# CC Prompt -- Slice 04 (Store-as-Source): cutover

You are CC implementing **Store-as-Source slice 04: cutover** in
`/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x`.

## Read first

1. `AGENTS.md`
2. `docs/design-v1.0.0/CDC-SESSION-BOOTSTRAP.md` sec. 0g
3. `docs/design-v1.0.0/project-plan.md` sec. 4a
4. `docs/design-v1.0.0/arc-store-as-source/arc-plan.md`
5. `docs/design/04-accepted/0026-store-as-source-model.md`
6. `docs/design-v1.0.0/arc-store-as-source/slice02-self-sourced-nodes/cdc-verification.md`
7. `docs/design-v1.0.0/arc-store-as-source/slice03-native-authoring/cdc-verification.md`
8. This slice's `slice-doc.md` and `ledger.md`

Run `./bin/odm orient` before making changes.

## Non-negotiables

- **Do not delete anything until the operator explicitly says GO** after seeing your dry-run and
  deletion/relocation manifest.
- Use odm-native store verbs for store state: `odm store status`, `odm store commit`, and `odm store
  sync` if needed. Keep store commits separate from code-branch commits.
- Preserve `CLAUDE.md -> AGENTS.md`. Edit `AGENTS.md`, not the symlink target through the symlink.
- Do not hand-edit store node files for normal authoring proof. The SS-6 proof must use
  `./bin/odm node new`, `node set`, and/or `node set-body`.
- Do not write external Codex memory. If a memory update is needed, record the proposed note in the
  close; the operator must request that separately.

## Phase A -- preflight, reconcile, preview, then stop

1. Record the starting state:
   - code branch status;
   - `./bin/odm store status --json`;
   - `./bin/odm check`;
   - `./bin/odm orient`.
2. If `check` reports uncovered docs from the recently formalized Store Lifecycle close set, reconcile
   them before cutover:
   - run `./bin/odm migrate --all --dry-run`;
   - run real `./bin/odm migrate --all` if the dry-run reports work;
   - persist store changes with `./bin/odm store commit -m "<message>"`;
   - rerun `./bin/odm check`.
3. Preview authored conversion:
   - run `./bin/odm migrate --to-authored --dry-run`;
   - inspect enough converted-node samples to prove former `source.paths` are preserved in
     `source.migrated_from`;
   - confirm no unrelated end-user docs are being converted as planning work.
4. Prepare the manifest:
   - delete list: `docs/design-v1.0.0/**`, `docs/design/**`;
   - classify list: `docs/dev/**` with a delete/keep recommendation and reason;
   - relocate/keep list: manual dashboard, CDC bootstrap, end-user docs, workflow instructions;
   - config plan for `[coverage] scan_root`, `legacy.docs_directory`, and `legacy.dev_directory`;
   - store/code commit plan and rollback path.
5. Stop and ask the operator for **GO**. Do not proceed to Phase B without it.

## Phase B -- only after operator GO

1. Run the real `./bin/odm migrate --to-authored`.
2. Run `./bin/odm migrate --all` and `./bin/odm check`.
3. Persist store changes with `./bin/odm store commit`.
4. Delete only the operator-approved legacy planning/design paths.
5. Apply the approved config boundary so future end-user `./docs` content is outside the planning
   corpus.
6. Move or preserve the dashboard/bootstrap exactly as approved.
7. Update `AGENTS.md` with the odm-native authoring workflow and any new bootstrap/dashboard location;
   keep `CLAUDE.md` as a symlink.
8. Run post-cutover verification:
   - `./bin/odm check`
   - `./bin/odm orient`
   - `./bin/odm node list`
   - `./bin/odm rollup --dry-run`
9. Reproduce SS-6 with a fresh context or equivalent clean transcript: author a real new arc+slice
   body using only `./bin/odm` commands, read it back with `node show`, and keep `check` green after
   each step.
10. Write `closing-report.md`, update this ledger, and bubble up to `arc-plan.md`, `project-plan.md`,
    and `project-status.html`.

## Acceptance

All F-1..F-12 rows in `ledger.md` are done or deliberately deferred with operator-approved follow-up.
No missing-source, uncovered-doc, body-hash, or source-path error may cite a deleted planning file.
The final state must have clean code and store worktrees, separate store/code commits, and the old
`./docs` planning authoring path retired.
