pub const OPENAPI: &str = include_str!("weather_station_v0/openapi.yaml");

const _: () = {
    assert!(!OPENAPI.is_empty());
};
