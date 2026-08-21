# Slice 04 -- cutover

<!-- Name/title carries no document-role metadata, per ODD-0013 sec. 2.1. -->

**Arc:** Store-as-source-of-truth & native authoring -- **Kind:** code + docs
**Opened:** 2026-08-21 -- **Design basis:** ODD-0026 sec. 2.3, 2.4, 7; Store-as-Source
slices 02/03; project-plan sec. 4a.

## Goal

Move odm from dual-track planning (`./docs` as source, store as faithful mirror) to store-only
planning: existing planning work nodes are converted to `origin: authored`, the legacy planning
and design trees are removed from the code branch, and the remaining workflow no longer treats
future end-user `./docs` content as the planning corpus.

The single sentence this slice makes true: *after a dry-run-first, explicit-go cutover, odm's
planning corpus is authored in the store, `./docs` no longer contains or drives planning/design
source files, and `check`/`orient`/`node list`/`rollup` still prove the same project state without
hand-edited store files.*

## Context

ODD-0026 is accepted. Slice 02 made self-sourced authored planning nodes valid and introduced the
explicit `migrate --to-authored` conversion pass. Slice 03 added the programmatic authoring surface
(`node new`, `node set`, `node set-body`) that lets future plan work happen through `./bin/odm`.
Store Lifecycle is closed, so store persistence and sync have native verbs.

The corpus is still in the old shape until this slice runs: most planning nodes remain migrated
mirrors with `source.paths` pointing into `docs/design-v1.0.0/` or `docs/design/`, and the
operational config still carries legacy `docs_directory`, `dev_directory`, and coverage roots.
That makes this slice the point of no return. It is git-recoverable, but it is still destructive
to the legacy authoring surface, so it must stop for an explicit operator **GO** after the dry-run
and deletion manifest.

## Scope -- in

1. **Preflight and evidence snapshot.** Confirm the code branch and store worktree state before any
   conversion or deletion. Record the code branch, store status, store HEAD if available, and the
   starting `check`/`orient` state. If current uncovered-doc errors are only newly added formal docs,
   run the normal `migrate --all`/artifact minting path first and make that explicit in the close.
2. **Final safety migrate before conversion.** Run `./bin/odm migrate --all --dry-run`, then the real
   `./bin/odm migrate --all` if the dry-run reports work. Commit store changes with `odm store
   commit`. `./bin/odm check` must be green, apart from already-accepted warnings, before
   `--to-authored` is allowed to run.
3. **Authored conversion, preview first.** Run `./bin/odm migrate --to-authored --dry-run` and record
   the exact planned conversion shape. The dry-run must show conversion of existing planning work
   nodes while preserving former source paths in `source.migrated_from`; it must not silently convert
   unrelated end-user docs.
4. **Deletion and relocation manifest.** Produce a manifest before deletion:
   - **Must delete:** `docs/design-v1.0.0/**` and `docs/design/**`, per ODD-0026 sec. 2.3.
   - **Classify before delete:** `docs/dev/**`, because the legacy config still names it as a dev-doc
     corpus. Delete it only if the manifest proves it is planning/legacy corpus, not end-user docs,
     and the operator includes it in the GO.
   - **Must keep or relocate:** end-user docs; the manual status dashboard; the CDC bootstrap; any
     non-store workflow instructions that must survive the cutover.
5. **Explicit operator GO.** Stop after the dry-run and manifest. The GO packet must include the
   commands run, the exact delete/keep/relocate list, planned config changes, planned store/code
   commits, and rollback path. No deletion happens without the operator's explicit GO in the
   session.
6. **Cut over after GO.** Run the real authored conversion, rerun `migrate --all`, rerun `check`, and
   commit store changes with `odm store commit`. Delete the approved legacy planning/design docs from
   the code branch. The final code commit should contain only the planned code-branch changes.
7. **Config boundary after deletion.** Update the operational config so future end-user `./docs`
   content is not treated as the planning corpus. That means removing, disabling, or narrowing
   legacy `[coverage]`, `docs_directory`, and `dev_directory` settings, with the exact choice recorded
   in the closing report. The invariant is: future conventional docs under `./docs` do not become
   coverage failures merely because they are not planning nodes.
8. **Workflow surface update.** Update `AGENTS.md` (canonical) and preserve `CLAUDE.md -> AGENTS.md`.
   The instructions must say new project/arc/slice work is authored through `./bin/odm`, not by
   creating files in `docs/design-v1.0.0/`. If the CDC bootstrap moves, update the bootstrap pointer.
   External Codex memory is not edited by CC; any memory update is a separate operator-explicit
   action.
9. **Arc-scale proof.** Demonstrate SS-6/SS-7: after the legacy planning docs are gone, a fresh
   context can author a real new arc/slice body through `./bin/odm` only, and `check`/`orient`/
   `node list`/`rollup --dry-run` remain green and coherent with no hand-edited store files.

## Scope -- out

- Interactive `$EDITOR` or heading-section editing (`node edit --body`, `--meta`, `--section`);
  that remains slice 07.
- New query/read surfaces from the LLM command-surface arc.
- A general export system or static HTML generator. This slice may move the manual dashboard out of
  the store-covered docs tree, but automated export is a later arc.
- Changing end-user documentation content beyond the minimum needed to preserve or relocate it.
- Writing external Codex memory. Record the proposed memory note or follow-up, but do not mutate it
  unless the operator explicitly asks in that moment.

## Verification approach

This is a live-corpus cutover, so the acceptance proof is primarily command evidence plus file-state
inspection. The close must include:

- preflight state, including code branch status and `odm store status`;
- `migrate --all --dry-run` and real `migrate --all` results before conversion;
- `migrate --to-authored --dry-run` results, followed by the operator GO;
- real conversion results, store commit evidence, and post-conversion `check`;
- the exact deletion/relocation manifest and final file tree evidence;
- post-deletion `check`, `orient`, `node list`, and `rollup --dry-run`;
- the SS-6 fresh-context authoring transcript.

Warnings are allowed only if they are already-known decomposition/sizing warnings explicitly
accepted by the arc. Missing-source, uncovered-doc, body-hash, or source-path errors tied to the
deleted planning docs are blocking.

## Exit criteria

`docs/design-v1.0.0/` and `docs/design/` are gone from the code branch; any `docs/dev/` disposition is
operator-approved and recorded; end-user docs are not lost; the dashboard/bootstrap have an explicit
post-cutover home or an explicit, time-boxed follow-up; `AGENTS.md` documents the odm-native workflow;
`CLAUDE.md` remains a compatibility symlink; the store is committed using `odm store commit`; the code
worktree is clean after the code commit; SS-6 and SS-7 can be CDC-verified.
