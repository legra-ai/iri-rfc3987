//! Percent-decoding iterators ([`Decode`], [`DecodedChunk`]) and the
//! UTF-8 chunked decoder.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::FusedIterator;
use core::mem;

use super::hex::decode_octet;
use crate::utf8::{self, Utf8Chunks};

/// An iterator used to decode an [`EStr`] slice.
///
/// See the [`DecodedChunk`] type for documentation of the items
/// yielded by this iterator.
///
/// [`EStr`]: super::EStr
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Decode<'a> {
    source: &'a str,
}

/// An item returned by the [`Decode`] iterator.
#[derive(Clone, Copy, Debug)]
pub(crate) enum DecodedChunk<'a> {
    /// An unencoded subslice.
    Unencoded(&'a str),
    /// A percent-encoded octet, decoded (for example, `"%20"` decoded
    /// as `0x20`).
    PctDecoded(u8),
}

impl<'a> Decode<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn next_if_unencoded(&mut self) -> Option<&'a str> {
        let i = self
            .source
            .bytes()
            .position(|x| x == b'%')
            .unwrap_or(self.source.len());

        if i == 0 {
            None
        } else {
            let (s, rem) = self.source.split_at(i);
            self.source = rem;
            Some(s)
        }
    }
}

impl<'a> Iterator for Decode<'a> {
    type Item = DecodedChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            None
        } else if let Some(s) = self.next_if_unencoded() {
            Some(DecodedChunk::Unencoded(s))
        } else {
            let (s, rem) = self.source.split_at(3);
            self.source = rem;

            let x = decode_octet(s.as_bytes()[1], s.as_bytes()[2]);
            Some(DecodedChunk::PctDecoded(x))
        }
    }
}

impl FusedIterator for Decode<'_> {}

pub(crate) enum DecodedUtf8Chunk<'a, 'b> {
    Unencoded(&'a str),
    Decoded { valid: &'b str, invalid: &'b [u8] },
}

impl<'a> Decode<'a> {
    pub(crate) fn decode_utf8(self, mut handle_chunk: impl FnMut(DecodedUtf8Chunk<'a, '_>)) {
        const BUF_SIZE: usize = 32;

        let mut buf = [0; BUF_SIZE];
        let mut len = 0;

        for chunk in self {
            match chunk {
                DecodedChunk::Unencoded(s) => {
                    if len > 0 {
                        for chunk in Utf8Chunks::new(&buf[..len]) {
                            handle_chunk(DecodedUtf8Chunk::Decoded {
                                valid: chunk.valid(),
                                invalid: chunk.invalid(),
                            });
                        }
                        len = 0;
                    }
                    handle_chunk(DecodedUtf8Chunk::Unencoded(s));
                }
                DecodedChunk::PctDecoded(x) => {
                    buf[len] = x;
                    len += 1;

                    if len >= BUF_SIZE {
                        let mut split_at = BUF_SIZE - 1;
                        while split_at >= BUF_SIZE - 3 && !utf8::is_char_boundary(buf[split_at]) {
                            split_at -= 1;
                        }

                        if split_at < BUF_SIZE - 3 {
                            split_at = BUF_SIZE;
                        }

                        for chunk in Utf8Chunks::new(&buf[..split_at]) {
                            handle_chunk(DecodedUtf8Chunk::Decoded {
                                valid: chunk.valid(),
                                invalid: chunk.invalid(),
                            });
                        }

                        for i in split_at..BUF_SIZE {
                            buf[i - split_at] = buf[i];
                        }
                        len = BUF_SIZE - split_at;
                    }
                }
            }
        }

        for chunk in Utf8Chunks::new(&buf[..len]) {
            handle_chunk(DecodedUtf8Chunk::Decoded {
                valid: chunk.valid(),
                invalid: chunk.invalid(),
            });
        }
    }

    fn decoded_len(&self) -> usize {
        self.source.len() - self.source.bytes().filter(|&x| x == b'%').count() * 2
    }

    fn borrow_all_or_prep_buf(&mut self) -> Result<&'a str, String> {
        if let Some(s) = self.next_if_unencoded() {
            if self.source.is_empty() {
                return Ok(s);
            }
            let mut buf = String::with_capacity(s.len() + self.decoded_len());
            buf.push_str(s);
            Err(buf)
        } else {
            Err(String::with_capacity(self.decoded_len()))
        }
    }

    /// Attempts to decode the slice to a string.
    ///
    /// This method allocates only when the slice contains any
    /// percent-encoded octet.
    ///
    /// # Errors
    ///
    /// Returns `Err` containing the decoded bytes if they are not
    /// valid UTF-8.
    pub(crate) fn into_string(mut self) -> Result<Cow<'a, str>, Vec<u8>> {
        if self.source.is_empty() {
            return Ok(Cow::Borrowed(""));
        }

        let mut buf = match self.borrow_all_or_prep_buf() {
            Ok(s) => return Ok(Cow::Borrowed(s)),
            Err(buf) => Ok::<_, Vec<u8>>(buf),
        };

        self.decode_utf8(|chunk| match chunk {
            DecodedUtf8Chunk::Unencoded(s) => match &mut buf {
                Ok(string) => string.push_str(s),
                Err(vec) => vec.extend_from_slice(s.as_bytes()),
            },
            DecodedUtf8Chunk::Decoded { valid, invalid } => match &mut buf {
                Ok(string) => {
                    string.push_str(valid);
                    if !invalid.is_empty() {
                        let mut vec = mem::take(string).into_bytes();
                        vec.extend_from_slice(invalid);
                        buf = Err(vec);
                    }
                }
                Err(vec) => {
                    vec.extend_from_slice(valid.as_bytes());
                    vec.extend_from_slice(invalid);
                }
            },
        });

        match buf {
            Ok(buf) => Ok(Cow::Owned(buf)),
            Err(buf) => Err(buf),
        }
    }
}
