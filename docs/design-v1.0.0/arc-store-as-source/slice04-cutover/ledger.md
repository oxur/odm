# Slice 04 ledger -- cutover

Per `LEDGER-DISCIPLINE.md` sec. A. Cutover slice: rows reach `reproduced` only from live-corpus
command evidence plus file-state inspection. Destructive rows remain `open` until the explicit
operator GO is recorded.

| ID | Criterion | Verify | Significance | Origin | Status |
|----|-----------|--------|--------------|--------|--------|
| F-1 | Preflight snapshot proves the starting state before any conversion/deletion: code branch, store status, and initial `check`/`orient` findings are recorded | closing report includes `git status`, `odm store status`, `odm check`, `odm orient` summaries | serious | house cutover discipline | open |
| F-2 | The current corpus is first brought to a clean migrated mirror: `migrate --all --dry-run` is reviewed, real `migrate --all` runs if needed, store changes are committed, and `check` is green before `--to-authored` | command transcript; `odm store commit`; `odm check` exit 0 before conversion | serious | arc-plan slice 04 | open |
| F-3 | Existing planning work nodes are converted with the explicit one-time conversion path, preserving former source paths in `source.migrated_from` | `migrate --to-authored --dry-run` reviewed; real `migrate --to-authored`; node samples show `origin: authored`, `source.class: authored`, and `migrated_from` | serious | ODD-0026 sec. 2.1; slice 02 | open |
| F-4 | The cutover has an explicit GO checkpoint: no deletion happens until the operator sees the conversion preview, deletion/keep/relocate manifest, config plan, commit plan, and rollback path | closing report quotes the operator GO and the manifest it approved | serious | arc-plan explicit-go gate | open |
| F-5 | Deletion is scoped correctly: `docs/design-v1.0.0/**` and `docs/design/**` are removed; end-user docs are preserved; `docs/dev/**` is deleted only if classified and approved | file tree evidence before/after; manifest maps delete/keep/relocate decisions to paths | serious | ODD-0026 sec. 2.3/2.4 | open |
| F-6 | Post-cutover config no longer treats future conventional `./docs` content as planning corpus by default | operational config diff; `check` on a docs tree with no planning sources does not produce uncovered planning-doc errors | serious | project-plan sec. 4a; ODD-0026 fork D | open |
| F-7 | The manual dashboard and CDC bootstrap are removed from the store-covered planning docs tree without being lost; their post-cutover homes or explicit follow-ups are recorded | file tree evidence and AGENTS/bootstrap pointer update, or closing-report follow-up with owner | correctness | bootstrap NEXT-SESSION agenda | open |
| F-8 | `AGENTS.md` documents the odm-native authoring workflow and `CLAUDE.md` remains a symlink to `AGENTS.md` | file read plus `test -L CLAUDE.md`; no hand-authored planning files are instructed under `docs/design-v1.0.0/` | serious | project governance | open |
| F-9 | With the legacy planning/design docs gone, `check`, `orient`, `node list`, and `rollup --dry-run` remain green/coherent; no missing-source/uncovered-doc/body-hash errors cite deleted planning files | post-deletion command transcript | serious | arc SS-7 | open |
| F-10 | SS-6 is reproduced at arc scale: a fresh context authors a real new arc+slice body end-to-end using only `./bin/odm`, no `./docs` planning tree and no hand-edited store files | fresh-context transcript; `node show`; `check` after each authored step | serious | arc SS-6 | open |
| F-11 | Store commits and code commits are separated: store changes are persisted with `odm store commit`; code-branch deletion/config/doc changes are committed separately; both worktrees end clean | `odm store status`; `git status`; commit summaries | correctness | Store Lifecycle SL-4 | open |
| F-12 | Bubble-up is completed: arc-plan, project plan/status, and close docs record SS-6/SS-7 results, residual warnings, and the next arc/slice after cutover | updated arc-plan/project status; closing-report bubble-up section | serious | project-management close discipline | open |

## Closure notes

This ledger is intentionally stricter than a normal docs slice. `migrate --to-authored` freezes
further `./docs` edits to converted nodes, and deleting the old planning tree removes the fallback
authoring surface. The preview and GO gate are part of the acceptance criteria, not ceremony.
