# Closing Report — Slice 05.1 (Arc 02): Evidence-transition dates

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 06 opens.

- **Implementation commit:** `5c238a8`.
- **Branch:** `arc02-slice05.1-evidence-dates` (based on
  `arc02-slice05-recomposition-integrity`; not pushed; not merged to `main`).
- **Scope delivered:** a contained, back-compatible `GateRecord` schema
  extension in odm-core — an optional `evidence_dates: BTreeMap<Evidence,
  NaiveDate>` recording the date each evidence level was *first* reached, plus
  the `set_gate` logic to populate it (first-reach per level, preserved across
  raises). Recording only; consumption is arc A7.
- **Result:** 8 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test --workspace` → all pass (5 new status tests
  incl. a populated-field round-trip proptest; slice03/04/05 unregressed);
  clippy `-D warnings` → exit 0; no `unsafe`; coverage TOTAL line 97.88% /
  region 97.28% (status.rs line 94.23%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `5c238a8`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-core gate_record_evidence_dates` → 1 passed; `evidence_dates` added, `reached`/`by`/`evidence` asserted unchanged. |
| H-2 | done | `cargo test -p odm-core set_gate_records_level_date` → 1 passed; first reach records `{level: date}`. |
| H-3 | done | `cargo test -p odm-core raise_preserves_prior_level_dates` → 1 passed; `attested@D1` then `reproduced@D2` ⇒ both dates kept. |
| H-4 | done | `cargo test -p odm-core resetting_same_level_keeps_first_date` → 1 passed; same level re-recorded keeps the original date. |
| H-5 | done | `cargo test -p odm-core back_compat_no_evidence_dates_roundtrip` → 1 (pre-field YAML re-emits byte-identical, no `evidence_dates` key) + `evidence_dates_roundtrip_identity` (128-case proptest). |
| H-6 | done | `cargo test --workspace` → all green; `reached`/`evidence`/terminal-gate/satisfaction semantics untouched. |
| H-7 | done | clippy `-D warnings` → exit 0; `! grep '\bunsafe\b' crates/odm-core/src` → no match; `cargo fmt --check` clean. |
| H-8 | done | `cargo llvm-cov -p odm-core … --ignore-filename-regex '(odm-graph\|odm-store\|odm-cli)/'` → TOTAL line 97.88% (status.rs 94.23%), from a clean `target/llvm-cov-target`. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Evidence regression is recorded, not flagged.** The spec scopes this slice
   to "first-reach per level only" and defers regression. With `entry().
   or_insert()`, recording a *lower* level after a higher one (e.g. `reproduced`
   then `attested`) simply adds the lower level's first-reach date and lets
   `evidence`/`reached` follow it down — the engine neither detects nor prevents
   a downgrade. This matches the slice-doc ("record first-reach only; regression
   handling deferred"); flagging it so CDC confirms the non-goal rather than
   reading it as a gap. A7 owns regression semantics.

2. **`reached` keeps its overwrite behavior; `evidence_dates` is the new home for
   history.** Slice03's `reached` is overwritten on a raise, and I left that
   exactly as-is to avoid consumer breakage (H-6). The consequence: `reached` is
   "current reach date," not "first reach of the gate" — the durable
   first-reach-per-level timing now lives in `evidence_dates`. The first-reach
   date of the gate as a whole is the minimum value in that map. Flagging the
   (intended) semantic split between the two fields.

3. **Map keys are `Evidence`, serialized via its existing lowercase string
   form.** `BTreeMap<Evidence, NaiveDate>` orders the wire output by evidence
   level (asserted → reconciled) and reuses `Evidence`'s `Serialize`/`Deserialize`
   as a string map key — no new wire vocabulary, deterministic ordering. The
   on-disk shape is `evidence_dates: { attested: <date>, reproduced: <date> }`
   nested under the gate record; §2.3's example does not show this field (it is
   the additive capture this slice introduces, per telemetry §6).

## Uncertainties named

- **No `set-gate` CLI flag yet.** Recording happens through
  `Status::set_gate(...)`; the CLI `set-gate` surface (and any flag to pass an
  explicit transition date rather than "today") is wired separately, out of
  scope here.
- **Day granularity.** Transition dates are `NaiveDate`, matching `reached`.
  Sub-day verification-latency (the git precision clock, telemetry §6) is a
  separate signal A7 derives from git, not from this field.
- **`status.rs` residue (~6%).** The uncovered lines are pre-existing trivial
  accessors; the new `set_gate`/`evidence_dates` paths are exercised. TOTAL line
  97.88% clears the bar.
- **Sandbox/CI parity.** All cargo evidence was produced on a local toolchain;
  CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target`.
