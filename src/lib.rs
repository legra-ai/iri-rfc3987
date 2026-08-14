//! Internationalized Resource Identifier handling for RFC 3987 and RFC 3986.
#![forbid(unsafe_code)]
// Vendored IRI parser code uses `i` as idiomatic index variables.
#![allow(clippy::disallowed_names)]
// Vendored IRI parser/normalizer has deeply nested parsing logic.
#![allow(clippy::excessive_nesting)]
// The implementation retains performance-oriented and compact parser idioms
// from fluent-uri where changing them would obscure the RFC algorithm.
#![allow(clippy::inline_always)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]

extern crate alloc;

mod component;
mod convert;
mod error;
mod imp;
#[cfg(test)]
mod normalize;
mod parse;
mod pct_enc;
mod resolve;
mod utf8;

pub use convert::TryIntoIri;
pub use error::IriParseError;
pub use imp::{Iri, IriRef};
pub use resolve::ResolveError;

#[cfg(test)]
mod tests;
