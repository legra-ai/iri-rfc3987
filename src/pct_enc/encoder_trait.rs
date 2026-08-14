//! The [`Encoder`] trait.

use super::table::Table;

/// A trait used by [`EStr`] to specify the table used for encoding.
///
/// [`EStr`]: super::EStr
pub(crate) trait Encoder: 'static {
    /// The table used for encoding.
    const TABLE: Table;
}
