//! Constant-space validation of byte-window IRI references.

mod authority;
mod grammar;

use crate::IriParseError;
use crate::error::IriParseErrorKind;
use grammar::Grammar;

/// Classification of a completely validated RFC 3987 IRI reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IriReferenceClassification {
    absolute: bool,
}

impl IriReferenceClassification {
    /// Whether the reference has a scheme and is an absolute IRI.
    #[must_use]
    pub fn is_absolute(self) -> bool {
        self.absolute
    }
}

/// Validates an IRI reference incrementally without retaining its bytes.
///
/// Windows may split UTF-8 scalars or percent escapes. A successful `push`
/// validates only a prefix: callers must call `finish` at verified input EOF.
/// Failure is sticky. Memory use is independent of the reference length.
#[derive(Debug, Default)]
pub struct IriReferenceValidator {
    grammar: Grammar,
    offset: usize,
    scalar: [u8; 4],
    scalar_len: usize,
    scalar_width: usize,
    percent_remaining: u8,
    error: Option<IriParseError>,
}

impl IriReferenceValidator {
    /// Creates an empty reference validator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes the next byte window.
    ///
    /// # Errors
    /// Returns an error for malformed UTF-8, percent escapes, or IRI grammar.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), IriParseError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        for &byte in bytes {
            if !self.byte(byte) {
                return Err(self.fail());
            }
            let Some(next) = self.offset.checked_add(1) else {
                return Err(self.fail());
            };
            self.offset = next;
        }
        Ok(())
    }

    /// Completes validation at input EOF and returns its classification.
    ///
    /// # Errors
    /// Returns the original failure or an error for incomplete input.
    pub fn finish(mut self) -> Result<IriReferenceClassification, IriParseError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        if self.scalar_len != 0 || self.percent_remaining != 0 || !self.grammar.finish() {
            return Err(self.fail());
        }
        Ok(IriReferenceClassification {
            absolute: self.grammar.absolute,
        })
    }

    fn fail(&mut self) -> IriParseError {
        let error = IriParseError {
            index: self.offset,
            kind: IriParseErrorKind::UnexpectedCharOrEnd,
        };
        self.error = Some(error);
        error
    }

    fn byte(&mut self, byte: u8) -> bool {
        if self.percent_remaining != 0 {
            if !byte.is_ascii_hexdigit() {
                return false;
            }
            self.percent_remaining -= 1;
            return true;
        }
        if self.scalar_len == 0 {
            if byte == b'%' {
                self.percent_remaining = 2;
                return self.grammar.atom('%', true);
            }
            if byte.is_ascii() {
                return self.grammar.atom(char::from(byte), false);
            }
            self.scalar_width = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => return false,
            };
        }
        self.scalar[self.scalar_len] = byte;
        self.scalar_len += 1;
        if self.scalar_len < self.scalar_width {
            return (self.scalar_len == 1) || (byte & 0xc0 == 0x80);
        }
        let Ok(text) = core::str::from_utf8(&self.scalar[..self.scalar_len]) else {
            return false;
        };
        let scalar = text.chars().next().expect("nonempty UTF-8 scalar");
        self.scalar_len = 0;
        self.grammar.atom(scalar, false)
    }
}
