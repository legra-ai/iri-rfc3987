//! IP-literal, IPv4, and IPv6 parsing on the byte [`Reader`].

use crate::imp::HostMeta;
use crate::parse::common::{Result, err};
use crate::parse::reader::Reader;
use crate::pct_enc;
use crate::pct_enc::encoder::{Hexdig, IpvFuture};

enum Seg {
    Normal(u16, bool),
    Ellipsis,
    MaybeV4(bool),
    SingleColon,
}

impl Reader<'_> {
    pub(super) fn read_v6(&mut self) -> Option<[u16; 8]> {
        let mut segs = [0; 8];
        let mut ellipsis_idx = 8;

        let mut i = 0;
        while i < 8 {
            match self.read_v6_segment() {
                Some(Seg::Normal(seg, colon)) => {
                    if colon == (i == 0 || i == ellipsis_idx) {
                        return None;
                    }
                    segs[i] = seg;
                    i += 1;
                }
                Some(Seg::Ellipsis) => {
                    if ellipsis_idx != 8 {
                        return None;
                    }
                    ellipsis_idx = i;
                }
                Some(Seg::MaybeV4(colon)) => {
                    if i > 6 || colon == (i == ellipsis_idx) {
                        return None;
                    }
                    let octets = self.read_v4()?.to_be_bytes();
                    segs[i] = u16::from_be_bytes([octets[0], octets[1]]);
                    segs[i + 1] = u16::from_be_bytes([octets[2], octets[3]]);
                    i += 2;
                    break;
                }
                Some(Seg::SingleColon) => return None,
                None => break,
            }
        }

        if ellipsis_idx == 8 {
            if i != 8 {
                return None;
            }
        } else if i == 8 {
            return None;
        } else {
            for j in (ellipsis_idx..i).rev() {
                segs[8 - (i - j)] = segs[j];
                segs[j] = 0;
            }
        }

        Some(segs)
    }

    fn read_v6_segment(&mut self) -> Option<Seg> {
        let colon = self.read_str(":");
        if !self.has_remaining() {
            return colon.then_some(Seg::SingleColon);
        }

        let first = self.peek(0).unwrap();
        let mut x = match pct_enc::decode_hexdigit(first) {
            Some(v) => u16::from(v),
            _ => {
                return colon.then(|| {
                    if first == b':' {
                        self.skip(1);
                        Seg::Ellipsis
                    } else {
                        Seg::SingleColon
                    }
                });
            }
        };
        let mut i = 1;

        while i < 4 {
            let Some(b) = self.peek(i) else {
                self.skip(i);
                return None;
            };
            match pct_enc::decode_hexdigit(b) {
                Some(v) => {
                    x = (x << 4) | u16::from(v);
                    i += 1;
                }
                _ if b == b'.' => return Some(Seg::MaybeV4(colon)),
                _ => break,
            }
        }
        self.skip(i);
        Some(Seg::Normal(x, colon))
    }

    pub(super) fn read_v4(&mut self) -> Option<u32> {
        let mut addr = self.read_v4_octet()? << 24;
        for i in (0..3).rev() {
            if !self.read_str(".") {
                return None;
            }
            addr |= self.read_v4_octet()? << (i * 8);
        }
        Some(addr)
    }

    fn read_v4_octet(&mut self) -> Option<u32> {
        let mut res = self.peek_digit(0)?;
        if res == 0 {
            self.skip(1);
            return Some(0);
        }

        for i in 1..3 {
            let Some(x) = self.peek_digit(i) else {
                self.skip(i);
                return Some(res);
            };
            res = res * 10 + x;
        }
        self.skip(3);

        u8::try_from(res).is_ok().then_some(res)
    }

    pub(super) fn peek_digit(&self, i: usize) -> Option<u32> {
        self.peek(i).and_then(|x| (x as char).to_digit(10))
    }

    pub(super) fn read_port(&mut self) {
        if self.read_str(":") {
            let mut i = 0;
            while self.peek_digit(i).is_some() {
                i += 1;
            }
            self.skip(i);
        }
    }

    pub(super) fn read_ip_literal(&mut self) -> Result<Option<HostMeta>> {
        if !self.read_str("[") {
            return Ok(None);
        }

        let start = self.pos;

        let meta = if self.read_v6().is_some() {
            HostMeta::Ipv6
        } else if self.pos == start {
            self.read_ipv_future()?;
            HostMeta::IpvFuture
        } else {
            err!(start, InvalidIpv6Addr);
        };

        if !self.read_str("]") {
            err!(self.pos, UnexpectedCharOrEnd);
        }
        Ok(Some(meta))
    }

    fn read_ipv_future(&mut self) -> Result<()> {
        if let Some(b'v' | b'V') = self.peek(0) {
            self.skip(1);
            if self.read::<Hexdig>()? && self.read_str(".") && self.read::<IpvFuture>()? {
                return Ok(());
            }
        }
        err!(self.pos, UnexpectedCharOrEnd);
    }
}

#[cfg(test)]
pub(crate) fn parse_v4_or_reg_name(bytes: &[u8]) -> HostMeta {
    let mut reader = Reader::new(bytes);
    match reader.read_v4() {
        Some(_addr) if !reader.has_remaining() => HostMeta::Ipv4,
        _ => HostMeta::RegName,
    }
}

#[cfg(test)]
pub(crate) fn parse_v6(bytes: &[u8]) -> [u16; 8] {
    Reader::new(bytes).read_v6().unwrap()
}
