//! Percent-encoding iterators and the `encode_byte` helper.

use core::iter::FusedIterator;
use core::str;

use super::table::Table;

pub(crate) fn encode_byte(x: u8) -> &'static str {
    const TABLE: &[u8; 256 * 3] = &{
        const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

        let mut i = 0;
        let mut table = [0; 256 * 3];
        while i < 256 {
            table[i * 3] = b'%';
            table[i * 3 + 1] = HEX_DIGITS[i >> 4];
            table[i * 3 + 2] = HEX_DIGITS[i & 0b1111];
            i += 1;
        }
        table
    };

    const TABLE_STR: &str = match str::from_utf8(TABLE) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    };

    &TABLE_STR[x as usize * 3..x as usize * 3 + 3]
}

/// An iterator used to percent-encode a string slice.
///
/// See the [`EncodedChunk`] type for documentation of the items
/// yielded by this iterator.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Encode<'s> {
    table: Table,
    source: &'s str,
    to_enc: &'s [u8],
}

impl<'s> Encode<'s> {
    pub(crate) fn new(table: Table, source: &'s str) -> Self {
        Self {
            table,
            source,
            to_enc: &[],
        }
    }
}

/// An item returned by the [`Encode`] iterator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EncodedChunk<'a> {
    /// An unencoded subslice.
    Unencoded(&'a str),
    /// A byte, percent-encoded.
    PctEncoded(&'static str),
}

impl<'a> Iterator for Encode<'a> {
    type Item = EncodedChunk<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let [x, rem @ ..] = self.to_enc {
            self.to_enc = rem;
            return Some(EncodedChunk::PctEncoded(encode_byte(*x)));
        }

        if self.source.is_empty() {
            return None;
        }

        let mut iter = self.source.char_indices();

        let first_disallowed_idx = iter
            .find_map(|(i, ch)| (!self.table.allows(ch)).then_some(i))
            .unwrap_or(self.source.len());

        let next_allowed_idx = iter
            .find_map(|(i, ch)| self.table.allows(ch).then_some(i))
            .unwrap_or(self.source.len());

        if first_disallowed_idx == 0 {
            let (disallowed, rem) = self.source.split_at(next_allowed_idx);
            self.source = rem;

            let (x, rem) = disallowed.as_bytes().split_first().unwrap();
            self.to_enc = rem;

            Some(EncodedChunk::PctEncoded(encode_byte(*x)))
        } else {
            let allowed = &self.source[..first_disallowed_idx];
            self.to_enc = &self.source.as_bytes()[first_disallowed_idx..next_allowed_idx];
            self.source = &self.source[next_allowed_idx..];

            Some(EncodedChunk::Unencoded(allowed))
        }
    }
}

impl FusedIterator for Encode<'_> {}
