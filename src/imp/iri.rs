//! `Iri<T>` — a fully-qualified IRI (scheme is required).

use alloc::string::String;

use borrow_or_share::BorrowOrShare;
#[cfg(test)]
use borrow_or_share::Bos;

use super::parts::{Constraints, Meta, Parse, RiMaybeRef, RmrRef};
use crate::component::{IAuthority, Scheme};
use crate::error::IriParseError;
#[cfg(test)]
use crate::normalize::Normalizer;
use crate::pct_enc::EStr;
use crate::pct_enc::encoder::{IFragment, IPath, IQuery};

// ════════════════════════════════════════════════════════════════════
//  Iri<T>  —  a fully-qualified IRI (scheme is required)
// ════════════════════════════════════════════════════════════════════

/// An IRI (Internationalized Resource Identifier) compliant with
/// [RFC 3987](https://datatracker.ietf.org/doc/html/rfc3987).
///
/// The scheme component is always present.
///
/// # Variants
///
/// `Iri<&str>` — borrowed, zero-copy. `Iri<String>` — owned.
///
/// # Comparison
///
/// `Iri`s are compared
/// [lexicographically](Ord#lexicographical-comparison) by their
/// byte values. Normalization is **not** performed prior to
/// comparison.
#[derive(Clone, Copy)]
pub struct Iri<T> {
    pub(crate) val: T,
    pub(crate) meta: Meta,
}

impl<T> RiMaybeRef for Iri<T> {
    type Val = T;
    type WithVal<U> = Iri<U>;

    const CONSTRAINTS: Constraints = Constraints {
        scheme_required: true,
    };

    fn new(val: T, meta: Meta) -> Self {
        Self { val, meta }
    }

    fn make_ref<'i, 'o>(&'i self) -> RmrRef<'o, 'i>
    where
        T: BorrowOrShare<'i, 'o, str>,
    {
        RmrRef::new(self.as_str(), &self.meta)
    }
}

impl<'a> Iri<&'a str> {
    /// Parses a borrowed IRI from a string slice.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string does not match the
    /// [`IRI`](https://datatracker.ietf.org/doc/html/rfc3987#section-2.2)
    /// ABNF rule from RFC 3987.
    pub fn parse(input: &'a str) -> Result<Self, IriParseError> {
        input.parse_iri()
    }
}

impl Iri<String> {
    /// Parses an owned IRI from a `String`.
    ///
    /// On failure the original `String` is returned alongside the
    /// error so it can be reused.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string does not match the
    /// [`IRI`](https://datatracker.ietf.org/doc/html/rfc3987#section-2.2)
    /// ABNF rule from RFC 3987.
    pub fn parse_owned(input: String) -> Result<Self, (IriParseError, String)> {
        input.parse_iri()
    }
}

impl Iri<String> {
    /// Borrows this `Iri<String>` as `Iri<&str>`.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn borrow(&self) -> Iri<&str> {
        Iri {
            val: &self.val,
            meta: self.meta,
        }
    }

    /// Consumes this `Iri<String>` and yields the underlying
    /// [`String`].
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.val
    }

    /// Parse a known-valid IRI constant.
    ///
    /// Use this for compile-time-known IRI strings (e.g., namespace
    /// constants or test fixtures.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not a valid IRI per RFC 3987.
    #[must_use]
    pub fn known(value: &str) -> Self {
        Iri::parse(value)
            .unwrap_or_else(|e| panic!("Iri::known called with invalid IRI {value:?}: {e}"))
            .to_owned()
    }
}

impl Iri<&str> {
    /// Creates a new `Iri<String>` by cloning the contents of this
    /// `Iri<&str>`.
    #[inline]
    #[must_use]
    pub fn to_owned(&self) -> Iri<String> {
        Iri {
            val: self.val.to_owned(),
            meta: self.meta,
        }
    }
}

impl<'i, 'o, T: BorrowOrShare<'i, 'o, str>> Iri<T> {
    /// Returns the IRI as a string slice.
    #[must_use]
    pub fn as_str(&'i self) -> &'o str {
        self.val.borrow_or_share()
    }

    /// Returns the [scheme] component.
    ///
    /// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
    #[must_use]
    pub(crate) fn scheme(&'i self) -> &'o Scheme {
        self.make_ref().scheme()
    }

    /// Returns the optional [authority] component.
    ///
    /// [authority]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2
    #[must_use]
    pub(crate) fn authority(&'i self) -> Option<IAuthority<'o>> {
        self.make_ref().authority()
    }

    /// Returns the [path] component.
    ///
    /// [path]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.3
    #[must_use]
    pub(crate) fn path(&'i self) -> &'o EStr<IPath> {
        self.make_ref().path()
    }

    /// Returns the optional [query] component.
    ///
    /// [query]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.4
    #[must_use]
    pub(crate) fn query(&'i self) -> Option<&'o EStr<IQuery>> {
        self.make_ref().query()
    }

    /// Returns the optional [fragment] component.
    ///
    /// [fragment]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.5
    #[must_use]
    pub(crate) fn fragment(&'i self) -> Option<&'o EStr<IFragment>> {
        self.make_ref().fragment()
    }
}

#[cfg(test)]
impl<T: Bos<str>> Iri<T> {
    /// Normalizes the IRI.
    ///
    /// Applies syntax-based normalization described in
    /// [Section 6.2.2 of RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986#section-6.2.2).
    ///
    /// # Panics
    ///
    /// Panics if normalization fails due to path underflow (should
    /// not happen with the default normalizer settings).
    #[must_use]
    pub(crate) fn normalize(&self) -> Iri<String> {
        Normalizer::new().normalize(self).unwrap()
    }

    /// Checks whether an authority component is present.
    #[must_use]
    pub(crate) fn has_authority(&self) -> bool {
        self.make_ref().has_authority()
    }

    /// Checks whether a query component is present.
    #[must_use]
    pub(crate) fn has_query(&self) -> bool {
        self.make_ref().has_query()
    }

    /// Checks whether a fragment component is present.
    #[must_use]
    pub(crate) fn has_fragment(&self) -> bool {
        self.make_ref().has_fragment()
    }

    /// Returns a slice with the fragment component removed.
    #[must_use]
    pub(crate) fn strip_fragment(&self) -> Iri<&str> {
        RiMaybeRef::new(self.make_ref().strip_fragment(), self.meta)
    }
}

#[cfg(test)]
impl Iri<String> {
    /// Replaces the fragment component of `self` with the given one.
    ///
    /// The fragment component is removed when `opt.is_none()`.
    pub(crate) fn set_fragment(&mut self, opt: Option<&EStr<IFragment>>) {
        RmrRef::set_fragment(&mut self.val, &self.meta, opt.map(EStr::as_str));
    }
}
