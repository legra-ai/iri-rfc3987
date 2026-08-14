//! UTF-8 utilities for IRI non-ASCII character handling.
//!
//! Adapted from `core::str` (Rust 1.81) and fluent-uri v0.4.1 (MIT).

#[cfg(test)]
use core::str;

const CONT_MASK: u8 = 0b0011_1111;

#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

/// Decodes the next UTF-8 code point starting at `bytes[i]`.
///
/// Returns `(code_point, byte_length)`.
// Make sure it's inlined into `Parser::read`.
// This improves performance significantly for non-ASCII case.
#[inline(always)]
pub(crate) const fn next_code_point(bytes: &[u8], i: usize) -> (u32, usize) {
    let x = bytes[i];
    if x < 128 {
        return (x as u32, 1);
    }

    let init = utf8_first_byte(x, 2);
    let y = bytes[i + 1];
    if x < 0xE0 {
        (utf8_acc_cont_byte(init, y), 2)
    } else {
        let z = bytes[i + 2];
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        if x < 0xF0 {
            ((init << 12) | y_z, 3)
        } else {
            let w = bytes[i + 3];
            (((init & 7) << 18) | utf8_acc_cont_byte(y_z, w), 4)
        }
    }
}

/// Returns `true` if `b` is the start of a UTF-8 character boundary.
#[cfg(test)]
pub(crate) const fn is_char_boundary(b: u8) -> bool {
    // This is bit magic equivalent to: b < 128 || b >= 192
    (b as i8) >= -0x40
}

/// A chunk of UTF-8 data with a valid prefix and an invalid suffix.
#[cfg(test)]
pub(crate) struct Utf8Chunk<'a> {
    valid: &'a str,
    invalid: &'a [u8],
}

#[cfg(test)]
impl<'a> Utf8Chunk<'a> {
    /// Returns the valid UTF-8 prefix.
    pub(crate) fn valid(&self) -> &'a str {
        self.valid
    }

    /// Returns the invalid byte suffix.
    pub(crate) fn invalid(&self) -> &'a [u8] {
        self.invalid
    }
}

/// An iterator that yields [`Utf8Chunk`]s from a byte slice.
#[cfg(test)]
pub(crate) struct Utf8Chunks<'a> {
    source: &'a [u8],
}

#[cfg(test)]
impl<'a> Utf8Chunks<'a> {
    /// Creates a new iterator over the given bytes.
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { source: bytes }
    }
}

#[cfg(test)]
impl<'a> Iterator for Utf8Chunks<'a> {
    type Item = Utf8Chunk<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Utf8Chunk<'a>> {
        if self.source.is_empty() {
            return None;
        }

        match str::from_utf8(self.source) {
            Ok(valid) => {
                self.source = &[];

                Some(Utf8Chunk {
                    valid,
                    invalid: &[],
                })
            }
            Err(e) => {
                let (valid, after_valid) = self.source.split_at(e.valid_up_to());

                let (invalid, rem) = if let Some(len) = e.error_len() {
                    let (invalid, rem) = after_valid.split_at(len);
                    (invalid, rem)
                } else {
                    (after_valid, &[][..])
                };
                self.source = rem;

                Some(Utf8Chunk {
                    valid: str::from_utf8(valid).unwrap(),
                    invalid,
                })
            }
        }
    }
}
