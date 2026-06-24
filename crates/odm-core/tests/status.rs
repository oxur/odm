//! Tests for the status vector, evidence levels, gate-sets, and `set-gate`.

use chrono::NaiveDate;
use odm_core::NodeType;
use odm_core::gates::{GateConfigError, GateSet, GateSets};
use odm_core::status::{Evidence, Status};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
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
