pub const OPENAPI: &str = include_str!("weather_station/openapi.yaml");

const _: () = {
    assert!(!OPENAPI.is_empty());
};
