//! Tests for the status vector, evidence levels, gate-sets, and `set-gate`.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use odm_core::NodeType;
use odm_core::gates::{GateConfigError, GateSet, GateSets};
use odm_core::status::{Evidence, GateRecord, Status};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
}

/// A distinct date `n` days after [`day`], for ordering transition dates.
fn day_plus(n: i64) -> NaiveDate {
    day() + chrono::Duration::days(n)
}

const ODM_TOML: &str = "\
author_name = \"ignored\"

[gates.slice]
sequence = [\"planned\", \"built\", \"tested\", \"deployed\"]

[gates.arc]
sequence = [\"planned\", \"in-progress\", \"complete\", \"verified\"]
";

// ----- H-1: Evidence total order --------------------------------------------

#[test]
fn evidence_total_order() {
    use Evidence::{Asserted, Attested, Reconciled, Reproduced};
    assert!(Asserted < Attested);
    assert!(Attested < Reproduced);
    assert!(Reproduced < Reconciled);
    // Transitive / total: sorting a shuffled set yields the canonical order.
    let mut levels = [Reconciled, Asserted, Reproduced, Attested];
    levels.sort();
    assert_eq!(levels, [Asserted, Attested, Reproduced, Reconciled]);
    assert_eq!(Reconciled.max(Asserted), Reconciled);
}

// ----- H-7: default evidence is asserted (least-confident) ------------------

#[test]
fn evidence_default_asserted() {
    assert_eq!(Evidence::default(), Evidence::Asserted);
    assert_eq!(Evidence::default().as_str(), "asserted");
    // It is the minimum of the order.
    assert!(Evidence::default() <= Evidence::Reconciled);
}

// ----- H-2: gate-sets load from odm.toml ------------------------------------

#[test]
fn gate_sets_from_config() {
    let sets = GateSets::from_toml_str(ODM_TOML).expect("valid config");
    assert_eq!(sets.len(), 2);
    let slice = sets.for_type(NodeType::Slice).expect("slice gates");
    assert_eq!(slice.sequence(), ["planned", "built", "tested", "deployed"]);
    let arc = sets.for_type(NodeType::Arc).expect("arc gates");
    assert_eq!(arc.sequence().first().map(String::as_str), Some("planned"));
    // A type with no [gates.<type>] table has no set.
    assert!(sets.for_type(NodeType::Note).is_none());
}

#[test]
fn gate_sets_reject_unknown_type_key() {
    let bad = "[gates.widget]\nsequence = [\"a\"]\n";
    assert_eq!(
        GateSets::from_toml_str(bad),
        Err(GateConfigError::UnknownType("widget".to_string()))
    );
    // Malformed TOML → a Toml error.
    assert!(matches!(
        GateSets::from_toml_str("[gates.slice]\n= bad"),
        Err(GateConfigError::Toml(_))
    ));
    // No [gates] at all → empty, not an error.
    assert!(GateSets::from_toml_str("author_name = \"x\"").unwrap().is_empty());
}

// ----- H-5: terminal gate ---------------------------------------------------

#[test]
fn terminal_gate() {
    let sets = GateSets::from_toml_str(ODM_TOML).unwrap();
    assert_eq!(sets.terminal(NodeType::Slice), Some("deployed"));
    assert_eq!(sets.terminal(NodeType::Arc), Some("verified"));
    assert_eq!(sets.terminal(NodeType::Note), None);
    // An empty sequence has no terminal gate.
    assert_eq!(GateSet::new(vec![]).terminal(), None);
}

// ----- H-3: set-gate records {reached, by, evidence} ------------------------

#[test]
fn set_gate_records_evidence() {
    let sets = GateSets::from_toml_str(ODM_TOML).unwrap();
    let slice = sets.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    status
        .set_gate(slice, "built", Some("duncan".to_string()), Evidence::Reproduced, day())
        .expect("built is a valid gate");

    let record = status.gate("built").expect("recorded");
    assert_eq!(record.reached, day());
    assert_eq!(record.by.as_deref(), Some("duncan"));
    assert_eq!(record.evidence, Evidence::Reproduced);

    // Re-recording overwrites (e.g. raising the evidence level).
    status.set_gate(slice, "built", None, Evidence::Reconciled, day()).unwrap();
    assert_eq!(status.gate("built").unwrap().evidence, Evidence::Reconciled);
    assert_eq!(status.gate("built").unwrap().by, None);
}

// ----- H-4: a gate not in the type's set is rejected ------------------------

#[test]
fn set_gate_rejects_unknown() {
    let sets = GateSets::from_toml_str(ODM_TOML).unwrap();
    let slice = sets.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    let err = status
        .set_gate(slice, "verified-live", None, Evidence::Asserted, day())
        .expect_err("not a slice gate");
    assert_eq!(err.gate, "verified-live");
    assert!(err.allowed.contains(&"built".to_string()));
    assert!(status.is_empty(), "nothing recorded on rejection");
}

// ----- H-6: status is a vector, not a scalar --------------------------------

#[test]
fn status_is_multigate() {
    let sets = GateSets::from_toml_str(ODM_TOML).unwrap();
    let slice = sets.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    status.set_gate(slice, "planned", None, Evidence::Asserted, day()).unwrap();
    status.set_gate(slice, "built", None, Evidence::Reproduced, day()).unwrap();

    // Multiple gates are held independently — not a single scalar state.
    assert_eq!(status.len(), 2);
    assert!(status.has_reached("planned") && status.has_reached("built"));
    assert!(!status.has_reached("tested"));
    assert_eq!(status.gate("planned").unwrap().evidence, Evidence::Asserted);
    assert_eq!(status.gate("built").unwrap().evidence, Evidence::Reproduced);
    let reached: Vec<&str> = status.reached().map(|(g, _)| g).collect();
    assert_eq!(reached, ["built", "planned"]); // gate-name order
}

// ----- wire-compatibility with the §2.3 status shape ------------------------

#[test]
fn status_round_trips_through_yaml() {
    let sets = GateSets::from_toml_str(ODM_TOML).unwrap();
    let slice = sets.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();
    status
        .set_gate(slice, "built", Some("duncan".to_string()), Evidence::Reproduced, day())
        .unwrap();
    status.set_gate(slice, "tested", None, Evidence::Reconciled, day()).unwrap();

    // Serializes to the §2.3 `status:` shape (map of gate → record) and back.
    let yaml = serde_norway::to_string(&status).unwrap();
    assert!(yaml.contains("built:") && yaml.contains("evidence: reproduced"));
    let parsed: Status = serde_norway::from_str(&yaml).unwrap();
    assert_eq!(parsed, status);
}

// ===== slice05.1: evidence-transition dates =================================

const SLICE_GATES: &str = "[gates.slice]\nsequence = [\"planned\", \"built\", \"tested\"]\n";

fn slice_gates() -> GateSets {
    GateSets::from_toml_str(SLICE_GATES).unwrap()
}

// ----- H-1: GateRecord carries the optional first-reached date map ----------

#[test]
fn gate_record_evidence_dates() {
    let gates = slice_gates();
    let slice = gates.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    status.set_gate(slice, "built", Some("ci".to_string()), Evidence::Attested, day()).unwrap();

    let record = status.gate("built").unwrap();
    // reached / by / evidence are exactly as before.
    assert_eq!(record.reached, day());
    assert_eq!(record.by.as_deref(), Some("ci"));
    assert_eq!(record.evidence, Evidence::Attested);
    // The new map carries the reached level's first-reached date.
    assert_eq!(record.evidence_dates, BTreeMap::from([(Evidence::Attested, day())]));
}

// ----- H-2: set_gate records the reached level's date on first reach --------

#[test]
fn set_gate_records_level_date() {
    let gates = slice_gates();
    let slice = gates.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    status.set_gate(slice, "tested", None, Evidence::Reproduced, day_plus(5)).unwrap();

    assert_eq!(
        status.gate("tested").unwrap().evidence_dates,
        BTreeMap::from([(Evidence::Reproduced, day_plus(5))])
    );
}

// ----- H-3: a raise preserves earlier levels' dates (the point of the slice) -

#[test]
fn raise_preserves_prior_level_dates() {
    let gates = slice_gates();
    let slice = gates.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    // attested @ D1, then raised to reproduced @ D2.
    status.set_gate(slice, "built", None, Evidence::Attested, day_plus(1)).unwrap();
    status.set_gate(slice, "built", None, Evidence::Reproduced, day_plus(2)).unwrap();

    let record = status.gate("built").unwrap();
    // Current reach reflects the raise...
    assert_eq!(record.evidence, Evidence::Reproduced);
    assert_eq!(record.reached, day_plus(2));
    // ...but both levels' first-reached dates survive (no overwrite).
    assert_eq!(
        record.evidence_dates,
        BTreeMap::from([(Evidence::Attested, day_plus(1)), (Evidence::Reproduced, day_plus(2)),])
    );
}

// ----- H-4: re-recording the same level keeps its original first-reach date -

#[test]
fn resetting_same_level_keeps_first_date() {
    let gates = slice_gates();
    let slice = gates.for_type(NodeType::Slice).unwrap();
    let mut status = Status::new();

    status.set_gate(slice, "built", None, Evidence::Attested, day_plus(1)).unwrap();
    // Same level re-recorded at a later date: the first-reach date is kept.
    status
        .set_gate(slice, "built", Some("ci".to_string()), Evidence::Attested, day_plus(3))
        .unwrap();

    let record = status.gate("built").unwrap();
    assert_eq!(record.evidence, Evidence::Attested);
    // `reached`/`by` follow the latest call (unchanged behavior)...
    assert_eq!(record.reached, day_plus(3));
    assert_eq!(record.by.as_deref(), Some("ci"));
    // ...but the level's first-reached date is the original D1.
    assert_eq!(record.evidence_dates, BTreeMap::from([(Evidence::Attested, day_plus(1))]));
}

// ----- H-5: back-compat — omitted when empty; round-trips identically -------

#[test]
fn back_compat_no_evidence_dates_roundtrip() {
    // A pre-slice05.1 record (no `evidence_dates` key) parses with an empty map.
    let old = "\
built:
  reached: 2026-06-12
  by: duncan
  evidence: reproduced
tested:
  reached: 2026-06-13
  evidence: reconciled
";
    let parsed: Status = serde_norway::from_str(old).unwrap();
    assert!(parsed.gate("built").unwrap().evidence_dates.is_empty());

    // Emitting must NOT invent the field, and must reproduce the input verbatim.
    let emitted = serde_norway::to_string(&parsed).unwrap();
    assert!(!emitted.contains("evidence_dates"), "empty map is omitted on the wire");
    assert_eq!(emitted, old, "byte-identical round-trip for a pre-field node");

    // A directly-built record with an empty map also omits the field on the
    // wire (Status serializes transparently as a map of gate → GateRecord).
    let record = GateRecord {
        reached: day(),
        by: None,
        evidence: Evidence::Asserted,
        evidence_dates: BTreeMap::new(),
    };
    assert!(!serde_norway::to_string(&record).unwrap().contains("evidence_dates"));
}

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    // With the field populated, a Status round-trips through YAML unchanged
    // (parse ∘ emit = identity over an arbitrary sequence of gate reaches).
    #[test]
    fn evidence_dates_roundtrip_identity(
        reaches in prop::collection::vec(
            (
                prop_oneof![Just("planned"), Just("built"), Just("tested")],
                prop_oneof![
                    Just(Evidence::Asserted),
                    Just(Evidence::Attested),
                    Just(Evidence::Reproduced),
                    Just(Evidence::Reconciled),
                ],
                0i64..30,
            ),
            1..12,
        )
    ) {
        let gates = slice_gates();
        let slice = gates.for_type(NodeType::Slice).unwrap();
        let mut status = Status::new();
        for (gate, evidence, offset) in reaches {
            status.set_gate(slice, gate, None, evidence, day_plus(offset)).unwrap();
        }
        let yaml = serde_norway::to_string(&status).unwrap();
        let parsed: Status = serde_norway::from_str(&yaml).unwrap();
        prop_assert_eq!(parsed, status);
    }
}
