# Slice 05.1 (Arc 02) — Evidence-transition dates (plan-of-record)

> Refs: ODD-0013 §2.3 (status schema) + §4.4 (evidence levels). `depends_on:`
> **slice03** (`GateRecord` + `Status::set_gate` + `Evidence`) — independent of
> slice04/05; lands before slice06 (check v2). Motivation:
> `workbench/forecasting-telemetry.md` §6 — the two-clock telemetry's
> verification-latency signal, a record-now-or-lose-it schema capture.

## Why this slice exists (now)

`GateRecord` records one `reached` date plus the *current* `evidence` level, and
`Status::set_gate` **overwrites** the record when evidence is raised (e.g. CI flips
`attested → reproduced`). So the **date a gate first reached each evidence level**
is discarded. That timing *is* the **verification-latency / evidence-churn** signal
the post-arc6 telemetry thread needs, and it cannot be recovered later if not
captured now — git-history reconstruction is fragile under squash-merge / rebase.
This slice makes the signal durable, with **zero behavior change** for existing
consumers. It is deliberately small and back-compatible.

## Goal

`GateRecord` optionally records the date each `Evidence` level was *first* reached
for that gate; `Status::set_gate` populates it on first-reach and on raise, **never
overwriting** an earlier level's date. **Done when** a raise preserves prior levels'
dates, the field is absent/empty on nodes with no transition history (back-compat),
and `parse ∘ emit = identity` holds with it populated.

## Scope

**In:** an optional per-evidence-level first-reached date map on `GateRecord`
(`reached` / `by` / `evidence` unchanged); `set_gate` records the level's date on
first-reach and preserves earlier levels' dates on a raise; canonical field order +
skip-when-empty serialization; round-trip + back-compat tests.

**Out:** consuming the signal (latency math, rollup surfacing → arc A7); evidence
**regression** semantics (record first-reach only; regression handling deferred);
the CLI `set-gate` flag surface (wired separately); any change to
satisfaction / terminal-gate (slice04) behavior.

## Verification

`cargo test -p odm-core` green; clippy `-D warnings`; no `unsafe`; coverage ≥ 90%
(target 95%) scoped to odm-core. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). Existing
slice03/04 tests still green (no consumer breakage).
