# Slice 03 (Arc 02): Gates, status & evidence recording

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | `Evidence` enum has a total order `asserted < attested < reproduced < reconciled` | `cargo test -p odm-core evidence_total_order` → ok | serious | 0013 §4.4 / 0001 D3 | done | `a1dce22`: `evidence_total_order` → 1 passed. Derived `Ord` follows declaration order; sorting `{reconciled, asserted, reproduced, attested}` yields the canonical order; `max` works. | |
| H-2 | Per-type gate-sets load from `odm.toml` `[gates.<type>]` | `cargo test -p odm-core gate_sets_from_config` → ok | serious | 0013 §5.1 | done | `a1dce22`: `gate_sets_from_config` → 1 passed (+ `gate_sets_reject_unknown_type_key`). `GateSets::from_toml_str` reads `[gates.slice]`/`[gates.arc]`, ignores other keys; unknown type key → `GateConfigError::UnknownType`. | Lives in odm-core (toml dep added). |
| H-3 | `set-gate` records `{reached, by, evidence}` on the node's status vector | `cargo test -p odm-core set_gate_records_evidence` → ok | serious | 0013 §5.1 / 0001 D3 | done | `a1dce22`: `set_gate_records_evidence` → 1 passed. `Status::set_gate` records `{reached, by, evidence}`; re-recording overwrites (raise evidence). | Operation is an odm-core method; CLI `set-gate` command wires later (see decision 2). |
| H-4 | A gate not in the node type's set is rejected | `cargo test -p odm-core set_gate_rejects_unknown` → ok | correctness | 0013 §5.1 | done | `a1dce22`: `set_gate_rejects_unknown` → 1 passed. A gate not in the slice set → `Err(UnknownGate{gate, allowed})`; nothing recorded. | |
| H-5 | Terminal-gate accessor returns the last gate in a type's sequence | `cargo test -p odm-core terminal_gate` → ok | correctness | 0013 §4.4 | done | `a1dce22`: `terminal_gate` → 1 passed. `terminal(Slice)=deployed`, `terminal(Arc)=verified`; empty sequence → `None`. | |
| H-6 | Status is a vector (multiple gates independently set), not a scalar | `cargo test -p odm-core status_is_multigate` → ok | serious | 0013 §5.1 / 0001 D1 | done | `a1dce22`: `status_is_multigate` → 1 passed. Two gates held independently with their own evidence; `len==2`; per-gate lookup. | `Status` is a `BTreeMap<gate, record>`, never a scalar. |
| H-7 | Default evidence is `asserted` when not specified (least-confident default) | `cargo test -p odm-core evidence_default_asserted` → ok | correctness | 0013 §4.4 | done | `a1dce22`: `evidence_default_asserted` → 1 passed. `Evidence::default() == Asserted` (the order's minimum); `GateRecord.evidence` defaults to asserted when absent (serde `#[serde(default)]`). | |
| H-8 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | done | `a1dce22`: clippy → exit 0; unsafe grep → no match. | |
| H-9 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | done | `a1dce22`: TOTAL **line 98.31%**, region 97.96% (gates.rs 100% line, status.rs 93.02% line). | Run from a clean `target/llvm-cov-target`. |

## What Worked

- **Total order for free via declaration order.** Declaring `Evidence`'s
  variants least- to most-confident and deriving `Ord` makes the canonical
  ordering a compiler-guaranteed property, and `#[default]` on the first variant
  makes `asserted` the least-confident default — H-1 and H-7 are both just
  derives, not hand-written comparisons that could drift.
- **Gate-set membership as a type invariant.** `Status::set_gate` takes the
  `&GateSet` and validates before recording, so an out-of-set gate (H-4) cannot
  be written — the rejection lives at the one mutation point rather than relying
  on callers to pre-check.
- **`#[serde(transparent)]` Status = the §2.3 shape.** Modeling `Status` as a
  transparent `BTreeMap<gate, record>` means it serializes to exactly the
  existing `status:` block, so the typed model *operates on* the arc01
  serialization without redefining it — proven by the YAML round-trip test.
- **Reusing `NodeType::from_str` for config keys** kept the `[gates.<type>]`
  parsing honest: an unknown type key is rejected with the same vocabulary the
  rest of the model uses, rather than silently accepted.

## Closure

Closed at `a1dce22` on 2026-06-24. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 9. Done: 9. Deferred: 0. No-op: 0.
