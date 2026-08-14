# iri-rfc3987

[![Crates.io](https://img.shields.io/crates/v/iri-rfc3987.svg)](https://crates.io/crates/iri-rfc3987)
[![Documentation](https://docs.rs/iri-rfc3987/badge.svg)](https://docs.rs/iri-rfc3987)
[![CI](https://github.com/legra-ai/iri-rfc3987/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/iri-rfc3987/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/iri-rfc3987.svg)](https://github.com/legra-ai/iri-rfc3987/blob/main/LICENSE-APACHE)
[![Downloads](https://img.shields.io/crates/d/iri-rfc3987.svg)](https://crates.io/crates/iri-rfc3987)

Zero-copy parsing, validation, and resolution for
Internationalized Resource Identifiers (IRIs), based on RFC 3987 and RFC 3986.

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

The implementation retains the IRI-specific portion of the parser adapted
from [`fluent-uri`](https://crates.io/crates/fluent-uri) 0.4.1. URI-only types
and builder APIs are intentionally not included.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
