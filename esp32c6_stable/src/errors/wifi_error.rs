

pub enum WifiError {
    DNSError(DNSError),
    Other
}

pub enum DNSError {
    InvalidName,
    NameTooLong,
    Failed,
    NotFound
}

impl From<embassy_net::dns::Error> for WifiError {
    fn from(e: embassy_net::dns::Error) -> Self {
        match e {
            embassy_net::dns::Error::InvalidName => Self::DNSError(DNSError::InvalidName),
            embassy_net::dns::Error::NameTooLong => Self::DNSError(DNSError::NameTooLong),
            embassy_net::dns::Error::Failed => Self::DNSError(DNSError::Failed),
        }
    }
}