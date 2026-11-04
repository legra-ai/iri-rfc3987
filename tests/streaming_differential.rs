//! Exhaustive short grammar combinations and unbounded component controls.

use iri_rfc3987::{Iri, IriRef, IriReferenceValidator};

fn compare(bytes: &[u8]) {
    let text = core::str::from_utf8(bytes).unwrap();
    let expected = IriRef::parse(text).is_ok();
    let mut validator = IriReferenceValidator::new();
    let mut error = false;
    for byte in bytes {
        error |= validator.push(core::slice::from_ref(byte)).is_err();
    }
    let result = validator.finish();
    assert_eq!(!error && result.is_ok(), expected, "{text:?}");
    if let Ok(classification) = result {
        assert_eq!(
            classification.is_absolute(),
            Iri::parse(text).is_ok(),
            "{text:?}"
        );
    }
}

#[test]
fn short_component_combinations_match_existing_grammar() {
    let alphabet = b"a1:/?#@[]%.+v";
    for prefix in ["", "//", "a:", "a://", "//u@", "a:/", "a?", "a#"] {
        // bounded: fixed prefix plus four generated ASCII characters.
        let mut bytes = [0; 16];
        bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
        for length in 0..=4 {
            for mut value in 0..alphabet.len().pow(length) {
                for offset in 0..length as usize {
                    bytes[prefix.len() + offset] = alphabet[value % alphabet.len()];
                    value /= alphabet.len();
                }
                compare(&bytes[..prefix.len() + length as usize]);
            }
        }
    }
}

#[test]
fn large_authority_and_ipvfuture_do_not_need_retained_text() {
    for (prefix, suffix) in [
        ("//", "@host/path"),
        ("//[v", ".address]/"),
        ("//[v1.", "]/"),
        ("urn:", ""),
    ] {
        let mut validator = IriReferenceValidator::new();
        validator.push(prefix.as_bytes()).unwrap();
        let window = [b'a'; 8192];
        for _ in 0..1024 {
            validator.push(&window).unwrap();
        }
        validator.push(suffix.as_bytes()).unwrap();
        validator.finish().unwrap();
    }
}
