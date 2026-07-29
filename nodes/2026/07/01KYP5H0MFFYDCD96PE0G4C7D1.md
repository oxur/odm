---
id: 01KYP5H0MFFYDCD96PE0G4C7D1
number: 559999000
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 02 (Arc 06): migrate odm''s own docs'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice02-migrate-odm-docs/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDP779VQWKQ8JBG77
---
# Closing report — Slice 02 (Arc 06): migrate odm's own docs

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc06-slice02-migrate-odm-docs`, branched off
> `arc06-slice01-migrate-importer-core` (slice01 unmerged; the prompt's fallback).
> Rebase onto `release/1.0.x` once slice01 merges.
>
> **The importer meets reality.** slice01 built the core on fixtures; this slice
> ran `odm migrate` on odm's **own** `docs/design`, settled the three flagged
> decisions against the real corpus, and holds `odm check` green on the imported
> graph. The plan-set self-host is slice03.

## Per-row walk

**N-1 — every real ODD accounted for; legacy intact — done (attested).**
`migrate_real_docs_all_accounted` (over a `TempDir` snapshot of `docs/design`) →
ok: `created + skipped == discover().len()` (12), 0 skipped, every node
`type=odd` with its number preserved (ODD-0013 and ODD-0019 present), and a
byte-snapshot of the copy is unchanged after migration (never-delete at real
scale). The **real import is committed on the branch**: `odm migrate docs/design`
created 12 `odd` nodes under `nodes/2026/07/`; the legacy `docs/design` (14 files)
is untouched.

**N-2 — `odm check` green on the imported graph — done (attested).**
`check_green_on_migrated_odm_docs` → ok (`odm check` exit 0 on the imported
corpus); the live committed corpus reports "ok (12 node(s), no problems)", exit 0.
This is green **by construction, not by force**: `recompose::check_orphan`
requires a containment parent only for work nodes (`is_work() && != project`), so
document (`odd`) nodes are never orphans; they are non-parent-capable (no
decomposition checks); and with no edges / `affects` / `deferred`, link-integrity,
supersession, stale-doc, and reenter-when all find nothing. No edge was fabricated
and no finding suppressed.

**N-3 — the three slice01 flags settled — done (attested).**
  - **Numbering space:** documented in the `odm-migrate` lib docs as *distinct* —
    the idempotence key is `(type == odd, number)` (`existing_odd_numbers` filters
    to `odd` nodes), so a legacy ODD `#13` cannot collide with a work node `#13`.
    Confirmed no collision on the real corpus (odd numbers `2`, `9`–`19`).
  - **Real `supersedes` shape:** the real corpus uses `null` throughout. The
    parser was hardened (`de_opt_ref`) to also accept a bare number and a human
    ref string (`"ODD-0011"`), so the plausible string form resolves rather than
    loud-skipping. `supersedes_real_shape_resolves` proves a `"ODD-0030"` ref →
    the target's ULID; `de_opt_ref_accepts_null_number_and_string` covers the shapes.
  - **Multi-supersession:** none present in the real corpus, so the model's single
    `supersedes` edge (with the warn-on-multiple guard from slice01) is sufficient —
    **no `odm-core` amendment raised**.

**N-4 — `deferred` mapping settled — done (attested).** There is **no
`07-deferred` ODD** in `docs/design` (the only states present are Draft, Accepted,
Final). The slice01 interim `deferred → retire` mapping therefore stands,
unexercised and revisitable; recorded here and in the importer docs. The A5
first-class `deferred` marker is not needed by the real corpus → no amendment.

**N-5 — idempotent + `--dry-run` at real scale — done (attested).**
`migrate_real_docs_idempotent_and_dry_run` → ok: a re-run creates 0 and skips all
as `AlreadyExists` (no duplicates); `--dry-run` plans the whole corpus and writes
nothing. Confirmed live: a second `odm migrate docs/design` reported "0 created,
12 skipped".

**N-6 — gates + no regression — done (attested).** clippy `--all-targets
--all-features -D warnings` exits 0; no `unsafe` in `crates/odm-migrate/src`; line
coverage `lib.rs` 99.56% / `mapping.rs` 99.05% / `legacy.rs` 93.86% (all ≥ 90);
`cargo test --workspace` green (49 suites). Every edge-case fix landed in
`odm-migrate` + a fixture (`test-data/legacy-refstr`), never by editing a legacy
ODD.

Rows: 6. Done: 6. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the real `docs/design` migration (12 `odd` nodes,
committed; legacy intact; none dropped), `odm check` green on the imported graph,
the three slice01 flags settled against reality, the `deferred` mapping settled,
and idempotent/`--dry-run` at real scale. Every "out" item stayed out: **no**
`design-v1.0.0` plan-set migration (slice03), **no** oxur `crates/design/docs`
(decided out of scope), **no** PM-skill work (slice04/05), **no** legacy-file edit.

One in-scope decision is disclosed, not dropped: the `supersedes` **string shape**
is handled proactively (`de_opt_ref`) even though the real corpus uses only
`null` — future-proofing, with a fixture, not a silent extension.

## Deviations / decisions flagged

See the ledger's **Deviations / decisions flagged**. In brief: all three slice01
flags settled **without amendment** (distinct odd numbering space; `supersedes`
parser hardened for the string form; no multi-supersession in the corpus →
single-edge model kept); `deferred → retire` stands (no real deferred ODD);
oxur's docs decided out of scope; the 12 real `odd` nodes are committed on the
branch (ULIDs freshly minted → idempotence, not byte-identity, is the
reproducibility guarantee). **No amendment to ODD-0013 §9 or the arc-plan mapping.**

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice02 deliver A-2 + the A-7 mechanism?** Yes. A-2 (migrate odm's own
docs) is a tested, real, committed import; and A-7 (compose: "runs cleanly on
odm's own `docs/design`; `check` passes on the imported graph") is now
**mechanism-complete** — to be reproduced at arc scale at arc-close. A-2 stays
`attested` until CDC reproduces.

**2. What the real corpus revealed for slice03 (the plan-set cutover).**
  - **`odd` and work nodes will coexist in one store — the numbering spaces must
    stay separated.** slice02 put 12 `odd` nodes under `nodes/`; slice03 adds
    `project`/`arc`/`slice` work nodes for the `design-v1.0.0` plan. The
    idempotence key is already `(type, number)`-aware for `odd`; slice03 must
    confirm the work-node side keys the same way (a `slice #13` and an `odd #13`
    are different nodes) and that `odm check`/`orient` handle a *mixed* corpus
    (work forest + standalone odd document nodes) — the orphan-exemption for
    document nodes (verified in N-2) is exactly what makes that mix green.
  - **The reflexive-import ordering is the slice03 crux.** slice03 migrates the
    plan set that *describes the migration* — including this very arc-plan and
    slice docs. The plan describing the migration must be imported **last** (per
    the arc-plan open Q), and slice03 should lean on `--dry-run` (the reflexive
    preview) + a git checkpoint before the cutover so a bad import is recoverable.
    slice02's never-delete + idempotence make the retry safe.
  - **Legacy retirement is now a live question.** With `odd` nodes committed
    alongside the legacy `docs/design`, the corpus has *two* representations of the
    same ODDs. slice02 deliberately keeps both (supersede-not-delete); slice03 /
    post-cutover decides whether/when the legacy files are retired (they are
    preserved in git regardless). Flagged so it is a decision, not a drift.
  - **No mapping amendments needed — but the model gaps are logged.** The
    single-`supersedes`-edge limit and the typed-`author` / first-class-`deferred`
    questions did not fire on odm's own docs. They remain candidate amendments if
    the plan-set (slice03) or a future corpus exercises them.

**3. Silent-drop diff at the arc altitude.** Nothing the arc-plan scoped for A-2 /
A-7 is missing. The one proactive decision (string-shape `supersedes`) and the
committed-nodes reproducibility caveat are disclosed here so the arc-plan records
them.

**4. Reusable finding.** *Green-by-construction* is the bar to aim for: rather than
teaching `check` to ignore odd nodes, the model already exempts document nodes from
the orphan rule — so a correct import is simply green. When a real corpus makes a
check fail, prefer finding the *model reason* it should pass (or reporting a genuine
issue) over special-casing the checker.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-2 row evidence + A-7 mechanism-complete + a v1.4 version-history
entry), not only here.
