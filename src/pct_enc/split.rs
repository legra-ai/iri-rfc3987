//! Iterator over `EStr` subslices separated by a delimiter.

use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::str;

use super::encoder_trait::Encoder;
use super::estr::EStr;

/// An iterator over subslices of an [`EStr`] slice separated by a
/// delimiter.
///
/// This struct is created by [`EStr::split`].
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Split<'a, E: Encoder> {
    pub(super) inner: str::Split<'a, char>,
    pub(super) encoder: PhantomData<E>,
}

impl<'a, E: Encoder> Iterator for Split<'a, E> {
    type Item = &'a EStr<E>;

    fn next(&mut self) -> Option<&'a EStr<E>> {
        self.inner.next().map(EStr::new_validated)
    }
}

impl<'a, E: Encoder> DoubleEndedIterator for Split<'a, E> {
    fn next_back(&mut self) -> Option<&'a EStr<E>> {
        self.inner.next_back().map(EStr::new_validated)
    }
}

impl<E: Encoder> FusedIterator for Split<'_, E> {}
