pub const WITT: &str = include_str!("the_beginner/wasm.wit");
pub const OPENAPI: &str = include_str!("the_beginner/openapi.yaml");

const _: () = {
    assert!(!WITT.is_empty());
    assert!(!OPENAPI.is_empty());
};