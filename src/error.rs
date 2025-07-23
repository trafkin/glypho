use markdown::message::Place;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlyphoError {
    #[error("{0:?}")]
    Io(#[from] std::io::Error),
    #[error("{place:?}, {reason:?}, {rule_id:?}, {m_source:?}")]
    MarkdownError {
        place: Option<Box<Place>>,
        reason: String,
        rule_id: String,
        m_source: String,
    },
}
