//! Tests for the frontmatter schema and its round-trip invariant.
//!
//! Integration tests (public API only), so library `src/` stays panic-free.
//! Test names contain the substrings the ledger Verify commands filter on
//! (`frontmatter_parse`, `schema_core_fields`, `schema_edges_block`,
//! `frontmatter_roundtrip`, `unknown_keys_preserved`, `canonical_field_order`,
//! `supersedes_kind`).

use std::str::FromStr;

use chrono::NaiveDate;
use odm_core::desired::{DesiredFact, FileExpect, ProbeSpec, ShellExpect};
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
         type: design\n\
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
    assert_eq!(fm.node_type(), NodeType::Design);
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

#[test]
fn insert_extra_carries_an_unmodeled_key_through_roundtrip() {
    // `odm migrate` uses this to carry a legacy `author` (no typed field).
    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let mut fm = Frontmatter::new(
        id,
        1,
        NodeType::Design,
        "Doc",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    );
    assert_eq!(fm.unknown_key_count(), 0);
    fm.insert_extra("author", "Ada Lovelace");
    assert_eq!(fm.unknown_key_count(), 1);

    let doc = Document::new(fm, "body\n");
    let emitted = doc.emit().unwrap();
    assert!(emitted.contains("author: Ada Lovelace"), "extra key emitted:\n{emitted}");
    // Round-trips: the unmodeled key survives parse ∘ emit.
    assert_eq!(Document::parse(&emitted).unwrap(), doc);
}

// ----- arc06 slice03 (ODD-0020): the schema marker -------------------------

#[test]
fn schema_marker_round_trip() {
    use odm_core::schema::{SchemaMarker, SchemaVersion};

    let id = Id::from_str(SAMPLE_ULID).unwrap();
    let mut fm = Frontmatter::new(
        id,
        1,
        NodeType::Design,
        "Doc",
        day(2026, 7, 6),
        day(2026, 7, 6),
        Origin::Planned,
    );
    fm.stamp_schema();
    assert_eq!(fm.schema(), Some(SchemaMarker::current(NodeType::Design)));
    assert_eq!(fm.schema_version(), SchemaVersion::CURRENT);

    let doc = Document::new(fm, "body\n");
    let emitted = doc.emit().unwrap();
    assert!(emitted.contains("schema: design/v1.0"), "schema marker emitted:\n{emitted}");
    // Additive round-trip: parse ∘ emit is identity.
    assert_eq!(Document::parse(&emitted).unwrap(), doc);
}

#[test]
fn schema_absent_is_v0_1() {
    use odm_core::schema::SchemaVersion;

    // A node with no `schema:` field reads as v0.1 (computed, not written).
    let text = format!(
        "---\n\
         id: {SAMPLE_ULID}\n\
         number: 3\n\
         type: design\n\
         name: Legacy\n\
         created: 2026-06-20\n\
         updated: 2026-06-20\n\
         origin: planned\n\
         ---\n\
         # Legacy\n"
    );
    let doc = Document::parse(&text).unwrap();
    assert_eq!(doc.frontmatter().schema(), None);
    assert_eq!(doc.frontmatter().schema_version(), SchemaVersion::LEGACY);
    // Absent stays absent on emit (no fabricated schema on a legacy node).
    assert!(!doc.emit().unwrap().contains("schema:"), "no schema written when absent");
}

// ----- I-5: unknown keys preserved -----------------------------------------

#[test]
fn unknown_keys_preserved_through_roundtrip() {
    // `status` became a typed field in arc02 slice04 (and `desired_facts` in
    // arc05 slice01), so neither is an "unknown" key any longer. A genuinely
    // unmodeled key (`provenance_note`) must still survive verbatim. Both still
    // round-trip — one typed, one preserved.
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
         provenance_note: hand-authored\n\
         ---\nbody\n"
    );
    let doc = Document::parse(&text).expect("valid");
    // Only `provenance_note` is unknown now; `status` is typed.
    assert_eq!(doc.frontmatter().unknown_key_count(), 1);
    assert!(doc.frontmatter().status().has_reached("built"));

    let reparsed = Document::parse(&doc.emit().expect("emit")).expect("reparse");
    assert_eq!(reparsed, doc, "typed status and unknown keys both survive a round-trip");
    // Both keys are still literally present after emission.
    let emitted = doc.emit().expect("emit");
    assert!(emitted.contains("status:"));
    assert!(emitted.contains("provenance_note: hand-authored"));
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
        Just(NodeType::Design),
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
                       "edges", "status", "decomposed", "desired_facts", "deferred"];
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

// ----- F-1 / F-2 (arc05 slice01): desired_facts schema + positioned errors ---

fn shell_fact(id: &str, describe: &str, run: &str, exit: i32, stdout: Option<&str>) -> DesiredFact {
    DesiredFact {
        id: id.to_string(),
        describe: describe.to_string(),
        probe: ProbeSpec::Shell {
            run: run.to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit, stdout_contains: stdout.map(str::to_string) },
        },
    }
}

fn doc_with_facts(node_type: NodeType, facts: Vec<DesiredFact>) -> Document {
    let fm = Frontmatter::new(
        Id::from_str(SAMPLE_ULID).unwrap(),
        1,
        node_type,
        "n",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    )
    .with_desired_facts(facts);
    Document::new(fm, "body\n")
}

#[test]
fn desired_facts_round_trip() {
    // A project node carrying program-level acceptance facts (no separate
    // layer — arc-plan v1.3): one fact checks exit only, one also matches stdout.
    let project = doc_with_facts(
        NodeType::Project,
        vec![
            shell_fact(
                "db-reachable",
                "the prod DB answers a trivial query",
                "pg_isready -h prod -t 2",
                0,
                None,
            ),
            shell_fact(
                "git-clean",
                "the working tree is clean",
                "git status --porcelain",
                0,
                Some(""),
            ),
        ],
    );
    // A slice node carrying a single fact — the model is uniform across types.
    let slice = doc_with_facts(
        NodeType::Slice,
        vec![shell_fact("built", "the crate builds", "cargo build", 0, None)],
    );
    // The default: a node that declares nothing.
    let empty = doc_with_facts(NodeType::Note, vec![]);

    for doc in [&project, &slice, &empty] {
        // parse ∘ emit == identity, on every node type.
        let reparsed = Document::parse(&doc.emit().expect("emit")).expect("parse");
        assert_eq!(&reparsed, doc);
    }

    // Accessor reflects the declared facts and the empty default.
    assert_eq!(project.frontmatter().desired_facts().len(), 2);
    assert_eq!(project.frontmatter().desired_facts()[0].id, "db-reachable");
    assert!(slice.frontmatter().desired_facts().len() == 1);
    assert!(empty.frontmatter().desired_facts().is_empty());

    // Empty is skipped on emit (a fact-free node gains no `desired_facts:` key);
    // a populated node emits the canonical `kind: shell` shape.
    assert!(!empty.emit().unwrap().contains("desired_facts"));
    let emitted = project.emit().unwrap();
    assert!(emitted.contains("desired_facts:"));
    assert!(emitted.contains("kind: shell"));

    // The proposed YAML shape (slice-doc) parses into the typed model.
    let text = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: arc\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n\
         desired_facts:\n  - id: db-reachable\n    describe: \"the prod DB answers a trivial query\"\n\
         \x20   probe:\n      kind: shell\n      run: \"pg_isready -h prod -t 2\"\n\
         \x20     expect:\n        exit: 0\n---\nbody\n"
    );
    let parsed = Document::parse(&text).expect("proposed shape parses");
    match &parsed.frontmatter().desired_facts()[0].probe {
        ProbeSpec::Shell { run, expect, .. } => {
            assert_eq!(run, "pg_isready -h prod -t 2");
            assert_eq!(expect.exit, 0);
            assert_eq!(expect.stdout_contains, None);
        }
        ProbeSpec::File { .. } => panic!("expected a shell probe"),
    }
}

#[test]
fn desired_facts_malformed_errors_with_position() {
    let base = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: project\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n"
    );
    // An unknown probe `kind`, a missing required field (`run`), and an empty
    // `id` are each a parse error that carries position — never a panic or a
    // silent drop.
    let unknown_kind = format!(
        "{base}desired_facts:\n  - id: f1\n    describe: d\n    probe:\n      kind: bogus\n\
         \x20     run: \"true\"\n      expect:\n        exit: 0\n---\nbody\n"
    );
    let missing_run = format!(
        "{base}desired_facts:\n  - id: f1\n    describe: d\n    probe:\n      kind: shell\n\
         \x20     expect:\n        exit: 0\n---\nbody\n"
    );
    let empty_id = format!(
        "{base}desired_facts:\n  - id: \"\"\n    describe: d\n    probe:\n      kind: shell\n\
         \x20     run: \"true\"\n      expect:\n        exit: 0\n---\nbody\n"
    );

    for (label, text) in
        [("unknown_kind", unknown_kind), ("missing_run", missing_run), ("empty_id", empty_id)]
    {
        match Document::parse(&text) {
            Err(FrontmatterError::Yaml(msg)) => {
                assert!(
                    msg.contains("line") && msg.contains("column"),
                    "{label}: error must carry a position, got: {msg}"
                );
            }
            other => panic!("{label}: expected a positioned Yaml error, got {other:?}"),
        }
    }

    // The empty-id error names the offending field (not a generic YAML fault).
    let empty_id_again = format!(
        "{base}desired_facts:\n  - id: \"  \"\n    describe: d\n    probe:\n      kind: shell\n\
         \x20     run: \"true\"\n      expect:\n        exit: 0\n---\nbody\n"
    );
    match Document::parse(&empty_id_again) {
        Err(FrontmatterError::Yaml(msg)) => assert!(msg.contains("`id` must not be empty")),
        other => panic!("expected empty-id error, got {other:?}"),
    }
}

// ----- G-1 / G-2 (arc05 slice02): `file` probe spec parse + positioned errors -

#[test]
fn file_probe_spec_round_trip() {
    // A file fact with every expectation declared round-trips through the serde
    // layer, on any node type.
    let fact = DesiredFact {
        id: "schema-present".to_string(),
        describe: "the schema file is intact".to_string(),
        probe: ProbeSpec::File {
            path: "db/schema.sql".to_string(),
            expect: FileExpect { exists: true, sha256: Some("a".repeat(64)), size: Some(2048) },
        },
    };
    let doc = doc_with_facts(NodeType::Project, vec![fact]);
    let reparsed = Document::parse(&doc.emit().expect("emit")).expect("parse");
    assert_eq!(reparsed, doc);
    let emitted = doc.emit().unwrap();
    assert!(emitted.contains("kind: file"));

    // The minimal source form — `kind: file` + `path`, no `expect` — defaults to
    // "the file should exist".
    let text = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: arc\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n\
         desired_facts:\n  - id: present\n    describe: d\n    probe:\n\
         \x20     kind: file\n      path: README.md\n---\nbody\n"
    );
    let parsed = Document::parse(&text).expect("minimal file spec parses");
    match &parsed.frontmatter().desired_facts()[0].probe {
        ProbeSpec::File { path, expect } => {
            assert_eq!(path, "README.md");
            assert!(expect.exists);
            assert_eq!(expect.sha256, None);
            assert_eq!(expect.size, None);
        }
        ProbeSpec::Shell { .. } => panic!("expected a file probe"),
    }
}

#[test]
fn file_probe_spec_malformed_errors_with_position() {
    let base = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: project\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n"
    );
    // Missing required `path`, a non-hex `sha256`, and an unknown `expect`
    // sub-field are each a positioned parse error (slice01 F-2 consistency).
    let missing_path = format!(
        "{base}desired_facts:\n  - id: f\n    describe: d\n    probe:\n      kind: file\n\
         \x20     expect:\n        exists: true\n---\nbody\n"
    );
    let bad_sha = format!(
        "{base}desired_facts:\n  - id: f\n    describe: d\n    probe:\n      kind: file\n\
         \x20     path: x\n      expect:\n        sha256: \"not-a-valid-hash\"\n---\nbody\n"
    );
    let unknown_field = format!(
        "{base}desired_facts:\n  - id: f\n    describe: d\n    probe:\n      kind: file\n\
         \x20     path: x\n      expect:\n        bogus: 1\n---\nbody\n"
    );

    for (label, text) in [
        ("missing_path", missing_path),
        ("bad_sha", bad_sha.clone()),
        ("unknown_field", unknown_field),
    ] {
        match Document::parse(&text) {
            Err(FrontmatterError::Yaml(msg)) => assert!(
                msg.contains("line") && msg.contains("column"),
                "{label}: error must carry a position, got: {msg}"
            ),
            other => panic!("{label}: expected a positioned Yaml error, got {other:?}"),
        }
    }

    // The bad-sha256 error names the field (a semantic check, not a generic fault).
    match Document::parse(&bad_sha) {
        Err(FrontmatterError::Yaml(msg)) => assert!(msg.contains("`sha256` must be 64")),
        other => panic!("expected sha256 error, got {other:?}"),
    }
}

#[test]
fn file_expect_defaults_exists_and_accepts_null_sha256() {
    // `expect` present but `exists` omitted → defaults to true (the serde default).
    let valid = "a".repeat(64);
    let text = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: arc\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n\
         desired_facts:\n  - id: f\n    describe: d\n    probe:\n      kind: file\n\
         \x20     path: x\n      expect:\n        sha256: {valid}\n---\nbody\n"
    );
    match &Document::parse(&text).expect("parses").frontmatter().desired_facts()[0].probe {
        ProbeSpec::File { expect, .. } => {
            assert!(expect.exists, "exists defaults to true when omitted");
            assert_eq!(expect.sha256.as_deref(), Some(valid.as_str()));
        }
        ProbeSpec::Shell { .. } => panic!("expected a file probe"),
    }

    // An explicit `sha256: null` deserializes to None (the optional-null branch).
    let text_null = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: arc\nname: n\n\
         created: 2026-06-20\nupdated: 2026-06-20\norigin: planned\nreserved: false\n\
         desired_facts:\n  - id: f\n    describe: d\n    probe:\n      kind: file\n\
         \x20     path: x\n      expect:\n        exists: false\n        sha256: null\n---\nbody\n"
    );
    match &Document::parse(&text_null).expect("parses").frontmatter().desired_facts()[0].probe {
        ProbeSpec::File { expect, .. } => {
            assert!(!expect.exists);
            assert_eq!(expect.sha256, None);
        }
        ProbeSpec::Shell { .. } => panic!("expected a file probe"),
    }
}

// ----- D-1 (arc05 slice06): the deferred marker round-trips ------------------

#[test]
fn deferred_marker_round_trip() {
    use odm_core::frontmatter::Deferral;

    // A node with a deferred marker referencing one of its own facts.
    let fact = DesiredFact {
        id: "db-back".to_string(),
        describe: "the prod DB is reachable again".to_string(),
        probe: ProbeSpec::Shell {
            run: "true".to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    let fm = Frontmatter::new(
        Id::from_str(SAMPLE_ULID).unwrap(),
        1,
        NodeType::Slice,
        "Parked",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    )
    .with_desired_facts(vec![fact])
    .with_deferred(Some(Deferral {
        because: "blocked on the prod outage".to_string(),
        reenter_when: "db-back".to_string(),
    }));
    let doc = Document::new(fm, "body\n");

    let reparsed = Document::parse(&doc.emit().expect("emit")).expect("parse");
    assert_eq!(reparsed, doc);
    let deferral = reparsed.frontmatter().deferred().expect("deferred present");
    assert_eq!(deferral.because, "blocked on the prod outage");
    assert_eq!(deferral.reenter_when, "db-back");
    assert!(doc.emit().unwrap().contains("deferred:"));

    // Absent ⇒ not deferred (default), and no `deferred:` key on emit.
    let plain = Frontmatter::new(
        Id::from_str(SAMPLE_ULID).unwrap(),
        2,
        NodeType::Slice,
        "Active",
        day(2026, 6, 20),
        day(2026, 6, 20),
        Origin::Planned,
    );
    let plain_doc = Document::new(plain, "body\n");
    assert!(plain_doc.frontmatter().deferred().is_none());
    assert!(!plain_doc.emit().unwrap().contains("deferred:"));
    assert_eq!(Document::parse(&plain_doc.emit().unwrap()).unwrap(), plain_doc);
}

// ----- K-1 (arc05 slice07): probe `inputs` + freshness class -----------------

#[test]
fn probe_inputs_round_trip() {
    use odm_core::ProbeSpec;

    // A shell probe declaring inputs round-trips; absent `inputs` ⇒ empty (default).
    let text = format!(
        "---\nid: {SAMPLE_ULID}\nnumber: 1\ntype: slice\nname: n\n\
         created: 2026-07-01\nupdated: 2026-07-01\norigin: planned\nreserved: false\n\
         desired_facts:\n  - id: schema\n    describe: d\n    probe:\n      kind: shell\n\
         \x20     run: \"sha256sum build/schema.sql\"\n      inputs: [build/schema.sql]\n\
         \x20     expect:\n        exit: 0\n---\nbody\n"
    );
    let doc = Document::parse(&text).expect("parses");
    match &doc.frontmatter().desired_facts()[0].probe {
        ProbeSpec::Shell { inputs, .. } => assert_eq!(inputs, &["build/schema.sql"]),
        ProbeSpec::File { .. } => panic!("expected a shell probe"),
    }
    // Round-trips (emit → parse == identity).
    assert_eq!(Document::parse(&doc.emit().unwrap()).unwrap(), doc);
    assert!(doc.emit().unwrap().contains("inputs:"));

    // A shell probe with no `inputs` emits no `inputs:` key (skip-if-empty).
    let plain = DesiredFact {
        id: "db".to_string(),
        describe: "d".to_string(),
        probe: ProbeSpec::Shell {
            run: "true".to_string(),
            inputs: Vec::new(),
            expect: ShellExpect { exit: 0, stdout_contains: None },
        },
    };
    let pdoc = doc_with_facts(NodeType::Slice, vec![plain]);
    assert!(!pdoc.emit().unwrap().contains("inputs:"));
    assert_eq!(Document::parse(&pdoc.emit().unwrap()).unwrap(), pdoc);
}

#[test]
fn probe_class_from_inputs() {
    use odm_core::{ProbeClass, ProbeSpec};

    // `file` → input-derived (its path is the input).
    let file = ProbeSpec::File { path: "ROLLUP.md".to_string(), expect: FileExpect::default() };
    assert_eq!(file.class(), ProbeClass::InputDerived);
    assert_eq!(file.inputs(), vec!["ROLLUP.md"]);

    // `shell` + inputs → input-derived.
    let shell_in = ProbeSpec::Shell {
        run: "sha256sum x".to_string(),
        inputs: vec!["x".to_string()],
        expect: ShellExpect { exit: 0, stdout_contains: None },
    };
    assert_eq!(shell_in.class(), ProbeClass::InputDerived);
    assert_eq!(shell_in.inputs(), vec!["x"]);

    // `shell` w/o inputs → volatile (honest staleness, ODD-0019 default).
    let shell_vol = ProbeSpec::Shell {
        run: "pg_isready".to_string(),
        inputs: Vec::new(),
        expect: ShellExpect { exit: 0, stdout_contains: None },
    };
    assert_eq!(shell_vol.class(), ProbeClass::Volatile);
    assert!(shell_vol.inputs().is_empty());
}
