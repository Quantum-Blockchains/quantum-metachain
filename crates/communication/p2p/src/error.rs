use libp2p::TransportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Parsing address error: address - {addr:?}")]
    ParsingAddressError {
        #[source]
        source: libp2p::multiaddr::Error,
        addr: String,
    },

    #[error("Address listening error")]
    AddressListeningError {
        #[from]
        source: TransportError<std::io::Error>,
    },

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
