//! Exercise IRI validation and resolution through the public consumer API.
use iri_rfc3987::{Iri, IriRef};

#[test]
fn relative_reference_preserves_query_and_fragment() {
    let base = Iri::parse("https://example.org/a/b/").expect("absolute base");
    let reference = IriRef::parse("../image.svg?size=2#part").expect("relative reference");
    let resolved = reference.resolve_against(&base).expect("resolved IRI");
    assert_eq!(
        resolved.as_str(),
        "https://example.org/a/image.svg?size=2#part"
    );
}

#[test]
fn unicode_iri_round_trips_through_the_owned_boundary() {
    let borrowed = Iri::parse("https://example.org/résumé").expect("Unicode IRI");
    let serialized = serde_json::to_string(&borrowed).expect("serialize IRI");
    let restored: Iri<String> = serde_json::from_str(&serialized).expect("validated IRI");
    assert_eq!(restored.as_str(), borrowed.as_str());
    assert!(Iri::parse("/relative").is_err());
    assert!(Iri::parse("https://example.org/%xy").is_err());
}
