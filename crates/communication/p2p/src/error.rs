use libp2p::TransportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Parsing address error: address - {0}")]
    ParsingAddressError(String),

    #[error("Address listening error")]
    AddressListeningError(#[from] TransportError<std::io::Error>),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
