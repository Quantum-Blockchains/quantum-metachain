use thiserror::Error;

#[derive(Error, Debug)]
pub enum RPCError {
    #[error("Parsing address error: address - {0}")]
    ParsingAddressError(String),
}
