//! `IriRef<T>` — an IRI reference (scheme is optional).

use alloc::string::String;

use borrow_or_share::{BorrowOrShare, Bos};

use super::parts::{Constraints, Meta, Parse, RiMaybeRef, RmrRef};
use crate::component::{IAuthority, Scheme};
use crate::error::IriParseError;
use crate::imp::iri::Iri;
use crate::pct_enc::EStr;
use crate::pct_enc::encoder::{IFragment, IPath, IQuery};
use crate::resolve::{self, ResolveError};

// ════════════════════════════════════════════════════════════════════
//  IriRef<T>  —  an IRI reference (scheme is optional)
// ════════════════════════════════════════════════════════════════════

/// An IRI reference, i.e., either an IRI or a relative reference.
///
/// Compliant with
/// [RFC 3987](https://datatracker.ietf.org/doc/html/rfc3987).
///
/// # Variants
///
/// `IriRef<&str>` — borrowed, zero-copy. `IriRef<String>` — owned.
///
/// # Comparison
///
/// `IriRef`s are compared
/// [lexicographically](Ord#lexicographical-comparison) by their
/// byte values. Normalization is **not** performed prior to
/// comparison.
#[derive(Clone, Copy)]
pub struct IriRef<T> {
    pub(crate) val: T,
    pub(crate) meta: Meta,
}

impl<T> RiMaybeRef for IriRef<T> {
    type Val = T;
    type WithVal<U> = IriRef<U>;

    const CONSTRAINTS: Constraints = Constraints {
        scheme_required: false,
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

impl<'a> IriRef<&'a str> {
    /// Parses a borrowed IRI reference from a string slice.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string does not match the
    /// [`IRI-reference`](https://datatracker.ietf.org/doc/html/rfc3987#section-2.2)
    /// ABNF rule from RFC 3987.
    pub fn parse(input: &'a str) -> Result<Self, IriParseError> {
        input.parse_iri()
    }
}

impl IriRef<String> {
    /// Parses an owned IRI reference from a `String`.
    ///
    /// On failure the original `String` is returned alongside the
    /// error so it can be reused.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string does not match the
    /// [`IRI-reference`](https://datatracker.ietf.org/doc/html/rfc3987#section-2.2)
    /// ABNF rule from RFC 3987.
    pub fn parse_owned(input: String) -> Result<Self, (IriParseError, String)> {
        input.parse_iri()
    }
}

impl IriRef<String> {
    /// Borrows this `IriRef<String>` as `IriRef<&str>`.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn borrow(&self) -> IriRef<&str> {
        IriRef {
            val: &self.val,
            meta: self.meta,
        }
    }

    /// Consumes this `IriRef<String>` and yields the underlying
    /// [`String`].
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.val
    }
}

impl IriRef<&str> {
    /// Creates a new `IriRef<String>` by cloning the contents.
    #[inline]
    #[must_use]
    pub fn to_owned(&self) -> IriRef<String> {
        IriRef {
            val: self.val.to_owned(),
            meta: self.meta,
        }
    }
}

impl<'i, 'o, T: BorrowOrShare<'i, 'o, str>> IriRef<T> {
    /// Returns the IRI reference as a string slice.
    #[must_use]
    pub fn as_str(&'i self) -> &'o str {
        self.val.borrow_or_share()
    }

    /// Returns the optional [scheme] component.
    ///
    /// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
    #[must_use]
    pub(crate) fn scheme(&'i self) -> Option<&'o Scheme> {
        self.make_ref().scheme_opt()
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

impl<T: Bos<str>> IriRef<T> {
    /// Resolves the IRI reference against the given base IRI.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the base has a fragment, or if the base has a
    /// rootless path and no authority while the reference is
    /// relative, non-empty, and does not start with `'#'`.
    pub fn resolve_against<U: Bos<str>>(&self, base: &Iri<U>) -> Result<Iri<String>, ResolveError> {
        resolve::resolve(base.make_ref(), self.make_ref(), true).map(RiMaybeRef::from_pair)
    }

    /// Checks whether a scheme component is present.
    #[must_use]
    pub fn has_scheme(&self) -> bool {
        self.make_ref().has_scheme()
    }
}
