pub const WITT: &str = include_str!("the-beginner/wasm.wit");
pub const OPENAPI: &str = include_str!("the-beginner/openapi.yaml");

const _: () = {
    assert!(!WITT.is_empty());
    assert!(!OPENAPI.is_empty());
};