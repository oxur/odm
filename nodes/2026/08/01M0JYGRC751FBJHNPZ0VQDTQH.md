---
id: 01M0JYGRC751FBJHNPZ0VQDTQH
number: 558406000
type: artifact
schema: artifact/v1.1
name: Slice 04 ledger -- cutover
created: 2026-08-21
updated: 2026-08-21
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-as-source/slice04-cutover/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-21
edges:
  part_of: 01M0JYGPK1P71JBXV699502MBN
---
# Slice 04 ledger -- cutover

Status: closed by CC on 2026-08-21.

| Row | Status | Evidence |
| --- | --- | --- |
| F-1 preflight | done | Code tree started clean after commit 988b4fa; store started clean at faa9ec3-ahead state; `odm orient` ran; initial `odm check` showed uncovered docs before reconciliation. |
| F-2 migrated mirror | done | `odm migrate --all --dry-run` previewed 4 source reconciles, 5 slice creates, 19 artifact mints, 2 artifact reconciles; real `odm migrate --all` ran; store commit `faa9ec32267ecfdd824dd88228e8814b4bddb42e`; `odm check` then exited 0 with 10 pre-existing warnings. |
| F-3 authored preview | done | `odm migrate --to-authored --dry-run` reported 85 work nodes would convert; code/test inspection confirmed former `source.paths` are preserved in `source.migrated_from`. |
| F-4 operator GO | done | Operator replied `GO` after the manifest for `docs/design-v1.0.0/**`, `docs/design/**`, `docs/dev/**`, bootstrap/dashboard relocation, config boundary, commits, and rollback path. |
| F-5 real authored conversion | done | `odm migrate --to-authored` converted 85 nodes; `odm migrate --all` then skipped authored nodes as self-sourced/untouched; store commit `00a314700c5bf8c12f47a4ff99fd7c2aff6e0c08`. Interim `odm check` failed only while legacy docs still existed and authored nodes no longer covered them. |
| F-6 approved deletion | done | Deleted the approved legacy planning/design corpus: `docs/design-v1.0.0/**`, `docs/design/**`, and classified legacy `docs/dev/**`; removed an untracked `.DS_Store` residue and empty directories. |
| F-7 retained files relocated | done | Moved `docs/design-v1.0.0/CDC-SESSION-BOOTSTRAP.md` to `CDC-SESSION-BOOTSTRAP.md`; moved `docs/design-v1.0.0/project-status.html` to `export/project-status.html`. |
| F-8 config boundary | done | Removed code locator `[legacy]`; removed store `[coverage] scan_root = "docs"` and `[legacy]` docs/dev roots. `odm migrate --all` now exits with named `no docs_directory configured`, confirming legacy migration is no longer active. |
| F-9 post-cutover commands | done | `odm check` exit 0 with the 10 known warnings; `odm orient` exit 0 with no structural errors and no drift; `odm node list` exit 0; `odm rollup --dry-run` exit 0. No missing-source, uncovered-doc, body-hash, or source-path error cites a deleted planning file. |
| F-10 workflow instructions | done | `AGENTS.md` now points to root `CDC-SESSION-BOOTSTRAP.md`, describes the deleted docs planning corpus, and preserves `CLAUDE.md -> AGENTS.md`. |
| F-11 SS-6 proof | done | Created authored arc #798832901 and slice #798832902 using `odm node new`; edited metadata with `odm node set`; edited body with `odm node set-body`; read both back with `odm node show`; `odm check` stayed at 0 errors. |
| F-12 commits and close | done | Store commits are separate from code commits: `faa9ec3` reconcile, `00a3147` authored conversion, `dea3e78` config boundary + SS-6 proof. This ledger and closing report are store-authored close artifacts; code-side cutover commit carries the deletions/relocations/instruction update. |

Known warnings are pre-existing decomposition/stub warnings except the proof arc, which was affirmed with `odm node decomposed 798832901` before close.
