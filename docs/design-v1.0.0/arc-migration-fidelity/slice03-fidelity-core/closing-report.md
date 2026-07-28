# Slice 03 closing report — Fidelity core

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 03 · **Feeds:** MF-2, MF-3
> **Realizes:** ODD-0025 §2.1/§2.2 (Accepted, slice02) · **Assignment:** `cc-prompt.md` ·
> **Ledger:** `ledger.md` (F-1…F-12) · **Implemented by:** CC · **Date:** 2026-07-27 ·
> **Branch:** `arc-migfidelity-slice01-coverage` (continued — see "Branch deviation" below) ·
> **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI/CDC.

## What shipped

The **faithful-import capability**: a migrated node's body is its source body
(no transformation), proven by a hard body-hash gate, with every migrated node
carrying an ODD-0025 `source` record and (for documents) preserved
`author`/`version`. Built in `odm-core` (three new typed fields) and
`odm-migrate` (a shared `fidelity` module + both importers), verified entirely
on fixtures — no live-corpus mutation.

**`odm-core` (`crates/odm-core/src/frontmatter.rs`):**
- `Source` struct (`paths: Vec<PathBuf>`, `class`, `normalization`,
  `migrated_by`, `migrated_on`) and `author: Option<String>` /
  `version: Option<String>` fields, in a documented canonical order
  (`… component, author, version, origin, reserved, retired, source, edges, …`).
- The round-trip proptest extended with the three fields; two new fixture
  tests for exact round-trip content and additive omission.
- `check.rs::check_field_validity` extended: `author`/`version` flagged on a
  work node; `source` — per a criterion correction, see below — is **not**
  flagged on a work node (it's valid on all types).

**`odm-migrate` (new `src/fidelity.rs` + both importers):**
- `fidelity::verify_body_hash` — `sha256(normalize(source)) ==
  sha256(normalize(node))`, `normalize = trim+lf`, hard-fails via a new
  `MigrateError::BodyHashMismatch`.
- `fidelity::build_source` — the shared `Source`-record constructor both
  importers call.
- `selfhost.rs`: the stub synthesis (`format!("# {}\n", fm.name())`) is
  **gone** — `body_source_path()` resolves and reads
  `arc-plan.md`/`slice-doc.md`/`project-plan.md` under `PlanNode.source`
  verbatim.
- `mapping.rs`: `author`/`version` moved off the `extra` catch-all onto the
  typed fields; `legacy.rs` gained a `version` field with a careful
  deserializer (`de_opt_version`) that reconstructs `"1.0"` from YAML's
  untyped-float parsing of `version: 1.0` rather than losing the trailing
  zero via naive `Display`.
- `MigrateError::SourceRead` for a source-body I/O failure (selfhost only —
  mapping.rs's body is already in memory from the frontmatter parse).

**Docs:** ODD-0013 v2.4 and ODD-0020 v2.1 — the ODD-0025 §4 amendments,
applied exactly as specified (§4 below has the deviation notes).

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Three typed fields, canonical slot | **done** | `Source` struct + fields; canonical order documented and matches emit order. |
| F-2 | Round-trip holds | **done** | Existing proptest extended; 2 new fixture tests. |
| F-3 | Per-type validity | **done, criterion corrected** | See "Deviation: F-3" below — `source` is valid on work nodes too; only `author`/`version` are document-only. |
| F-4 | selfhost verbatim body | **done** | Stub synthesis removed; body-equality test against the fixture source, including a case with real content beyond the H1. |
| F-5 | mapping verbatim + typed fields | **done** | Body already faithful; `author`/`version` now typed. |
| F-6 | Hard body-hash gate | **done** | Unit-tested pass/fail on the primitive; wired into both importers, dry-run-independent. |
| F-7 | `source` populated | **done** | Both importers, all four source-doc classes covered by tests. |
| F-8 | `author`/`version` preserved | **done** | Including the float-formatting fix for `version:`. |
| F-9 | ODD-0013/0020 amendments applied | **done** | Both ODDs amended + versioned; grep-verified. |
| F-10 | No live-store mutation | **done** | Store byte-hash unchanged across the whole slice (matches slice01's baseline). |
| F-11 | Clippy/unsafe/coverage | **done** | Clean; 0 `unsafe`; all changed modules ≥ 94.5% line coverage. |
| F-12 | No decided-model drift | **done** | Cross-read confirms `trim+lf`, `source` (never `provenance`), no stored hash. |

**Rows: 12. Done: 12 (1 with a corrected criterion). Deferred: 0. No-op: 0.**
No silent drops — the slice-doc's four explicit "Out" items (live re-migration,
`artifact` type + minting, doc-coverage-in-check wiring, synthesis) were not
touched; verified by the unchanged live-store hash (F-10) and by grep (no
`NodeType::Artifact`, no `Vec` on `Edges.supersedes`).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace` | all green (no failures), including `odm-cli`'s self-hosted-corpus tests |
| `cargo test -p odm-core` | 30 (`frontmatter`) + 16 (`check`) + unit tests, all green |
| `cargo test -p odm-migrate` | 51 (lib) + 6 (`fidelity`) + existing integration suites, all green |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied, clean |
| `unsafe` in changed modules | none |
| `cargo llvm-cov --workspace --summary-only` | `check.rs` 95.61%, `frontmatter.rs` 97.07%, `fidelity.rs` 100%, `legacy.rs` 94.85%, `odm-migrate/lib.rs` 94.50%, `mapping.rs` 98.75%, `selfhost.rs` 95.10% |
| Live store (`.worktrees/odm`) | `git status --porcelain` empty; node-file byte hash unchanged (`40aecffaa88be3aa8764d9423594c8b0`) before and after the entire slice |

## Deviations (flagged, per the working agreement)

### Deviation: F-3's criterion, corrected against ODD-0025's own text

The ledger row and slice-doc both state "author/version/source valid on
document-family nodes; a `check` finding if present on a work node," bundling
all three fields together. **ODD-0025 §2.2 itself does not say this for
`source`:** its opening sentence is "every migrated node carries a `source`
sub-map" (no type restriction — line 76 of the ODD), and the "Both are
document-node fields" sentence (line 102) grammatically refers back to the two
fields just introduced as "separately preserved" (`author`, `version`), not to
`source`.

My first implementation followed the ledger's literal (bundled) wording and
flagged `source` on work nodes too. This immediately broke
`odm-cli`'s `check_green_on_self_hosted_corpus` and
`check_no_orphan_work_nodes` tests, because `selfhost.rs` — per F-7 and the
cc-prompt's own Task item 2 — populates `source` on every self-hosted node,
**including** the arc/slice/project work nodes it mints. A work node
literally cannot satisfy both "source populated" (F-7, required) and "source
invalid on work nodes" (F-3, as first read) at once — a real internal
inconsistency between two rows of the same ledger, not a coding mistake to
route around.

**Resolution:** re-read ODD-0025 §2.2's actual prose (not the ledger's
compressed summary), confirmed `source` is meant to be valid on all node
types, corrected `check.rs` to only flag `author`/`version` on work nodes,
updated the test to assert `source` is valid on a work node, and re-ran the
full workspace suite green. This is a correction *toward* the Accepted ODD's
literal text, not a design re-decision — no ODD-0025 amendment was needed,
since the ODD was already right; only the ledger's shorthand summary and my
first-pass code were wrong. Recorded here per "flag every deviation" rather
than silently fixed.

### Deviation: continued on the existing feature branch, not a new one

The cc-prompt names `arc-migfidelity-slice03-fidelity-core` as the branch.
This work landed on `arc-migfidelity-slice01-coverage` instead, continuing the
branch slices 01 and 02 were committed to in this same session (per the
session's established pattern — slice02 also landed there rather than on its
own named branch). Flagged rather than silently deviated from; not
re-branched retroactively to avoid rewriting shared history mid-session.

### Note: the schema-minor bump (ODD-0025 §4) is recorded, not executed

ODD-0025 §4 asks ODD-0020 to "record that adding `source`/`author`/`version`
bumps the affected types' schema minor when typed in s03." This slice's
ledger has no row requiring `SchemaMarker::current()`'s version numbers to
actually change, and doing so touches the `check`-level
"unsupported-newer-schema" contract workspace-wide — a bigger, separately
decidable change. ODD-0020 v2.1 records the argument for the bump and
explicitly defers executing it (see its §4 note). Not a silent drop: named in
the ODD itself, with the reasoning for deferring.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s03 deliver the MF-2/MF-3 capability the arc-plan planned against?**
Yes, at the fixture-verified level the arc-plan scoped this slice to: MF-2
("body-hash gate green across all migrated nodes; zero stub bodies remain")
now has its *mechanism* — the gate exists, is wired into both importers, and
is proven to hard-fail on a real mismatch (unit-tested) — but MF-2 itself
stays **planned**, since "zero stub bodies remain" is a live-corpus outcome
that only s04's re-migration produces. MF-3 ("every migrated node carries a
`source` sub-map") is now **mechanically true for any node either importer
mints from this commit forward** — verified by fixture tests for all four
source classes (project-plan, arc-plan, slice-doc, odd) — but, again, stays
**planned** at arc scale until s04 runs it against the live 60-node corpus and
actually gives every existing node one.

**What implementing it revealed the arc-plan didn't anticipate:**

1. **A real internal inconsistency between two ledger rows (F-3 vs. F-7), not
   just a "canonical-slot" ambiguity.** The plan assumed "document-node only"
   applied uniformly to `author`/`version`/`source`, but `selfhost.rs`
   populating `source` on work nodes (which the *same slice's* F-7 requires)
   makes that assumption false by construction. This wasn't caught at
   plan-time because ODD-0025's own prose is more precise than its ledger
   summary — a compression artifact, not a design gap. **Concrete
   consequence for later slices:** MF-6 (s05, "doc-coverage wired into `odm
   check`") and any future per-type validity extension should cross-check the
   *ODD's actual §2.2 wording*, not just the ledger row's one-line
   restatement, before encoding a per-type rule.
2. **A genuine data-fidelity hazard in `version:` parsing** the plan didn't
   name: the real corpus's `version: 1.0`/`version: 2.3` are YAML floats, and
   Rust's default `Display` for a whole float drops the trailing zero
   (`1.0_f64` → `"1"`). A naive `String`-typed deserializer would have
   silently corrupted every two-part version number on migration — caught
   before it reached a fixture assertion by checking the real corpus's
   frontmatter shape first, not by a test failure. `legacy::de_opt_version`
   (with its own unit test) is the fix; worth flagging for s04's live run,
   which will be the first time this reads all ~19 real ODD files with
   `version:` fields at once.
3. **The schema-minor bump ODD-0025 §4 asked for is bigger than "record a
   note."** Actually bumping `SchemaMarker::current()` versions touches the
   `content_validity`/"unsupported newer schema" contract workspace-wide — a
   decision with its own blast radius, not a one-line follow to F-1. Recorded
   in ODD-0020 v2.1 as deferred, not silently dropped; whichever slice (s04,
   s05, or a dedicated one) decides to execute it should treat it as its own
   ledger row, not a rider on another slice's typing work.

**The slice-scale silent-drop diff:** scope-as-specified (slice-doc.md's
"In"/"Out") vs. scope-as-delivered — no drops beyond the one corrected
criterion (F-3, disclosed above). All four "Out" items (live-corpus
re-migration, `artifact` NodeType + minting, doc-coverage-in-check wiring,
synthesis) are confirmed absent — verified by the unchanged live-store hash
and by grep for the types/wiring that would mark their presence.

**Recommended arc-ledger update (MF-2, MF-3):** both remain **planned**. The
*capability* they describe now exists and is fixture-proven; the *live-corpus
outcome* they actually assert (zero stubs; every node has a source record)
is s04's job. Recorded as pointers from MF-2/MF-3 to this closing report as
baseline evidence, per LEDGER-DISCIPLINE v2.0 §B (composition rows reproduce
at arc scale, at arc close — never inherited from a slice's fixture-only
attestation).
