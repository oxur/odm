//! Tests for the frontmatter schema and its round-trip invariant.
//!
//! Integration tests (public API only), so library `src/` stays panic-free.
//! Test names contain the substrings the ledger Verify commands filter on
//! (`frontmatter_parse`, `schema_core_fields`, `schema_edges_block`,
//! `frontmatter_roundtrip`, `unknown_keys_preserved`, `canonical_field_order`,
//! `supersedes_kind`).

use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::frontmatter::{
    Dependency, Document, Edges, Frontmatter, FrontmatterError, Retirement, SupersedeKind,
    Supersedes, TornEdge,
};
use odm_core::gates::GateSets;
use odm_core::status::Evidence;
use odm_core::{Id, NodeType, Origin};
use proptest::prelude::*;

const SAMPLE_ULID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
}

fn minimal_doc_text(body: &str) -> String {
    format!(
        "---\n\
         id: {SAMPLE_ULID}\n\
         number: 7\n\
         type: slice\n\
         name: Store layer\n\
         created: 2026-06-20\n\
         updated: 2026-06-21\n\
         origin: planned\n\
         reserved: false\n\
         ---\n{body}"
    )
}

// ----- I-1: parse splits frontmatter from body; errors are typed ------------

#[test]
fn frontmatter_parse_splits_block_and_body() {
    let doc = Document::parse(&minimal_doc_text("# Title\n\nBody.\n")).expect("valid doc");
    assert_eq!(doc.frontmatter().number(), 7);
    assert_eq!(doc.body(), "# Title\n\nBody.\n");
}

#[test]
fn frontmatter_parse_rejects_malformed() {
    // No opening fence.
    assert_eq!(Document::parse("no fence here"), Err(FrontmatterError::MissingOpen));
    // Opening but no closing fence.
    assert_eq!(Document::parse("---\nid: x\nnumber: 1\n"), Err(FrontmatterError::Unterminated));
    // Closing fence but invalid YAML / schema (number is not an int).
    let bad = "---\nid: x\nnumber: not-a-number\ntype: slice\nname: n\n\
               created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\n---\n";
    assert!(matches!(Document::parse(bad), Err(FrontmatterError::Yaml(_))));
}

// ----- I-2: core fields parse ----------------------------------------------

#[test]
fn schema_core_fields_parse() {
    let text = format!(
        "---\n\
         id: {SAMPLE_ULID}\n\
         number: 42\n\
         type: odd\n\
         name: Architecture\n\
         created: 2026-06-20\n\
         updated: 2026-06-22\n\
         tags: [arch, design]\n\
         component: odm-core\n\
         origin: discovered\n\
         reserved: true\n\
         ---\nbody\n"
    );
    let fm = Document::parse(&text).expect("valid").frontmatter().clone();
    assert_eq!(fm.id(), Id::from_str(SAMPLE_ULID).unwrap());
    assert_eq!(fm.number(), 42);
    assert_eq!(fm.node_type(), NodeType::Odd);
    assert_eq!(fm.name(), "Architecture");
    assert_eq!(fm.created(), day(2026, 6, 20));
    assert_eq!(fm.updated(), day(2026, 6, 22));
    assert_eq!(fm.tags(), &["arch".to_string(), "design".to_string()]);
    assert_eq!(fm.component(), Some("odm-core"));
    assert_eq!(fm.origin(), Origin::Discovered);
    assert!(fm.reserved());
}

// ----- I-3: edges block parses ---------------------------------------------

#[test]
fn schema_edges_block_parses_every_kind() {
    let p = SAMPLE_ULID;
    let text = format!(
        "---\n\
         id: {p}\n\
         number: 1\n\
         type: slice\n\
         name: n\n\
         created: 2026-06-20\n\
         updated: 2026-06-20\n\
         origin: planned\n\
         reserved: false\n\
         edges:\n\
        \x20 part_of: {p}\n\
        \x20 depends_on:\n\
        \x20   - {p}\n\
        \x20   - {{ node: {p}, satisfied_at: tested }}\n\
        \x20 blocked_by: [{p}]\n\
        \x20 verifies: [{p}]\n\
        \x20 consumes: [{p}]\n\
        \x20 affects: [{p}]\n\
        \x20 supersedes: {{ node: {p}, kind: obsoletes }}\n\
        \x20 tears:\n\
        \x20   - edge: {p}\n\
        \x20     because: assumed for cycle break\n\
         ---\nbody\n"
    );
    let fm = Document::parse(&text).expect("valid edges").frontmatter().clone();
    let e = fm.edges();
    let id = Id::from_str(p).unwrap();
    assert_eq!(e.part_of, Some(id));
    assert_eq!(e.depends_on.len(), 2);
    assert_eq!(e.depends_on[0], Dependency::Bare(id));
    assert_eq!(
        e.depends_on[1],
        Dependency::Qualified { node: id, satisfied_at: "tested".to_string() }
    );
    assert_eq!(e.blocked_by, vec![id]);
    assert_eq!(e.verifies, vec![id]);
    assert_eq!(e.consumes, vec![id]);
    assert_eq!(e.affects, vec![id]);
    assert_eq!(e.supersedes, Some(Supersedes { node: id, kind: SupersedeKind::Obsoletes }));
    assert_eq!(
        e.tears,
        vec![TornEdge {
            edge: Dependency::Bare(id),
            because: "assumed for cycle break".to_string()
        }]
    );
}

// ----- I-6: canonical field order (snapshot) -------------------------------

#[test]
fn canonical_field_order_snapshot() {
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let fm = Frontmatter::new(
        id,
        7,
        NodeType::Slice,
        "Store layer",
        day(2026, 6, 20),
        day(2026, 6, 21),
        Origin::Planned,
    )
    .with_tags(vec!["store".to_string()])
    .with_component("odm-store")
    .with_edges(Edges { part_of: Some(id), ..Edges::default() });

    let emitted = Document::new(fm, "body\n").emit().expect("emit");
    let expected = format!(
        "---\n\
         id: {SAMPLE_ULID}\n\
         number: 7\n\
         type: slice\n\
         name: Store layer\n\
         created: 2026-06-20\n\
         updated: 2026-06-21\n\
         tags:\n\
         - store\n\
         component: odm-store\n\
         origin: planned\n\
         reserved: false\n\
         edges:\n\
        \x20 part_of: {SAMPLE_ULID}\n\
         ---\nbody\n"
    );
    assert_eq!(emitted, expected);
}

// ----- I-7: supersedes kind ∈ {obsoletes, updates} -------------------------

#[test]
fn supersedes_kind_roundtrips_both_variants() {
    for (kind, word) in
        [(SupersedeKind::Obsoletes, "obsoletes"), (SupersedeKind::Updates, "updates")]
    {
        let id = Id::from_str(SAMPLE_ULID).unwrap();
        let fm = Frontmatter::new(
            id,
            1,
            NodeType::Adr,
            "decision",
            day(2026, 6, 20),
            day(2026, 6, 20),
            Origin::Planned,
        )
        .with_edges(Edges { supersedes: Some(Supersedes { node: id, kind }), ..Edges::default() });
        let emitted = Document::new(fm.clone(), "").emit().expect("emit");
        assert!(emitted.contains(&format!("kind: {word}")), "kind word in YAML");
        let parsed = Document::parse(&emitted).expect("reparse");
        assert_eq!(parsed.frontmatter().edges().supersedes.as_ref().unwrap().kind, kind);
    }
}

// ----- mutators + retirement round-trip (slice05 additions) -----------------

#[test]
fn mutators_and_retirement_roundtrip() {
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let fm = Frontmatter::new(
        id,
        1,
        NodeType::Slice,
        "Original",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    );
    let mut doc = Document::new(fm, "body\n");

    // In-place edits via frontmatter_mut.
    let f = doc.frontmatter_mut();
    f.set_name("Renamed");
    f.set_updated(day(2026, 6, 23));
    f.retire("folded into slice 6", day(2026, 6, 23));
    assert_eq!(doc.frontmatter().name(), "Renamed");
    assert_eq!(doc.frontmatter().updated(), day(2026, 6, 23));
    assert_eq!(
        doc.frontmatter().retired(),
        Some(&Retirement { reason: "folded into slice 6".to_string(), on: day(2026, 6, 23) })
    );

    // The retirement marker survives a round-trip.
    let reparsed = Document::parse(&doc.emit().unwrap()).unwrap();
    assert_eq!(reparsed, doc);
    assert!(reparsed.emit().unwrap().contains("retired:"));
}

// ----- I-5: unknown keys preserved -----------------------------------------

#[test]
fn unknown_keys_preserved_through_roundtrip() {
    // `status` became a typed field in arc02 slice04, so it is no longer an
    // "unknown" key; `desired_facts` is still unmodeled and must survive. Both
    // still round-trip — one typed, one preserved.
    let text = format!(
        "---\n\
         id: {SAMPLE_ULID}\n\
         number: 3\n\
         type: slice\n\
         name: n\n\
         created: 2026-06-20\n\
         updated: 2026-06-20\n\
         origin: planned\n\
         reserved: false\n\
         status:\n\
        \x20 built:\n\
        \x20   reached: 2026-06-12\n\
        \x20   evidence: reproduced\n\
         desired_facts:\n\
        \x20 - id: db-wired\n\
        \x20   describe: prod connects\n\
         ---\nbody\n"
    );
    let doc = Document::parse(&text).expect("valid");
    // Only `desired_facts` is unknown now; `status` is typed.
    assert_eq!(doc.frontmatter().unknown_key_count(), 1);
    assert!(doc.frontmatter().status().has_reached("built"));

    let reparsed = Document::parse(&doc.emit().expect("emit")).expect("reparse");
    assert_eq!(reparsed, doc, "typed status and unknown keys both survive a round-trip");
    // Both keys are still literally present after emission.
    let emitted = doc.emit().expect("emit");
    assert!(emitted.contains("status:"));
    assert!(emitted.contains("desired_facts:"));
    assert!(emitted.contains("evidence: reproduced"));
}

// ----- H-15 (arc02 s04): typed Status field on Frontmatter ------------------

#[test]
fn status_typed_field_round_trips() {
    use std::str::FromStr;
    let gates =
        GateSets::from_toml_str("[gates.slice]\nsequence = [\"planned\", \"built\"]").unwrap();
    let gset = gates.for_type(NodeType::Slice).unwrap();

    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let mut fm = Frontmatter::new(
        id,
        7,
        NodeType::Slice,
        "Store layer",
        day(2026, 6, 20),
        day(2026, 6, 21),
        Origin::Planned,
    );
    // Record a reached gate on the *typed* status field (not a preserved key).
    fm.status_mut()
        .set_gate(gset, "built", Some("duncan".into()), Evidence::Reproduced, day(2026, 6, 22))
        .unwrap();
    let doc = Document::new(fm, "body\n");

    // The typed status survives a round-trip and is not counted as unknown.
    let reparsed = Document::parse(&doc.emit().unwrap()).unwrap();
    assert_eq!(reparsed, doc);
    let record = reparsed.frontmatter().status().gate("built").expect("typed status read back");
    assert_eq!(record.evidence, Evidence::Reproduced);
    assert_eq!(
        reparsed.frontmatter().unknown_key_count(),
        0,
        "status is typed, not preserved-unknown"
    );
    assert!(reparsed.emit().unwrap().contains("status:"));
}

// ----- I-4: parse ∘ emit == identity (proptest over typed fields) ----------

prop_compose! {
    fn arb_date()(
        // Stay within chrono's always-valid range to avoid generating
        // impossible calendar dates.
        days in 0i64..40_000
    ) -> NaiveDate {
        day(1970, 1, 1) + chrono::Duration::days(days)
    }
}

fn arb_node_type() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        Just(NodeType::Project),
        Just(NodeType::Arc),
        Just(NodeType::Slice),
        Just(NodeType::Odd),
        Just(NodeType::Adr),
        Just(NodeType::Note),
    ]
}

fn arb_origin() -> impl Strategy<Value = Origin> {
    prop_oneof![Just(Origin::Planned), Just(Origin::Discovered), Just(Origin::Amendment)]
}

// Plain, non-adversarial text: keeps generated YAML well-formed (no embedded
// fences or control characters), matching "arbitrary valid nodes".
fn arb_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ._-]{0,40}"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn frontmatter_roundtrip_identity(
        number in any::<u32>(),
        node_type in arb_node_type(),
        name in arb_text(),
        created in arb_date(),
        updated in arb_date(),
        origin in arb_origin(),
        reserved in any::<bool>(),
        tags in prop::collection::vec(arb_text(), 0..4),
        component in prop::option::of(arb_text()),
        has_parent in any::<bool>(),
        n_deps in 0usize..3,
        body_lines in prop::collection::vec(arb_text(), 0..4),
    ) {
        let id = Id::new();
        let mut edges = Edges::default();
        if has_parent {
            edges.part_of = Some(Id::new());
        }
        for i in 0..n_deps {
            // Mix bare and qualified dependencies.
            if i % 2 == 0 {
                edges.depends_on.push(Dependency::Bare(Id::new()));
            } else {
                edges.depends_on.push(Dependency::Qualified {
                    node: Id::new(),
                    satisfied_at: "tested".to_string(),
                });
            }
        }

        let mut fm = Frontmatter::new(id, number, node_type, name, created, updated, origin)
            .with_tags(tags)
            .with_reserved(reserved)
            .with_edges(edges);
        if let Some(c) = component {
            fm = fm.with_component(c);
        }

        let doc = Document::new(fm, body_lines.join("\n"));
        let emitted = doc.emit()?;
        let reparsed = Document::parse(&emitted)?;
        prop_assert_eq!(reparsed, doc);
    }
}

// ----- C-1: a tear entry carries both the torn edge and the rationale -------

#[test]
fn tear_carries_rationale() {
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let target = Id::new();
    let fm = Frontmatter::new(
        id,
        1,
        NodeType::Slice,
        "n",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    )
    .with_edges(Edges {
        tears: vec![TornEdge {
            edge: Dependency::Bare(target),
            because: "B is assumed to ship first".to_string(),
        }],
        ..Edges::default()
    });
    let emitted = Document::new(fm, "body\n").emit().unwrap();

    // Both the torn edge target and its rationale are persisted (not dropped).
    assert!(emitted.contains(&target.to_string()), "edge persisted:\n{emitted}");
    assert!(emitted.contains("because:"), "rationale key persisted:\n{emitted}");

    // The typed entry round-trips: edge + because both survive.
    let reparsed = Document::parse(&emitted).unwrap();
    let tears = &reparsed.frontmatter().edges().tears;
    assert_eq!(tears.len(), 1);
    assert_eq!(tears[0].edge, Dependency::Bare(target));
    assert_eq!(tears[0].because, "B is assumed to ship first");
}

// ----- C-3: a populated `tears` round-trips (parse ∘ emit = identity) -------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn tears_roundtrip_identity(
        n_tears in 1usize..4,
        // Non-empty rationale text (a tear always carries one).
        becauses in prop::collection::vec("[a-zA-Z0-9][a-zA-Z0-9 ._-]{0,39}", 1..4),
    ) {
        let id = Id::new();
        let mut edges = Edges::default();
        for i in 0..n_tears {
            let because = becauses.get(i % becauses.len()).cloned().unwrap();
            // Mix bare and gate-qualified torn edges.
            let edge = if i % 2 == 0 {
                Dependency::Bare(Id::new())
            } else {
                Dependency::Qualified { node: Id::new(), satisfied_at: "tested".to_string() }
            };
            edges.tears.push(TornEdge { edge, because });
        }
        let fm = Frontmatter::new(
            id, 1, NodeType::Slice, "n", day(2026, 6, 20), day(2026, 6, 20), Origin::Planned,
        )
        .with_edges(edges);

        let doc = Document::new(fm, "body\n");
        let emitted = doc.emit()?;
        prop_assert!(emitted.contains("tears:"), "tears emitted: {emitted}");
        prop_assert!(emitted.contains("because:"), "rationale emitted: {emitted}");
        let reparsed = Document::parse(&emitted)?;
        prop_assert_eq!(reparsed, doc);
    }
}

// ----- C-4: empty `tears` is omitted; no-tears nodes round-trip identically -

#[test]
fn empty_tears_roundtrip() {
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let fm = Frontmatter::new(
        id,
        1,
        NodeType::Slice,
        "n",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    )
    .with_edges(Edges { part_of: Some(id), ..Edges::default() });
    let doc = Document::new(fm, "body\n");
    let emitted = doc.emit().unwrap();

    // An empty `tears` must not invent the key (arc01/02 nodes have none).
    assert!(!emitted.contains("tears:"), "empty tears omitted; got:\n{emitted}");
    // And such a node round-trips byte-identically.
    let reparsed = Document::parse(&emitted).unwrap();
    assert_eq!(reparsed, doc);
    assert_eq!(reparsed.emit().unwrap(), emitted);
}

// ----- I-5 (proptest form): arbitrary unknown scalar keys survive ----------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn unknown_keys_preserved_proptest(
        extra_keys in prop::collection::hash_map(
            "[a-z_]{3,12}",
            prop_oneof![
                "[a-zA-Z0-9 ]{0,20}".prop_map(|s| format!("\"{s}\"")),
                any::<i64>().prop_map(|n| n.to_string()),
                any::<bool>().prop_map(|b| b.to_string()),
            ],
            0..5,
        )
    ) {
        // Skip keys that collide with modeled fields (now-typed `status`,
        // `retired`, `decomposed` included — a scalar value on those would fail
        // to deserialize into their typed shapes rather than land in `extra`).
        let modeled = ["id", "number", "type", "name", "created", "updated",
                       "tags", "component", "origin", "reserved", "retired",
                       "edges", "status", "decomposed"];
        let mut yaml = String::from(
            "---\nid: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nnumber: 1\ntype: note\nname: n\n\
             created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n",
        );
        let mut count = 0;
        for (k, v) in &extra_keys {
            if modeled.contains(&k.as_str()) {
                continue;
            }
            yaml.push_str(&format!("{k}: {v}\n"));
            count += 1;
        }
        yaml.push_str("---\nbody\n");

        let doc = Document::parse(&yaml)?;
        prop_assert_eq!(doc.frontmatter().unknown_key_count(), count);
        let reparsed = Document::parse(&doc.emit()?)?;
        prop_assert_eq!(reparsed, doc);
    }
}
