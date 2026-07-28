# odm migration / provenance / synthesis — design decisions & open forks

> **Status:** LIVING NOTES, not a plan. Capture surface for the design conversation
> so nothing is silently dropped. Feeds (later) the reconciliation-arc definition and
> a provenance/migration ODD. Nothing here is committed to code or to the store.
> Started 2026-07-27.

---

## 0. Frame

- Migrating to a multi-worktree structure: `main` (frozen latest stable, 0.3.5, → fast-forwards to 1.0.0 on UAT pass), release worktrees `1.0.x` / `1.1.x`, long-running multi-release `bitubardos`, and a **canonical, branch-independent doc store** = `.worktrees/odm` (orphan branch, to be renamed `odm-store`).
- The 1.0.x migration did **not** fully migrate documents or document bodies. We are (a) finding ALL holes, (b) building tooling to discover & describe them, (c) then methodically fixing to 100% — no panic solution.
- **This is a repeatable capability**: the migration tool must work across *numerous* projects, not just this corpus. "No file left behind" is its acceptance test.
- Meta-goal: make each class of hole **loud** (a check fails) instead of relying on someone noticing.
- **Where this work is planned:** `design-v1.0.0/` on `release/1.0.x` (existing convention). Arc slug confirmed: **`arc-migration-fidelity`**. Adding it updates `project-plan.md` roadmap + version history (tracked plan change).
- **Owed / out-of-band:** the `collaboration-framework` `PROJECT-MANAGEMENT.md` encodes the *old* single-tree layout; it can't be updated until post-1.0. Part VI (operator owns the layout) covers us for now — the framework doc is reconciled-later work, not a conflict to solve in this arc.
- **Post-arc workflow:** once this lands, planning/execution moves to being `odm`-mediated — the session either builds & calls `./bin/odm` directly, or writes prompts + temp files for CC to call `./bin/odm`. (Context for how this arc's own outputs get consumed downstream.)

---

## 1. Verified substrate & code findings (evidence-backed)

- **Canonical store model:** ULID nodes under `nodes/YYYY/MM/*.md`; binary `.odm/index` (`ODMINDEX`); `.odm/context.json` (current-arc pointer); `.odm/drift` (snapshot, `fact_count:0` now); store-local `config.toml` (operational config lives *in* the store, ODD-0022 §4.2).
- **Node frontmatter model** (`odm-core/src/frontmatter.rs`): `id, number, type, schema` (per-type marker, ODD-0020), `name, created, updated, tags, component, origin, reserved, edges{...}, status{gate→{reached,evidence,evidence_dates}}, decomposed{on,children}, retired{reason,on}`.
- **`origin` is a semantic enum** (`Planned | Discovered | Amendment`) — *why a node exists*, NOT where it came from.
- **`edges`**: `part_of` (Option), `depends_on` (Vec<Dependency>, gate-qualified), `blocked_by`, `verifies`, `consumes`, `affects`, **`supersedes: Option<Supersedes{node, kind: Obsoletes|Updates}>`**, `tears`. Supersession is modeled & wired (index/adapter/CLI). It is **single-target**; there is **no stored `superseded_by`** (reverse is derived).
- **KEYSTONE HOLE — provenance never implemented:** no source-path, no body hash, no provenance map anywhere in code or data. It was *always intended* (per Duncan) → a designed primitive with **zero implementation**. Nothing checks for its own absence.
- **Two source conventions** → two importers:
  - Frontmatter docs (`docs/design` ODDs; also dev & research docs): YAML frontmatter (`number/title/state/tags/...`) → `migrate`.
  - Frontmatter-LESS planning corpus (`project-plan.md`/`arc-plan.md`/`slice-doc.md`): metadata is purely **structural** (path `arcNN/sliceMM/...`) → `self-host`.
  - Supporting docs (`ledger`, `cc-prompt`, `cdc-verification`, `closing-report`, ADRs, amendments, UAT) map to **no node**.
- **Corpus size:** `1.0.x/docs` = **320 .md**. Cohorts: 46 closing-report · 43 ledger · 43 slice-doc · 42 cc-prompt · 42 cdc-verification · 11 arc-plan · ODDs · **~25 dev docs** · **~5 research** · one-offs. Store = **60 nodes** (6 arc, 39 slice, 9 design, 5 research, 1 project). Audit's "211 uncovered" is a **floor** — dev & research cohorts weren't in its frame. **[LIVE-VERIFIED 2026-07-27:** 44 stub bodies = 6 arc + 38 slice (all non-retired work nodes); 6 arcs = A1–A6 exactly (numbers 1100–1600) → matches `arc_in_scope`; 14 design/research floaters; 1 tombstone.**]**
- **Fidelity holes (body-present ≠ body-faithful):**
  - 44 bodyless skeleton stubs (`# {name} (plan-of-record)` only) — every arc, all but one slice. **Root cause: self-host mints a synthesized stub body, never imports the `arc-plan.md`/`slice-doc.md` body.**
  - Project node body = ~2 KB vision summary ≠ its ~40 KB source `project-plan.md`. **Root cause: `replan.rs::vision_from_plan` extracts §1 of the plan into the project body — a bespoke, project-node-only path.** Under the 1:1 rule this is a *synthesis*, not a migration.
  - 14 `design`/`research` nodes have **no `part_of`** — outside the containment tree. **RECALIBRATED: possibly not a defect** — design/research docs may legitimately relate via `affects`/`consumes`, not `part_of` containment. Needs the model decision (see F7), not an assumed hole.
  - Apparent **source-internal staleness**: `arc04/arc-plan.md` still reads "planned, not started" though the arc shipped & verified. (Unconfirmed — only read the header blockquote.)
  - **CORRECTION (was overclaimed):** I earlier called the design-node lifecycle "flattened / fabricated-uniform." `mapping.rs::reach_cumulative` is actually **faithful + honesty-calibrated**: gates up to the source's terminal state are marked `Evidence::Asserted` ("claimed from the legacy record, not reproduced") on the *only date the legacy record had*. Under the frontmatter-fidelity check this **passes**. Not a hole. My error — reaching for a dramatic finding.

- **Self-host root causes (`odm-migrate/src/selfhost.rs`) — the real defect cluster:**
  - **`arc_in_scope(major) = (1..=MAX_MVP_ARC)` — hardcoded A1–A6.** The 5 "missing" arcs aren't drift/silent-drop; the importer was **scoped to the MVP arcs and never widened** (docstring: "importing the project + A1–A6 arcs + their slices").
  - **Structural-primary only:** self-host mints arc-node←`arc-plan.md`, slice-node←`slice-doc.md`, and *nothing else*. The ~211 `ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT docs are never walked.
  - **Idempotence is create-or-skip, keyed on `(type, number)`.** Re-running does **not** re-mint stubs — it **skips** them and changes nothing. ⇒ a fixed importer needs an explicit **update/upgrade** path; "just re-run migrate" cannot repair existing nodes. **[implication for the fix]**
  - **Provenance is cheap to add here:** `PlanNode.source: PathBuf` already holds the source path at build time (used for git-history dates) and is **discarded** rather than persisted. And `Frontmatter.extra` (`#[serde(flatten)]`) round-trips an untyped `provenance:` block safely, so it can land additively before being formally typed (ODD-0020 style).
- **Frontmatter model — DEFINITIVE (`frontmatter.rs:171`):** fields are `id, number, type, schema, name, created, updated, tags, component, origin, reserved, retired, edges, status, decomposed, desired_facts, deferred, + extra(flatten)`. **No `provenance`/`source`/hash field — keystone hole confirmed at the type level.**
- **Stale-artifact holes:**
  - `main`-tree `docs/design/.odm/state.json` is the **old 0.3.x index format** — superseded, unlabeled, still looks authoritative.
  - Repo-root `CLAUDE.md` (from `main`) describes a **single-crate** workspace; `1.0.x` is an **8-crate** workspace (`odm-core/store/index/graph/reconcile/migrate/cli` + `oxur-odm`).
  - A dated **tombstone** node (retired UAT slice) — a self-documented instance of "trusted the filesystem over the plan-of-record." Coverage tooling must treat tombstone node ↔ tombstone dir as *covered*, not a hole.

---

## 2. Decisions (made this session)

1. **Scope:** every `.md` under `1.0.x/docs/*` migrates. No file left behind. (dev + research + planning corpus + supporting docs all in.)
2. **Migration is strictly 1:1 and verbatim.** One source → one node; node body == source body. **Hard-fail** if `sha256(trim(source_body)) != sha256(trim(node_body))` → migration errors, does not proceed.
3. **Synthesis is a separate, later step** — never part of migration. It creates a *new* node that **supersedes** its originals (originals become *superseded-by*). This keeps the hard-fail hash gate exception-free.
4. **Provenance is a sub-map** on the node, with **`source_paths` as a list** (1+), so synthesis (many sources) fits the same shape.
5. **Record synthesis type** when a node is synthesized (e.g. `concatenation` | `editorial/conceptual merge` | `other`).
6. **Frontmatter fidelity is checked** via a **schema mapping**, over **only the fields present in the original**.

### Provenance schema — working sketch

Migration (1:1):
```yaml
provenance:
  source_paths:
    - docs/design-v1.0.0/arc04-index-cache/slice07-early-cutoff/slice-doc.md
  source_class: slice-doc
  body_sha256: <sha256 of source body, leading/trailing whitespace stripped>
  normalization: trim            # records WHAT was stripped, so the hash stays interpretable
  migrated_by: odm-migrate/1.0.0 # tool + version → pre-fix imports become queryable
  migrated_on: 2026-07-27
```

Synthesis (later, superseding — sketch):
```yaml
provenance:
  synthesis: editorial-merge     # concatenation | editorial-merge | other
  source_paths: [ <path A>, <path B> ]
  # body_sha256 only meaningful when synthesis == concatenation (mechanically checkable)
edges:
  supersedes: [ <id A>, <id B> ] # NOTE: today this is single-target — see fork F1
```

---

## 3. Open forks (need Duncan's call)

- **F1 — many-to-one supersede. [DECIDED 2026-07-27]** `edges.supersedes` → **`Vec`** (a synthesized node supersedes many originals). Reverse (`superseded_by`) stays **derived**, NOT hand-stored. **Hard requirement:** the bidirectional lineage must be **guaranteed by the tooling on every synthesis** — never a manual/hand-maintained back-edge (that drifts silently). Build the both-directions invariant into the synthesis command + a check.
- **F2 — synthesis type sets the verification regime. [DECIDED 2026-07-27]** `concatenation` **stays hash-gated** (define a deterministic join: order, separator, per-source trim). `editorial/conceptual merge` is not hash-checkable → verified by supersede lineage + explicit attestation. Keep as much hard-fail as each type physically allows. (Synthesis-type = *how merged*; existing `SupersedesKind` Obsoletes/Updates = *what it does to the target* — two axes, keep both.)
- **F3 — body definition + no-transform rule. [DECIDED 2026-07-27]** "Body" = post-frontmatter for FM docs; whole file for FM-less corpus. The importer **must stop synthesizing** the `# {name} (plan-of-record)` H1 / injecting headers. Duncan: *"that old behaviour is what got us into trouble"* — the no-transform change is explicitly a fix, not a regression.
- **F4 — normalization boundary. [OPEN — model slice]** `trim` ends only (strictest — catches any internal change), plus line-endings (CRLF→LF) so a cross-platform checkout doesn't fail spuriously? Not blocking; lands in slice 2.
- **F5 — metadata-fidelity storage. [DECIDED 2026-07-27]** **Compute at migration time; store nothing.** Rationale (Duncan): we make no guarantee content stays fixed — docs explicitly change (e.g. Version-History sections) — so a stored hash for drift-detection is unwanted. Frontmatter fidelity = a migration-time mapping-equivalence check over the fields the source had. **Derived consequence (carry unless overruled):** the *body* hash is likewise migration-time-only — provenance stores `source_paths` + `source_class` + `normalization` + tool/version, **no stored hashes**; both body and frontmatter fidelity are import-time gates, nothing re-verified later.
- **F6 — other branches. [DECIDED 2026-07-27]** `1.0.x` now; `1.1.x` / `bitubardos` at any time after, no rush (and 1.0.x is expected to pull in most content anyway). Not a silent narrowing — an explicit sequence.
- **F7 — containment for design/research nodes. [DECIDED 2026-07-27]** **Allow both.** Some design/research nodes are top-level (uncontained); some are specific to an arc/slice (`part_of` it), created during — or pausing — planning. So containment is *optional* for doc nodes, and the coverage/orphan check must **not** flag a legitimately top-level doc node as an orphan. The "14 floaters" are (mostly) the intended shape, not a hole.
- **F8 — fix vector: update-in-place. [CLARIFIED + resolution 2026-07-27]** `self_host` is create-or-skip on `(type, number)`, so re-running skips the 44 stubs and never replaces their bodies. Repair needs an **update-in-place** op: match each existing node → its source (by structural coordinates, since provenance is currently absent), write real body + provenance into the *same* node, hard-fail the hash gate, preserve id/edges/status. Stub-vs-hand-edited is decidable by "is the body a lone H1?" (a stub). Lands in slices 2–3.
- **F9 — supporting-doc node class + chunk-scale granularity. [OPEN — model slice; surfaced by s01]** The ~211 uncovered supporting docs (`ledger`/`cc-prompt`/`cdc-verification`/`closing-report`/ADR/amendment/UAT) need a node class when s05 mints them. Two sub-questions: (a) *one* class or several? (b) *containment* — supporting docs come at **two modeled scales** (per-slice: `ledger`/`slice closing-report`; per-arc: `arc closing-report`) plus a **chunk** grouping the Release-Hardening workflow introduced (`cN-closing-report.md`, `cN-cdc-verification.md`), which is **not** one of the project/arc/slice scales. s01's detector currently folds chunk artifacts into their slice-sibling class (fine for a read-only count; undecided for the model). **CDC-proposed ruling (confirm):** a single **`artifact`** (supporting-doc) node class, distinct from work nodes, `part_of` its **nearest *modeled* scale** — per-slice artifacts → the slice; arc-level and chunk-level artifacts → the **arc** (do **not** introduce a "chunk" node scale; that would break the constant project/arc/slice vocabulary the framework exists to protect). Related to the arc-plan's "supporting-doc artifact-vs-work node class" scope line.
- **F10 — report/verification artifact self-coverage. [OPEN — model slice; surfaced by s01]** `odm migrate --coverage` renders to stdout and the operator redirects into `coverage-report.md` under `design-v1.0.0/`; likewise every slice emits `closing-report.md`/`cdc-verification.md`. So **each regeneration adds ≥1 permanently-uncovered `.md`** (the 326→328 source-count drift observed at s01 verification is the first instance). When s05 wires doc-coverage into `odm check` (MF-6), the arc's own generated reports will fail the enforced gate **forever** unless their disposition is decided here. **CDC-proposed ruling (confirm):** fold into F9 — `closing-report`/`cdc-verification` are ordinary `artifact` nodes minted in s05 (so they're *covered*, "no file left behind" stays literally true; their bodies changing per-run is fine under F5's no-stored-hash rule). The one genuine open sub-question: does the *regenerable* `coverage-report.md` get an `artifact` node too, or an explicit **coverage-exemption** (an ignore rule) since it is a derived output, not authored source? **CDC lean:** exempt the coverage-report specifically (it is tool output, re-derivable), node-mint the authored reports. Needs your call.
- **Naming note (not a fork; for the s02 ODD):** "research" names **three distinct things** in this corpus — `docs/dev/research/` (5 files), `docs/dev/` generally (26), and the tag-based `research` node *type* (5, inside the `odd` class). The model ODD should disambiguate them in one line so a later reader does not conflate them.

- **F11 — scope cap: removed, not widened. [DECIDED 2026-07-27 — operator]** The `MAX_MVP_ARC` / `arc_in_scope` A1–A6 cap (`selfhost.rs`) is **deleted**, not raised. It bought nothing, was the root cause of the 6-missing-arcs hole ("scoped to the MVP arcs and never widened", §1), and contradicts "no file left behind" (a cap and that property cannot both hold). `self_host` imports every arc/slice dir; `arc_in_scope` is shared with `coverage.rs:512`, so its representation logic updates too. No feature flag / replacement gate — a project this size needs no scope protection, and hardcoded values that get overlooked are the trap. Lands in s04.
- **F12 — arc numbers are non-structural handles. [DECIDED 2026-07-27 — operator]** `number` (`u32`, required) is a **human handle only** — not identity (the ULID) and not order (the dependency DAG; 0013 §2.3). Named arcs (`arc-store-home`, `arc-release-hardening`, `arc-llm-command-surface`, `arc-migration-fidelity`) get a handle by a deterministic rule (a band above A1–A8's 1100–1800), assigned in s04 — **no "canonical numbering" decision was ever required** (an earlier CDC over-escalated the un-numbered named arcs into a blocker; they are not one). *DECIDED — done now in s05 (v1.9, 2026-07-28):* `source` becomes the identity / idempotence + coverage key (retiring `(type, number)` as a correctness key); every node gets a one-time `source` backfill; and the named-arc handle goes name-derived + stable. Promoted from "future refinement" after the v1.8 finding showed the position-based handle is a live re-run-duplicate hazard — deferring it is what let the number keep biting.

---

## 4. Open verification TODO (my side, read-only)

- ~~Enumerate the full `Frontmatter` struct field list~~ **DONE** — no provenance field (`frontmatter.rs:171`).
- ~~Locate the body-synthesis write path~~ **DONE** — both importers live in `odm-migrate`: `mapping.rs` (frontmatter docs, faithful), `selfhost.rs` (structural, mints stub bodies), `replan.rs` (project vision extraction = a synthesis). Still to pin: the exact line in `selfhost.rs` that writes the `# {name}` stub body (after `build_node`).
- ~~Confirm the two importers and their source roots~~ **DONE** — `migrate` over `docs/design` (frontmatter docs); `self_host` over the `design-v1.0.0` `plan_root` (structural), A1–A6 only.
- ~~Build & run the **doc-coverage detector** (320 sources ↔ 60 nodes) — the first real hole count.~~ **DONE — s01 (coverage-discovery), closed & CDC-verified 2026-07-27.** Live exact inventory (`slice01-coverage-discovery/coverage-report.md`): **326 source docs, 266 uncovered across 10 classes; 6/12 arc dirs + 39/44 slice dirs represented; 44 stub bodies (6 arc + 38 slice); 60/60 provenance-absent.** Matching is heuristic (structural coordinates + frontmatter number) and each entry carries its basis, as noted — provenance (this arc) makes it an exact set-difference later. This inventory supersedes the audit's ≈44/≈211/5 estimates as the arc's work-list.
- Read the rest of `replan.rs` + `mapping.rs::build_node` body-attachment to confirm whether the frontmatter (`migrate`) path imports source bodies verbatim (it may already satisfy the 1:1 rule — only self-host clearly violates it).
- Read `AI-CONSTITUTION-SUPPLEMENT.md` + `AI-ENGINEERING-METHODOLOGY.md` **in full** before drafting the arc-plan (project MUST-rule per CDC-SESSION-BOOTSTRAP; skill summary alone is documented-insufficient).

## 5. Hard dependencies & gates (from the roadmap)

- **G-1 — id-scheme — ✅ CLOSED 2026-07-27 (ODD-0024).** Decision: **retain ULID** identity; register-style `D-YYMM-XXXX` **rejected**; `number` unchanged. The minting freeze is **LIFTED** — the arc's minting slices are unblocked. Recorded in `docs/design/04-accepted/0024-id-scheme-retain-ulid.md`; freeze struck in `project-plan.md` (§3 + v1.12 Version History), `CDC-SESSION-BOOTSTRAP.md` (§0b + struck §0a/§0), gap-review G-1 row. All on `release/1.0.x` disk — **operator to `git commit`.** (Was: BLOCKER on all minting, re-litigated 3×.)
- **L-8b — reconcile the four drifted ODDs (0013, 0017, +2).** Standing pre-ship release-gate; the audit scoped this arc to **subsume L-8b**. Fold it in as a slice → clears a loose gate.
- **Placement:** new **named, number-deferred** arc in project-plan §2a (like RH / store-home / LLM-command-surface); slug `arc-migration-fidelity`. Read as **release-blocking** (on the P-12 self-host DoD path; subsumes L-8b) — pending operator confirm.
- **Role:** CDC seat — this session draws arc-plan + per-slice open sets + cc-prompts and verifies; CC implements on a real toolchain. AskUserQuestion popup is flaky this project → **decisions in prose**.
