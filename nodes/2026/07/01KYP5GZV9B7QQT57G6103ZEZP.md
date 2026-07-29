---
id: 01KYP5GZV9B7QQT57G6103ZEZP
number: 568786300
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 06 / Slice 01: `migrate` importer core'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice01-migrate-importer-core/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKPJ5YSKRRPDZBX90X
---
# CDC Verification — Arc 06 / Slice 01: `migrate` importer core

> Independent verification of CC's closed ledger (impl + close on
> `arc06-slice01-migrate-importer-core`, commits `4479076` + `bc55c35`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run. Branch cut from **`release/1.0.x`** (the new base).

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Diff base is `release/1.0.x` (not `main` — `main` was reset onto the pre-rebuild
`release/0.3.x`, so a `main` diff spuriously shows all of A4/A5 as "added").

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **M-1** — `crates/odm-migrate/{lib,mapping,legacy}.rs`; in `[workspace] members`; no version
  literals; `Command::Migrate { legacy_path, dry_run }` wired (`odm-cli/src/lib.rs:400,524`).
  Vs `release/1.0.x` the diff is odm-migrate (5) + odm-cli (4) + odm-core (2, the `extra`
  insert helper) — **odm-index untouched**. ✔
- **M-2** — `mapping.rs`: legacy `number` preserved (`:73–74`, also the idempotence key) +
  fresh ULID; `MapError::{MissingNumber, UnknownState}`; `NodeType::Odd`; `supersedes` edge;
  metadata carried; author → `extra`; name fallback `ODD-{:04}`. `maps_legacy_fields_to_node`
  present. ✔ (see ruling 1)
- **M-3** — idempotent via the preserved-`number` map (seeded with existing `odd` nodes);
  `migrate_is_idempotent` present (2nd run creates 0 / skips 6 `AlreadyExists`). ✔
- **M-4** — `migrate_dry_run_writes_nothing` present (plan printed, no `nodes/`). ✔
- **M-5** — `migrate_never_deletes_legacy` (byte-snapshot of a copied corpus, unchanged after
  a commit run — *proves* never-mutate) + `dustbin_imports_as_superseded`. ✔
- **M-6** — `SkipReason::{AlreadyExists, Malformed, Unmappable}` (`lib.rs:76`) — three distinct
  typed reasons; `migrate_malformed_reports_not_panics` + a malformed-frontmatter skip; a
  dangling `supersedes` → warning + node still imported (no dangling edge). Loud, never a
  silent drop. ✔
- **M-7 (no `unsafe`)** — grep empty in `odm-migrate/src`. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; line coverage
lib 99.6% / mapping 99.1% / legacy 91.0% / cli-migrate 96.6%; 48 suites green. → **PENDING CI.**

## Rulings on CC's flagged candidate-amendments

1. **`state` → cumulative gate reach (not a single gate), at `Evidence::Asserted`. Accepted —
   and the right call.** A `Final` doc *passed through* draft→…→final; recording only a lone
   `final` gate would misrepresent its history, so reaching every gate up to the mapped
   position is faithful. And **`Asserted` is the honest evidence level for a historical
   migration** — the states are *claimed from the legacy record*, not independently reproduced
   (exactly what `asserted` means). Good use of the evidence ladder.
2. **`deferred` grouped with the dustbin as a retirement. Accepted as an interim — but flagged
   for slice02/03 reconsideration.** `07-deferred` is semantically *not* dead: deferred = "set
   aside, may return," distinct from rejected/withdrawn/superseded. Mapping it to `retire`
   risks reading a paused doc as terminated. **A5 just built a first-class `deferred` marker**
   (with a re-entry predicate) — a legacy-deferred ODD arguably maps better to that than to
   retirement. For a fixtures-only core slice this is a defensible simplification (and the odd
   gate-set may not yet have a non-retired deferred position), but it should be **revisited
   against the real corpus in slice02** (does odm's own `docs/design` have `07-deferred`
   ODDs? if so, decide: A5 `deferred` marker vs a distinct gate vs retire). Recorded as a
   real open question, not silently accepted.
3. **`author` → `extra` (not a typed field). Accepted.** The new frontmatter has no typed
   `author`; riding it in the existing forward-compat `extra` map (via `insert_extra`) is the
   mechanism the schema already provides — no ad-hoc schema change. A typed `author` field is
   a future call, not a migration blocker.
4. **Single `supersedes` edge per node (>1 → warn). Accepted; flagged for slice02.** The
   model's `supersedes` is single (`Option<Supersedes>`), so a legacy doc that supersedes >1
   can't be fully represented → a warning (loud, not silent). Fine for fixtures; **the real
   corpus may need multi-supersession** (CC's bubble-up flags it) — if so, that's an
   `odm-core` model question (an amendment), settled in slice02.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-1: the importer core (mapping + idempotent + dry-run +
  never-delete), tested on fixtures. A-6 marked mechanism-complete (reproduce at arc scale).
- **Silent-drop diff honest?** ✔ — 7/7; all four design decisions are flagged as candidate
  amendments, not buried.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-1 attested + A-6
  mechanism-complete + arc-plan v1.3), with a concrete **slice02** list: settle the
  odd-vs-work numbering space (the code already keys on `(type=odd, number)` — recommend
  documenting `odd` as a distinct space); confirm the real `supersedes` value shape (may not
  be bare `u32`s); check for multiple supersessions per node. All correct.

## Verdict

**Arc 06 / Slice 01 CDC-verified on structure; all four flags ruled; cargo rows pending CI.**
The importer core is honest and safe: never-delete is *proven* by a byte-snapshot (not
asserted), malformed input is loud with three typed skip reasons, idempotence keys on the
preserved legacy `number`, and the mapping faithfully carries a legacy ODD into an `odd` node
(number preserved, cumulative gates at `Asserted`, supersedes edge, dustbin→retire). One
mapping question worth revisiting at real-corpus scale: **`deferred`→retire** (A5's `deferred`
marker may be the better target). A-1 attested-on-close; flips `done` on CI green.

**This is the arc opener — no arc-close follows.** Next: **slice02** (run `migrate` on odm's
own `docs/design`; resolve the real-corpus edge cases CC flagged; `odm check` green on the
imported graph) — the step where the flagged decisions (deferred, multi-supersede, numbering
space) get settled against reality.

CDC: planning thread, 2026-07-06. Iterations used: 1.
