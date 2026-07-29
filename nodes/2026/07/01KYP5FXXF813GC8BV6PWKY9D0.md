---
id: 01KYP5FXXF813GC8BV6PWKY9D0
number: 569115200
type: artifact
schema: artifact/v1.1
name: 'Slice 09 (Migration Fidelity) — CDC verification: Coverage enforcement *capability*'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice09-coverage-enforcement/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYP5FRXJBKY1GHW4QF9JQZD7
---
# Slice 09 (Migration Fidelity) — CDC verification: Coverage enforcement *capability*

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 09 · **Verifier:** CDC
> (independent) · **Date:** 2026-07-29 · **Method:** LEDGER-DISCIPLINE v2.0 §A CDC protocol +
> PROJECT-MANAGEMENT Part IV bubble-up check. **Fixture-only slice — no live class-(b) row.** Code +
> test *existence and correctness* reproduced by direct read on `release/1.0.x`; runtime execution
> (`cargo test`/`clippy`/`llvm-cov`) **attested-by-CC → CI**.
> **Under review:** `closing-report.md`, `ledger.md` (F-1…F-10), the five commits `012fad5` (type +
> Findings 2/3), `1760d7b` (discovery reach), `bc6e193` (check-wiring), `9cccdde` (F-8 + tests),
> `381acba` (close) — all on `release/1.0.x`.

## Verdict

**PASS — CDC-verified.** Every capability claim reproduces in code, the tests exist and are
non-vacuous, the two CDC v2.8 Findings are genuinely fixed, and **no live mutation occurred** (the `odm`
branch HEAD is unchanged and no ODD was edited — both reproduced, not attested). CC's one flagged
deviation — `artifact/v1.1` rather than the cc-prompt/ODD-0025-§4 `artifact/v1.0` — is **correct given
the codebase's global schema-version model**, and I ratify it; it leaves **one LOW spec-keeping
follow-up** (ODD-0025 §4 still literally reads "v1.0" and should be amended to match). The
`commands.rs` sub-90% coverage note is a fair, honestly-disclosed reading of pre-existing debt in
untouched code.

## Environment & what "reproduced" means here

This is a **capability slice — fixture-only**, so there is no live store to reproduce against (that is
s10). The CDC Linux bridge cannot run the macOS `odm` binaries or edition-2024 `cargo`, so **all runtime
rows are attested-by-CC → CI**. What I reproduce is *structural*: the code artifacts exist and do what
the ledger claims (read on `release/1.0.x`), the tests exist and assert the right things (read the
bodies), and the no-mutation guarantee holds (git state on the `odm` branch + the ODD tree).

## 1. Capability reproduction (code + tests) — confirmed

| Claim | Reproduced in code | Test evidence |
|-------|--------------------|---------------|
| **F-1** `NodeType::Artifact` | `node_type.rs`: variant (`:47`), `as_str`→`"artifact"` (`:63`), in the `is_document` group (`:82`), `valid_child_types`→`&[]` (`:103`), `FromStr` (`:134`) | `schema.rs` round-trip; `check.rs` field-validity (attested→CI) |
| **F-1** schema `artifact/v1.1` | `SchemaMarker::current` stamps the **global** `SchemaVersion::CURRENT = {1,1}` (`schema.rs:46,103`) — see §2 | dedicated round-trip test |
| **F-2** containment = nearest modeled scale | `artifact.rs::scale_index` (`:113`) builds a **directory→node-id index from every persisted arc/slice node's own `source.paths`** (the s08 canonical key) → resolves numbered **and** named scales uniformly, top-level → uncontained (§2.7) | `artifact_containment_resolves_nearest_modeled_scale` (4 cases) |
| **F-3** mint-all (§2.6) | `artifact.rs::mint_artifacts` (`:170`) — every supporting-doc class, `coverage-report.md` included, no exemption | `artifact_mint_all_covers_supporting_docs_incl_reports`, `_is_idempotent`, `_dry_run_writes_nothing`, `_body_is_verbatim_1to1` |
| **F-4** design/research `source` backfill | `mapping.rs::backfill_source` (`:298`) — the `reconcile_source` counterpart for the pre-`source` family, matched by `number`, gated | `tests/backfill_source.rs` (×5) |
| **F-5** doc-coverage `check` Error | `commands.rs`: `coverage_scan_root` (`:540`) reads `[coverage] scan_root`; the `(c3)` block (`:1632`) emits Error code `uncovered-doc`; `None`→documented no-op | `check_coverage.rs` (×4): uncovered→Error(code 1, names file), cover→green, **no-op without config**, relative≡absolute |
| **F-6** Finding 2 (named-arc undercount) | `coverage.rs::resolve_arc_dir_numbers` (`:593`, used `:537`) replays `named_arc_number`'s collision handling → a named arc with a node reads represented | `tests/coverage.rs` (+1 new) |
| **F-7** Finding 3 (stale `provenance:` scan) | `provenance_absence` retargeted to `frontmatter().source().is_none()` (`:686`); **`has_provenance_key`/`frontmatter_yaml` removed** (grep count 0) + dead test dropped | typed-accessor test |
| **F-8** optional containment | structural since F-1 (`check_orphan` gates on `is_work()`); `artifact` named in the doc comment | new `recompose.rs::detect_orphan` case |

**Tests:** **20 new** functions confirmed across the six files (`artifact.rs` unit ×4, `tests/artifact.rs`
×5, `tests/backfill_source.rs` ×5, `check_coverage.rs` ×4, `tests/coverage.rs` +1, `schema.rs` +1) — the
29 `#[test]` I counted in those files minus the pre-existing ones in the two modified test files. Spot-read
bodies (the enforcement + containment tests) are **non-vacuous** — they assert exit codes, the finding
code, the named file, and the covered→silent transition, and would fail if the rule regressed.

## 2. The `artifact/v1.1` deviation — adjudicated (ratified, with a spec-keeping follow-up)

CC stamped `artifact/v1.1`, not the `artifact/v1.0` the cc-prompt and **ODD-0025 §4** name. I reproduced
the mechanism: `SchemaVersion::CURRENT` is a **single global constant `{1,1}`** (`schema.rs:46`) and
`SchemaMarker::current(node_type)` applies it to *every* type (`schema.rs:103`). There is no per-type
version table. So a newly-added type necessarily stamps the shared current — **`v1.1`** — and
hand-writing the literal `"artifact/v1.0"` would *bypass* `SchemaMarker::current`, creating exactly the
inconsistency the marker mechanism exists to prevent. **CC's choice is correct; I ratify it.** (It is
also semantically apt: `v1.1` is the schema generation that carries the `source` field, and an
`artifact` is born *with* `source` — a `v1.0` artifact, meaning "before source," never exists.)

**The residual gap (LOW — spec-keeping):** ODD-0025 §4 still literally reads *"add an `artifact/v1.0`
schema marker."* Delivery produced `v1.1`. CC judged "no ODD edit needed" — right about the *semantics*
(§2.5/§2.6/§2.7 are untouched), but §4 is the **amendment-specification** section and its text now
mismatches what shipped. The clean close is a **one-line ODD-0025 §4 amendment**: the `artifact` type
stamps the shared `SchemaVersion::CURRENT` (`v1.1`), not a type-local `v1.0`, because the schema minor is
a global generation counter, not a per-type axis. This is disclosed, not silent — but leaving §4
diverged is a small spec-softening, and the discipline is to reconcile the spec to delivery. *I can draft
the amendment; it does not block s09.*

**Deeper observation (not an s09 finding):** the model docs (ODD-0020) describe **per-type** schema
markers (`project/v1.0`, `arc/v1.0`, …) while the code implements a **global** current minor. The two
coexist coherently today (per-type *field-validity* + a global *version generation*), and the global
model yields the right answer for `artifact` and for s10's design/research backfill (both become `v1.1`
as they gain/carry `source`). Worth a note on the schema-versioning backlog if per-type divergence is
ever actually wanted; no action for this arc.

## 3. No live mutation (F-9) — reproduced

- **`odm` branch HEAD is still `7226797`** (slice08's rewrite) — **no s09 commit on the store branch**.
- **No ODD edited:** `git diff --name-only 012fad5^ 381acba -- docs/design/` is **empty** (the ODD tree
  is untouched; the only `docs/design-v1.0.0/` changes are the arc-plan bubble-up + the slice09 close
  docs).
- **`[coverage]` is absent from the live `.worktrees/odm/config.toml`**, so the new `check` rule is
  **inert on the live corpus** — `odm check` there is unchanged (CC: 0 errors, 8 pre-existing warnings,
  exit 0). This is the **no-red-window** guarantee, and it is exactly the seam s10 activates (add the
  key after minting). s10's open set remains correctly scoped.
- s10/s11/s12 not pulled forward: no live mint/backfill, no synthesis, no reconcile code.

## 4. `commands.rs` coverage note (F-10) — ratified

CC disclosed that `commands.rs` sits at 88.93% file-wide, under the ledger's "≥ 90% changed modules."
Reproduced context: it is a **2521-line pre-existing file** (the single home for every `check`/`validate`
rule); the *new* code (`coverage_scan_root` + the `(c3)` block) has all branches exercised by the four
`check_coverage.rs` tests. **I ratify CC's reading** — the criterion's intent (the new code is
well-tested) is met, and the shortfall is pre-existing debt in code s09 did not touch. **Observation
(not a finding):** `commands.rs` as a 2521-line god-file is a refactor candidate for a future slice, and
"coverage ≥ 90% changed *modules*" is ambiguous for such a file — measuring changed *regions/functions*
would be a sharper future criterion.

## 5. Ledger — per-row CDC disposition

| ID | CC | CDC verdict | Strength |
|----|----|-------------|----------|
| F-1 | done | **confirmed** — variant + methods + `artifact/v1.1` (adjudicated §2) | reproduced (code) / attested→CI (tests) |
| F-2 | done | **confirmed** — `scale_index` directory-index technique; 4-case test | reproduced (code) / attested→CI |
| F-3 | done | **confirmed** — `mint_artifacts` mint-all; idempotent/dry-run/verbatim tests | reproduced (code) / attested→CI |
| F-4 | done | **confirmed** — `backfill_source`; ×5 tests | reproduced (code) / attested→CI |
| F-5 | done | **confirmed** — `(c3)` rule, config-gated, inert-on-live (reproduced); ×4 non-vacuous tests | reproduced (rule + inert-live) / attested→CI (exec) |
| F-6 | done | **confirmed** — `resolve_arc_dir_numbers` | reproduced (code) / attested→CI |
| F-7 | done | **confirmed** — retargeted to `source().is_none()`; stale helpers removed (0) | reproduced |
| F-8 | done | **confirmed** — `is_work()`-gated orphan check; fixture | reproduced (code) / attested→CI |
| F-9 | done | **confirmed** — `odm` HEAD unchanged; no ODD edit; `[coverage]` absent live | **reproduced** |
| F-10 | done | **clippy/unsafe/cov attested→CI**; the sub-90% `commands.rs` note ratified (§4) | attested→CI |

**Row count: 10 opened / 10 dispositioned — no silent drops.** Done: 10.

## 6. Findings (severity-classified)

1. **ODD-0025 §4 says `artifact/v1.0`; delivery is `artifact/v1.1`** — **LOW (spec-keeping).** The
   delivery is correct (§2); the model doc's amendment-spec text should be reconciled with a one-line
   §4 amendment. Disclosed by CC, not silent. *Disposition: recommend the amendment (CDC can draft);
   does not block s09 or s10.*
2. **`commands.rs` god-file / ambiguous "changed modules" coverage unit** — **observation, no action
   this arc.** Refactor candidate; sharpen the coverage criterion to changed regions in future ledgers.
3. **Positive (Safety-II):** CC's bubble-up surfaced two reusable insights worth keeping — the
   *containment-by-directory-index* technique (simpler + more general than containment-by-numbering when
   the ancestor is already in the store), and the `slice_number()` precondition trap (raw-major, not a
   handle) caught by a **loud fixture failure** rather than inspection — CC's suggestion to strengthen
   `slice_number()`'s doc comment is endorsed as a tiny future follow-up. Finding 3 was also revealed to
   have been **unconditionally wrong since s02** (every node flagged provenance-missing, uncaught
   because nothing depended on the count until s09 wired a check near it) — a good latent-bug catch.

## 7. Bubble-up check (PROJECT-MANAGEMENT Part IV)

- **Did s09 deliver its assigned arc piece?** Yes — the coverage-enforcement **capability** (the
  `artifact` type, both discovery-reach families, the config-gated `check` rule, and both v2.8 Findings)
  exists in code and is fixture-proven, with the live corpus untouched.
- **Silent-drop diff:** none. Every slice-doc "In" item landed; every "Out" item (live mint/backfill,
  live check activation, synthesis, reconcile) is confirmed untouched.
- **What it revealed:** the schema-version model is **global, not per-type** — which is the root of the
  `artifact/v1.1` deviation and worth the s09-scoped ODD-0025 §4 amendment above; and the
  containment-by-index technique is a keeper.
- **Arc-plan change forced:** flip s09 → **CDC-verified PASS**; MF-1/MF-6 → "capability done,
  live-pending"; confirm **s10 (coverage live run) unblocked + next**; record the ODD-0025 §4
  amendment as a LOW follow-up. Applied via the plan-change discipline (v2.14, this slice named).

## Closure

s09 **CDC-verified PASS** on 2026-07-29. Capability reproduced in code; tests non-vacuous; **no live
mutation** (reproduced). The `artifact/v1.1` deviation is correct and ratified, leaving a LOW ODD-0025
§4 spec-keeping amendment. s10 (coverage live run) is unblocked and next; its open set remains correctly
scoped (the `[coverage]` key it adds is the confirmed inert-on-live activation seam).

_Verified by: CDC (independent), 2026-07-29 — against `release/1.0.x` commits `012fad5`…`381acba`;
`odm@7226797` unchanged._
