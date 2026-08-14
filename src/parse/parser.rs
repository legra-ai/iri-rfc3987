//! The recursive-descent [`Parser`] driving
//! scheme/authority/path/query/fragment.

use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};

use crate::imp::{AuthMeta, Constraints, HostMeta, Meta};
use crate::parse::common::{Result, err};
use crate::parse::reader::Reader;
use crate::pct_enc::encoder::{
    IFragment, IPath, IQuery, IRegName, ISegmentNzNc, IUserinfo, Scheme,
};

pub(crate) fn parse(bytes: &[u8], constraints: Constraints) -> Result<Meta> {
    let mut parser = Parser {
        constraints,
        reader: Reader::new(bytes),
        out: Meta::default(),
    };
    parser.parse_from_scheme()?;
    Ok(parser.out)
}

struct Parser<'a> {
    constraints: Constraints,
    reader: Reader<'a>,
    out: Meta,
}

impl<'a> Deref for Parser<'a> {
    type Target = Reader<'a>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl DerefMut for Parser<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PathKind {
    General,
    AbEmpty,
    ContinuedNoScheme,
}

impl Parser<'_> {
    fn read_v4_or_reg_name(&mut self) -> Result<HostMeta> {
        Ok(match (self.read_v4(), self.read::<IRegName>()?) {
            (Some(_addr), false) => HostMeta::Ipv4,
            _ => HostMeta::RegName,
        })
    }

    fn read_host(&mut self) -> Result<HostMeta> {
        match self.read_ip_literal()? {
            Some(host) => Ok(host),
            None => self.read_v4_or_reg_name(),
        }
    }

    fn parse_from_scheme(&mut self) -> Result<()> {
        self.read::<Scheme>()?;

        if self.peek(0) == Some(b':') {
            if self.pos > 0 && self.bytes[0].is_ascii_alphabetic() {
                self.out.scheme_end = NonZeroUsize::new(self.pos);
            } else {
                err!(0, UnexpectedCharOrEnd);
            }

            self.skip(1);
            return if self.read_str("//") {
                self.parse_from_authority()
            } else {
                self.parse_from_path(PathKind::General)
            };
        } else if self.constraints.scheme_required {
            err!(self.pos, UnexpectedCharOrEnd);
        } else if self.pos == 0 && self.read_str("//") {
            return self.parse_from_authority();
        }
        self.parse_from_path(PathKind::ContinuedNoScheme)
    }

    fn parse_from_authority(&mut self) -> Result<()> {
        let host_start = self.pos;
        let host_meta = self.read_host()?;

        let mut auth_meta = AuthMeta {
            host_bounds: (host_start, self.pos),
            host_meta,
        };

        self.read_port();

        if let HostMeta::Ipv4 | HostMeta::RegName = host_meta {
            let userinfo_read = self.read::<IUserinfo>()?;

            if self.peek(0) == Some(b'@') {
                self.skip(1);

                let host_start = self.pos;
                let host_meta = self.read_host()?;

                auth_meta = AuthMeta {
                    host_bounds: (host_start, self.pos),
                    host_meta,
                };

                self.read_port();
            } else if userinfo_read {
                err!(self.pos, UnexpectedCharOrEnd);
            }
        }

        self.out.auth_meta = Some(auth_meta);
        self.parse_from_path(PathKind::AbEmpty)
    }

    fn parse_from_path(&mut self, kind: PathKind) -> Result<()> {
        let path_start;

        match kind {
            PathKind::General | PathKind::AbEmpty => {
                path_start = self.pos;
            }
            PathKind::ContinuedNoScheme => {
                path_start = 0;
                self.read::<ISegmentNzNc>()?;

                if self.peek(0) == Some(b':') {
                    err!(self.pos, UnexpectedCharOrEnd);
                }
            }
        }

        if self.read::<IPath>()? && kind == PathKind::AbEmpty && self.bytes[path_start] != b'/' {
            err!(path_start, UnexpectedCharOrEnd);
        }

        self.out.path_bounds = (path_start, self.pos);

        if self.read_str("?") {
            self.read::<IQuery>()?;
            self.out.query_end = NonZeroUsize::new(self.pos);
        }

        if self.read_str("#") {
            self.read::<IFragment>()?;
        }

        if self.has_remaining() {
            err!(self.pos, UnexpectedCharOrEnd);
        }
        Ok(())
    }
}
