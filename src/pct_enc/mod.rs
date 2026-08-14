//! Percent-encoding utilities.

pub(crate) mod encoder;

#[cfg(test)]
mod decode;
#[cfg(test)]
mod encode;
mod encoder_trait;
mod estr;
mod hex;
mod path;
#[cfg(test)]
mod split;
pub(crate) mod table;

#[cfg(test)]
pub(crate) use decode::{Decode, DecodedUtf8Chunk};
#[cfg(test)]
pub(crate) use encode::{Encode, EncodedChunk, encode_byte};
pub(crate) use encoder_trait::Encoder;
pub(crate) use estr::EStr;
pub(crate) use hex::{decode_hexdigit, is_hexdig, is_hexdig_pair};
pub(crate) use table::Table;
