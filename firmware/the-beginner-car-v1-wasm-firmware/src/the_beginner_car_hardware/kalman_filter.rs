use std::sync::{Arc, Mutex};

use the_beginner_car_spi_protocol::Position;

use crate::the_beginner_car_hardware::wheel::tachometer::Tachometer;

pub struct KalmanFilterIterateState {
    pub dt: f32,                                     // timestep in ms
    pub wingspan: f32,                               // = 18.5;
    pub left_tachometer_total_pulses_previous: i32,  //,  = self.left_tachometer.get_total_pulses();
    pub right_tachometer_total_pulses_previous: i32, // = self.right_tachometer.get_total_pulses();
    pub current_rotation: f32, //  = 0.0; // TODO: derived from current position
}

#[derive(Clone)]
pub struct KalmanFilter {
    position: Arc<Mutex<Option<Position>>>, // 0.1: rotation, distance moved
    // 0.2: rotation, distance moved
    left_tachometer: Tachometer,
    right_tachometer: Tachometer,
}

impl KalmanFilter {
    pub fn new(left_tachometer: Tachometer, right_tachometer: Tachometer) -> Self {
        let position = Arc::new(Mutex::new(Some(Position {
            x: 0.0,
            y: 0.0,
            rotation_radians: 0.0,
            timestamp_unix_epoch_ms: 0,
        })));

        Self {
            position,
            left_tachometer,
            right_tachometer,
        }
    }

    pub fn create_iterate_state(&self) -> KalmanFilterIterateState {
        KalmanFilterIterateState {
            dt: 0.0,
            wingspan: 18.5,
            left_tachometer_total_pulses_previous: self.left_tachometer.get_total_pulses(),
            right_tachometer_total_pulses_previous: self.right_tachometer.get_total_pulses(),
            current_rotation: 0.0,
        }
    }

    pub fn iterate(&self, state: &mut KalmanFilterIterateState) -> Option<Position> {
        let left_tachometer_total_pulses = self.left_tachometer.get_total_pulses();
        let right_tachometer_total_pulses = self.right_tachometer.get_total_pulses();
        let left_tachometer_pulses =
            left_tachometer_total_pulses - state.left_tachometer_total_pulses_previous;
        let right_tachometer_pulses =
            right_tachometer_total_pulses - state.right_tachometer_total_pulses_previous;

        state.left_tachometer_total_pulses_previous = left_tachometer_total_pulses;
        state.right_tachometer_total_pulses_previous = right_tachometer_total_pulses;

        let distance_left_cm = left_tachometer_pulses as f32 * self.left_tachometer.cm_per_pulse();
        let distance_right_cm =
            right_tachometer_pulses as f32 * self.right_tachometer.cm_per_pulse();
        let position_delta = Self::predict(
            distance_left_cm,
            distance_right_cm,
            state.wingspan,
            state.current_rotation,
        );

        state.current_rotation += position_delta.rotation_radians;

        let position;
        {
            let mut position_guard = self.position.lock().unwrap();
            if let Some(pos) = position_guard.as_mut() {
                pos.add(position_delta);
                position = Some(pos.clone());
            } else {
                position = None;
            }
        }

        position
    }

    pub fn update(&mut self, position: Position) {
        let mut position_guard = self.position.lock().unwrap();
        position_guard.replace(position);
    }

    pub fn predict(
        distance_left_cm: f32,
        distance_right_cm: f32,
        wingspan: f32,
        current_rotation: f32,
    ) -> Position {
        let distance = (distance_left_cm + distance_right_cm) / 2.0;
        let delta_theta = (distance_right_cm - distance_left_cm) / wingspan;

        let (dx, dy) = if delta_theta.abs() < 1e-6 {
            (
                distance * current_rotation.cos(),
                distance * current_rotation.sin(),
            )
        } else {
            let radius = distance / delta_theta;

            (
                radius * ((current_rotation + delta_theta).sin() - current_rotation.sin()),
                -radius * ((current_rotation + delta_theta).cos() - current_rotation.cos()),
            )
        };

        Position {
            x: dx,
            y: dy,
            rotation_radians: delta_theta,
            timestamp_unix_epoch_ms: 0,
        }
    }

    // expected to be called, i.e. every 100ms
    pub fn position(&self) -> Option<Position> {
        let position = self.position.lock().unwrap();
        position.clone()
    }
}
