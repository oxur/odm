# Slice 07 (Arc 02) — CLI graph-mutators (plan-of-record)

> Refs: ODD-0013 §7 (command surface), §3 (edges), §4.3 (tears), §5.1 (gates).
> `depends_on:` arc02 slices 02 (Tear), 03 (Status/set_gate), 04 (typed edges/status
> in Frontmatter) + arc01 store. *Not* dependent on slice06 (check v2); sequenced
> after it only because CC is one agent.
>
> **Why this slice exists:** arc02 built the graph engine + queries (`next`/`blocked`/
> `path`) but the CLI can *create nodes* and *query* — it cannot yet *wire edges*,
> *set gates*, or *tear cycles*. So a graph can only be built by hand-editing
> frontmatter. This slice closes that gap. It is a **self-hosting prerequisite**:
> post-A3 we migrate the plan into odm, which needs CLI edge-wiring + gate-setting.
> (Surfaced in slice04 CDC; the `05.1` insert underlined it.)

## Goal

Wire the existing odm-core mutators (`edges_mut`, `Status::set_gate`, `Tear::new`)
to the CLI and persist them through odm-store. **Done when a graph can be
constructed and advanced end-to-end through `odm` alone** — no hand-editing.

## Scope

**In (all in-process `dispatch`, persisted atomically via odm-store):**
- `odm link X <edge> Y` — add an edge on the **source** X: `depends_on` (+ optional
  `--satisfied-at <gate>`), `blocked_by`, `consumes`, `verifies`, `affects`,
  `part_of` (enforces single-parent — replaces the existing parent). Reverse is
  derived, never written.
- `odm unlink X <edge> Y` — remove that edge (absent edge → clear no-op message).
- `odm set-gate X <gate> [--by <who>] [--evidence <level>]` — record via
  `Status::set_gate` (validates against the type's gate-set → `UnknownGate`
  otherwise); default evidence `asserted`; records the `evidence_dates` first-reach
  (slice05.1).
- `odm tear X depends_on Y --because <rationale>` — declare a tear (rationale
  required → `MissingRationale`); persists to `tears:` on X.
- `odm decomposed X --children <ref…>` — affirm X's decomposition complete (wraps
  `affirm_decomposed`; records the child set + date). Gives `check`'s
  advanced-without-decomposition / drift findings a real fix command (slice06 CDC).
- Confirm/extend `odm new --parent <ref>` sets `part_of`.
- Endpoint resolution by **id | number | unique name-prefix** (reuse slice05's
  resolver); unresolvable → typed error naming the fix.
- `--dry-run` (no write) + `--yes` on every mutator; errors-as-affordances.

**Out:** the `check`/reconcile consumers (slice06 / Arc A5); rollup/orient rendering
(Arc A3); sequence-order / monotonic-evidence *policy* (owned by the satisfaction
model, not the CLI).

## Verification

`cargo test -p odm-cli` (+ in-process dispatch tests) green; a node graph built
purely via `link`/`set-gate`/`tear` then queried with `next`/`blocked` behaves;
mutations round-trip on reload; clippy `-D warnings`; coverage ≥ 90% (line). Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). With this, **odm
is self-host-usable**: a plan can be authored entirely through the CLI. Arc 02 done.
