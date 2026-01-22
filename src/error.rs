use std::sync::PoisonError;

use markdown::message::Place;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlyphoError {
    #[error("File not provided")]
    NotProvided,
    #[error("File not found")]
    NotFound,
    #[error("Markdown parsing{place:?}, {reason:?}, {rule_id:?}, {m_source:?}")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::sync::Mutex;

    // ==================== From<PoisonError<T>> Tests ====================

    #[test]
    fn test_from_poison_error() {
        // Create a poisoned mutex by panicking inside a lock
        let mutex = Mutex::new(42);
        let poison_result = std::panic::catch_unwind(|| {
            let _guard = mutex.lock().unwrap();
            panic!("Intentional panic to poison the mutex");
        });
        assert!(poison_result.is_err());

        // Now the mutex is poisoned
        let lock_result = mutex.lock();
        assert!(lock_result.is_err());

        if let Err(poison_err) = lock_result {
            let glypho_err: GlyphoError = poison_err.into();
            assert!(matches!(glypho_err, GlyphoError::PoisonError));
        }
    }

    // ==================== From<std::io::ErrorKind> Tests ====================

    #[rstest]
    #[case(std::io::ErrorKind::NotFound, GlyphoError::NotFound)]
    #[case(std::io::ErrorKind::Other, GlyphoError::Unknown)]
    #[case(std::io::ErrorKind::InvalidData, GlyphoError::InvalidData)]
    fn test_from_io_error_kind_mapped(
        #[case] io_kind: std::io::ErrorKind,
        #[case] expected: GlyphoError,
    ) {
        let result: GlyphoError = io_kind.into();
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&expected)
        );
    }

    #[rstest]
    #[case(std::io::ErrorKind::PermissionDenied)]
    #[case(std::io::ErrorKind::ConnectionRefused)]
    #[case(std::io::ErrorKind::ConnectionReset)]
    #[case(std::io::ErrorKind::ConnectionAborted)]
    #[case(std::io::ErrorKind::NotConnected)]
    #[case(std::io::ErrorKind::AddrInUse)]
    #[case(std::io::ErrorKind::AddrNotAvailable)]
    #[case(std::io::ErrorKind::BrokenPipe)]
    #[case(std::io::ErrorKind::AlreadyExists)]
    #[case(std::io::ErrorKind::WouldBlock)]
    #[case(std::io::ErrorKind::InvalidInput)]
    #[case(std::io::ErrorKind::TimedOut)]
    #[case(std::io::ErrorKind::WriteZero)]
    #[case(std::io::ErrorKind::Interrupted)]
    #[case(std::io::ErrorKind::Unsupported)]
    #[case(std::io::ErrorKind::UnexpectedEof)]
    #[case(std::io::ErrorKind::OutOfMemory)]
    fn test_from_io_error_kind_unknown(#[case] io_kind: std::io::ErrorKind) {
        let result: GlyphoError = io_kind.into();
        assert!(matches!(result, GlyphoError::Unknown));
    }

    // ==================== Debug Trait Tests ====================

    #[test]
    fn test_error_debug() {
        let err = GlyphoError::NotFound;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_markdown_error_debug() {
        let err = GlyphoError::MarkdownError {
            place: None,
            reason: "test".to_string(),
            rule_id: "rule".to_string(),
            m_source: "src".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("MarkdownError"));
        assert!(debug_str.contains("test"));
    }

    // ==================== Error Trait Tests ====================

    #[test]
    fn test_error_is_std_error() {
        fn assert_std_error<E: std::error::Error>(_: &E) {}

        let err = GlyphoError::NotFound;
        assert_std_error(&err);
    }

    #[test]
    fn test_eyre_report_conversion() {
        let eyre_err = eyre::eyre!("Test error");
        let glypho_err: GlyphoError = eyre_err.into();
        assert!(matches!(glypho_err, GlyphoError::ErrReport(_)));
    }

    // ==================== Variant Coverage Tests ====================

    #[test]
    fn test_all_variants_exist() {
        // This test ensures all variants can be constructed
        let variants: Vec<GlyphoError> = vec![
            GlyphoError::NotProvided,
            GlyphoError::NotFound,
            GlyphoError::MarkdownError {
                place: None,
                reason: String::new(),
                rule_id: String::new(),
                m_source: String::new(),
            },
            GlyphoError::InvalidData,
            GlyphoError::Unknown,
            GlyphoError::PoisonError,
            GlyphoError::ErrReport(eyre::eyre!("test")),
        ];

        assert_eq!(variants.len(), 7);
    }
}
