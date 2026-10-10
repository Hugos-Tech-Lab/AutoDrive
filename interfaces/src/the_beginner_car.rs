pub mod hardware_mcu;

pub const WITT: &str = include_str!("the_beginner_car/wasm.wit");
pub const OPENAPI: &str = include_str!("the_beginner_car/openapi.yaml");

const _: () = {
    assert!(!WITT.is_empty());
    assert!(!OPENAPI.is_empty());
};