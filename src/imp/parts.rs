//! Shared internal helpers: marker traits, metadata, parse trait, and the
//! lightweight shared reference `RmrRef` used by both `Iri` and `IriRef`.

use alloc::string::String;
use core::num::NonZeroUsize;

use crate::component::{IAuthority, Scheme};
use crate::error::IriParseError;
use crate::parse;
use crate::pct_enc::encoder::{IFragment, IPath, IQuery};
use crate::pct_enc::{EStr, Encoder};

// ── Internal marker traits and metadata ────────────────────────────

pub(crate) trait Value: Default {}
impl Value for &str {}
impl Value for String {}

pub(crate) struct Constraints {
    pub(crate) scheme_required: bool,
}

/// Trait that unifies `Iri` and `IriRef`.
pub(crate) trait RiMaybeRef: Sized {
    type Val;
    type WithVal<T>: RiMaybeRef<Val = T>;

    const CONSTRAINTS: Constraints;

    fn new(val: Self::Val, meta: Meta) -> Self;

    fn from_pair((val, meta): (Self::Val, Meta)) -> Self {
        Self::new(val, meta)
    }

    fn make_ref<'i, 'o>(&'i self) -> RmrRef<'o, 'i>
    where
        Self::Val: borrow_or_share::BorrowOrShare<'i, 'o, str>;
}

// ── Metadata ───────────────────────────────────────────────────────

#[derive(Clone, Copy, Default)]
pub(crate) struct Meta {
    /// Index of the trailing colon of the scheme component.
    pub(crate) scheme_end: Option<NonZeroUsize>,
    pub(crate) auth_meta: Option<AuthMeta>,
    pub(crate) path_bounds: (usize, usize),
    /// One byte past the last byte of the query component.
    pub(crate) query_end: Option<NonZeroUsize>,
}

impl Meta {
    #[inline]
    pub(crate) fn query_or_path_end(&self) -> usize {
        self.query_end
            .map_or(self.path_bounds.1, core::num::NonZeroUsize::get)
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct AuthMeta {
    pub(crate) host_bounds: (usize, usize),
    pub(crate) host_meta: HostMeta,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum HostMeta {
    Ipv4,
    Ipv6,
    IpvFuture,
    #[default]
    RegName,
}

// ── Parsing trait ──────────────────────────────────────────────────

pub(crate) trait Parse {
    type Val;
    type Err;

    fn parse_iri<R: RiMaybeRef<Val = Self::Val>>(self) -> Result<R, Self::Err>;
}

impl<'a> Parse for &'a str {
    type Val = &'a str;
    type Err = IriParseError;

    fn parse_iri<R: RiMaybeRef<Val = Self::Val>>(self) -> Result<R, Self::Err> {
        parse::parse(self.as_bytes(), R::CONSTRAINTS).map(|meta| R::new(self, meta))
    }
}

impl Parse for String {
    type Val = Self;
    type Err = (IriParseError, Self);

    fn parse_iri<R: RiMaybeRef<Val = Self::Val>>(self) -> Result<R, Self::Err> {
        match parse::parse(self.as_bytes(), R::CONSTRAINTS) {
            Ok(meta) => Ok(R::new(self, meta)),
            Err(e) => Err((e, self)),
        }
    }
}

// ── RmrRef (lightweight shared reference) ──────────────────────────

/// Lightweight shared reference into an IRI or IRI reference.
#[derive(Clone, Copy)]
pub(crate) struct RmrRef<'v, 'm> {
    val: &'v str,
    meta: &'m Meta,
}

impl<'v, 'm> RmrRef<'v, 'm> {
    pub(crate) fn new(val: &'v str, meta: &'m Meta) -> Self {
        Self { val, meta }
    }

    pub(crate) fn as_str(self) -> &'v str {
        self.val
    }

    fn slice(self, start: usize, end: usize) -> &'v str {
        &self.val[start..end]
    }

    fn eslice<E: Encoder>(self, start: usize, end: usize) -> &'v EStr<E> {
        EStr::new_validated(self.slice(start, end))
    }

    pub(crate) fn scheme_opt(self) -> Option<&'v Scheme> {
        let end = self.meta.scheme_end?.get();
        Some(Scheme::new_validated(self.slice(0, end)))
    }

    pub(crate) fn scheme(self) -> &'v Scheme {
        let end = self.meta.scheme_end.map_or(0, NonZeroUsize::get);
        Scheme::new_validated(self.slice(0, end))
    }

    pub(crate) fn authority(self) -> Option<IAuthority<'v>> {
        let mut meta = self.meta.auth_meta?;
        let start = match self.meta.scheme_end {
            Some(i) => i.get() + 3,
            None => 2,
        };
        let end = self.meta.path_bounds.0;

        meta.host_bounds.0 -= start;
        meta.host_bounds.1 -= start;

        Some(IAuthority::new(self.slice(start, end), meta))
    }

    pub(crate) fn path(self) -> &'v EStr<IPath> {
        self.eslice(self.meta.path_bounds.0, self.meta.path_bounds.1)
    }

    pub(crate) fn query(self) -> Option<&'v EStr<IQuery>> {
        let end = self.meta.query_end?.get();
        Some(self.eslice(self.meta.path_bounds.1 + 1, end))
    }

    fn fragment_start(self) -> Option<usize> {
        Some(self.meta.query_or_path_end())
            .filter(|&i| i != self.val.len())
            .map(|i| i + 1)
    }

    pub(crate) fn fragment(self) -> Option<&'v EStr<IFragment>> {
        self.fragment_start()
            .map(|i| self.eslice(i, self.val.len()))
    }

    #[cfg(test)]
    pub(crate) fn set_fragment(buf: &mut String, meta: &Meta, opt: Option<&str>) {
        buf.truncate(meta.query_or_path_end());
        if let Some(s) = opt {
            buf.reserve_exact(s.len() + 1);
            buf.push('#');
            buf.push_str(s);
        }
    }

    #[cfg(test)]
    pub(crate) fn strip_fragment(self) -> &'v str {
        &self.val[..self.meta.query_or_path_end()]
    }

    #[inline]
    pub(crate) fn has_scheme(self) -> bool {
        self.meta.scheme_end.is_some()
    }

    #[inline]
    pub(crate) fn has_authority(self) -> bool {
        self.meta.auth_meta.is_some()
    }

    #[inline]
    #[cfg(test)]
    pub(crate) fn has_query(self) -> bool {
        self.meta.query_end.is_some()
    }

    #[inline]
    pub(crate) fn has_fragment(self) -> bool {
        self.meta.query_or_path_end() != self.val.len()
    }
}
