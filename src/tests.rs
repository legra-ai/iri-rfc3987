//! Tests for the `iri-rfc3987` crate.

use crate::{Iri, IriParseError, IriRef};

// ── Parsing: valid IRIs ──────────────────────────────────────────────

#[test]
fn parse_simple_http_iri() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert_eq!(iri.as_str(), "http://example.com");
    assert_eq!(iri.scheme().as_str(), "http");
}

#[test]
fn parse_https_with_path() {
    let iri = Iri::parse("https://example.com/path/to/resource").unwrap();
    assert_eq!(iri.scheme().as_str(), "https");
    assert_eq!(iri.path().as_str(), "/path/to/resource");
}

#[test]
fn parse_iri_with_all_components() {
    let iri = Iri::parse("http://user@example.com:8080/path?query=1#frag").unwrap();
    assert_eq!(iri.scheme().as_str(), "http");
    let auth = iri.authority().unwrap();
    assert_eq!(auth.userinfo().unwrap().as_str(), "user");
    assert_eq!(auth.host(), "example.com");
    assert_eq!(auth.port().unwrap().as_str(), "8080");
    assert_eq!(iri.path().as_str(), "/path");
    assert_eq!(iri.query().unwrap().as_str(), "query=1");
    assert_eq!(iri.fragment().unwrap().as_str(), "frag");
}

#[test]
fn parse_urn() {
    let iri = Iri::parse("urn:isbn:0451450523").unwrap();
    assert_eq!(iri.scheme().as_str(), "urn");
    assert!(iri.authority().is_none());
    assert_eq!(iri.path().as_str(), "isbn:0451450523");
}

#[test]
fn parse_iri_with_unicode() {
    let iri = Iri::parse("http://example.com/\u{00E9}l\u{00E8}ve").unwrap();
    assert_eq!(iri.scheme().as_str(), "http");
}

#[test]
fn parse_iri_with_percent_encoding() {
    let iri = Iri::parse("http://example.com/a%20b").unwrap();
    assert_eq!(iri.path().as_str(), "/a%20b");
}

#[test]
fn parse_iri_empty_authority() {
    let iri = Iri::parse("file:///etc/hosts").unwrap();
    assert_eq!(iri.scheme().as_str(), "file");
    let auth = iri.authority().unwrap();
    assert_eq!(auth.host(), "");
    assert_eq!(iri.path().as_str(), "/etc/hosts");
}

#[test]
fn parse_ipv6_host() {
    let iri = Iri::parse("http://[::1]/path").unwrap();
    let auth = iri.authority().unwrap();
    assert_eq!(auth.host(), "[::1]");
}

#[test]
fn parse_xsd_integer_iri() {
    let iri = Iri::parse("http://www.w3.org/2001/XMLSchema#integer").unwrap();
    assert_eq!(iri.scheme().as_str(), "http");
    assert_eq!(iri.fragment().unwrap().as_str(), "integer");
}

#[test]
fn parse_rdf_type_iri() {
    let iri = Iri::parse("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
    assert_eq!(iri.fragment().unwrap().as_str(), "type");
}

// ── Parsing: invalid IRIs ──────────────────────────────────────────

#[test]
fn parse_empty_string_as_iri_fails() {
    assert!(Iri::<&str>::parse("").is_err());
}

#[test]
fn parse_relative_as_iri_fails() {
    assert!(Iri::<&str>::parse("relative/path").is_err());
}

#[test]
fn parse_no_scheme_as_iri_fails() {
    assert!(Iri::<&str>::parse("//example.com/path").is_err());
}

#[test]
fn parse_iri_error_details() {
    let err = Iri::<&str>::parse("").unwrap_err();
    assert_eq!(err.index(), 0);
}

// ── Parsing: IRI references ────────────────────────────────────────

#[test]
fn parse_absolute_iri_reference() {
    let iri_ref = IriRef::parse("http://example.com/path").unwrap();
    assert_eq!(iri_ref.scheme().unwrap().as_str(), "http");
}

#[test]
fn parse_relative_iri_reference() {
    let iri_ref = IriRef::parse("../relative/path").unwrap();
    assert!(iri_ref.scheme().is_none());
    assert_eq!(iri_ref.path().as_str(), "../relative/path");
}

#[test]
fn parse_empty_iri_reference() {
    let iri_ref = IriRef::parse("").unwrap();
    assert!(iri_ref.scheme().is_none());
    assert!(iri_ref.authority().is_none());
    assert!(iri_ref.path().is_empty());
}

#[test]
fn parse_fragment_only_iri_reference() {
    let iri_ref = IriRef::parse("#fragment").unwrap();
    assert!(iri_ref.scheme().is_none());
    assert_eq!(iri_ref.fragment().unwrap().as_str(), "fragment");
}

#[test]
fn parse_query_only_iri_reference() {
    let iri_ref = IriRef::parse("?query").unwrap();
    assert!(iri_ref.scheme().is_none());
    assert_eq!(iri_ref.query().unwrap().as_str(), "query");
}

// ── Iri<String> ────────────────────────────────────────────────────

#[test]
fn parse_owned_iri() {
    let iri = Iri::parse_owned("http://example.com".to_owned()).unwrap();
    assert_eq!(iri.as_str(), "http://example.com");
    let s: String = iri.into_string();
    assert_eq!(s, "http://example.com");
}

#[test]
fn parse_owned_iri_error_returns_string() {
    let result = Iri::parse_owned("not an iri".to_owned());
    let (_err, original) = result.unwrap_err();
    assert_eq!(original, "not an iri");
}

#[test]
fn borrow_owned_iri() {
    let owned = Iri::parse_owned("http://example.com".to_owned()).unwrap();
    let borrowed = owned.borrow();
    assert_eq!(borrowed.as_str(), "http://example.com");
}

#[test]
fn to_owned_borrowed_iri() {
    let borrowed = Iri::parse("http://example.com").unwrap();
    let owned = borrowed.to_owned();
    assert_eq!(owned.as_str(), "http://example.com");
}

// ── Equality and comparison ────────────────────────────────────────

#[test]
fn iri_equality() {
    let a = Iri::parse("http://example.com").unwrap();
    let b = Iri::parse("http://example.com").unwrap();
    assert_eq!(a, b);
}

#[test]
fn iri_inequality() {
    let a = Iri::parse("http://example.com").unwrap();
    let b = Iri::parse("http://example.org").unwrap();
    assert_ne!(a, b);
}

#[test]
fn iri_eq_str() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert_eq!(iri, "http://example.com");
    assert_eq!("http://example.com", iri);
}

#[test]
fn iri_ordering() {
    let a = Iri::parse("http://a.example.com").unwrap();
    let b = Iri::parse("http://b.example.com").unwrap();
    assert!(a < b);
}

// ── Display and Debug ──────────────────────────────────────────────

#[test]
fn iri_display() {
    let iri = Iri::parse("http://example.com/path").unwrap();
    assert_eq!(format!("{iri}"), "http://example.com/path");
}

#[test]
fn iri_debug() {
    let iri = Iri::parse("http://example.com").unwrap();
    let debug = format!("{iri:?}");
    assert!(debug.contains("Iri"));
    assert!(debug.contains("http"));
}

// ── FromStr ────────────────────────────────────────────────────────

#[test]
fn iri_from_str() {
    let iri: Iri<String> = "http://example.com".parse().unwrap();
    assert_eq!(iri.as_str(), "http://example.com");
}

#[test]
fn iri_from_str_invalid() {
    let result: Result<Iri<String>, IriParseError> = "".parse();
    assert!(result.is_err());
}

// ── TryFrom ────────────────────────────────────────────────────────

#[test]
fn iri_try_from_str() {
    let iri: Iri<&str> = "http://example.com".try_into().unwrap();
    assert_eq!(iri.as_str(), "http://example.com");
}

#[test]
fn iri_try_from_string() {
    let iri: Iri<String> = "http://example.com".to_owned().try_into().unwrap();
    assert_eq!(iri.as_str(), "http://example.com");
}

// ── Conversion: Iri → IriRef ───────────────────────────────────────

#[test]
fn iri_to_iri_ref() {
    let iri = Iri::parse("http://example.com").unwrap();
    let iri_ref: IriRef<&str> = iri.into();
    assert_eq!(iri_ref.as_str(), "http://example.com");
}

// ── Normalization ──────────────────────────────────────────────────

#[test]
fn normalize_scheme_lowercasing() {
    let iri = Iri::parse("HTTP://example.com/").unwrap();
    let normalized = iri.normalize();
    assert_eq!(normalized.as_str(), "http://example.com/");
}

#[test]
fn normalize_host_lowercasing() {
    let iri = Iri::parse("http://EXAMPLE.COM/").unwrap();
    let normalized = iri.normalize();
    assert_eq!(normalized.as_str(), "http://example.com/");
}

#[test]
fn normalize_percent_decoding() {
    let iri = Iri::parse("http://example.com/%7Euser").unwrap();
    let normalized = iri.normalize();
    assert_eq!(normalized.as_str(), "http://example.com/~user");
}

#[test]
fn normalize_dot_segments() {
    let iri = Iri::parse("http://example.com/a/b/../c").unwrap();
    let normalized = iri.normalize();
    assert_eq!(normalized.as_str(), "http://example.com/a/c");
}

#[test]
fn normalize_preserves_valid_encoding() {
    let iri = Iri::parse("http://example.com/a%20b").unwrap();
    let normalized = iri.normalize();
    assert_eq!(normalized.as_str(), "http://example.com/a%20b");
}

// ── Reference resolution (RFC 3986 Section 5.4) ───────────────────

const BASE: &str = "http://a/b/c/d;p?q";

fn resolve(reference: &str) -> String {
    let base = Iri::parse(BASE).unwrap();
    let r = IriRef::parse(reference).unwrap();
    r.resolve_against(&base).unwrap().into_string()
}

#[test]
fn resolve_normal_examples() {
    // RFC 3986 Section 5.4.1 — Normal Examples
    assert_eq!(resolve("g:h"), "g:h");
    assert_eq!(resolve("g"), "http://a/b/c/g");
    assert_eq!(resolve("./g"), "http://a/b/c/g");
    assert_eq!(resolve("g/"), "http://a/b/c/g/");
    assert_eq!(resolve("/g"), "http://a/g");
    assert_eq!(resolve("//g/h"), "http://g/h");
    assert_eq!(resolve("?y"), "http://a/b/c/d;p?y");
    assert_eq!(resolve("g?y"), "http://a/b/c/g?y");
    assert_eq!(resolve("#s"), "http://a/b/c/d;p?q#s");
    assert_eq!(resolve("g#s"), "http://a/b/c/g#s");
    assert_eq!(resolve("g?y#s"), "http://a/b/c/g?y#s");
    assert_eq!(resolve(";x"), "http://a/b/c/;x");
    assert_eq!(resolve("g;x"), "http://a/b/c/g;x");
    assert_eq!(resolve("g;x?y#s"), "http://a/b/c/g;x?y#s");
    assert_eq!(resolve(""), "http://a/b/c/d;p?q");
    assert_eq!(resolve("."), "http://a/b/c/");
    assert_eq!(resolve("./"), "http://a/b/c/");
    assert_eq!(resolve(".."), "http://a/b/");
    assert_eq!(resolve("../"), "http://a/b/");
    assert_eq!(resolve("../g"), "http://a/b/g");
    assert_eq!(resolve("../.."), "http://a/");
    assert_eq!(resolve("../../"), "http://a/");
    assert_eq!(resolve("../../g"), "http://a/g");
}

#[test]
fn resolve_abnormal_examples() {
    // RFC 3986 Section 5.4.2 — Abnormal Examples
    assert_eq!(resolve("../../../g"), "http://a/g");
    assert_eq!(resolve("../../../../g"), "http://a/g");
    assert_eq!(resolve("/./g"), "http://a/g");
    assert_eq!(resolve("/../g"), "http://a/g");
    assert_eq!(resolve("g."), "http://a/b/c/g.");
    assert_eq!(resolve(".g"), "http://a/b/c/.g");
    assert_eq!(resolve("g.."), "http://a/b/c/g..");
    assert_eq!(resolve("..g"), "http://a/b/c/..g");
}

// ── Fragment handling ──────────────────────────────────────────────

#[test]
fn has_fragment() {
    let iri = Iri::parse("http://example.com#frag").unwrap();
    assert!(iri.has_fragment());
}

#[test]
fn no_fragment() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert!(!iri.has_fragment());
}

#[test]
fn strip_fragment() {
    let iri = Iri::parse("http://example.com/path#frag").unwrap();
    let stripped = iri.strip_fragment();
    assert_eq!(stripped.as_str(), "http://example.com/path");
}

#[test]
fn set_fragment_on_owned() {
    let mut iri = Iri::parse_owned("http://example.com/path".to_owned()).unwrap();
    let frag = crate::pct_enc::EStr::<crate::pct_enc::encoder::IFragment>::new("new_frag");
    iri.set_fragment(frag);
    assert_eq!(iri.as_str(), "http://example.com/path#new_frag");
}

// ── Query handling ─────────────────────────────────────────────────

#[test]
fn has_query() {
    let iri = Iri::parse("http://example.com?q=1").unwrap();
    assert!(iri.has_query());
}

#[test]
fn no_query() {
    let iri = Iri::parse("http://example.com").unwrap();
    assert!(!iri.has_query());
}

// ── Authority handling ─────────────────────────────────────────────

#[test]
fn has_authority() {
    let iri = Iri::parse("http://example.com/path").unwrap();
    assert!(iri.has_authority());
}

#[test]
fn no_authority() {
    let iri = Iri::parse("urn:isbn:0451450523").unwrap();
    assert!(!iri.has_authority());
}

#[test]
fn authority_port_to_u16() {
    let iri = Iri::parse("http://example.com:8080/path").unwrap();
    let auth = iri.authority().unwrap();
    assert_eq!(auth.port_to_u16().unwrap(), Some(8080));
}

// ── Path segments ──────────────────────────────────────────────────

#[test]
fn path_segments() {
    let iri = Iri::parse("http://example.com/a/b/c").unwrap();
    let segs: Vec<&str> = iri
        .path()
        .segments_if_absolute()
        .unwrap()
        .map(crate::pct_enc::EStr::as_str)
        .collect();
    assert_eq!(segs, ["a", "b", "c"]);
}

#[test]
fn path_is_absolute() {
    let iri = Iri::parse("http://example.com/path").unwrap();
    assert!(iri.path().is_absolute());
}

// ── Serde ─────────────────────────────────────────────────────────

mod serde_tests {
    use crate::{Iri, IriRef};

    #[test]
    fn serialize_iri() {
        let iri = Iri::parse("http://example.com").unwrap();
        let json = serde_json::to_string(&iri).unwrap();
        assert_eq!(json, "\"http://example.com\"");
    }

    #[test]
    fn deserialize_iri_owned() {
        let iri: Iri<String> = serde_json::from_str("\"http://example.com\"").unwrap();
        assert_eq!(iri.as_str(), "http://example.com");
    }

    #[test]
    fn deserialize_invalid_iri_fails() {
        let result: Result<Iri<String>, _> = serde_json::from_str("\"not a valid iri\"");
        assert!(result.is_err());
    }

    #[test]
    fn serde_roundtrip_iri() {
        let original = Iri::parse_owned("http://example.com/path?q=1#frag".to_owned()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Iri<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn serde_roundtrip_iri_ref() {
        let original = IriRef::parse_owned("../relative#frag".to_owned()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: IriRef<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}

// ── EStr and percent encoding ──────────────────────────────────────

#[test]
fn estr_decode_to_string() {
    let iri = Iri::parse("http://example.com/a%20b").unwrap();
    let decoded = iri.path().decode().into_string().unwrap();
    assert_eq!(&*decoded, "/a b");
}

#[test]
fn estr_split_query_params() {
    let iri = Iri::parse("http://example.com/?a=1&b=2").unwrap();
    let query = iri.query().unwrap();
    let params: Vec<&str> = query.split('&').map(crate::pct_enc::EStr::as_str).collect();
    assert_eq!(params, ["a=1", "b=2"]);
}

// ── Normalizer configuration ───────────────────────────────────────

#[test]
fn normalizer_default_port_removal() {
    use crate::normalize::Normalizer;

    fn default_port(scheme: &str) -> Option<u16> {
        match scheme {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        }
    }

    let normalizer = Normalizer::new().default_port_with(default_port);

    let iri = Iri::parse("http://example.com:80/path").unwrap();
    let normalized_iri = normalizer.normalize(&iri).unwrap();
    assert_eq!(normalized_iri.as_str(), "http://example.com/path");

    let iri2 = Iri::parse("http://example.com:8080/path").unwrap();
    let normalized2 = normalizer.normalize(&iri2).unwrap();
    assert_eq!(normalized2.as_str(), "http://example.com:8080/path");
}

// ── Resolver configuration ─────────────────────────────────────────

#[test]
fn resolver_with_base() {
    use crate::resolve::Resolver;

    let base = Iri::parse("http://example.com/a/b/c").unwrap();
    let resolver = Resolver::with_base(base);

    let reference = IriRef::parse("../d").unwrap();
    let resolved_iri = resolver.resolve(&reference).unwrap();
    assert_eq!(resolved_iri.as_str(), "http://example.com/a/d");
}

// ── Scheme comparison ──────────────────────────────────────────────

#[test]
fn scheme_case_insensitive_equality() {
    use crate::component::Scheme;

    let a = Scheme::new_or_panic("http");
    let b = Scheme::new_or_panic("HTTP");
    assert_eq!(a, b);
}

#[test]
fn scheme_new_invalid() {
    use crate::component::Scheme;

    assert!(Scheme::new("").is_none());
    assert!(Scheme::new("123").is_none());
}

// ── TryIntoIri ──────────────────────────────────────────────────────

mod try_into_iri_tests {
    use crate::{Iri, TryIntoIri};

    #[test]
    fn from_str_valid() {
        let iri = "http://example.com".try_into_iri().unwrap();
        assert_eq!(iri.as_str(), "http://example.com");
    }

    #[test]
    fn from_str_invalid() {
        assert!("not a valid iri".try_into_iri().is_err());
    }

    #[test]
    fn from_string_valid() {
        let s = String::from("http://example.com/path");
        let iri = s.try_into_iri().unwrap();
        assert_eq!(iri.as_str(), "http://example.com/path");
    }

    #[test]
    fn from_string_invalid() {
        let s = String::from(":::invalid");
        assert!(s.try_into_iri().is_err());
    }

    #[test]
    fn from_iri_string_passthrough() {
        let original = Iri::parse("http://example.com").unwrap().to_owned();
        let cloned = original.clone();
        let result = original.try_into_iri().unwrap();
        assert_eq!(result, cloned);
    }

    #[test]
    fn from_iri_borrowed() {
        let borrowed = Iri::parse("http://example.com").unwrap();
        let result = borrowed.try_into_iri().unwrap();
        assert_eq!(result.as_str(), "http://example.com");
    }

    #[test]
    fn from_iri_ref() {
        let owned = Iri::parse("http://example.com").unwrap().to_owned();
        let result = (&owned).try_into_iri().unwrap();
        assert_eq!(result, owned);
    }
}

// ── Iri::known ──────────────────────────────────────────────────────

mod known_tests {
    use crate::Iri;

    #[test]
    fn known_valid_iri() {
        let iri = Iri::known("http://example.com/resource");
        assert_eq!(iri.as_str(), "http://example.com/resource");
    }

    #[test]
    fn known_urn() {
        let iri = Iri::known("urn:isbn:0451450523");
        assert_eq!(iri.scheme().as_str(), "urn");
    }

    #[test]
    #[should_panic(expected = "Iri::known called with invalid IRI")]
    fn known_invalid_panics() {
        let _ = Iri::known("not a valid iri");
    }

    #[test]
    #[should_panic(expected = "Iri::known called with invalid IRI")]
    fn known_empty_panics() {
        let _ = Iri::known("");
    }
}
