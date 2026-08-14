//! Cross-type conversions and shared trait impls.
//!
//! Holds `From<Iri<T>> for IriRef<T>` plus the `impl_common_traits!`
//! macro that generates `Default`, `PartialEq`, `Hash`, `Display`,
//! `Debug`, etc. for both `Iri` and `IriRef`.

use alloc::string::String;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::str::FromStr;
use core::{fmt, hash};

use borrow_or_share::Bos;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::iri::Iri;
use super::iri_ref::IriRef;
use super::parts::{Meta, Value};
use crate::error::IriParseError;

// ── Conversion: Iri → IriRef ───────────────────────────────────────

impl<T> From<Iri<T>> for IriRef<T> {
    fn from(iri: Iri<T>) -> Self {
        IriRef {
            val: iri.val,
            meta: iri.meta,
        }
    }
}

// ── Shared trait impls via macro ───────────────────────────────────

macro_rules! impl_common_traits {
    ($Ty:ident, $name:literal) => {
        impl<T: Value> Default for $Ty<T> {
            fn default() -> Self {
                Self {
                    val: T::default(),
                    meta: Meta::default(),
                }
            }
        }

        impl<T: Bos<str>, U: Bos<str>> PartialEq<$Ty<U>> for $Ty<T> {
            fn eq(&self, other: &$Ty<U>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl<T: Bos<str>> PartialEq<str> for $Ty<T> {
            fn eq(&self, other: &str) -> bool {
                self.as_str() == other
            }
        }

        impl<T: Bos<str>> PartialEq<$Ty<T>> for str {
            fn eq(&self, other: &$Ty<T>) -> bool {
                self == other.as_str()
            }
        }

        impl<T: Bos<str>> PartialEq<&str> for $Ty<T> {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl<T: Bos<str>> PartialEq<$Ty<T>> for &str {
            fn eq(&self, other: &$Ty<T>) -> bool {
                *self == other.as_str()
            }
        }

        impl<T: Bos<str>> Eq for $Ty<T> {}

        impl<T: Bos<str>> hash::Hash for $Ty<T> {
            fn hash<H: hash::Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }

        impl<T: Bos<str>> PartialOrd for $Ty<T> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<T: Bos<str>> Ord for $Ty<T> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl<T: Bos<str>> AsRef<str> for $Ty<T> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl<T: Bos<str>> Borrow<str> for $Ty<T> {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl<'a> TryFrom<&'a str> for $Ty<&'a str> {
            type Error = IriParseError;

            #[inline]
            fn try_from(value: &'a str) -> Result<Self, Self::Error> {
                $Ty::parse(value)
            }
        }

        impl TryFrom<String> for $Ty<String> {
            type Error = (IriParseError, String);

            #[inline]
            fn try_from(value: String) -> Result<Self, Self::Error> {
                $Ty::parse_owned(value)
            }
        }

        impl<'a> From<$Ty<&'a str>> for &'a str {
            #[inline]
            fn from(value: $Ty<&'a str>) -> &'a str {
                value.val
            }
        }

        impl From<$Ty<String>> for String {
            #[inline]
            fn from(value: $Ty<String>) -> String {
                value.val
            }
        }

        impl From<$Ty<&str>> for $Ty<String> {
            #[inline]
            fn from(value: $Ty<&str>) -> Self {
                value.to_owned()
            }
        }

        impl FromStr for $Ty<String> {
            type Err = IriParseError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $Ty::parse(s).map(|r| r.to_owned())
            }
        }

        impl<T: Bos<str>> fmt::Debug for $Ty<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct($name)
                    .field("scheme", &self.scheme())
                    .field("authority", &self.authority())
                    .field("path", &self.path())
                    .field("query", &self.query())
                    .field("fragment", &self.fragment())
                    .finish()
            }
        }

        impl<T: Bos<str>> fmt::Display for $Ty<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self.as_str(), f)
            }
        }

        impl<T: Bos<str>> Serialize for $Ty<T> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $Ty<&'de str> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = <&str>::deserialize(deserializer)?;
                $Ty::parse(s).map_err(|e| {
                    de::Error::custom(format_args!("failed to parse {:?} as {}: {}", s, $name, e))
                })
            }
        }

        impl<'de> Deserialize<'de> for $Ty<String> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                $Ty::parse_owned(s).map_err(|(e, _)| {
                    de::Error::custom(format_args!("failed to parse as {}: {}", $name, e))
                })
            }
        }
    };
}

impl_common_traits!(Iri, "Iri");
impl_common_traits!(IriRef, "IriRef");
