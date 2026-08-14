# iri-rfc3987

[![Crates.io](https://img.shields.io/crates/v/iri-rfc3987.svg)](https://crates.io/crates/iri-rfc3987)
[![Documentation](https://docs.rs/iri-rfc3987/badge.svg)](https://docs.rs/iri-rfc3987)
[![CI](https://github.com/legra-ai/iri-rfc3987/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/iri-rfc3987/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/iri-rfc3987.svg)](https://github.com/legra-ai/iri-rfc3987/blob/main/LICENSE-APACHE)
[![Downloads](https://img.shields.io/crates/d/iri-rfc3987.svg)](https://crates.io/crates/iri-rfc3987)

Zero-copy parsing, validation, and resolution for Internationalized Resource
Identifiers (IRIs), based on RFC 3987 and RFC 3986.

## Why this crate

IRIs are structured identifiers, not arbitrary strings. Parsing at the
boundary lets applications reject malformed values early while retaining the
original text and avoiding an allocation when the input is already borrowed.

`iri-rfc3987` keeps the public API deliberately small:

- parsing is explicit and fallible;
- borrowed and owned forms use the same generic types;
- relative references can be resolved against an absolute base;
- parse and resolution failures are typed and carry useful context;
- URI-only types and builder APIs are outside the scope of this crate.

The crate provides two validated types:

- `Iri<T>` for an absolute IRI with a required scheme.
- `IriRef<T>` for an IRI reference, including relative references.

Both types support borrowed parsing with `&str` and owned storage with
`String`:

```rust
use iri_rfc3987::{Iri, IriRef};

let iri = Iri::parse("https://example.org/resource")?;
assert_eq!(iri.as_str(), "https://example.org/resource");

let reference = IriRef::parse("../other")?;
assert!(!reference.has_scheme());
# Ok::<(), iri_rfc3987::IriParseError>(())
```

## Borrowed and owned values

Parsing an `&str` produces a zero-copy `Iri<&str>` or `IriRef<&str>`. Convert
to an owned value only when it must outlive the source string:

```rust
use iri_rfc3987::{Iri, IriRef};

let borrowed: Iri<&str> = Iri::parse("https://example.org/resource")?;
let owned: Iri<String> = borrowed.to_owned();
assert_eq!(owned.as_str(), "https://example.org/resource");

let reference: IriRef<String> = IriRef::parse_owned("../other".to_owned()).unwrap();
assert_eq!(reference.as_str(), "../other");
# Ok::<(), iri_rfc3987::IriParseError>(())
```

## Resolving references

`IriRef` accepts absolute and relative references. Resolution follows the
path-merging rules from RFC 3986:

```rust
use iri_rfc3987::{Iri, IriRef};

let base = Iri::parse("https://example.org/a/b/").unwrap();
let reference = IriRef::parse("../image.svg").unwrap();
let resolved = reference.resolve_against(&base).unwrap();

assert_eq!(resolved.as_str(), "https://example.org/a/image.svg");
```

An absolute reference is returned as its own resolved IRI, while an empty or
relative reference inherits the appropriate components from the base. A base
with a fragment and invalid opaque-base combinations are rejected with a
typed [`ResolveError`](https://docs.rs/iri-rfc3987/latest/iri_rfc3987/enum.ResolveError.html).

## Validation errors

Parsing never silently accepts an invalid value. `IriParseError::index`
identifies the byte position where validation stopped:

```rust
use iri_rfc3987::Iri;

let error = Iri::<&str>::parse("not an absolute iri").unwrap_err();
assert_eq!(error.index(), 3);
# Ok::<(), iri_rfc3987::IriParseError>(())
```

For a known compile-time constant, `Iri::known` is available when a panic is
preferable to carrying a construction error:

```rust
use iri_rfc3987::Iri;

let rdf_type = Iri::known("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
assert_eq!(rdf_type.as_str(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
```

## Serde

`Iri` and `IriRef` serialize as their validated string representation and
validate again when deserialized:

```rust
use iri_rfc3987::Iri;

let iri = Iri::parse_owned("https://example.org/resource".to_owned()).unwrap();
let encoded = serde_json::to_string(&iri)?;
let decoded: Iri<String> = serde_json::from_str(&encoded)?;

assert_eq!(decoded, iri);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## API overview

| Type | Purpose |
| --- | --- |
| `Iri<T>` | Absolute IRI with a required scheme. |
| `IriRef<T>` | Absolute or relative IRI reference. |
| `IriParseError` | Typed parse failure with a byte index. |
| `ResolveError` | Typed failure while resolving a reference. |
| `TryIntoIri` | Fallible conversion into `Iri<String>`. |

The implementation retains the IRI-specific portion of the parser adapted
from [`fluent-uri`](https://crates.io/crates/fluent-uri) 0.4.1. URI-only types
and builder APIs are intentionally not included.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
