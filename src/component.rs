//! IRI components.

use core::marker::PhantomData;
#[cfg(test)]
use core::num::ParseIntError;
use core::{hash, iter};

use ref_cast::{RefCastCustom, ref_cast_custom};

use crate::imp::{AuthMeta, HostMeta};
use crate::pct_enc::encoder::{IRegName, IUserinfo, Port};
#[cfg(test)]
use crate::pct_enc::table;
use crate::pct_enc::{EStr, Encoder};

pub(crate) type IAuthority<'a> = Authority<'a, IUserinfo, IRegName>;

/// A [scheme] component.
///
/// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
///
/// # Comparison
///
/// `Scheme`s are compared case-insensitively.
#[derive(RefCastCustom)]
#[repr(transparent)]
pub(crate) struct Scheme {
    inner: str,
}

const ASCII_CASE_MASK: u8 = 0b0010_0000;

impl Scheme {
    #[ref_cast_custom]
    #[inline]
    pub(crate) const fn new_validated(scheme: &str) -> &Self;

    /// Converts a string slice to `&Scheme`.
    ///
    /// # Panics
    ///
    /// Panics if the string is not a valid scheme name according to
    /// [Section 3.1 of RFC 3986][scheme]. For a non-panicking
    /// variant, use [`new`](Self::new).
    ///
    /// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new_or_panic(s: &str) -> &Self {
        match Self::new(s) {
            Some(scheme) => scheme,
            None => panic!("invalid scheme"),
        }
    }

    /// Converts a string slice to `&Scheme`, returning `None` if the
    /// conversion fails.
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new(s: &str) -> Option<&Self> {
        if matches!(s.as_bytes(), [first, rem @ ..]
        if first.is_ascii_alphabetic() && table::SCHEME.validate(rem))
        {
            Some(Self::new_validated(s))
        } else {
            None
        }
    }

    /// Returns the scheme component as a string slice.
    #[inline]
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        &self.inner
    }
}

impl PartialEq for Scheme {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (self.inner.as_bytes(), other.inner.as_bytes());
        a.len() == b.len()
            && iter::zip(a, b).all(|(x, y)| x | ASCII_CASE_MASK == y | ASCII_CASE_MASK)
    }
}

impl Eq for Scheme {}

impl hash::Hash for Scheme {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let mut buf = [0; 8];
        for chunk in self.inner.as_bytes().chunks(8) {
            let len = chunk.len();
            for i in 0..len {
                buf[i] = chunk[i] | ASCII_CASE_MASK;
            }
            state.write(&buf[..len]);
        }
    }
}

impl core::fmt::Debug for Scheme {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl core::fmt::Display for Scheme {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}

#[derive(Clone, Copy)]
struct AuthorityInner<'a> {
    val: &'a str,
    meta: AuthMeta,
}

impl<'a> AuthorityInner<'a> {
    fn userinfo(&self) -> Option<&'a EStr<IUserinfo>> {
        let host_start = self.meta.host_bounds.0;
        (host_start != 0).then(|| EStr::new_validated(&self.val[..host_start - 1]))
    }

    fn host(&self) -> &'a str {
        let (start, end) = self.meta.host_bounds;
        &self.val[start..end]
    }

    fn port(&self) -> Option<&'a EStr<Port>> {
        let host_end = self.meta.host_bounds.1;
        (host_end != self.val.len()).then(|| EStr::new_validated(&self.val[host_end + 1..]))
    }

    #[cfg(test)]
    fn port_to_u16(&self) -> Result<Option<u16>, ParseIntError> {
        self.port()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str().parse())
            .transpose()
    }
}

/// An [authority] component.
///
/// [authority]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2
#[derive(Clone, Copy)]
pub(crate) struct Authority<'a, UserinfoE = IUserinfo, RegNameE = IRegName> {
    inner: AuthorityInner<'a>,
    _marker: PhantomData<(UserinfoE, RegNameE)>,
}

impl<'a, UserinfoE: Encoder, RegNameE: Encoder> Authority<'a, UserinfoE, RegNameE> {
    pub(crate) const fn new(val: &'a str, meta: AuthMeta) -> Self {
        Self {
            inner: AuthorityInner { val, meta },
            _marker: PhantomData,
        }
    }

    pub(crate) fn meta(&self) -> AuthMeta {
        self.inner.meta
    }

    /// Returns the authority component as a string slice.
    #[inline]
    #[must_use]
    pub(crate) fn as_str(&self) -> &'a str {
        self.inner.val
    }

    /// Returns the optional [userinfo] subcomponent.
    ///
    /// [userinfo]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1
    #[must_use]
    pub(crate) fn userinfo(&self) -> Option<&'a EStr<UserinfoE>> {
        self.inner.userinfo().map(EStr::cast)
    }

    /// Returns the [host] subcomponent as a string slice.
    ///
    /// The host subcomponent is always present, although it may be
    /// empty.
    ///
    /// The square brackets enclosing an IPv6 or `IPvFuture` address
    /// are included.
    ///
    /// Note that ASCII characters within a host are
    /// *case-insensitive*.
    ///
    /// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
    #[must_use]
    pub(crate) fn host(&self) -> &'a str {
        self.inner.host()
    }

    /// Returns the parsed [host] subcomponent.
    ///
    /// Note that ASCII characters within a host are
    /// *case-insensitive*.
    ///
    /// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
    #[must_use]
    pub(crate) fn host_parsed(&self) -> Host<'a, RegNameE> {
        match self.inner.meta.host_meta {
            HostMeta::Ipv4 => Host::Ipv4,
            HostMeta::Ipv6 => Host::Ipv6,
            HostMeta::IpvFuture => Host::IpvFuture,
            HostMeta::RegName => Host::RegName(EStr::new_validated(self.host())),
        }
    }

    /// Returns the optional [port] subcomponent.
    ///
    /// A scheme may define a default port to use when the port is
    /// not present or is empty.
    ///
    /// [port]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.3
    #[must_use]
    pub(crate) fn port(&self) -> Option<&'a EStr<Port>> {
        self.inner.port()
    }

    /// Converts the [port] subcomponent to `u16`, if present and
    /// nonempty.
    ///
    /// Returns `Ok(None)` if the port is not present or is empty.
    /// Leading zeros are ignored.
    ///
    /// [port]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.3
    ///
    /// # Errors
    ///
    /// Returns `Err` if the port cannot be parsed into `u16`.
    #[cfg(test)]
    pub(crate) fn port_to_u16(&self) -> Result<Option<u16>, ParseIntError> {
        self.inner.port_to_u16()
    }
}

impl<UserinfoE: Encoder, RegNameE: Encoder> core::fmt::Debug
    for Authority<'_, UserinfoE, RegNameE>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Authority")
            .field("userinfo", &self.userinfo())
            .field("host", &self.host())
            .field("host_parsed", &self.host_parsed())
            .field("port", &self.port())
            .finish()
    }
}

impl core::fmt::Display for Authority<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}

/// A parsed [host] component.
///
/// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
#[derive(Clone, Copy)]
pub(crate) enum Host<'a, RegNameE: Encoder = IRegName> {
    /// An IPv4 address.
    #[non_exhaustive]
    Ipv4,
    /// An IPv6 address.
    #[non_exhaustive]
    Ipv6,
    /// An IP address of future version.
    #[non_exhaustive]
    IpvFuture,
    /// A registered name.
    ///
    /// Note that ASCII characters within a registered name are
    /// *case-insensitive*.
    RegName(&'a EStr<RegNameE>),
}

impl<RegNameE: Encoder> core::fmt::Debug for Host<'_, RegNameE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Host::Ipv4 => f.debug_struct("Ipv4").finish_non_exhaustive(),
            Host::Ipv6 => f.debug_struct("Ipv6").finish_non_exhaustive(),
            Host::IpvFuture => f.debug_struct("IpvFuture").finish_non_exhaustive(),
            Host::RegName(name) => f.debug_tuple("RegName").field(name).finish(),
        }
    }
}
