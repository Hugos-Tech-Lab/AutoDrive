use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpServerError {
    #[error("{0}")]
    UserError(String),
    #[error("Internal Error: `{0}`")]
    InternalProgrammerError(String),
}
