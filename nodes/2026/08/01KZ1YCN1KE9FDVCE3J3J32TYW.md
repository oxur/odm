---
id: 01KZ1YCN1KE9FDVCE3J3J32TYW
number: 518162700
type: artifact
schema: artifact/v1.1
name: 'Closing Report — Slice 16 (Migration Fidelity): orient P-12 readiness'
created: 2026-08-02
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice16-orient-p12-readiness/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KZ1YCK41BNR96RQ537Y9KBZW
---
# Closing Report — Slice 16 (Migration Fidelity): orient P-12 readiness

> Verified by: CC (this session). F-1/F-4 attested **and reproduced** (a real `odm orient` run against
> `.worktrees/odm`, the actual frozen corpus); F-2/F-3/F-5 attested. CDC reproduction is the open item.
> Closed 2026-08-02 on `release/1.0.x`. Fixture-only slice — no store commit; the real-store verification
> below is a read (`odm orient` writes nothing).

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative behind it.

**F-1/F-2 landed, but not on the first attempt — and the failure is the reason the diff looks the way it
does.** The literal criterion wording ("the `node_type() == Project` filter... also requires
`retired().is_none()`") describes checking `Frontmatter::retired()` directly. The first implementation did
exactly that, on the frontmatters `orient` already had in hand
(`commands::index_frontmatters`, which reconstructs `Frontmatter`s from the `.odm/` index rather than
parsing every file). It compiled, passed clippy, and **did nothing**: the fixture
(`orient_excludes_retired_project_from_selection`) still saw "2 projects, none selected" with the retired one
listed.

The reason: retirement is deliberately **not** part of the index projection (ODD-0014 §3.5) — the
index-reconstructed `Frontmatter` never carries a `Retirement`, only `IndexRecord.retired` (a bare flag,
kept for `list`'s F-15 default-hide, which reads it straight off the record rather than through the
frontmatter adapter). `odm-cli/src/commands.rs` already documents this exact gap, in the `check` command's
L-3b project-vision rule:

> `retired` is not part of the index projection (only id/type/edges/status — `Derived::load`'s doc), so it's
> read off the freshly-loaded `doc` below, not the index-reconstructed `fm`.

That comment predates this slice and describes precisely the trap the first attempt fell into. The fix
matches the pattern it already established: `is_retired(store, id)` does a **targeted per-node
`store.load()`** and checks the freshly-parsed `Document`'s frontmatter, not the index-reconstructed one.
This keeps the diff entirely inside `orient.rs` (no `odm-index`/`odm-core` change was needed once the right
predicate was found), satisfying F-4's "diff limited to `orient.rs`" — but it's worth naming clearly, since a
different (heavier) path was seriously considered and rejected: extending `odm-index`'s
`frontmatter_from_record` adapter to reconstruct a placeholder `Retirement` from the record's flag. That
would also have worked, but it would have widened the diff into a shared adapter every index-backed command
uses, for a fix only `orient`/`check` need — rejected in favor of the narrower, already-precedented
per-node-load pattern.

**F-3's audit found a second instance of the same leak, not just confirmed absence of one.**
`Rollup::assemble`'s `ready`/`blocked` computation has no notion of retirement at all (confirmed by grep —
no mention of "retired" anywhere in `odm-core`'s `rollup.rs`/`graph.rs`/`satisfaction.rs`), so a retired node
whose last-reached gate is non-terminal shows up in **both** `model.ready` and `model.blocked` identically.
The slice-doc's F-2 criterion named only READY; the audit (F-3) found BLOCKED needed the identical fix and
applied it — one `Vec::retain` call each, filtered once in `orient()` itself (before either the human
renderer or `orient_json` reads `model`), so both output paths stay consistent by construction rather than
by two separately-maintained filters.

**Everything else in `orient`'s output was audited and left alone**, because it is either already covered by
the F-1 fix (VISION only ever resolves to the now-non-retired project) or genuinely not an "orient
enumeration" of nodes by identity: CURRENT FOCUS shows whatever the operator explicitly `use`d (a selection,
not a list `orient` composes); INTEGRITY reports structural-validity findings, not a node roster; DRIFT/
DEFERRED are fact-level and probe-level, not retirement-keyed.

## Real-store reproduction (the point of the slice)

The fixture is necessary but, per the F-1 near-miss above, not sufficient evidence on its own — so this slice
was verified against the actual corpus the P-12 demonstration is about, not only a hand-built scenario. The
`odm` orphan-branch worktree wasn't checked out in this session's filesystem; `git worktree add .worktrees/odm
odm` attached the existing branch (a read-only checkout of already-committed history — no commit, no branch
mutation, and the directory is gitignored so it leaves no trace in `git status`). Then:

```
$ ./target/debug/odm orient
odm — orient

VISION  #1000 odm v1.0.0 — Project Plan (arc roadmap)
  ...
```

No "2 projects, none selected" prompt; `#1001` (the retired project) does not appear anywhere in the human
output or `--json` (`grep -n "1001"` over both: no match). This is the literal P-12 success criterion the
freeze surfaced as broken, now demonstrated clean.

## Scope discipline

Diff is `crates/odm-cli/src/orient.rs` + `crates/odm-cli/tests/orient.rs` only. No `odm-core` change, no
`odm-index` change (the rejected alternative above), no change to `next` or any other command, no change to
retirement semantics or the collapse — `#1001` is read (checked for a retirement marker) but never written,
by this slice or by anything it touches. `auto_stage_git`/store mutation are not in play at all (this is a
pure read command).

## Iterations

One implementation pass produced a plausible-looking but non-functional fix (the index-frontmatter check);
the fixture failure caught it immediately, and the corrected predicate (a targeted load, matching
`commands.rs`'s own established precedent) passed on the next attempt. Counted as a single iteration under
the five-iteration cap — the fixture did its job on the first real run, before any code was believed "done."

## v2.0 bubble-up → `../arc-plan.md`

- **s16 done**: `odm orient` demonstrates P-12 cleanly, verified against both a fixture corpus and the real
  frozen store (`.worktrees/odm`) — no retired node (project, ready, or blocked) is ever surfaced.
- **Carried forward as context for the arc-close and for F-22's remaining LLM-arc scope**: the
  "retirement is not part of the index projection" gap is a real, reusable trap — this slice is the second
  place (after `check`'s L-3b) to have hit it. Worth a note if/when F-22's broader scope (`next`, and any
  other surface) is picked up in the LLM-command-surface arc: check whether that surface reads
  index-reconstructed frontmatters before assuming a `.retired()` check on them will do anything.
- Per the arc-plan, **the arc-close now runs**: MF-9 composition (fidelity already reproduced, 0 drift) + the
  P-12 demonstration (this slice, now clean) + the bubble-up → Migration Fidelity closes.
