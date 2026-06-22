//! Invariant tests for the slice-02 identity primitives.
//!
//! These live as integration tests (against the public API only) so the
//! library sources in `src/` stay free of `unwrap`/`expect` — see ledger row
//! G-12. Test names contain the substrings the ledger's Verify commands filter
//! on (`id_uniqueness`, `id_roundtrip`, `id_creation_ordered`,
//! `identity_not_number`, `nodetype_variants`, `nodetype_classification`,
//! `valid_child_types`, `origin_roundtrip`, `node_identity_stable`).

use std::collections::HashSet;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use odm_core::{Id, IdParseError, Node, NodeType, Origin};
use proptest::prelude::*;

// ----- Id: uniqueness (G-1) ------------------------------------------------

proptest! {
    // Mint a large batch of ids and require every one distinct.
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn id_uniqueness(n in 1_000usize..=10_000) {
        let ids: HashSet<Id> = (0..n).map(|_| Id::new()).collect();
        prop_assert_eq!(ids.len(), n);
    }
}

// ----- Id: string round-trip (G-2) -----------------------------------------

proptest! {
    #[test]
    fn id_roundtrip(_ in 0u8..=u8::MAX) {
        let id = Id::new();
        let parsed = Id::from_str(&id.to_string())?;
        prop_assert_eq!(parsed, id);
    }
}

#[test]
fn id_roundtrip_rejects_garbage() {
    // Wrong length and bad characters map to distinct, typed errors.
    assert_eq!(Id::from_str("too-short"), Err(IdParseError::InvalidLength));
    assert_eq!(
        Id::from_str("!!!!!!!!!!!!!!!!!!!!!!!!!!"), // 26 chars, none base32
        Err(IdParseError::InvalidChar)
    );
}

// ----- Id: creation ordering (G-3) -----------------------------------------

#[test]
fn id_creation_ordered() {
    // ULID encodes the creation timestamp in its most-significant bits, so ids
    // minted in distinct milliseconds compare in creation order. A >=1ms gap
    // between mints makes the comparison deterministic (no same-ms tie).
    let mut previous = Id::new();
    for _ in 0..5 {
        sleep(Duration::from_millis(2));
        let next = Id::new();
        assert!(next > previous, "later id {next} should exceed earlier {previous}");
        previous = next;
    }
}

// ----- Id is not the human number (G-4) ------------------------------------

#[test]
fn identity_not_number() {
    // Two nodes that share every human-visible field still get distinct ids:
    // identity is independent of `number` (and of name/type/origin).
    let a = Node::new(7, NodeType::Slice, "same", Origin::Planned, false);
    let b = Node::new(7, NodeType::Slice, "same", Origin::Planned, false);
    assert_eq!(a.number(), b.number());
    assert_ne!(a.id(), b.id());
}

// ----- NodeType: variants + round-trip (G-5) -------------------------------

const ALL_NODE_TYPES: [NodeType; 6] = [
    NodeType::Project,
    NodeType::Arc,
    NodeType::Slice,
    NodeType::Odd,
    NodeType::Adr,
    NodeType::Note,
];

#[test]
fn nodetype_variants() {
    // Exactly six types, each round-tripping through its string form.
    assert_eq!(ALL_NODE_TYPES.len(), 6);
    for ty in ALL_NODE_TYPES {
        assert_eq!(NodeType::from_str(ty.as_str()), Ok(ty));
    }
    // Canonical spellings are the lowercase type names.
    assert_eq!(NodeType::Project.as_str(), "project");
    assert_eq!(NodeType::Note.as_str(), "note");
    // Parsing is case-insensitive; unknown names error.
    assert_eq!(NodeType::from_str("SLICE"), Ok(NodeType::Slice));
    assert!(NodeType::from_str("step").is_err());
}

// ----- NodeType: work vs document (G-6) ------------------------------------

#[test]
fn nodetype_classification() {
    for ty in [NodeType::Project, NodeType::Arc, NodeType::Slice] {
        assert!(ty.is_work(), "{ty} should be a work node");
        assert!(!ty.is_document(), "{ty} should not be a document node");
    }
    for ty in [NodeType::Odd, NodeType::Adr, NodeType::Note] {
        assert!(ty.is_document(), "{ty} should be a document node");
        assert!(!ty.is_work(), "{ty} should not be a work node");
    }
}

// ----- NodeType: containment data (G-7) ------------------------------------

#[test]
fn valid_child_types() {
    assert_eq!(NodeType::Project.valid_child_types(), &[NodeType::Arc]);
    assert_eq!(NodeType::Arc.valid_child_types(), &[NodeType::Slice]);
    // A slice is a leaf in the work-decomposition tree.
    assert!(NodeType::Slice.valid_child_types().is_empty());
    // Document nodes have no work children.
    for ty in [NodeType::Odd, NodeType::Adr, NodeType::Note] {
        assert!(ty.valid_child_types().is_empty(), "{ty} should have no work children");
    }
}

// ----- Origin: round-trip (G-8) --------------------------------------------

#[test]
fn origin_roundtrip() {
    for origin in [Origin::Planned, Origin::Discovered, Origin::Amendment] {
        assert_eq!(Origin::from_str(origin.as_str()), Ok(origin));
    }
    assert_eq!(Origin::Planned.as_str(), "planned");
    assert_eq!(Origin::from_str("AMENDMENT"), Ok(Origin::Amendment));
    assert!(Origin::from_str("invented").is_err());
}

// ----- Node: identity stable under edits (G-9) -----------------------------

#[test]
fn node_identity_stable() {
    let mut node = Node::new(1, NodeType::Odd, "Original", Origin::Planned, false);
    let id = node.id();

    node.set_name("Renamed");
    node.set_number(42);

    assert_eq!(node.id(), id, "id must not move when name/number change");
    assert_eq!(node.name(), "Renamed");
    assert_eq!(node.number(), 42);
    // The other fields are untouched by the edits.
    assert_eq!(node.node_type(), NodeType::Odd);
    assert_eq!(node.origin(), Origin::Planned);
    assert!(!node.reserved());
}

// ----- Trait surface: Default, Display, and error messages -----------------

#[test]
fn id_default_mints_fresh() {
    // `Default` delegates to `new`, so two defaults are distinct fresh ids.
    assert_ne!(Id::default(), Id::default());
}

#[test]
fn display_forms_render_canonically() {
    // `Display` agrees with `as_str` for the enums.
    assert_eq!(NodeType::Slice.to_string(), "slice");
    assert_eq!(NodeType::Odd.to_string(), "odd");
    assert_eq!(Origin::Discovered.to_string(), "discovered");
}

#[test]
fn parse_errors_carry_messages() {
    // Each typed parse error renders a human-readable message (and, for the
    // enum errors, echoes the offending input).
    let id_err = Id::from_str("nope").unwrap_err();
    assert!(id_err.to_string().contains("invalid id"));

    let ty_err = NodeType::from_str("widget").unwrap_err();
    assert!(ty_err.to_string().contains("widget"));

    let origin_err = Origin::from_str("guessed").unwrap_err();
    assert!(origin_err.to_string().contains("guessed"));
}

proptest! {
    // Property form: for any name/number edits, the id is invariant.
    #[test]
    fn node_identity_stable_under_arbitrary_edits(
        n0 in any::<u32>(),
        n1 in any::<u32>(),
        name0 in ".{0,40}",
        name1 in ".{0,40}",
        reserved in any::<bool>(),
    ) {
        let mut node = Node::new(n0, NodeType::Arc, name0, Origin::Discovered, reserved);
        let id = node.id();
        node.set_name(name1.clone());
        node.set_number(n1);
        prop_assert_eq!(node.id(), id);
        prop_assert_eq!(node.number(), n1);
        prop_assert_eq!(node.name(), name1.as_str());
    }
}
