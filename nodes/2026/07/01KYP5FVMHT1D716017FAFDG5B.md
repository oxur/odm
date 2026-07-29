---
id: 01KYP5FVMHT1D716017FAFDG5B
number: 561375900
type: artifact
schema: artifact/v1.1
name: Slice 04 closing report — Scope + repair capability
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice04-scope-repair-capability/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SM22NCEXWSSF2D5E2
---
# Slice 04 closing report — Scope + repair capability

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 04 · **Feeds:** MF-2, MF-3, MF-5
> **Realizes:** ODD-0025 §2.8 (update-in-place) + §2.1/§2.2 (gate + `source`); ODD-0020 §4
> (schema-minor bump) · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-12)
> **Implemented by:** CC · **Date:** 2026-07-28 · **Branch:** `release/1.0.x` (per the arc's
> established fast-forward pattern — no per-slice branch, as the cc-prompt itself directs)
> **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI/CDC.

## What shipped

The **scope + repair capability** — fixture-proven, no live mutation:

1. **The `MAX_MVP_ARC`/`arc_in_scope` cap is deleted, not raised.** `self_host`'s
   `discover()` now walks every `arc*` directory: numbered (`arcNN-*`, including
   the former post-MVP `arc07`/`arc08`) and **named** (`arc-<slug>`) alike.
2. **Named arcs get a deterministic, collision-free `number` handle.**
   `NAMED_ARC_BASE = arc_number(8) + NAMED_ARC_STEP` (= 1900, derived as a
   `const fn` expression rather than a repeated magic number), then `+100`
   per subsequent named arc in directory-sort order. `slice_number`'s offset
   logic (`slice_position`) is shared between numbered and named arcs.
3. **`coverage.rs`'s shared `arc_coordinate` no longer filters by scope** —
   a previously-excluded numbered arc (`arc07`) now correctly resolves once
   `self_host` mints it. Named-arc coverage-matching stays a disclosed
   heuristic limitation (see Deviations).
4. **Update-in-place repair** (`selfhost::repair`): matches an existing
   **stub** work node (`fidelity::is_stub_body`, shared with the s01
   detector) to its plan-set source by structural coordinate, imports the
   verbatim body + a fresh `source` record through the hard body-hash gate,
   and persists the **same** node via `Store::persist` — `id`, `edges`,
   `status`, and `number` all survive untouched; only `body`, `source`,
   `updated`, and the schema marker change. No `delete` anywhere in the path.
5. **The ODD-0020 schema-minor bump, executed.** `SchemaVersion::CURRENT` is
   now `v1.1` (was `v1.0`, deferred by s03). New and repaired nodes stamp
   `<type>/v1.1`; an existing `v1.0` node remains valid — verified by a new
   forward-compatibility test, not just by the constant's value.

Entirely fixture-verified. The live `.worktrees/odm` corpus is untouched
throughout (byte-identical, matching the hash recorded at s01/s03 close).

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Cap removed | **done** | `MAX_MVP_ARC`/`arc_in_scope` deleted; grep clean (including two doc-comment mentions of the identifier, removed for a clean literal grep). |
| F-2 | Named-arc handles | **done** | Deterministic, `const`-derived base; collision-freedom asserted by a set-uniqueness test. |
| F-3 | `coverage.rs` updated | **done** | `arc_in_scope` no longer imported/called; the arc07 fixture assertion flipped and passes. |
| F-4 | Update-in-place repair | **done** | Body verbatim, `source` populated, id/edges/status/number preserved — asserted directly. |
| F-5 | Repair gate | **done** | Pass-case live; fail-case unit-tested on the shared primitive (same structural argument as s03). |
| F-6 | Schema bump executed | **done** | `CURRENT = v1.1`; symbolic + literal assertions updated workspace-wide. |
| F-7 | Forward-compat | **done** | New test proves a `v1.0` node still validates after the bump. |
| F-8 | Stub predicate reused | **done** | `fidelity::is_stub_body` is the one definition; two call sites. |
| F-9 | No live mutation | **done** | Store hash unchanged across the whole slice. |
| F-10 | ODD-0020 §4 in sync | **done** | v1.3 entry + inline note; frontmatter bumped. |
| F-11 | Clippy/unsafe/coverage | **done** | Clean; 0 `unsafe`; all changed modules ≥ 93.7% line coverage. |
| F-12 | No decided-model drift | **done** | Cross-read confirms update-in-place, `source` naming, no stored hash, cap deleted not raised. |

**Rows: 12. Done: 12. Deferred: 0. No-op: 0.** No silent drops — the slice-doc's
"Out" items (the live run, `artifact` minting + check-wiring, synthesis,
re-pointing the live `context.json`, re-keying idempotence on `source`) were
not touched; verified by the unchanged live-store hash and by grep (no
`context.json` write in `selfhost.rs`, no `NodeType::Artifact` anywhere).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace` | all green (no failures) |
| `cargo test -p odm-migrate` | 45 (lib) + 6 (`coverage`) + 9 (`fidelity`) + existing suites, all green |
| `cargo test -p odm-core` | 17 (`check`) + 28 (`frontmatter`) + others, all green |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all` | applied, clean |
| `unsafe` in changed modules | none |
| `cargo llvm-cov --workspace --summary-only` | `schema.rs` 96.61%, `coverage.rs` 93.77%, `fidelity.rs` 100%, `selfhost.rs` 94.46% |
| Live store (`.worktrees/odm`) | `git status --porcelain` empty; node-file byte hash unchanged (`40aecffaa88be3aa8764d9423594c8b0`) — identical to slice01/slice03's recorded baseline |
| `grep -rnE 'MAX_MVP_ARC' crates/odm-migrate/src/` | no matches |

## Deviations (flagged, per the working agreement)

### Deviation: named-arc coverage-matching stays heuristically incomplete (disclosed, not fixed)

`coverage.rs`'s `arc_coordinate` can resolve a **numbered** arc's number from
its directory name alone (`arc07-horizon` → `7`), but a **named** arc's actual
`number` is an *assigned handle* that depends on the full sorted list of every
named arc directory in the corpus — information a single-directory-name
function cannot recover. So even after `self_host` mints a named arc a real
node (this slice), the doc-coverage/representation detectors will still
report its docs as heuristically uncovered — a false negative, not a crash.

This is explicitly disclosed in `arc_coordinate`'s doc comment rather than
fixed, for two reasons: (1) it is out of this slice's stated scope (the
cc-prompt's Task item 1 only asks to update `coverage.rs`'s scope *filter*,
not to add named-arc-aware matching); (2) ODD-0025 §5 already anticipates
that coverage's heuristic matching is a stopgap — "the `source` record makes
coverage an exact set-difference rather than the heuristic match s01 used" —
so building named-arc-aware structural matching now would be throwaway work
once s06+ wires coverage against `source` directly. Named as a known
follow-on in the doc comment rather than silently left for someone to
rediscover.

### Note: `repair()`'s hash-gate fail-case is proven at the primitive, not the integration path

Identical situation to s03's F-6: `repair()` constructs `node_body` from the
exact same read as `source_body`, so a real mismatch cannot occur through its
own code today (an invariant-check gate, not a live-diverging comparison). No
new deviation — the same accepted shape, carried forward.

### Note: `body_source_path`'s catch-all branch is now provably unreachable, left as a safety net

`body_source_path` (s03) has a `_ => node.source.clone()` arm for node types
`discover()` never produces. Repair calls the same function, so this remains
true. Not touched — removing it would need a `#[non_exhaustive]`-safe
justification this slice doesn't need to make.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s04 deliver the MF-2/MF-3/MF-5 capability the arc-plan planned against?**
Yes, at the fixture-verified level the arc-plan scoped this slice to. MF-2
("body-hash gate green... zero stub bodies remain") now has both its
*mechanism* (s03) and its *repair path* (this slice) — but "zero stub bodies
remain" is a live-corpus outcome only s05's actual run produces. MF-3 ("every
migrated node carries a `source` sub-map") is now mechanically true for any
node either importer *or* the repair op touches — but the live 60-node corpus
still needs s05 to run it for real. MF-5 ("all arcs represented... scope no
longer capped") is now **mechanically true by construction** — there is no
code path left that excludes an arc by number — but again, the live corpus's
6 previously-missing arcs won't exist as nodes until s05 mints them.

**What implementing it revealed the arc-plan didn't anticipate:**

1. **A genuinely reusable design pattern for "preserve everything except X":**
   cloning the existing `Frontmatter` and mutating only the touched fields
   (rather than rebuilding one field-by-field) is both simpler code and a
   *stronger* correctness guarantee for "id/edges/status preserved" than the
   arc-plan's phrasing implied was needed. Worth naming as the default
   pattern for **s06's supporting-doc minting**, if it ever needs to update
   an existing node rather than only create new ones.
2. **The named-arc coverage-matching gap (Deviation above) is a concrete,
   nameable piece of s06+ scope**, not just an abstract "coverage will
   eventually be exact" note in ODD-0025 §5. Recommend s06's scope explicitly
   lists "wire `arc_coordinate` (or its replacement) against `source.paths`
   so a named arc's docs resolve correctly" as a named item, rather than
   leaving it to be rediscovered when `check` starts flagging false
   "uncovered" findings for already-covered named-arc docs.
3. **The schema-minor bump's actual blast radius was smaller than ODD-0020's
   v1.2 deferral note worried it would be** — because every production call
   site already used `SchemaMarker::current()`/`SchemaVersion::CURRENT`
   symbolically (never a hardcoded `"v1.0"` literal), the bump was a
   one-line constant change; the only work was updating **test** assertions
   that hardcoded the literal string, which a single targeted grep found
   completely (7 call sites across 4 files, all fixed, all now
   symbolic-or-intentionally-historical). Worth naming for whoever eventually
   does the *next* schema bump: grep for the literal version string in
   `tests/`, not just `src/`, before assuming the change is "just the
   constant."

**The slice-scale silent-drop diff:** scope-as-specified (slice-doc.md's
"In"/"Out") vs. scope-as-delivered — no drops. All five "Out" items (the live
run, `artifact` minting + check-wiring, synthesis, live `context.json`
re-pointing, re-keying idempotence on `source`) are confirmed absent —
verified by the unchanged live-store hash and by grep for their
would-be-visible traces (no `context.json` write path, no `NodeType::Artifact`
reference, no re-keying of `existing_work_keys`' `(type, number)` lookup).

**Recommended arc-ledger update (MF-2, MF-3, MF-5):** all three remain
**planned**. The capability each describes now exists and is fixture-proven
in full; the live-corpus outcome each actually asserts is s05's job.
Recorded as pointers from MF-2/MF-3/MF-5 to this closing report as baseline
evidence, per LEDGER-DISCIPLINE v2.0 §B (composition rows reproduce at arc
scale, at arc close — never inherited from a slice's fixture-only
attestation).
