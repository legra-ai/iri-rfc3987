//! Byte cursor with low-level reading primitives over the input bytes.

use crate::parse::common::{Result, err};
use crate::pct_enc::{self, Encoder};
use crate::utf8;

pub(super) struct Reader<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Reader { bytes, pos: 0 }
    }

    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn has_remaining(&self) -> bool {
        self.pos < self.len()
    }

    pub(super) fn peek(&self, i: usize) -> Option<u8> {
        self.bytes.get(self.pos + i).copied()
    }

    pub(super) fn skip(&mut self, n: usize) {
        self.pos += n;
        debug_assert!(self.pos <= self.len());
    }

    #[cold]
    pub(super) fn invalid_pct(&self) -> Result<bool> {
        let mut i = self.pos + 1;
        if let Some(&x) = self.bytes.get(i)
            && pct_enc::is_hexdig(x)
        {
            i += 1;
        }
        err!(i, UnexpectedCharOrEnd);
    }

    #[inline(always)]
    pub(super) fn read<E: Encoder>(&mut self) -> Result<bool> {
        let start = self.pos;
        let mut i = self.pos;

        while i < self.len() {
            let x = self.bytes[i];
            if E::TABLE.allows_pct_encoded() && x == b'%' {
                let [hi, lo, ..] = self.bytes[i + 1..] else {
                    return self.invalid_pct();
                };
                if !pct_enc::is_hexdig_pair(hi, lo) {
                    return self.invalid_pct();
                }
                i += 3;
            } else if E::TABLE.allows_non_ascii() {
                let (x, len) = utf8::next_code_point(self.bytes, i);
                if !E::TABLE.allows_code_point(x) {
                    break;
                }
                i += len;
            } else {
                if !E::TABLE.allows_ascii(x) {
                    break;
                }
                i += 1;
            }
        }

        self.pos = i;
        Ok(self.pos > start)
    }

    pub(super) fn read_str(&mut self, s: &str) -> bool {
        if self.bytes[self.pos..].starts_with(s.as_bytes()) {
            self.skip(s.len());
            true
        } else {
            false
        }
    }
}
