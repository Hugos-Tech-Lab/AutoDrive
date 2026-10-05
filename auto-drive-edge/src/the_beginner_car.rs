// 
pub struct VehicleTelemetry {
  motor_a_set_speed: i8,
  motor_b_set_speed: i8,
  battery_voltage: f32,

}

// Answers questions like: 
// Is CPU usage too high?
// What's the average over 5 minutes?
// Hows the Memory?
pub struct SoftwareTelemetry {
  cpu_percentage: metrics::Gauge
}

pub struct TheBeginnerCar {
  connected: bool,
  software_telemetry: SoftwareTelemetry,
  vehicle_telemetry: VehicleTelemetry
}


