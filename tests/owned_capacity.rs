//! Owned IRI allocation accounting through the public API.

use iri_rfc3987::Iri;

#[test]
fn owned_iri_exposes_retained_string_capacity() {
    let mut text = String::with_capacity(4096);
    text.push_str("https://example.org/resource");
    let expected = text.capacity();
    let iri = Iri::parse_owned(text).unwrap();

    assert!(expected > iri.as_str().len());
    assert_eq!(iri.capacity(), expected);
    assert_eq!(iri.into_string().capacity(), expected);
}
