use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PwmToRpm {
    pub pwm: i8,
    pub rpm: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MotorConfig {
    /// ranged from 1 - 2,147,483,647
    pub min_pos_rpm: i32,
    pub max_pos_rpm: i32,
    /// ranged from -2,147,483,647 to -1
    pub min_neg_rpm: i32,
    pub max_neg_rpm: i32,
    pub pwm_to_rpm: Vec<PwmToRpm>,
}

impl MotorConfig {
    /// Sorts the lookup table, checks monotonicity, and validates configuration bounds.
    fn initialize(&mut self) -> Result<(), String> {
        self.pwm_to_rpm.sort_by_key(|p| p.pwm);

        if self.min_pos_rpm <= 0 || self.max_pos_rpm <= 0 {
            return Err("Positive RPM limits must be greater than 0.".into());
        }
        if self.min_pos_rpm > self.max_pos_rpm {
            return Err("min_pos_rpm cannot be greater than max_pos_rpm.".into());
        }

        if self.min_neg_rpm >= 0 || self.max_neg_rpm >= 0 {
            return Err("Negative RPM limits must be less than 0.".into());
        }
        // Assuming min_neg_rpm is the lower absolute magnitude (closer to 0) and max_neg_rpm is higher magnitude
        if self.min_neg_rpm < self.max_neg_rpm {
            return Err(
                "min_neg_rpm magnitude must be less than or equal to max_neg_rpm magnitude.".into(),
            );
        }

        // 3. Verify that pwm_to_rpm is monotonic (RPM should increase as PWM increases)
        if self.pwm_to_rpm.windows(2).any(|w| w[0].rpm >= w[1].rpm) {
            return Err(
                "pwm_to_rpm table must be strictly monotonic (RPM must increase with PWM).".into(),
            );
        }

        Ok(())
    }

    pub fn min_pos_rpm(&self) -> i32 {
        self.min_pos_rpm
    }

    pub fn max_pos_rpm(&self) -> i32 {
        self.max_pos_rpm
    }

    pub fn min_neg_rpm(&self) -> i32 {
        self.min_neg_rpm
    }

    pub fn max_neg_rpm(&self) -> i32 {
        self.max_neg_rpm
    }

    /// Converts a PWM value to RPM using linear interpolation.
    pub fn speed_to_rpm(&self, pwm: i8) -> i32 {
        if self.pwm_to_rpm.is_empty() {
            return 0;
        }

        if pwm <= self.pwm_to_rpm.first().unwrap().pwm {
            return self.pwm_to_rpm.first().unwrap().rpm;
        }
        if pwm >= self.pwm_to_rpm.last().unwrap().pwm {
            return self.pwm_to_rpm.last().unwrap().rpm;
        }

        for window in self.pwm_to_rpm.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];

            if pwm >= p1.pwm && pwm <= p2.pwm {
                if p1.pwm == p2.pwm {
                    return p1.rpm;
                }
                let pwm_range = (p2.pwm - p1.pwm) as i32;
                let rpm_range = p2.rpm - p1.rpm;
                let delta = (pwm - p1.pwm) as i32;

                return p1.rpm + (delta * rpm_range) / pwm_range;
            }
        }

        0
    }

    /// Converts a target RPM to a PWM value using inverse linear interpolation.
    pub fn rpm_to_pwm(&self, target_rpm: i32) -> i8 {
        if self.pwm_to_rpm.is_empty() {
            return 0;
        }

        // Handle out-of-bounds target RPM
        if target_rpm <= self.pwm_to_rpm.first().unwrap().rpm {
            return self.pwm_to_rpm.first().unwrap().pwm;
        }
        if target_rpm >= self.pwm_to_rpm.last().unwrap().rpm {
            return self.pwm_to_rpm.last().unwrap().pwm;
        }

        // Find the two points in the lookup table that bracket the target RPM
        for window in self.pwm_to_rpm.windows(2) {
            let p1 = &window[0];
            let p2 = &window[1];

            let min_rpm = std::cmp::min(p1.rpm, p2.rpm);
            let max_rpm = std::cmp::max(p1.rpm, p2.rpm);

            if target_rpm >= min_rpm && target_rpm <= max_rpm {
                if p1.rpm == p2.rpm {
                    return p1.pwm;
                }
                // Inverse linear interpolation..
                let rpm_range = p2.rpm - p1.rpm;
                let pwm_range = (p2.pwm - p1.pwm) as i32;
                let delta_rpm = target_rpm - p1.rpm;

                let calculated_pwm = p1.pwm as i32 + (delta_rpm * pwm_range) / rpm_range;
                return calculated_pwm.clamp(i8::MIN as i32, i8::MAX as i32) as i8;
            }
        }

        0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareConfig {
    pub left_motor: MotorConfig,
    pub right_motor: MotorConfig,
}

impl HardwareConfig {
    /// Initializes and validates both motor configurations.
    pub fn initialize(&mut self) -> Result<(), String> {
        self.left_motor
            .initialize()
            .map_err(|e| format!("Left motor error: {}", e))?;
        self.right_motor
            .initialize()
            .map_err(|e| format!("Right motor error: {}", e))?;
        Ok(())
    }

    pub fn min_pos_rpm(&self) -> i32 {
        std::cmp::min(
            self.left_motor.min_pos_rpm(),
            self.right_motor.min_pos_rpm(),
        )
    }

    pub fn max_pos_rpm(&self) -> i32 {
        std::cmp::min(
            self.left_motor.max_pos_rpm(),
            self.right_motor.max_pos_rpm(),
        )
    }

    pub fn min_neg_rpm(&self) -> i32 {
        std::cmp::max(
            self.left_motor.min_neg_rpm(),
            self.right_motor.min_neg_rpm(),
        )
    }

    pub fn max_neg_rpm(&self) -> i32 {
        std::cmp::min(
            self.left_motor.max_neg_rpm(),
            self.right_motor.max_neg_rpm(),
        )
    }

    pub fn symmetric_speed_limits(&self) -> (i32, i32) {
        let global_max_pos = self.max_pos_rpm();
        let global_max_neg_magnitude = self.max_neg_rpm().abs();
        let symmetric_limit = std::cmp::min(global_max_pos, global_max_neg_magnitude);
        (symmetric_limit, -symmetric_limit)
    }

    /// Converts a speed (-100 to 100, where 0 is stopped)
    /// to a target RPM based on the global symmetric speed limits.
    ///
    /// - 0 -> 0 RPM
    /// - 1 -> Lowest positive symmetric RPM
    /// - 100 -> Max positive symmetric RPM
    /// - -1 -> Lowest negative symmetric RPM magnitude (negative)
    /// - -100 -> Max negative symmetric RPM magnitude (negative)
    pub fn speed_to_rpm(&self, speed: i8) -> i32 {
        if speed == 0 {
            return 0;
        }

        let (max_pos, max_neg_magnitude) = self.symmetric_speed_limits();

        // Fetch minimum movement thresholds based on the motors
        let min_pos = std::cmp::min(
            self.left_motor.min_pos_rpm(),
            self.right_motor.min_pos_rpm(),
        );
        let min_neg_magnitude = std::cmp::max(
            self.left_motor.min_neg_rpm().abs(),
            self.right_motor.min_neg_rpm().abs(),
        );

        if speed > 0 {
            // Map 1..=100 from min_pos..=max_pos
            let clamped_speed = speed.clamp(1, 100) as i32;
            if max_pos <= min_pos {
                return min_pos;
            }
            // Linear scaling from percentage to RPM range
            let range = max_pos - min_pos;
            min_pos + ((clamped_speed - 1) * range) / 99
        } else {
            // Map -1..=-100 from -min_neg_magnitude..=-max_neg_magnitude
            let clamped_speed = speed.clamp(-100, -1).abs() as i32;
            if max_neg_magnitude <= min_neg_magnitude {
                return -min_neg_magnitude;
            }
            let range = max_neg_magnitude - min_neg_magnitude;
            let magnitude = min_neg_magnitude + ((clamped_speed - 1) * range) / 99;
            -magnitude
        }
    }
}
