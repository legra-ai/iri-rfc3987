//! Incremental validation and complete-input classification regressions.

use iri_rfc3987::{Iri, IriRef, IriReferenceValidator};

#[test]
fn incremental_reference_matches_owned_parser_at_every_boundary() {
    let cases = [
        "",
        "child",
        "../child",
        "urn:x",
        "https://例.example/é?q=私#片",
        "//user:password@example.test:443/path",
        "http://[::1]/",
        "http://[::ffff:192.0.2.1]/",
        "//[v1.address:part]/",
        "a%20b",
        "1:bad",
        "a b",
        "//host:bad/",
        "//user@@host/",
        "//[:::]/",
        "//[v1.]/",
        "%",
        "%0",
        "%xx",
        "x#fragment#again",
        "https://example.test/\u{e000}",
        "https://example.test/?\u{e000}",
    ];
    for value in cases {
        let expected = IriRef::parse(value).is_ok();
        for split in 0..=value.len() {
            let mut validator = IriReferenceValidator::new();
            let actual = validator
                .push(&value.as_bytes()[..split])
                .and_then(|()| validator.push(&value.as_bytes()[split..]))
                .and_then(|()| validator.finish());
            assert_eq!(actual.is_ok(), expected, "{value:?} split {split}");
            if let Ok(result) = actual {
                assert_eq!(result.is_absolute(), Iri::parse(value).is_ok());
            }
        }
    }
}

#[test]
fn late_invalid_byte_is_not_accepted_as_a_large_iri() {
    let mut validator = IriReferenceValidator::new();
    validator.push(b"urn:").unwrap();
    let window = [b'a'; 4096];
    for _ in 0..1024 {
        validator.push(&window).unwrap();
    }
    assert!(validator.push(b" ").is_err());
    assert!(validator.finish().is_err());
}

#[test]
fn incomplete_or_invalid_utf8_fails_at_eof_or_input() {
    for bytes in [
        &b"\xc3"[..],
        &b"\xc0\x80"[..],
        &b"\xed\xa0\x80"[..],
        &b"\xf4\x90\x80\x80"[..],
    ] {
        let mut validator = IriReferenceValidator::new();
        let result = validator.push(bytes).and_then(|()| validator.finish());
        assert!(result.is_err());
    }
}
