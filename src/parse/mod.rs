//! IRI recursive-descent parser.
//!
//! Always uses IRI encoders (no ASCII-only branch).

mod common;
mod ip;
mod parser;
mod reader;

#[cfg(test)]
pub(crate) use ip::{parse_v4_or_reg_name, parse_v6};
pub(crate) use parser::parse;
