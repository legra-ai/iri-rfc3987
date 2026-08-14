//! IRI parse error types.

use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IriParseErrorKind {
    UnexpectedCharOrEnd,
    InvalidIpv6Addr,
}

/// An error occurred when parsing an IRI or IRI reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IriParseError {
    pub(crate) index: usize,
    pub(crate) kind: IriParseErrorKind,
}

impl IriParseError {
    /// Returns the byte index at which the error occurred.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }
}

impl fmt::Display for IriParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.kind {
            IriParseErrorKind::UnexpectedCharOrEnd => {
                "unexpected character or end of input at index "
            }
            IriParseErrorKind::InvalidIpv6Addr => "invalid IPv6 address at index ",
        };
        write!(f, "{}{}", msg, self.index)
    }
}

impl std::error::Error for IriParseError {}
