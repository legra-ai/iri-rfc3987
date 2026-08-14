//! IRI normalization.

use alloc::string::String;
use core::fmt::{self, Write};
use core::num::NonZeroUsize;

use borrow_or_share::Bos;

use crate::imp::{HostMeta, Iri, Meta, RiMaybeRef as _, RmrRef};
use crate::pct_enc::encoder::IData;
use crate::pct_enc::{self, Decode, DecodedUtf8Chunk, Encode, EncodedChunk, Encoder};
use crate::{parse, resolve};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizeError {
    PathUnderflow,
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::PathUnderflow => "underflow occurred in path resolution",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for NormalizeError {}

#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
#[must_use]
pub(crate) struct Normalizer {
    allow_path_underflow: bool,
    default_port_f: fn(&str) -> Option<u16>,
}

impl Normalizer {
    pub(crate) const fn new() -> Self {
        Self {
            allow_path_underflow: true,
            default_port_f: |_| None,
        }
    }

    /// Sets the function with which to get the default port for a
    /// scheme.
    ///
    /// This defaults to `|_| None`.
    ///
    /// The scheme will be lowercased before being passed to the
    /// function.
    pub(crate) const fn default_port_with(mut self, f: fn(&str) -> Option<u16>) -> Self {
        self.default_port_f = f;
        self
    }

    /// Normalizes the given IRI.
    ///
    /// # Errors
    ///
    /// Returns `Err` if path normalization underflows.
    pub(crate) fn normalize<T: Bos<str>>(
        &self,
        iri: &Iri<T>,
    ) -> Result<Iri<String>, NormalizeError> {
        normalize(
            iri.make_ref(),
            self.allow_path_underflow,
            self.default_port_f,
        )
        .map(|(val, meta)| Iri { val, meta })
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn normalize(
    r: RmrRef<'_, '_>,
    allow_path_underflow: bool,
    default_port_f: fn(&str) -> Option<u16>,
) -> Result<(String, Meta), NormalizeError> {
    let mut buf = String::with_capacity(r.as_str().len());
    let mut meta = Meta::default();

    if let Some(scheme) = r.scheme_opt() {
        buf.push_str(scheme.as_str());
        buf.make_ascii_lowercase();
        meta.scheme_end = NonZeroUsize::new(buf.len());
        buf.push(':');
    }

    if let Some(auth) = r.authority() {
        buf.push_str("//");

        if let Some(userinfo) = auth.userinfo() {
            normalize_estr(&mut buf, userinfo.as_str(), false);
            buf.push('@');
        }

        let mut auth_meta = auth.meta();
        auth_meta.host_bounds.0 = buf.len();
        match auth_meta.host_meta {
            HostMeta::Ipv4 => buf.push_str(auth.host()),
            HostMeta::Ipv6 => {
                buf.push('[');
                write_v6(&mut buf, parse::parse_v6(&auth.host().as_bytes()[1..]));
                buf.push(']');
            }
            HostMeta::IpvFuture => {
                let start = buf.len();
                buf.push_str(auth.host());
                buf[start..].make_ascii_lowercase();
            }
            HostMeta::RegName => {
                let start = buf.len();
                let host = auth.host();
                normalize_estr(&mut buf, host, true);

                if buf.len() < start + host.len() {
                    auth_meta.host_meta = parse::parse_v4_or_reg_name(&buf.as_bytes()[start..]);
                }
            }
        }
        auth_meta.host_bounds.1 = buf.len();
        meta.auth_meta = Some(auth_meta);

        if let Some(port) = auth.port()
            && !port.is_empty()
        {
            let mut eq_default = false;
            if let Some(scheme_end) = meta.scheme_end
                && let Some(default) = default_port_f(&buf[..scheme_end.get()])
            {
                eq_default = port.as_str().parse().ok() == Some(default);
            }
            if !eq_default {
                buf.push(':');
                buf.push_str(port.as_str());
            }
        }
    }

    let path_start = buf.len();
    meta.path_bounds.0 = path_start;

    let path = r.path().as_str();

    if r.has_scheme() && path.starts_with('/') {
        let mut path_buf = String::with_capacity(path.len());
        normalize_estr(&mut path_buf, path, false);

        let underflow = resolve::remove_dot_segments(&mut buf, &path_buf, None);
        if underflow && !allow_path_underflow {
            return Err(NormalizeError::PathUnderflow);
        }

        if !r.has_authority() && buf[path_start..].starts_with("//") {
            buf.insert_str(path_start, "/.");
        }
    } else {
        normalize_estr(&mut buf, path, false);
    }

    meta.path_bounds.1 = buf.len();

    if let Some(query) = r.query() {
        buf.push('?');
        normalize_estr(&mut buf, query.as_str(), false);
        meta.query_end = NonZeroUsize::new(buf.len());
    }

    if let Some(fragment) = r.fragment() {
        buf.push('#');
        normalize_estr(&mut buf, fragment.as_str(), false);
    }

    Ok((buf, meta))
}

/// IRI normalization always uses UTF-8 path (no ASCII-only branch).
fn normalize_estr(buf: &mut String, s: &str, to_ascii_lowercase: bool) {
    Decode::new(s).decode_utf8(
        #[inline(always)]
        |chunk| match chunk {
            DecodedUtf8Chunk::Unencoded(s) => {
                let i = buf.len();
                buf.push_str(s);
                if to_ascii_lowercase {
                    buf[i..].make_ascii_lowercase();
                }
            }
            DecodedUtf8Chunk::Decoded { valid, invalid } => {
                for chunk in Encode::new(IData::TABLE, valid) {
                    match chunk {
                        EncodedChunk::Unencoded(s) => {
                            let i = buf.len();
                            buf.push_str(s);
                            if to_ascii_lowercase {
                                buf[i..].make_ascii_lowercase();
                            }
                        }
                        EncodedChunk::PctEncoded(s) => {
                            buf.push_str(s);
                        }
                    }
                }
                for &x in invalid {
                    buf.push_str(pct_enc::encode_byte(x));
                }
            }
        },
    );
}

/// Write an IPv6 address in canonical form.
fn write_v6(buf: &mut String, segments: [u16; 8]) {
    if let [0, 0, 0, 0, 0, 0xffff, ab, cd] = segments {
        let [a, b] = ab.to_be_bytes();
        let [c, d] = cd.to_be_bytes();
        write!(buf, "::ffff:{a}.{b}.{c}.{d}").unwrap();
    } else {
        #[derive(Copy, Clone, Default)]
        struct Span {
            start: usize,
            len: usize,
        }

        let zeroes = {
            let mut longest = Span::default();
            let mut current = Span::default();

            for (i, &segment) in segments.iter().enumerate() {
                if segment == 0 {
                    if current.len == 0 {
                        current.start = i;
                    }

                    current.len += 1;

                    if current.len > longest.len {
                        longest = current;
                    }
                } else {
                    current = Span::default();
                }
            }

            longest
        };

        #[inline]
        fn write_subslice(buf: &mut String, chunk: &[u16]) {
            if let Some((first, tail)) = chunk.split_first() {
                write!(buf, "{first:x}").unwrap();
                for segment in tail {
                    write!(buf, ":{segment:x}").unwrap();
                }
            }
        }

        if zeroes.len > 1 {
            write_subslice(buf, &segments[..zeroes.start]);
            buf.push_str("::");
            write_subslice(buf, &segments[zeroes.start + zeroes.len..]);
        } else {
            write_subslice(buf, &segments);
        }
    }
}
