//! Delayed userinfo/host disambiguation without retaining authority text.

use super::grammar::allows;
use crate::IriRef;
use crate::pct_enc::encoder::{IRegName, IUserinfo, IpvFuture};

#[derive(Debug)]
pub(super) struct Authority {
    userinfo: bool,
    seen_at: bool,
    host: Host,
}

impl Default for Authority {
    fn default() -> Self {
        Self {
            userinfo: true,
            seen_at: false,
            host: Host::default(),
        }
    }
}

impl Authority {
    pub(super) fn atom(&mut self, scalar: char, encoded: bool) -> bool {
        if !encoded && scalar == '@' {
            if self.seen_at || !self.userinfo {
                return false;
            }
            self.seen_at = true;
            self.host = Host::default();
            return true;
        }
        self.userinfo &= allows::<IUserinfo>(scalar, encoded);
        self.host.atom(scalar, encoded);
        self.host.valid || (!self.seen_at && self.userinfo)
    }

    pub(super) fn finish(&self) -> bool {
        self.host.valid
            && matches!(
                self.host.phase,
                HostPhase::Name | HostPhase::Port | HostPhase::AfterIp
            )
    }
}

#[derive(Debug, Default)]
enum HostPhase {
    #[default]
    Name,
    Port,
    IpStart,
    Ipv6,
    FutureVersion {
        nonempty: bool,
    },
    FutureAddress {
        nonempty: bool,
    },
    AfterIp,
}

#[derive(Debug)]
struct Host {
    phase: HostPhase,
    valid: bool,
    empty: bool,
    // bounded: an IPv6 address has at most 45 ASCII bytes plus brackets;
    // two leading slashes let the existing reference parser validate it.
    ip: [u8; 49],
    ip_len: usize,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            phase: HostPhase::Name,
            valid: true,
            empty: true,
            ip: [0; 49],
            ip_len: 0,
        }
    }
}

impl Host {
    fn atom(&mut self, scalar: char, encoded: bool) {
        if !self.valid {
            return;
        }
        self.valid = match &mut self.phase {
            HostPhase::Name => {
                if !encoded && scalar == '[' && self.empty {
                    self.ip[..3].copy_from_slice(b"//[");
                    self.ip_len = 3;
                    self.phase = HostPhase::IpStart;
                    true
                } else if !encoded && scalar == ':' {
                    self.phase = HostPhase::Port;
                    true
                } else {
                    allows::<IRegName>(scalar, encoded)
                }
            }
            HostPhase::Port => !encoded && scalar.is_ascii_digit(),
            HostPhase::IpStart => {
                if !encoded && matches!(scalar, 'v' | 'V') {
                    self.phase = HostPhase::FutureVersion { nonempty: false };
                    true
                } else {
                    self.phase = HostPhase::Ipv6;
                    self.ipv6(scalar, encoded)
                }
            }
            HostPhase::Ipv6 => self.ipv6(scalar, encoded),
            HostPhase::FutureVersion { nonempty } => {
                if !encoded && scalar == '.' && *nonempty {
                    self.phase = HostPhase::FutureAddress { nonempty: false };
                    true
                } else if !encoded && scalar.is_ascii_hexdigit() {
                    *nonempty = true;
                    true
                } else {
                    false
                }
            }
            HostPhase::FutureAddress { nonempty } => {
                if !encoded && scalar == ']' && *nonempty {
                    self.phase = HostPhase::AfterIp;
                    true
                } else {
                    *nonempty = true;
                    allows::<IpvFuture>(scalar, encoded)
                }
            }
            HostPhase::AfterIp => {
                if !encoded && scalar == ':' {
                    self.phase = HostPhase::Port;
                    true
                } else {
                    false
                }
            }
        };
        self.empty = false;
    }

    fn ipv6(&mut self, scalar: char, encoded: bool) -> bool {
        if encoded || !scalar.is_ascii() || self.ip_len == self.ip.len() {
            return false;
        }
        self.ip[self.ip_len] = scalar as u8;
        self.ip_len += 1;
        if scalar == ']' {
            self.phase = HostPhase::AfterIp;
            let text = core::str::from_utf8(&self.ip[..self.ip_len]).expect("ASCII IP literal");
            IriRef::parse(text).is_ok()
        } else {
            scalar.is_ascii_hexdigit() || matches!(scalar, ':' | '.')
        }
    }
}
