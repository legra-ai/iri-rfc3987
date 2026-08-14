//! Path-specific extension trait and methods.

use super::encoder;
use super::encoder_trait::Encoder;
use super::estr::EStr;
#[cfg(test)]
use super::split::Split;

/// A marker trait for path encoders.
///
/// This trait is sealed and cannot be implemented outside this crate.
pub(crate) trait PathEncoder: Encoder + sealed::Sealed {}

mod sealed {
    /// Prevents external implementations of `PathEncoder`.
    pub(crate) trait Sealed {}
    impl Sealed for super::encoder::IPath {}
}

impl PathEncoder for encoder::IPath {}

/// Extension methods for the [path] component.
///
/// [path]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.3
impl<E: PathEncoder> EStr<E> {
    /// Checks whether the path is absolute, i.e., starting with
    /// `'/'`.
    #[inline]
    #[must_use]
    pub(crate) fn is_absolute(&self) -> bool {
        self.as_str().starts_with('/')
    }

    /// Checks whether the path is rootless, i.e., not starting with
    /// `'/'`.
    #[inline]
    #[must_use]
    pub(crate) fn is_rootless(&self) -> bool {
        !self.as_str().starts_with('/')
    }

    /// Returns an iterator over the path segments, separated by
    /// `'/'`.
    ///
    /// Returns `None` if the path is [rootless]. Use [`split`]
    /// instead if you need to split a rootless path on occurrences
    /// of `'/'`.
    ///
    /// Note that the path can be [empty] when authority is present,
    /// in which case this method will return `None`.
    ///
    /// [rootless]: Self::is_rootless
    /// [`split`]: Self::split
    /// [empty]: Self::is_empty
    #[inline]
    #[must_use]
    #[cfg(test)]
    pub(crate) fn segments_if_absolute(&self) -> Option<Split<'_, E>> {
        self.as_str()
            .strip_prefix('/')
            .map(|s| Self::new_validated(s).split('/'))
    }
}
