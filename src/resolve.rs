//! IRI reference resolution.

use alloc::string::String;
use core::num::NonZeroUsize;
use core::{fmt, iter};

#[cfg(test)]
use borrow_or_share::Bos;

#[cfg(test)]
use crate::imp::{Iri, IriRef, RiMaybeRef as _};
use crate::imp::{Meta, RmrRef};

/// An error occurred while resolving an IRI reference against a base IRI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolveError {
    /// The base IRI contains a fragment component.
    BaseWithFragment,
    /// The reference cannot be resolved against an opaque base IRI.
    InvalidReferenceAgainstOpaqueBase,
    /// Path resolution attempted to traverse above the root.
    PathUnderflow,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::BaseWithFragment => "base should not have fragment",
            Self::InvalidReferenceAgainstOpaqueBase => {
                "when base has a rootless path and no authority, \
                 reference should either have scheme, be empty or \
                 start with '#'"
            }
            Self::PathUnderflow => "underflow occurred in path resolution",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ResolveError {}

#[derive(Clone, Copy)]
#[must_use]
#[cfg(test)]
pub(crate) struct Resolver<T> {
    base: Iri<T>,
    allow_path_underflow: bool,
}

#[cfg(test)]
impl<T: Bos<str>> fmt::Debug for Resolver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resolver")
            .field("base", &self.base)
            .field("allow_path_underflow", &self.allow_path_underflow)
            .finish()
    }
}

#[cfg(test)]
impl<T: Bos<str>> Resolver<T> {
    pub(crate) fn with_base(base: Iri<T>) -> Self {
        Self {
            base,
            allow_path_underflow: true,
        }
    }

    /// Resolves the given reference against the configured base.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the base/reference pair is invalid or path
    /// resolution underflows.
    pub(crate) fn resolve<U: Bos<str>>(
        &self,
        reference: &IriRef<U>,
    ) -> Result<Iri<String>, ResolveError> {
        resolve(
            self.base.make_ref(),
            reference.make_ref(),
            self.allow_path_underflow,
        )
        .map(|(val, meta)| Iri { val, meta })
    }
}

pub(crate) fn resolve(
    base: RmrRef<'_, '_>,
    r: RmrRef<'_, '_>,
    allow_path_underflow: bool,
) -> Result<(String, Meta), ResolveError> {
    assert!(base.has_scheme());

    if base.has_fragment() {
        return Err(ResolveError::BaseWithFragment);
    }
    if !base.has_authority()
        && base.path().is_rootless()
        && !r.has_scheme()
        && !matches!(r.as_str().bytes().next(), None | Some(b'#'))
    {
        return Err(ResolveError::InvalidReferenceAgainstOpaqueBase);
    }

    let (t_scheme, t_authority, t_path, t_query, t_fragment);

    let r_scheme = r.scheme_opt();
    let r_authority = r.authority();
    let r_path = r.path();
    let r_query = r.query();
    let r_fragment = r.fragment();

    if let Some(r_scheme) = r_scheme {
        t_scheme = r_scheme;
        t_authority = r_authority;
        t_path = (r_path.as_str(), None);
        t_query = r_query;
    } else {
        if r_authority.is_some() {
            t_authority = r_authority;
            t_path = (r_path.as_str(), None);
            t_query = r_query;
        } else {
            if r_path.is_empty() {
                t_path = (base.path().as_str(), None);
                if r_query.is_some() {
                    t_query = r_query;
                } else {
                    t_query = base.query();
                }
            } else {
                if r_path.is_absolute() {
                    t_path = (r_path.as_str(), None);
                } else {
                    let base_path = base.path();
                    let base_path = if base_path.is_empty() {
                        "/"
                    } else {
                        base_path.as_str()
                    };

                    let last_slash_idx = base_path.bytes().rposition(|b| b == b'/').unwrap();
                    let last_seg = &base_path.as_bytes()[last_slash_idx + 1..];
                    let base_path_stripped = match classify_segment(last_seg) {
                        SegKind::DoubleDot => base_path,
                        _ => &base_path[..=last_slash_idx],
                    };

                    t_path = (base_path_stripped, Some(r_path.as_str()));
                }
                t_query = r_query;
            }
            t_authority = base.authority();
        }
        t_scheme = base.scheme();
    }
    t_fragment = r_fragment;

    // Calculate the output length.
    let mut len = t_scheme.as_str().len() + 1;
    if let Some(authority) = t_authority {
        len += authority.as_str().len() + 2;
    }
    len += t_path.0.len() + t_path.1.map_or(0, str::len);
    if let Some(query) = t_query {
        len += query.len() + 1;
    }
    if let Some(fragment) = t_fragment {
        len += fragment.len() + 1;
    }

    let mut buf = String::with_capacity(len);
    let mut meta = Meta::default();

    buf.push_str(t_scheme.as_str());
    meta.scheme_end = NonZeroUsize::new(buf.len());
    buf.push(':');

    if let Some(authority) = t_authority {
        let mut auth_meta = authority.meta();
        buf.push_str("//");

        auth_meta.host_bounds.0 += buf.len();
        auth_meta.host_bounds.1 += buf.len();

        buf.push_str(authority.as_str());
        meta.auth_meta = Some(auth_meta);
    }

    let path_start = buf.len();
    meta.path_bounds.0 = path_start;

    if t_path.0.starts_with('/') {
        let underflow = remove_dot_segments(&mut buf, t_path.0, t_path.1);
        if underflow && !allow_path_underflow {
            return Err(ResolveError::PathUnderflow);
        }
    } else {
        buf.push_str(t_path.0);
    }

    // Close the loophole in the original algorithm.
    if t_authority.is_none() && buf[path_start..].starts_with("//") {
        buf.insert_str(path_start, "/.");
    }

    meta.path_bounds.1 = buf.len();

    if let Some(query) = t_query {
        buf.push('?');
        buf.push_str(query.as_str());
        meta.query_end = NonZeroUsize::new(buf.len());
    }

    if let Some(fragment) = t_fragment {
        buf.push('#');
        buf.push_str(fragment.as_str());
    }

    debug_assert!(buf.len() <= len);

    Ok((buf, meta))
}

pub(crate) fn remove_dot_segments(buf: &mut String, abs: &str, rel: Option<&str>) -> bool {
    debug_assert!(abs.starts_with('/'));

    let min_len = buf.len() + 1;
    let mut underflow = false;

    for part in iter::once(abs).chain(rel) {
        let bytes = part.as_bytes();
        let len = bytes.len();

        let mut start = 0;
        while start < len {
            let mut end = start;
            while end < len && bytes[end] != b'/' {
                end += 1;
            }
            let seg = &bytes[start..end];

            match classify_segment(seg) {
                SegKind::Dot => {}
                SegKind::DoubleDot => {
                    if buf.len() <= min_len {
                        underflow = true;
                    } else {
                        let prev_slash_idx = buf.as_bytes()[..buf.len() - 1]
                            .iter()
                            .rposition(|&b| b == b'/')
                            .unwrap();
                        buf.truncate(prev_slash_idx + 1);
                    }
                }
                SegKind::Normal => {
                    buf.push_str(&part[start..len.min(end + 1)]);
                }
            }

            if end == len {
                break;
            }
            start = end + 1;
        }
    }
    underflow
}

enum SegKind {
    Dot,
    DoubleDot,
    Normal,
}

fn classify_segment(s: &[u8]) -> SegKind {
    fn is_pct2e(s: &[u8]) -> bool {
        &s[..2] == b"%2" && (s[2] | 0x20) == b'e'
    }

    match s.len() {
        1 if s == b"." => SegKind::Dot,
        2 if s == b".." => SegKind::DoubleDot,
        3 if is_pct2e(s) => SegKind::Dot,
        4 if (s[0] == b'.' && is_pct2e(&s[1..])) || (s[3] == b'.' && is_pct2e(&s[..3])) => {
            SegKind::DoubleDot
        }
        6 if is_pct2e(&s[..3]) && is_pct2e(&s[3..]) => SegKind::DoubleDot,
        _ => SegKind::Normal,
    }
}
