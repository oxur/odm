# Closing Report — Slice 03 (Arc 02): Gates, status & evidence recording

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 04 opens.

- **Implementation commit:** `a1dce22`.
- **Branch:** `arc02-slice03-gates` (based on `arc02-slice02-cycles`; not pushed;
  not merged to `main`).
- **Scope delivered:** the recording half of evidence-leveled status, all in
  odm-core — `Evidence` (total order), `GateSets` from `odm.toml`, `Status` +
  `set-gate`, terminal-gate accessor. slice04 consumes `Evidence` + terminal.
- **Result:** 9 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-core` → all pass (9 new status/gate
  tests; arc01/02 tests unregressed); clippy `-D warnings` → exit 0; no
  `unsafe`; coverage TOTAL line 98.31% / region 97.96% (gates.rs 100%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `a1dce22`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-core evidence_total_order` → 1 passed; derived `Ord`, sort yields canonical order. |
| H-2 | done | `cargo test -p odm-core gate_sets_from_config` → 1 passed; `[gates.<type>]` parsed; unknown type key rejected. |
| H-3 | done | `cargo test -p odm-core set_gate_records_evidence` → 1 passed; `{reached, by, evidence}` recorded. |
| H-4 | done | `cargo test -p odm-core set_gate_rejects_unknown` → 1 passed; out-of-set gate → `UnknownGate`, nothing recorded. |
| H-5 | done | `cargo test -p odm-core terminal_gate` → 1 passed; terminal = last gate; empty → `None`. |
| H-6 | done | `cargo test -p odm-core status_is_multigate` → 1 passed; two gates held independently. |
| H-7 | done | `cargo test -p odm-core evidence_default_asserted` → 1 passed; `Evidence::default() == Asserted`. |
| H-8 | done | clippy exit 0; `! grep '\bunsafe\b' crates/odm-core/src` → no match. |
| H-9 | done | `cargo llvm-cov -p odm-core … --ignore-filename-regex '(odm-graph\|odm-store\|odm-cli)/'` → line 98.31%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Typed `Status` is defined standalone; `Frontmatter` is NOT modified.** The
   slice-doc lists status *serialization* as out of scope ("already in the arc01
   schema — this slice operates on it") and `depends_on: arc01 (the node + status
   schema field)`. In fact arc01-slice03 **deliberately** left `status` as a
   *preserved unknown key* (in the frontmatter `extra` map), explicitly deferring
   the typed model to here. So I built the typed model — `Evidence`/`GateRecord`/
   `Status` — and proved it serializes to the exact §2.3 `status:` shape (the
   `status_round_trips_through_yaml` test), but I did **not** wire it into the
   `Frontmatter` struct. Reasons: (a) the ledger is entirely odm-core type/logic
   with no frontmatter row; (b) wiring it would move `status` out of `extra` and
   change the closed arc01 `unknown_keys_preserved` test (count 2→1). The
   field-wiring (replacing the extra-preservation, and updating that arc01 test)
   belongs with the slice that persists `set-gate` via the store/CLI. **Flagging**
   the small doc mismatch ("status schema field" implies a typed field that arc01
   never added) and the deferral of the Frontmatter integration.

2. **`set-gate` is an odm-core operation, not (yet) a CLI command.** H-3's verify
   is `cargo test -p odm-core set_gate_records_evidence`, so `set-gate` is
   implemented as `Status::set_gate(&GateSet, …)`. The `odm set-gate <node> …`
   CLI command (§7) is not in this ledger; it wires when status is persisted on
   the node (per decision 1). The `--by`/`--evidence` flags and the
   "default evidence asserted" are modeled as the `by: Option`, `evidence` param,
   and `Evidence::default()` here; the CLI arg-defaulting lands with the command.

3. **Gate-set config parsing lives in odm-core (added a `toml` dep).** Because
   H-2 is verified in odm-core (`gate_sets_from_config`), `GateSets::from_toml_str`
   parses the `[gates]` table directly in odm-core rather than going through
   odm-store's `StoreConfig`. odm-store will likely call this when it loads
   `odm.toml`; for now odm-core owns the gate-set type and its parsing. Flagging
   the new `toml` dependency on odm-core.

4. **`Status` uses a `BTreeMap` (gate-name order), not gate-sequence order.**
   On-disk, gates serialize in name order, which may differ from the configured
   sequence order shown in §2.3 (built before tested). This keeps round-trip
   deterministic and model-equality stable; the *sequence* order is available
   from the `GateSet` when display needs it. Flagging the serialization-order
   choice.

## Uncertainties named

- **Status is not yet attached to a node.** Until decision (1)'s integration
  lands, a `Status` is a free-standing value; nothing reads/writes it on a
  persisted node. The YAML round-trip test demonstrates wire-compatibility, but
  end-to-end "set a gate on node X and reload it" is a later slice.
- **No gate *ordering* enforcement on `set-gate`.** This slice records any gate
  in the set; it does not require reaching gates in sequence (e.g. `tested`
  before `deployed`) or check monotonic evidence. Out-of-order / staleness is
  §4.4's *consuming* half (slice04). `set-gate` here is purely the recorder.
- **`status.rs` residue (~7%).** The few uncovered lines are trivial accessors on
  rarely-hit paths; TOTAL line 98.31% clears the bar comfortably.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target`.
