use std::fmt;

#[derive(Debug)]
pub enum ServerError {
    PortInUse,
    PermissionDenied,
    TlsError(String),
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCreds,
    Locked,
    Expired,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortInUse => write!(f, "port already in use"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::TlsError(e) => write!(f, "TLS error: {e}"),
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCreds => write!(f, "invalid credentials"),
            Self::Locked => write!(f, "account locked"),
            Self::Expired => write!(f, "session expired"),
        }
    }
}

impl std::error::Error for ServerError {}
impl std::error::Error for AuthError {}
