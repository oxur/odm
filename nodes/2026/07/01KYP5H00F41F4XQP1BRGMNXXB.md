---
id: 01KYP5H00F41F4XQP1BRGMNXXB
number: 510459900
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 01 (Arc 06): `migrate` importer core'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice01-migrate-importer-core/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKPJ5YSKRRPDZBX90X
---
# Closing report — Slice 01 (Arc 06): `migrate` importer core

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc06-slice01-migrate-importer-core`, branched off `release/1.0.x`
> (the A6 base — `main` is the pre-rebuild import).
>
> **The arc opener.** A6 closes the bootstrap loop (migrate → self-host → retire
> the prose odm replaces). This slice builds the importer *core* — the legacy →
> new mapping + `odm migrate`, idempotent / `--dry-run`-able / never-delete —
> tested on **fixtures**. slice02 runs it on odm's real `docs/design`.

## Per-row walk

**M-1 — crate + command exist — done (attested).** New workspace member
`odm-migrate` (added to `[workspace] members` in publish order, after
`odm-reconcile`), depending on `odm-core` + `odm-store`; the manifest is
all-`.workspace = true` (11 refs) with zero version literals. `odm migrate
<legacy-path> [--dry-run]` is a `clap` subcommand wired into `odm-cli`'s
`dispatch`. `migrate_command_exists` → ok (dry-run plans; commit persists the six
fixture docs).

**M-2 — faithful mapping — done (attested).** `maps_legacy_fields_to_node` → ok:
a legacy ODD becomes an `odd` node with its `number` preserved and a fresh ULID
identity; `title→name`, `tags`/`component` carried, `author` carried into `extra`;
the `state` scalar maps to a cumulative `odd` gate reach (a `Final` doc reaches
`draft…final`); the `supersedes`/`superseded-by` pair becomes a single
`supersedes` edge (kind `obsoletes`) on the superseding node, pointing at the
superseded node's freshly-minted ULID; the state-*directory* is dropped. (Mapping
decisions — cumulative reach, `author→extra`, `deferred→retire` — are flagged in
the ledger Deviations.)

**M-3 — idempotent — done (attested).** `migrate_is_idempotent` → ok: the first
run creates six, the second creates **zero** and skips six as `AlreadyExists`,
with no duplicates on disk. Idempotence keys on the preserved legacy `number`
(the id is a fresh ULID that cannot be re-minted): the run's `number → id` map is
seeded with the store's existing `odd` nodes and extended as ids are reserved, so
a duplicate `number` — across runs *or* within one run — is caught.

**M-4 — `--dry-run` writes nothing — done (attested).**
`migrate_dry_run_writes_nothing` → ok: the plan lists all six creations, but
`load_all` is empty and no `nodes/` directory is created. The report carries a
`dry_run` flag the CLI surfaces.

**M-5 — never-delete / supersede-not-delete — done (attested).**
`migrate_never_deletes_legacy` → ok: a byte-for-byte snapshot of a copied legacy
corpus is identical after a commit run — no legacy file removed or mutated (the
crate is read-legacy / write-new only). `dustbin_imports_as_superseded` → ok: the
`Superseded` (#4) and `Rejected` (#6) docs import as **retired** nodes whose
`reason` preserves the legacy state; the `supersedes` edge lives on the
superseding node (#5), not the superseded one. Git is the history; a dustbin state
is a node marker, never a file deletion.

**M-6 — malformed/edge → reported, never a panic — done (attested).**
`migrate_malformed_reports_not_panics` → ok over the edge corpus: a missing
`number` and an unknown `state` are reported `Unmappable` skips; a dangling
`supersedes` target is a **warning** and the node still imports (without the
dangling edge) — loud, never a silent drop. `migrate_malformed_frontmatter_is_a_reported_skip`
→ ok: an unterminated frontmatter block is a `Malformed` skip, not a panic. The
run always completes cleanly.

**M-7 — gates — done (attested).** clippy `--all-targets --all-features -D
warnings` exits 0; no `unsafe` in `crates/odm-migrate/src`. Line coverage:
`lib.rs` 99.56%, `mapping.rs` 99.05%, `legacy.rs` 91.03% — all ≥ 90; the
`odm-cli` `migrate.rs` render path is 96.61%. Full workspace green (48 suites).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the `odm-migrate` crate + `odm migrate` command,
the full field mapping (number→number+ULID, state→gate/retire,
supersedes-pair→edge, dustbin→retire, type=odd, metadata carried), idempotent
describe-or-create, `--dry-run`, never-delete/supersede-not-delete, and
malformed/edge handling — all proven on fixtures. Every "out" item stayed out:
**no** run against `docs/design` (slice02), **no** self-host cutover (slice03),
**no** PM-skill work (slice04/05), **no** `odm-index` change (git diff empty).

Two fields are handled by *disclosed decision*, not dropped: `author` is carried
into `extra` (no typed field — Deviation 3), and legacy `version` is intentionally
not carried onto the node (preserved in the untouched legacy file + git —
Deviation 4).

## Deviations / decisions flagged

See the ledger's **Deviations / decisions flagged** for the full set. In brief:
(1) `state`→**cumulative** gate reach at `Asserted`; (2) `deferred` grouped with
the dustbin as a **retirement** (candidate amendment: a first-class `deferred`
disposition); (3) `author`→`extra` (candidate amendment: a typed `author`);
(4) legacy `version` not carried; (5) single `supersedes` edge per node (>1 target
→ warn); (6) fixtures-only, canonical `odd` gate-set, numbering-space deferred.
**No amendment to ODD-0013 §9 or the arc-plan was required** — the mapping fits;
the flags are candidate amendments for slice02+, not blockers.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice01 deliver A-1?** Yes — the `migrate` importer core is a tested
library + command: faithful mapping, idempotent, `--dry-run`-able, never-delete,
loud on malformed. A-1 stays `attested` until CDC reproduces. This makes A-6
(compose: import legacy ODDs — idempotent, `--dry-run`, supersede-not-delete)
**mechanism-complete**, to be reproduced at arc scale at arc-close.

**2. What slice01 reveals for slice02 (the real-corpus run).**
  - **The odd-vs-work numbering-space question is now concrete and must be
    settled.** slice01 preserves the legacy `number` verbatim and keys idempotence
    on "a `number` already exists **as an `odd` node**." odm's real `docs/design`
    numbers ODDs 0001–0019; the `design-v1.0.0` work-node plan (slice03's target)
    will have its own numbers. If odd and work nodes share one space, collisions
    are possible; slice02 should decide whether `odd` numbers live in a separate
    space (and, if so, whether the idempotence key should be `(type, number)`
    rather than `number`-among-odds). Recommend `(type=odd, number)` — which is
    already what the code checks — and documenting odd numbers as a distinct space.
  - **Real `supersedes` values may not be bare numbers.** slice01 parses
    `supersedes`/`superseded-by` as `Option<u32>`. odm's real docs may reference by
    a different form (e.g. `ODD-0005`, a list, or null). slice02 must confirm the
    real shape and, if needed, widen the parse (a non-integer currently → a
    `Malformed` skip, which is loud but would drop the doc). Flagged as the most
    likely real-corpus surprise.
  - **Multiple supersessions per node.** The model's single `supersedes` edge
    (Deviation 5) may be exercised by the real corpus; slice02 should check whether
    any ODD supersedes more than one predecessor and, if so, whether the model
    needs a `Vec<Supersedes>` (an amendment) or the warn-and-keep-one behavior is
    acceptable.
  - **`deferred`/`author` decisions meet real data.** slice02 is where the
    `deferred→retire` grouping and `author→extra` carry are validated against
    actual ODD states/authors — the point to promote either to an amendment if the
    real corpus makes the case.

**3. Silent-drop diff at the arc altitude.** Nothing the arc-plan scoped for A-1 is
missing. The mapping decisions (cumulative reach, `deferred`/`author` handling,
single-supersedes) are disclosed here and in the ledger so the arc-plan records
them rather than a reader discovering them in slice02.

**4. Reusable finding.** The **copy-then-byte-snapshot** pattern is the way to
*prove* a read-only importer never mutates its source — reuse it for slice02/03's
self-host safety (where reflexivity makes never-mutate even more load-bearing).
The **`number → id` map shared by idempotence and edge-resolution** is the clean
shape for any two-pass import.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-1 row evidence + a v1.3 version-history entry), not only here.
