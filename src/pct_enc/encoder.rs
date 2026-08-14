#![allow(missing_debug_implementations)]

//! Percent-encoders for IRI components.

use super::table::*;
use super::{Encoder, Table};

/// An encoder for IRI userinfo.
#[derive(Clone, Copy)]
pub(crate) struct IUserinfo(());

impl Encoder for IUserinfo {
    const TABLE: Table = IUSERINFO;
}

/// An encoder for IRI registered name.
#[derive(Clone, Copy)]
pub(crate) struct IRegName(());

impl Encoder for IRegName {
    const TABLE: Table = IREG_NAME;
}

/// An encoder for IRI/URI port.
#[derive(Clone, Copy)]
pub(crate) struct Port(());

impl Encoder for Port {
    const TABLE: Table = DIGIT;
}

/// An encoder for IRI path.
///
/// `EStr` has [extension methods] for the path component.
///
/// [extension methods]: super::EStr#impl-EStr<E>-1
#[derive(Clone, Copy)]
pub(crate) struct IPath(());

impl Encoder for IPath {
    const TABLE: Table = IPATH;
}

/// An encoder for IRI query.
#[derive(Clone, Copy)]
pub(crate) struct IQuery(());

impl Encoder for IQuery {
    const TABLE: Table = IQUERY;
}

/// An encoder for IRI fragment.
#[derive(Clone, Copy)]
pub(crate) struct IFragment(());

impl Encoder for IFragment {
    const TABLE: Table = IFRAGMENT;
}

#[derive(Clone, Copy)]
#[cfg(test)]
pub(crate) struct IData(());

#[cfg(test)]
impl Encoder for IData {
    const TABLE: Table = UNRESERVED.or_pct_encoded().or_ucschar();
}

// The following are used only in the parser.

pub(crate) struct Hexdig;

impl Encoder for Hexdig {
    const TABLE: Table = HEXDIG;
}

pub(crate) struct IpvFuture;

impl Encoder for IpvFuture {
    const TABLE: Table = IPV_FUTURE;
}

pub(crate) struct Scheme;

impl Encoder for Scheme {
    const TABLE: Table = SCHEME;
}

pub(crate) struct ISegmentNzNc;

impl Encoder for ISegmentNzNc {
    const TABLE: Table = ISEGMENT_NZ_NC;
}
