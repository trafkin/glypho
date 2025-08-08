use std::sync::PoisonError;

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
    #[error("Poison Error")]
    PoisonError,
    #[error("ErrReport")]
    ErrReport(#[from] eyre::ErrReport),
}

impl<T> From<PoisonError<T>> for GlyphoError {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonError
    }
}

impl From<std::io::ErrorKind> for GlyphoError {
    fn from(io_err_kind: std::io::ErrorKind) -> Self {
        match io_err_kind {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::Other => Self::Unknown,
            std::io::ErrorKind::InvalidData => Self::InvalidData,
            _ => GlyphoError::Unknown,
        }
    }
}
