use std::fmt;
use std::error::Error;
use sled::Error as SledError;
use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum LWWMapError {
    SledError(SledError),
    SerdeError(SerdeError),
}

impl fmt::Display for LWWMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LWWMapError::SledError(e) => write!(f, "Sled error: {}", e),
            LWWMapError::SerdeError(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl Error for LWWMapError {}

impl From<SledError> for LWWMapError {
    fn from(error: SledError) -> Self {
        LWWMapError::SledError(error)
    }
}

impl From<SerdeError> for LWWMapError {
    fn from(error: SerdeError) -> Self {
        LWWMapError::SerdeError(error)
    }
}
