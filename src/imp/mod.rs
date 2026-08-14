//! Core IRI types: `Iri<T>` and `IriRef<T>`.
//!
//! Hand-written from fluent-uri v0.4.1 (MIT), IRI-only (no URI
//! types).

#![allow(missing_debug_implementations)]

mod conversions;
mod iri;
mod iri_ref;
mod parts;

pub use iri::Iri;
pub use iri_ref::IriRef;
#[cfg(test)]
pub(crate) use parts::RiMaybeRef;
pub(crate) use parts::{AuthMeta, Constraints, HostMeta, Meta, RmrRef};
