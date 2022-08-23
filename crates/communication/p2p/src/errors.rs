use libp2p::TransportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StartP2PServiceError {

    #[error("Transport construct error")]
    TransportConstructError{
        #[from]
        source: std::io::Error
    },

    #[error("Parsing address error")]
    ParsingAddressError {
        #[from]
        source: libp2p::multiaddr::Error
    },

    #[error("Address listening error")]
    AddressListeningError {
        #[from]
        source: TransportError<std::io::Error>
    }

}