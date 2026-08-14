//! Conversion traits for IRI types.
//!
//! [`TryIntoIri`] provides fallible conversion into a validated
//! `Iri<String>` from strings and already-validated IRI types.

use alloc::string::String;

use crate::{Iri, IriParseError};

/// Fallible conversion into a validated [`Iri<String>`].
///
/// Implemented for string types (fallible — requires RFC 3987
/// validation) and for already-validated IRI types (infallible
/// passthrough).
///
/// # Examples
///
/// ```
/// use iri_rfc3987::{
///     Iri,
///     TryIntoIri,
/// };
///
/// // From &str:
/// let iri: Iri<String> = "http://example.com".try_into_iri().unwrap();
///
/// // Passthrough for Iri<String>:
/// let same = iri.clone().try_into_iri().unwrap();
/// assert_eq!(iri, same);
/// ```
pub trait TryIntoIri {
    /// Attempt to convert `self` into a validated `Iri<String>`.
    ///
    /// # Errors
    ///
    /// Returns [`IriParseError`] if the value is not a valid IRI
    /// per RFC 3987.
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError>;
}

impl TryIntoIri for &str {
    #[inline]
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError> {
        self.parse()
    }
}

impl TryIntoIri for String {
    #[inline]
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError> {
        Iri::parse_owned(self).map_err(|(e, _)| e)
    }
}

impl TryIntoIri for Iri<String> {
    #[inline]
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError> {
        Ok(self)
    }
}

impl TryIntoIri for Iri<&str> {
    #[inline]
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError> {
        Ok(self.to_owned())
    }
}

impl TryIntoIri for &Iri<String> {
    #[inline]
    fn try_into_iri(self) -> Result<Iri<String>, IriParseError> {
        Ok(self.clone())
    }
}
