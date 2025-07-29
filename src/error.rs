use markdown::message::Place;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlyphoError {
    #[error("File not found")]
    NotFound,
    #[error("Makrdown parsing{place:?}, {reason:?}, {rule_id:?}, {m_source:?}")]
    MarkdownError {
        place: Option<Box<Place>>,
        reason: String,
        rule_id: String,
        m_source: String,
    },

    #[error("File format is not text")]
    InvalidData,
    #[error("Unknown Error")]
    Unknown,
}

impl From<std::io::ErrorKind> for GlyphoError {
    fn from(io_err_kind: std::io::ErrorKind) -> Self {
        match io_err_kind {
            std::io::ErrorKind::NotFound => GlyphoError::NotFound,
            std::io::ErrorKind::Other => GlyphoError::Unknown,
            std::io::ErrorKind::InvalidData => GlyphoError::InvalidData,
            _ => GlyphoError::Unknown,
        }
    }
}
