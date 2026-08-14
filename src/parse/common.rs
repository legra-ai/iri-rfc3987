//! Shared parser result type and the `err!` early-return macro.

use crate::error::IriParseError;

pub(super) type Result<T> = core::result::Result<T, IriParseError>;

/// Returns immediately with an error.
macro_rules! err {
    ($index:expr, $kind:ident) => {
        return Err($crate::error::IriParseError {
            index: $index,
            kind: $crate::error::IriParseErrorKind::$kind,
        })
    };
}

pub(super) use err;
