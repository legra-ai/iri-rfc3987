//! The [`EStr`] type — percent-encoded string slices.

use core::cmp::Ordering;
use core::marker::PhantomData;
use core::{hash, str};

use ref_cast::{RefCastCustom, ref_cast_custom};

#[cfg(test)]
use super::decode::Decode;
use super::encoder_trait::Encoder;
#[cfg(test)]
use super::split::Split;
#[cfg(test)]
use super::table;

/// Percent-encoded string slices.
///
/// # Type parameter
///
/// The `EStr<E>` type is parameterized over a type `E` that
/// implements [`Encoder`]. The associated constant `E::TABLE` of type
/// [`Table`] specifies the byte patterns allowed in a string.
///
/// [`Table`]: super::Table
///
/// # Comparison
///
/// `EStr` slices are compared
/// [lexicographically](Ord#lexicographical-comparison) by their byte
/// values. Normalization is **not** performed prior to comparison.
#[derive(RefCastCustom)]
#[repr(transparent)]
pub(crate) struct EStr<E: Encoder> {
    encoder: PhantomData<E>,
    inner: str,
}

impl<E: Encoder> EStr<E> {
    #[cfg(test)]
    pub(crate) const ASSERT_ALLOWS_PCT_ENCODED: () = assert!(
        E::TABLE.allows_pct_encoded(),
        "table does not allow percent-encoded octets"
    );

    /// Converts a string slice to an `EStr` slice assuming validity.
    #[ref_cast_custom]
    pub(crate) const fn new_validated(s: &str) -> &Self;

    /// An empty `EStr` slice.
    pub(crate) const EMPTY: &'static Self = Self::new_validated("");

    pub(crate) fn cast<F: Encoder>(&self) -> &EStr<F> {
        EStr::new_validated(&self.inner)
    }

    /// Converts a string slice to an `EStr` slice.
    ///
    /// # Panics
    ///
    /// Panics if the string is not properly encoded with `E`.
    /// For a non-panicking variant, use [`new`](Self::new).
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new(s: &str) -> Option<&Self> {
        if E::TABLE.validate(s.as_bytes()) {
            Some(Self::new_validated(s))
        } else {
            None
        }
    }

    /// Yields the underlying string slice.
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        &self.inner
    }

    /// Returns the length of the `EStr` slice in bytes.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    /// Checks whether the `EStr` slice is empty.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator used to decode the `EStr` slice.
    ///
    /// Always **split before decoding**, as otherwise the data may be
    /// mistaken for component delimiters.
    ///
    /// Note that the iterator will **not** decode `U+002B` (+) as
    /// `0x20` (space).
    ///
    /// # Panics
    ///
    /// Panics at compile time if `E::TABLE` does not
    /// [allow percent-encoded octets].
    ///
    /// [allow percent-encoded octets]: super::Table::allows_pct_encoded
    #[cfg(test)]
    pub(crate) fn decode(&self) -> Decode<'_> {
        () = Self::ASSERT_ALLOWS_PCT_ENCODED;
        Decode::new(&self.inner)
    }

    /// Returns an iterator over subslices of the `EStr` slice
    /// separated by the given delimiter.
    ///
    /// # Panics
    ///
    /// Panics if the delimiter is not a [reserved] character.
    ///
    /// [reserved]: https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
    #[cfg(test)]
    pub(crate) fn split(&self, delim: char) -> Split<'_, E> {
        assert!(
            table::RESERVED.allows(delim),
            "splitting with non-reserved character"
        );
        Split {
            inner: self.inner.split(delim),
            encoder: PhantomData,
        }
    }
}

impl<E: Encoder> AsRef<Self> for EStr<E> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<E: Encoder> AsRef<str> for EStr<E> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<E: Encoder> PartialEq for EStr<E> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<E: Encoder> PartialEq<str> for EStr<E> {
    fn eq(&self, other: &str) -> bool {
        &self.inner == other
    }
}

impl<E: Encoder> PartialEq<EStr<E>> for str {
    fn eq(&self, other: &EStr<E>) -> bool {
        self == &other.inner
    }
}

impl<E: Encoder> Eq for EStr<E> {}

impl<E: Encoder> hash::Hash for EStr<E> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<E: Encoder> PartialOrd for EStr<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: Encoder> Ord for EStr<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<E: Encoder> Default for &EStr<E> {
    /// Creates an empty `EStr` slice.
    fn default() -> Self {
        EStr::EMPTY
    }
}

/// Debug impl for `EStr`.
impl<E: Encoder> core::fmt::Debug for EStr<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}

/// Display impl for `EStr`.
impl<E: Encoder> core::fmt::Display for EStr<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}
