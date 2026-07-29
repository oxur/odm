---
id: 01KYP5H0FCH3C86WAZH4AQZKRA
number: 526639000
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 06 / Slice 02: migrate odm''s own docs'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice02-migrate-odm-docs/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDP779VQWKQ8JBG77
---
# CDC Verification — Arc 06 / Slice 02: migrate odm's own docs

> Independent verification of CC's closed ledger (impl + close on
> `arc06-slice02-migrate-odm-docs`, commits `028d21d` + `bfcdd95`), per LEDGER-DISCIPLINE
> v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows route to CI /
> a local 1.85+ run. Branch cut from the slice01 tip (off `release/1.0.x`).

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Diff base is `release/1.0.x`.

## Row dispositions

**Row count:** 6 opened, 6 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **N-1** — `nodes/2026/07/` holds **12** `odd` nodes; legacy `docs/design` **12** ODD files
  intact. `migrate_real_docs_all_accounted` present (created+skipped == corpus, byte-snapshot
  unchanged). ✔
- **N-2** — `check_green_on_migrated_odm_docs` present (`odm check` exit 0). Green by
  construction (see ruling 2). ✔
- **N-3** — `de_opt_ref` (`legacy.rs:66`) accepts `null` | bare number | `"ODD-00NN"` string;
  `first_number("ODD-0011") == 11` tested (`:234`); numbering-space note in the importer docs;
  multi-supersession — none in the real corpus → single-edge model kept, no amendment. ✔
- **N-4** — no `07-deferred` ODD in `docs/design` (states = Draft/Accepted/Final) →
  deferred→retire interim stands, recorded. ✔
- **N-5** — `migrate_real_docs_idempotent_and_dry_run` present (re-run 0 created / 12 skipped;
  dry-run writes nothing). ✔
- **N-6 (no `unsafe`)** — grep empty; **odm-index untouched** vs `release/1.0.x`. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; line coverage lib
99.6% / mapping 99.1% / legacy 93.9%; full workspace green (49 suites). → **PENDING CI.**

## Rulings on CC's decisions

1. **The three slice01 flags all settled without amendment — honest, because reality was
   benign.** odm's real corpus is all progression states, all `supersedes = null`, no dustbin,
   no `07-deferred` — so the flags resolved *against the corpus* (numbering space distinct; no
   multi-supersession; deferred→retire untested-so-interim), not by hand-waving. Good: CC
   also **hardened the `supersedes` parser prophylactically** for the `"ODD-00NN"` string
   shape (plausible given the human ref convention) with a fixture — future-proofing a real
   shape without waiting for it to break, and without touching a legacy file. That's the right
   kind of forward work.
2. **`odm check` green "by construction" — accepted as honest, with a calibration note.**
   `odd` (document) nodes are orphan-exempt (`recompose::check_orphan` requires a parent only
   for work nodes) and non-parent-capable, and with all `supersedes = null` they have no
   edges — so the imported graph produces **zero findings** legitimately (no fabricated edge,
   no suppressed finding). **Calibration for the arc-close:** this green is *real but modest*
   — the 12 `odd` nodes are isolated document islands, so it exercises check lightly. The
   **connected-graph** demonstration (a `part_of` tree — project→arc→slice — driving
   `orient`/`rollup` end-to-end) is **slice03's** job (A-8, the self-host trigger). slice02's
   green should not be over-read at arc-close as "self-host proven"; it proves "the ODDs
   import into a structurally-sound isolated set." That distinction is correct scope, flagged
   so the arc-close reproduces A-8 on the *connected* corpus, not on isolated docs.
3. **Real nodes committed on the branch (self-host begins).** Accepted — the 12 `odd` nodes
   live alongside the untouched legacy `docs/design` (supersede-not-delete); `.odm/` stayed
   gitignored. Legacy retirement is correctly deferred to a post-cutover question.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-2: the importer runs clean on odm's real docs; A-7
  mechanism-complete (reproduce at arc scale). The three flags + deferred settled against
  reality.
- **Silent-drop diff honest?** ✔ — 6/6; the modest-green nature, the prophylactic parser
  hardening, and the committed-nodes decision are all disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-2 attested + A-7
  mechanism-complete + arc-plan v1.4), with a concrete **slice03** list: odd + work nodes
  coexist in one store (keep numbering spaces separate; the document-node orphan-exemption is
  what keeps the mixed corpus green); the **reflexive-import ordering is the crux** (import
  the plan that describes the migration *last*; lean on `--dry-run` + a git checkpoint);
  legacy retirement becomes a live post-cutover question. All correct — these are slice03's
  design points.

## Verdict

**Arc 06 / Slice 02 CDC-verified on structure; decisions ruled; cargo rows pending CI.** The
importer met reality with zero surprises: 12 ODDs → 12 `odd` nodes, legacy intact
(byte-proven), `odm check` green, and the three slice01 flags settled against the actual
corpus without a model amendment (the parser hardened prophylactically for the `ODD-NN`
string shape). One calibration carried to the arc-close: slice02's check-green is over
*isolated* document nodes — the connected-graph self-host demonstration (A-8) lands in
slice03. A-2 attested-on-close; flips `done` on CI green. **A6 at 2/5.**

**Next: slice03 — the self-host cutover.** The `design-v1.0.0` plan-set (project-plan,
arc-plans, slice-docs — a real `part_of` tree) comes into `nodes/`, odd + work nodes coexist,
`orient`/`rollup` run on the self-hosted corpus, and **the loop closes**. Its crux is the
reflexive-import ordering (migrate the plan that governs the migration last) — the meatiest,
highest-stakes slice of the arc.

CDC: planning thread, 2026-07-06. Iterations used: 1.
