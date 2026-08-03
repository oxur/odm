---
id: 01KYP5FSFQ4TYBDXX6YNAVA499
number: 543468700
type: artifact
schema: artifact/v1.1
name: CDC Session Bootstrap — pick up where we left off
created: 2026-07-25
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/CDC-SESSION-BOOTSTRAP.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-03
---
# CDC Session Bootstrap — pick up where we left off

> **The canonical bootstrap for the CDC/CC collaboration on `odm` — living resume + genesis.**
> Read this first in a new session to reach full situational awareness without re-reading the
> whole history. Updated at session close. **Resume last updated: 2026-08-02 (evening)** (§0f — current; §§0e–0 and §§1–7 below are history). **2026-07-25:** the genesis doc (formerly
> `workbench/odm-session-bootstrap.md`, 2026-06-19 — why the project exists) was **merged in
> as §8** and this file made the single canonical bootstrap; §§1–7 (the "what's true now / do
> this next" resume) are unchanged from 2026-07-07.

---

## 0f. Resume update — 2026-08-02 (evening) (closed arcs recorded DONE; Store-as-Source arc born; store commit index fixed — START HERE)

**This is the current "read first." §§0e–0 below are prior resumes, now history.** You are **CDC** —
the independent planner/verifier. Loop unchanged: **draw** each slice's open set
(`slice-doc`/`ledger`/`cc-prompt`), **CDC-verify** CC's reports (reproduce-don't-attest), **maintain the
arc-plan + the dashboard** via bubble-up. Peer frame, own your errors plainly, calibrated honesty. Read the
collaboration-framework docs IN FULL at start.

### What changed since §0e
- **The closed arcs are now recorded DONE (operator call).** `project-plan.md` → **v1.18**: SH bumped
  `attested → done` (**P-14**), and **P-15 (Release Hardening)** + **P-16 (Migration Fidelity)** added as
  `done` (own P-rows, resolving the v1.8 open question). The **dashboard** (`project-status.html`) recolors
  `--closed` → green so **RH/SH/MF render green** like `complete`; **SL/A6 (paused) stay their own
  orange** — distinct status, not recolored. A1–A5 already `complete`.
- **Store-as-Source arc BORN** (`arc-store-as-source/arc-plan.md` **v1.6**). This is the "take the training
  wheels off" arc: odm authors its **own** planning docs (store = source of truth); `./docs` is freed for
  **end-user documentation only**. Slices: **01** ODD-0026 model (forks **A/B/D/E open**, C settled) · **02**
  self-sourced planning nodes (no `undeveloped`/missing-source error for a store-authored node) · **03**
  native authoring commands (`node new --from-file/--body/$EDITOR`; `node edit <ref>`) — **this is the
  "native editing / content support"** · **04** cutover (delete the `./docs` planning subtree) · **05**
  decomposition-bookkeeping consistency (**CDC-verified PASS**, `c0f4773`) · **06** auto-extend authored
  decomposition (**CDC-verified PASS**, `7cd5e01`).
- **`migrate --all` is now HANDS-OFF** for authored additions (slice 06 / SS-9): an already-affirmed parent
  that gains a plan-tree-declared work-child auto-extends its decomposition — **no manual `node
  decomposed`**. Reproduced on the real corpus: `RECOMPOSE` = `0 re-affirmed, 1 auto-extended, 0 drift`,
  `check` 0 errors. Removals and never-affirmed parents still behave conservatively (CC correctly
  distinguished a *vanished* work-child from a *present non-work* id).
- **`store commit` leaves the git index CLEAN** (Store-Lifecycle **slice 04 / SL-6**, **CDC-verified PASS**,
  `e8e69de`). `commit_all` now syncs the index from the committed tree (`sync_index_to_tree`), so a raw
  `git status` reads clean after a commit. **This root-causes + kills the recurring "phantom
  staged-deletion" scare** (the commit was always correct; the index was stale) and **retires the manual
  `git reset` step** on the operator's next `store commit` with the rebuilt binary.
- **LLM-command-surface arc RECONCILED to v1.4** (not started). B2-3 folded in as **slice 10** (name/title
  de-redundancy — restore ODD-0013 §2.1; ledger **A-12**) = **the "title cleanups"**; Batch-2 (B2-1→02,
  B2-2→08) + F-21→01, F-22→02, CDC-ARC-1/L-8→09; re-grounded 60→400. Critical path 01(+07)→08→02→03;
  slice 10 pairs with 08.

### Where "native editing" and "title cleanups" are tracked (operator asked 2026-08-02)
- **Native editing (content) support** → **Store-as-Source slice 03** (`node new --from-file/--body/$EDITOR`,
  `node edit`); ledger **SS-4**, rolls into arc DoD **SS-6**; design half = **ODD-0026 fork E** (lean, not
  decided). **Open** — gated behind slice 01 (ODD-0026) + slice 02 (self-sourced nodes).
- **Title cleanups** (the `(plan-of-record)`/role-metadata redundancy) → **LLM arc slice 10**, ledger
  **A-12**. **Open** — the whole LLM arc is reconciled but not started. Going-forward half already in effect
  (CDC stops adding the labels); the repair (24 store names + 57 H1 titles) + normalizer fix + `check` guard
  are unbuilt.

### Arc board (2026-08-02 evening)
A1–A5 **complete** · **RH / SH / MF closed = DONE** (green) · **SL paused** after s01+s04 (s02 `status` /
s03 `sync` deferred) · **A6 paused** (PM-skill s05 / retire-prose s06 remain — still P-12's gate) ·
**LLM-command-surface reconciled v1.4, NOT started** · **Store-as-Source design-first** (s05/s06 done; s01
ODD-0026 + s02/s03/s04 remain) · A7/A8 **scoped**.

### HEADs & canonical files (2026-08-02 evening)
- `release/1.0.x` HEAD **`e8e69de`** (slice-04 store-commit index fix) — **CI-green, operator-attested**.
- odm store branch HEAD **`24f1037`** (**425 nodes**), migrated clean + committed.
- Canonical: `project-plan.md` **v1.18** · `project-status.html` (green recolor) · `arc-store-as-source/`,
  `arc-llm-command-surface/` (v1.4), `arc-store-lifecycle/` (v1.1 + s04) arc-plans.

### Do-this-next
- **Operator:** rebuild `./bin/odm` to pick up slice 04; on the next `store commit`, confirm raw `git
  status` is clean (closes SL-6's real leg, retires the manual `git reset`).
- **Decide ODD-0026 forks A/B/D/E** (leans recorded in `arc-store-as-source/arc-plan.md` slice 01) to open
  Store-as-Source → then s02 (self-sourced) → s03 (native authoring / "native editing").
- **LLM arc is ready to start** (v1.4): 01(+07)→08→02→03; slice 10 (title cleanups) pairs with 08.

### Gotchas re-confirmed this session (they WILL bite)
- **Stale mount-cache:** the file-staging pipeline can serve a STALE copy of a plan-tree file. **Ground-truth
  via `device_bash` before editing** any plan-tree file (bit us twice: dashboard + this bootstrap both
  looked older in `/tmp` than live).
- **File-transfer tools don't reach the worktree:** `device_commit_files` / `device_list_dir` reject
  `.worktrees/1.0.x/...` ("not inside a connected folder"), **but `device_bash` writes there fine.** So
  **edit live plan-tree files in place via `device_bash`** (python/sed), not SendUserFile→commit.
- **git:** `GIT_DIR=$R/.git/worktrees/1.0.x` + `GIT_WORK_TREE=$R/.worktrees/1.0.x` (store: `.../worktrees/odm`).
- **CDC sandbox can't run macOS `odm`/cargo** — attest cargo/exec rows → CI; reproduce structural rows by
  direct file read. `device_bash` **cannot delete** — I moved a stray backup to
  `design-v1.0.0/_to_delete/project-status.html.bak-precolor`; **operator: delete that `_to_delete/` folder.**

## 0e. Resume update — 2026-08-02 (Migration Fidelity CLOSED; Store Lifecycle paused; the LLM arc is next) — HISTORY (superseded by §0f)

**This is the current "read first." §§0d–0 below are prior resumes, now history.** You are **CDC** —
the independent planner/verifier. Loop unchanged: **draw** each slice's open set
(`slice-doc`/`ledger`/`cc-prompt`), **CDC-verify** CC's reports (`cdc-verification.md`,
reproduce-don't-attest), **maintain the arc-plan + the dashboard** via bubble-up. Peer frame, own your
errors plainly, calibrated honesty. Read the collaboration-framework docs IN FULL at start (§0's process
rule still holds).

### Where the project stands (the big shift since §0d)
**Migration Fidelity is DONE, and it was the release-blocking one.** The whole arc that §0d was mid-flight
on (s08…) ran to completion: **s04–s16 all CDC-verified PASS**, the arc-close freeze fired (project-vision
pair collapsed to one faithful 1:1 node — `#1000` re-cast, `#1001` retired — **0 drift**, `check` exit 0),
the **P-12 `orient` self-host demo runs clean**, and the store-side close landed (arc node `58837400` →
`complete` gate + `decomposed` affirmed, 17 children). **Closed on doc AND store AND CI-green.**

Current arc board (design order):

| Arc | State (2026-08-02) |
|-----|--------|
| A1–A3 | ✅ complete (MVP) |
| A4 Index / A5 Reconciliation | ✅ complete, CI-green |
| **A6 Migrate/self-host/PM-skill** | ⏸ **PAUSED after slice04** — resumes at **slice05 (PM-skill)** *after* the LLM arc, against the settled surface. Its `→ done when reproduces (CI green)` ledger rows flip at **A6's own arc-close**, not on general branch-green. |
| Release Hardening (RH) | ✅ **CLOSED 2026-07-27**, CI-green (C-1…C-6, C-8; C-7 folded into C-5) |
| Store Home & init (SH) | ✅ **CLOSED**, CI-green (orphan `odm` branch is live; two-config split) |
| **Migration Fidelity (MF)** | ✅ **CLOSED 2026-08-02**, CI-green. Carry: **MF-4 → L-8 / CDC-ARC-1** (14 RH-era design nodes dropped source `version` → design-corpus migrate + a standing frontmatter-fidelity check) — **routed into the LLM arc.** |
| **Store Lifecycle (SL)** | ⏸ **PAUSED after s01** — `odm store commit` landed + CDC-verified PASS + CI-green (SL-1). **s02 `store status` / s03 `store sync` deferred** by operator call. Resume only if Duncan lifts the pause. |
| **LLM command surface** | ◆ **THE NEXT ARC** — shaped, not started (`arc-llm-command-surface/arc-plan.md`; reconciled vs `../odm-command-inventory.md`). Now holds carried F-21 (RH) + CDC-ARC-1/L-8 (from MF). Still §4's oxur-route history behind it, but that's settled (oxur-term extracted; see RH close). |
| A7/A8 | post-MVP, **another CDC owns** — ignore. |

### ⭐ The dashboard is YOURS to maintain (I forgot this once — don't)
`docs/design-v1.0.0/project-status.html` is a **CDC-maintained** status dashboard — not Duncan's. It's a
self-contained data-driven HTML: a `const DATA = {groups:[{arcs:[…]}]}` object rendered into cards; the
summary tallies auto-compute from it. To update: edit the `DATA` object (arc `status ∈
complete|closed|active|planned|scoped|tentative|paused`; slices `[["01","name","done|plan|wip"]]`), then
**validate it parses** before shipping — `node -e` a brace-balanced extract of `const DATA` through
`Function("return ("+lit+")")()`. Deliver via **SendUserFile → device_commit_files**; Duncan commits it.
Do **not** push it to the artifact gallery (`create_artifact`) — he declined that; it lives in-repo.

### ⚙ Bridge/git mechanics that WILL bite you (hard-won this session)
The session may start with **the mount gone** (fresh cloud container). Re-establish: `get_device_info` →
folder is `/Users/oubiwann/lab/oxur/odm`, mounted at `$(pwd)/mnt/odm` under `device_bash` (path like
`/sessions/<id>/mnt/odm`). **`device_bash` runs in a Linux VM on Duncan's Mac** — so **macOS `odm`/cargo
binaries won't run there** (Exec-format): keep reproducing store state by direct git/file read, attest
runtime rows → CI.

Committing on `release/1.0.x` from the bridge, the reliable recipe:
- Address the worktree explicitly: `export GIT_DIR="$R/.git/worktrees/1.0.x" GIT_WORK_TREE="$R/.worktrees/1.0.x"` (the worktree `.git` file holds absolute macOS paths that don't resolve in the VM).
- **Set identity every session** — a fresh container has none, and a bare commit dies `exit=128 "author identity unknown"`. Export `GIT_AUTHOR_NAME=CDC GIT_AUTHOR_EMAIL=oubiwann@gmail.com` + the two `GIT_COMMITTER_*` twins. (Prior CDC commits are authored `CDC <oubiwann@gmail.com>`.)
- **Lock litter:** the mount **blocks `unlink`**, so git leaves `index.lock`/`HEAD.lock` behind and the *next* git op dies `exit=128 "File exists"`. Clear them by **atomic rename to a UNIQUE name** — `mv -f "$GD/index.lock" "$GD/index.lock.stale.$$"`. A fixed `.trash` name can itself get stuck; the `$$` suffix is what finally worked.
- **Single-lock-cycle commit:** (call 1) clear locks → `git add <files>`; (call 2) clear locks → `git commit` on the **already-staged index (no add)**. Two adds in one lock window collide.
- `warning: unable to unlink '…/objects/…/tmp_obj_…'` is **harmless** (objects land via rename); filter with `grep -v tmp_obj`. Commit success = `git log -1` moved, regardless of the warnings.
- Re-stage a file before editing if Duncan may have committed it meanwhile (his commit changes the device mtime; `device_commit_files`' `expectedMtimeMs` guard will reject a stale write — get the fresh mtime via `device_list_dir`).

### Evidence convention that got set this session
**Operator-attested CI is a legitimate evidence source** — when Duncan says "the push is green," record it
as `CI-green (release/1.0.x @<sha>, operator-attested)`, not reproduced-by-you (you can't run the macOS
CI). **Don't rewrite dated history** to reflect new CI state — add a **new dated changelog/version entry**
and update only the *live* status surfaces (headers, current-state cells, DoD rows, dashboard). Green SHA
this session: **`e4e0a95`**.

### One lesson worth carrying (own your prompt defects)
In s16 my cc-prompt specified the retired-node filter as `retired().is_none()` on the frontmatter `orient`
already has — a **silent no-op**, because retirement isn't in the `.odm/` index projection those
frontmatters are reconstructed from (ODD-0014 §3.5). CC caught it via a real-store fixture and used the
correct `store.load()`-based check (the `commands.rs` `check` precedent). I owned it in the verification as
a **CDC spec defect caught by CC's fixture discipline** — not a CC defect. Two transferable rules: **any
predicate touching retirement/supersession must `store.load()`, never trust the index projection**; and
when you plant a spec, name your own defect plainly when the fixture exposes it.

### HEADs & canonical files (2026-08-02)
- `release/1.0.x` HEAD: **`a9d3132`** (CI-green trueup) atop `e4e0a95` (dashboard) / `9cf2cdb`, `34daad4` (MF close).
- `odm` store branch HEAD: **`6225d1f`** (arc node complete+decomposed) atop `41ace1f` (the MF freeze, committed by `odm store commit`).
- Live store = `.worktrees/odm/` (nodes under `nodes/YYYY/MM/<ULID>.md`). Plan tree = `.worktrees/1.0.x/docs/design-v1.0.0/`.
- **`command-surface-uat-checklist.md`** — the live punch-list; Batch-2 open (B2-1 `next` grouping/tree; **B2-2 retire numeric handles from display AND input — `node show` takes a ULID**; B2-3 title "(plan-of-record)" redundancy). These are LLM-arc-adjacent.
- Node identity (decided, don't relitigate): **ULID `id` is identity; every edge references ULIDs; `number` is display-only** and Duncan wants it gone from display + input (→ the LLM arc / B2-2).
- Canonical: `arc-migration-fidelity/{arc-plan.md (v2.35), closing-report.md}`, `arc-store-lifecycle/arc-plan.md`, `arc-llm-command-surface/arc-plan.md`, `project-plan.md` (v1.17; P-12 = self-host DoD), `project-status.html`.

### Do-this-next
1. Read this + `arc-llm-command-surface/arc-plan.md` + `../odm-command-inventory.md` + `command-surface-uat-checklist.md`.
2. Confirm with Duncan whether to **start the LLM arc** (its slice01/02 are small — the data is already computed, just not printed) or lift the **Store Lifecycle** pause for s02/s03 first. Ask in prose.
3. When drawing LLM slices, fold in the carried **CDC-ARC-1/L-8** (design-corpus migrate + frontmatter-fidelity check) and **F-21**.
4. Keep the dashboard current at each milestone; commit via the recipe above.

---

## 0d. Resume update — 2026-07-28 (Migration Fidelity arc mostly landed; verify CC's s08 next) — HISTORY (superseded by §0e; MF is now CLOSED)

**This is the current "read first." §§0c–0 below are prior resumes, now history.** You are **CDC** —
the independent planner/verifier in the CDC/CC collaboration on `odm`. Your loop: **draw** each slice's
open set (`slice-doc.md` / `ledger.md` / `cc-prompt.md`), **CDC-verify** CC's implementation reports
(writing `cdc-verification.md`), and **maintain the arc-plan** via bubble-up. CC implements; you plan
and verify. Peer frame, own your errors, calibrated honesty (collaboration-framework).

### The arc and where it stands
**`arc-migration-fidelity/`** (release-blocking, P-12 DoD path) makes migration faithful/verifiable and
repairs odm's own self-hosted corpus (was 44 stubs / 6-of-12 arcs / 0 provenance) to 100%. Model ODD:
**ODD-0025** (Accepted; you authored it). Arc-plan: **`arc-migration-fidelity/arc-plan.md` — v2.9**.

- **s01–s07 CLOSED.** s04–s07 **CDC-verified PASS**. s07 was the arc's **first live mutation** — the
  live corpus is now faithful: **77 nodes, 0 stubs, 61 source-bearing, all 12 arcs represented**, `check`
  green (committed on the `odm` branch, `7b4eb57`).
- **s08 — source-path portability: DRAWN, CC-ready** (`slice08-source-path-portability/`). **← your
  pickup is verifying CC's s08 report.** It relativizes `source.paths` (absolute today → repo-content-root
  relative) and live-rewrites the 61 nodes. The crux/risk: the **transition-safe `by_source` guard**
  (F-4) — an absolute-stored node and a relative-discovered lookup must collapse to the same key, or the
  rewrite mints 61 duplicates. The dry-run MUST show **61 rewrites / 0 creates**; any create = guard
  wrong = stop.
- **s09** coverage enforcement (artifact mint + doc-coverage into `check` + `coverage.rs` Findings 2–3 +
  the 14 design/research nodes' `source`), **s10** synthesis + L-8b, **s11** reconcile run — planned.

### How to CDC-verify a report (the method that's worked)
Read `closing-report.md` + `ledger.md` + the committed artifacts, then **reproduce, don't just attest**:
- **Live store** = `.worktrees/odm/` (orphan `odm` branch), nodes under `nodes/`. **Plan tree** =
  `.worktrees/1.0.x/docs/design-v1.0.0/`. On the device both are under `mnt/odm/` (`device_bash`).
- **Constraints:** git is unreachable in the VM; there is no runnable `odm`/cargo (the built binaries are
  macOS — Exec-format-error on Linux). So cargo/`check`/`orient`/git rows are **attested-by-CC → CI**;
  everything about *store state* you reproduce by **direct file read** + independent recomputation:
  count stubs/source-bearing/schema; enumerate the source-less plan nodes (should be exactly project +
  retired); **recompute the body-hash gate yourself** (no stored hash: re-read each node's `source.paths`,
  normalize `trim+lf`, compare to body — remap the `/Users/oubiwann/...` prefix to `mnt/odm/` to resolve
  files, until s08 makes them relative); **coverage set-difference** = plan-tree `arc-plan.md`/`slice-doc.md`
  vs node `source.paths`.
- **Assess each finding's severity independently** — elevate if warranted (I raised s07's absolute-path
  finding from "routine follow-up" to s08's own slice). Ratify/adjust the MF bubble-up. Write
  `cdc-verification.md` in the slice dir; then update `arc-plan.md`: flip the slice to **CDC-verified
  PASS**, carry findings forward.

### Mechanics / gotchas
- **Ground-truth before every arc-plan edit** (`device_bash cat`/`grep`) — a stale mount-cache once
  served an old version and nearly reverted bubble-ups. Edit in place via **python truncate-rewrite**
  (`sed -i`'s unlink is blocked); anchor on ASCII substrings (unicode em/en-dashes bite `.replace`).
- Deliver docs with **SendUserFile → device_commit_files** (needs the *real* returned `file_uuid`;
  target `/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/.../slice-dir/`).
- **Renumbering** slices is an accepted operation — record the mapping in a new arc-plan history entry;
  don't rewrite old history entries.

### Decided model / standing preferences (don't relitigate)
- **ODD-0025:** `origin` (why) / `source` (where content came from — stored) / `provenance` (derived,
  reserved). 1:1 verbatim bodies + a **hard body-hash gate** (`normalize`=trim+lf, **no stored hash**).
  Project node (synthesis) and **retired** nodes are excluded from 1:1 `source`. `author`/`version` are
  document-node fields (N/A on the frontmatter-less plan corpus).
- **s05:** `source.paths` is the **identity key**; `number` is a display handle, keys nothing for
  correctness ("don't keep running into the number thing"). **s08 makes that key portable** — absolute
  paths reintroduced the same class of fragility via the path, which is why it's its own slice.
- **Operator preferences:** collaboration-framework peer frame; **build capability, then live run,
  unbundled** (s06/s07, s08); don't defer needlessly; own errors plainly.
- **s11 carry:** living plan docs drift from their migrated snapshots the instant they're re-edited, and
  `reconcile_source` *rejects* a non-stub whose body ≠ source — s11's arc-close reconcile must treat a
  legitimate source change as update-to-re-snapshot, not drift-to-reject.

### Canonical files
`arc-migration-fidelity/{arc-plan.md (v2.9), design-notes.md}`; `docs/design/04-accepted/0025-migration-fidelity-model.md`;
each `slice0N-*/` holds its `{slice-doc,ledger,cc-prompt,closing-report,cdc-verification}.md`.

---

## 0c. Resume update — 2026-07-27 (Migration Fidelity arc shaped; slice 1 ready — start here)

**A new release-blocking arc is planned: `arc-migration-fidelity/`** — it makes migration faithful and verifiable (1:1 verbatim bodies + a hard body-hash gate; a `provenance` sub-map; frontmatter-fidelity; a **doc-coverage** check) and repairs odm's own self-hosted corpus from skeleton — **44 stub bodies, 6 of 11 arcs, ~211 uncovered docs, 0 provenance** — to **100%**. Shaped from `reconciliation-audit-2026-07-27.md`; decisions in `arc-migration-fidelity/design-notes.md`; roadmap in project-plan §2a + v1.13. **Subsumes L-8b; on the P-12 DoD path.**

**Pick up here:** the arc's **slice 01 (coverage-discovery)** open set is written and CC-ready — `arc-migration-fidelity/slice01-coverage-discovery/{slice-doc,ledger,cc-prompt}.md`. It is **read-only** (build a coverage/gap detector, produce the exact inventory `coverage-report.md`; mints nothing). Hand `cc-prompt.md` to CC. **G-1 is closed** (ODD-0024) so the later minting slices are unblocked.

---

## 0b. Resume update — 2026-07-27 (G-1 CLOSED — read this first)

**✅ G-1 is CLOSED; the minting freeze is LIFTED.** Per **ODD-0024** (Accepted,
`docs/design/04-accepted/`): odm **retains ULID** identity; the register-style `D-YYMM-XXXX`
scheme is **rejected**; `number` is unchanged; no id is re-stamped. **You may mint new nodes.**
Any "do not mint before G-1" warning below (§0a, §0) is **superseded** by this. The id-scheme
question does not reopen without superseding ODD-0024.

---

## 0a. Resume update — 2026-07-26 (RH C-5: the cutover — read this first)

**⚠ The corpus moved. `nodes/` is not in the working tree any more.** odm now
dogfoods the store home its own arc built: the 60 nodes live on the orphan
**`odm` branch**, checked out at **`.worktrees/odm/`**. `odm.toml` at the repo
root is a **locator only**; the operational config (gate-sets, display,
`docs_directory`) is `.worktrees/odm/config.toml`, versioned with the data it
governs. Run `odm` from the repo root exactly as before — resolution follows the
locator — but read node files under `.worktrees/odm/nodes/`, and **commit corpus
changes on the `odm` branch** (`git -C .worktrees/odm …`), which is a separate
history from the code. `/.worktrees/` is gitignored on the code branch on purpose.

**`odm orient` now tells the truth on odm's own repo.** It used to print "no
vision text yet" and "(no current arc)". The project node carries a `# Vision`
body from project-plan §1, and the current focus (arc #1600) is committed *with*
the store, so a fresh session gets both — which is the project's stated success
test.

**Names and dates are real now.** Work-node names dropped their coordinates and
role suffixes (44 of 45 changed): `"Slice 01 (Arc 02) — Foo (plan-of-record)"` is
just `"Foo"`. `created` is the earliest git add-date of each node's own plan
directory, so the dates span **2026-06-20 → 2026-07-25** across 13 days instead
of reading 45× `2026-07-07`. If you have older notes quoting a node name or date,
they are stale — re-query.

**G-1 — SUPERSEDED: now CLOSED (2026-07-27, ODD-0024); minting freeze LIFTED.** ~~G-1 is
untouched and still in force.~~ The cutover **preserved every ULID** —
it relocated and re-stamped existing files, and minted nothing. ~~Do not mint new
nodes before the G-1 decision.~~ *(ULID retained — new nodes may now be minted.)*

**What this cost, worth knowing:** the dogfood found three defects that four
green slices had not, all of the same shape — a path that could not be exercised
where it was written, or success and failure looking identical. The sharpest:
`odm use arc X` would print `✓` and `odm orient` would still say "(no current
arc)", because `use` wrote the store root and `orient` read the invocation root —
indistinguishable until odm's own store moved out of the invocation root. Full
account: `arc-release-hardening/c5-closing-report.md`. Lesson recorded on the
arc: **an opt-in feature nobody has opted into is not verified.**

**Next actions:** (1) CDC verifies C-5 (`cdc-verification.md`) — including an
independent `orient`/`check` **in the relocated home**; (2) G-1 ODD, still the
gate on everything that mints; (3) the remaining RH chunks — C-4, C-6, C-8 — and
F-21/F-16/F-17.

---

## 0. Resume update — 2026-07-25 (UAT session; read this, then skim §§1–7 as history)

**Phase: UAT, two passes by design** (operator decision): pass 1 = human
usability (Duncan's `arc-release-hardening/uat-punch-list.md`, F-1…F-14 →
chunks C-1…C-5); pass 2 = LLM acceptance
(`arc-release-hardening/uat-report-llm-pass-batch2.md`, findings L-1…L-9,
written by a CDC session that **built odm from source in its sandbox — 2m54s,
stock toolchain — and drove the self-hosted store**; orient 17 ms, check 8 ms,
59 nodes green). The next CDC session can and should do the same.

**Operator priority (2026-07-25): ship v2 ASAP** — multi-stream PM pain in the
lykn project is the driver. Ship-blocking shortlist and full evidence:
**`arc-release-hardening/workflow-gap-coverage-review.md`** (G-1…G-12; its
Ownership section explains what is odm's vs lykn's).

**Authorities landed today:**
- **`../odm-command-inventory.md`** — THE command-surface authority (current /
  UAT-decided / future / legacy), reconstructed by the operator from the four
  kickoff transcripts after the spec was found to exist nowhere in-repo.
- **`arc-llm-command-surface/arc-plan.md`** (v1.1, unnumbered) — 7 shaped
  slices closing the LLM gaps; reconciled against the inventory.

**⚠ Before minting ANY new node: G-1. — CLOSED 2026-07-27 (ODD-0024): ULID retained, freeze lifted; see §0b. (Historical below.)** The operator intends to switch the ID
scheme (register-style `D-YYMM-XXXX` vs ULID) — recorded ONLY in the gap
review. Write the ODD and get the decision **first**; identity cannot change
after ship. CDC input is in the review (keep ULID as `id`; D-style competes
with `number`).

**Cautionary artifact:** node **#1605** was minted against a stale directory
listing and retired the same day (`odm retire --because`, reason preserved) —
the tombstone `arc06-…/slice05-uat-cli-feedback/` misled a reader exactly as
UAT L-2 predicts. `check` was green throughout. Trust the arc-plan, not `ls`.

**Next actions, in order:** (1) operator commits today's docs — everything
above is `??`-untracked until then; (2) G-1 ODD; (3) triage the shortlist into
release-hardening chunks (G-2 tear-rationale fix = **C-6**; its old routing
"arc02 slice08" is a phantom — that slice does not exist); (4) scope G-5
(generalize the importer; lykn `--dry-run` = UAT pass 3 — the long pole).

**Process rule that does NOT transfer between Cowork projects** (memory stores
are per-project; the lykn store will not follow you here): **read the
collaboration-framework's `AI-CONSTITUTION-SUPPLEMENT.md` and
`AI-ENGINEERING-METHODOLOGY.md` IN FULL at session start** — a 2026-07-25
Opus 5 session ran on the SKILL.md summary alone and logged 4
unwalked-consequence errors; MUST-gated docs were read, softly-worded ones
skipped. Structural floor regardless: query-before-claim, counts not
adjectives, `[ran:]/[read:]/[inferred]` provenance markers on state claims.

---

## 1. The frame (how we work)

- **odm** = a markdown/git-native, dependency-ordered planning + documentation substrate,
  rebuilt at v1.0.0 as a multi-crate Rust workspace. It mechanically actualizes the
  collaboration-framework. Repo: `github.com/oxur/odm` (pkg `oxur-odm`, binary `odm`).
- **Roles.** I am **CDC** (the independent planner/verifier): I draw each slice's open doc-set
  (slice-doc + ledger + cc-prompt), CDC-verify **CC**'s implementation reports (writing
  `cdc-verification.md`), keep arc-plans via bubble-up, run arc-closes (composition check +
  independent subagent gate + project-plan bubble-up), and keep the project-status dashboard
  current. CC implements on a real toolchain; I verify structurally + rule on flags. **Peer
  frame, amend-don't-work-around, flag every deviation.**
- **Ledgers.** LEDGER-DISCIPLINE v2.0 (scale-free: slice/arc/project). Evidence strength
  `asserted < attested < reproduced < reconciled`; a `done` row reaches ≥ `reproduced` at its
  scale. **Stable IDs over renumbering** (append, never disturb rows with real evidence).
  PROJECT-MANAGEMENT.md governs plan-late-plan-deep + bubble-up/close + plan-change discipline
  (dated version-history entries naming which child surfaced each change).
- **Environment.** The CDC sandbox has **no 1.85+ cargo** (edition 2024). So cargo/executable
  rows are **attested-by-CC → reproduced-on-CI**; I reproduce *structural* rows by code +
  on-disk inspection. CC builds/tests on local 1.95.0.
- **⚠ Tooling note:** the `AskUserQuestion` popup has been failing this session
  ("stream closed"). **Ask decisions in prose**, not via the tool, until confirmed fixed.
- **Git:** `release/0.3.x` = pre-rebuild import; `main` reset onto it; **all rebuild work is on
  `release/1.0.x`** (Duncan set this up / freshened it this session). Branches cut from
  `release/1.0.x`.

## 2. Where the project stands

**MVP (A1–A3) + core (A4–A5) are in; A6 is nearly done; a UAT arc runs before A6 finishes.**

| Arc | State |
|-----|-------|
| A1 Substrate & node CRUD | ✅ complete (merged, CI-green) |
| A2 Graph, gates & derived order | ✅ complete |
| A3 Rollup & orient (MVP capstone) | ✅ complete |
| A4 Index & cache | ✅ closed (CI-green on release/1.0.x) |
| A5 Reconciliation (ODD-0019 freshness) | ✅ closed (CI-green on release/1.0.x) |
| **A6 Migrate, self-host & PM-skill** | **⏸ PAUSED after slice04** (slices 01–04 CDC-verified; **odm SELF-HOSTS**). Resumes at slice05 (PM-skill) after the Release Hardening arc. |
| **Release Hardening (UAT)** | **▶ ACTIVE** — named arc, canonical number deferred. Batch-1 punch list triaged; no code yet. |
| A7 telemetry / A8 forecasting | post-MVP horizon, **owned by another CDC** — ignore. |

**The big milestone this session: odm self-hosts (A6 slice04).** `odm self-host <plan>`
imported the `design-v1.0.0` plan-set → **45 work nodes** (1 project + 6 arcs + 38 slices)
under `nodes/`, each schema-stamped `<type>/v1.0`, in a containment tree (arc `part_of`
project, slice `part_of` arc); `odm check` green on **58 nodes** (13 `odd` + 45 work);
`rollup`/`orient` reproduce the real state. The hand-maintained truth (project-plan, the
dashboard) is now **derivable from odm querying itself**. project-plan **P-12**
reproducible-at-arc-close. Numbering scheme used: project=1000, arc N=1000+100·N,
slice=arc+position. `[gates.*]` added to `odm.toml` (ODD-0013 §5.1). No `odm-index` change.

Also this session: **slice03 (schema versioning, ODD-0020)** CDC-verified — per-type
`schema: <type>/vN.N` markers, absent⇒v0.1, per-type field-validity a `check` Error,
forward-compat; 13 nodes stamped `odd/v1.0`.

## 3. The pivot: A6 paused → Release Hardening arc

After slice04, Duncan moved into **hands-on UAT** of the self-hosted tool. The feedback
(CLI/naming/type/output) was too large + too model-level for a slice, so — per Duncan's call
and the mid-arc-pause precedent — it was **extracted to its own arc**:

**`docs/design-v1.0.0/arc-release-hardening/arc-plan.md`** — *Release Hardening: CLI, types,
naming & output (UAT-driven).* Decisions settled with Duncan:
- **Placement:** *named arc, canonical A-number deferred* (don't renumber the other CDC's
  A7/A8; numbering is itself under review here). A6 was briefly given a UAT slice05 (arc06
  v1.8) then reverted (v1.9) — A6 is back to slice05=PM-skill, slice06=retire-prose, marked
  `paused` on the dashboard.
- **`odm path` → `odm chain`** (decided).

**The UAT flow (different from other arcs):** feedback → CDC triages each into **surface**
(cc-prompt) / **model** (ODD-0013/0020 amendment or ADR first) / **question** → chunked
cc-prompts → CC implements → CDC verifies → **re-run `odm self-host`** to validate. Findings
accrete as **F-rows**; chunks as **C-rows**.

### Batch-1 punch list (Duncan's, first-pass, non-authoritative)

Verbatim source: **`arc-release-hardening/uat-punch-list.md`**. Triaged into 14 F-rows +
5 chunks in the arc-plan. Chunk breakdown, dependency-ordered:

| Chunk | Scope | Kind | F-rows |
|-------|-------|------|--------|
| **C-1** Adopt Oxur table styling/theming (colours, warm-orange theme) | **model/arch (ADR); ROUTE OPEN — see §4** | model | F-1 |
| **C-2** Type taxonomy: `odd`→`design`, add `research`; re-stamp the 13 nodes; reclassify research docs | **model (ODD-0013 + ODD-0020 amendment)** | model | F-2, F-3 |
| **C-3** `odm list` overhaul: drop number col; date-first + `--date=updated`; status col after type; branch-and-leaf tree (drop name-prefixing); max-width config+flag + ` ...` elision; de-numbered names | surface | F-4…F-9 |
| **C-4** Command cleanup: `context`→`project` (+`--name`); `path`→`chain`; `new` warns-not-displays on re-run; `rollup` help + md/json + `--out` (defaults md/ROLLUP) | surface | F-10…F-13 |
| **C-5** Fold `self-host` into `migrate` | surface/medium | F-14 |

**Order:** C-1 + C-2 are foundational (the renderer + the type names everything else uses) →
C-3 → C-4/C-5 anytime. **More batches expected (~3–4 total).**

**Model amendments queued** (draft the amendment *before* the cc-prompt): ODD-0013 (type
taxonomy), ODD-0020 (schema markers `design/v1.0`, `research/v1.0` + re-stamp path), + a new
**ADR** for the styling-crate route (§4). **Reflexive loop:** after C-2/C-3, re-run
`odm self-host` so the corpus reflects `design`/`research` + de-numbered names, re-verify green
(arc-plan row RH-7).

## 4. ⭐ THE OPEN QUESTION (start here next session): the oxur-cli / oxur-table route

C-1 (the styling work, Duncan's top-priority-for-v1 item) has a **route decision that is NOT
made** — it needs **mutual investigation, then discussion, then an ADR**. Do **not** just draft
it.

**The record (found this session — Duncan's memory was right, we'd discussed/built this before):**
- `oxur-table` was **originally its own crate** — ODD-0001 (Oxur Letter of Intent) lists it
  "✅ IMPLEMENTED"; there's a **Final** oxur design doc **0015 "oxur-table API (re)Design"**
  (2025-12-31).
- `oxur-cli/src/table/README.md` says it plainly: *"In late 2025 this module was in its own
  crate but as oxur-cli started to take shape, oxur-table was moved to oxur-cli/src/table."* It
  was **used by `oxd`** (odm's direct ancestor) for `list` — the warm-orange theme.
- **odm's own intent already mandates oxur-cli styling:** ODD-0012 + ODD-0013 §11 spec
  `odm-cli` output as "oxur-cli/tabled"; odm's `CLAUDE.md` says depend on `oxur-cli`
  `default-features = false` for `common::output` + `table`, don't enable `binary`. The
  compiler-stack fear is *already avoided* by `default-features = false` (lang/comp/repl/clap
  are behind the `binary` feature; lib-only deps are just tabled/serde/toml/colored/dirs).
- **The code drifted:** odm-cli currently uses raw `tabled` + `writeln!`, no oxur-cli, no theme
  → the plain colourless output Duncan sees. So C-1 **realigns code to intent** — I had earlier
  mislabeled the CLAUDE.md line as "doc-drift to fix"; it's the opposite (the intent is right,
  the code drifted). *(This is corrected in the arc-plan.)*

**The two routes (decide together):**
- **(A) Depend on `oxur-cli` lib-only** (`default-features = false`). Fast; matches CLAUDE.md
  verbatim. Cost: couples odm to oxur-cli's release cadence + drags its whole lib surface.
- **(B) Re-extract a standalone `oxur-table` crate** (reverse the late-2025 fold; matches
  ODD-0001 + the 0015 redesign + Duncan's "split term out of cli" instinct). Cleaner boundary
  (odm depends on one small crate). Cost: **upstream work in the oxur repo** (publish, version).

**Scoping question for the ADR:** table **only**, or table **+ terminal output helpers**
(`common::output`: success/error/info/warning)? Duncan said "table/**terminal**" → likely both.

**CDC lean:** B (it was always its own crate; there's a Final API design to lean on; cleaner
deps) — but it's more work and it's Duncan's repo, so it's his call. **API to lean on:**
`oxur_cli::table::OxurTable::new(data).render()`, generic over `Tabled`, `ColoredString` cells,
TOML theme (ANSI + hex), warm-orange default.

**Next-session action:** (1) jointly investigate A vs B — read oxur design doc 0015, weigh
coupling vs upstream-publish cost; (2) decide; (3) write the ADR; (4) settle the **base-branch**
question below; (5) then draft the C-1 cc-prompt.

## 5. Other open items / carry-ins

- **Base branch for the RH cc-prompts** (decide at C-1): the chunks build on the
  migrate+self-host+schema code, which lives on the **A6 slice04 tip**
  (`arc06-slice04-self-host-cutover`, off `release/1.0.x`, **unmerged**). Branch off that tip,
  or **merge green-A6-so-far to `release/1.0.x` first** and branch from there? **CDC lean:
  merge A6-so-far first** (keeps `release/1.0.x` the clean base, avoids a long dependent chain)
  — but confirm CI is green first.
- **A6 remaining after this arc:** slice05 (PM-skill, from ODD-0001, in
  `billosys/ai-engineering`) → slice06 (retire redundant framework prose; folds the CLAUDE.md
  oxur-cli fix — now the C-1 styling change, not a doc-only fix) → **A6 arc-close** → v1.0.0
  MVP-plus self-hosting.
- **Named-follow (out of scope, noted):** `odm rollup --json` regenerating
  `project-status.html` — the dashboard becomes derivable from odm.
- **Parked (post-MVP backlog):** the two-reads-per-bare-command optimization (odm-reconcile
  perf); the schema type↔`type:` mismatch check (slice03 CDC note); the per-class-vs-per-type
  validity tightening (slice03 CDC note).
- **CI:** A6 slices 01–04 cargo rows are attested-pending-CI; they flip to reproduced on CI
  green.

## 6. Key files

- **Dashboard:** `docs/design-v1.0.0/project-status.html` (keep updated each milestone — A6
  `paused`, Release Hardening `active`).
- **Project plan / ledger:** `docs/design-v1.0.0/project-plan.md` (P-12 = self-host DoD).
- **Release Hardening arc:** `docs/design-v1.0.0/arc-release-hardening/arc-plan.md` +
  `uat-punch-list.md` (Duncan's verbatim batch 1).
- **A6 arc-plan (paused):** `docs/design-v1.0.0/arc06-migrate-self-host/arc-plan.md` (v1.9).
- **Genesis (why odm exists):** §8 of this file (merged 2026-07-25); the verbatim
  original is git-tracked as `docs/dev/0025-odm-discussion-bootstrap-rebuild-planning.md`.
- **oxur-table records:** `oxur` repo — `crates/oxur-cli/src/table/` (+ README),
  `crates/design/docs/06-final/0015-phase-2-oxur-table-api-design.md`, ODD-0001.
- **Memory:** `odm-redesign-decisions`, `odm-release-hardening-arc`, `odm-status-dashboard`
  (in the space's memory dir).

## 7. Do-this-next (the crisp version)

1. Read this doc + `arc-release-hardening/arc-plan.md` + `uat-punch-list.md`.
2. **Investigate the oxur-cli/oxur-table route (§4)** — read oxur-0015, weigh A vs B — then
   **discuss with Duncan and decide.** (Ask in prose; the popup tool is flaky.)
3. Settle the base-branch question (§5); confirm A6 CI is green.
4. Write the **ADR** for the styling route, then the **C-1 cc-prompt**.
5. Expect more UAT batches from Duncan — triage into F-rows/chunks as they land.

---

## 8. Genesis — why odm exists

> Merged 2026-07-25 from `workbench/odm-session-bootstrap.md` (authored 2026-06-19 by
> Claude, CDC/team-support thread, with Duncan, at the close of the pos-loyalty-svc
> session where the failure this tool fixes surfaced in the worst way). The
> load-bearing sections are carried near-verbatim below. **Elided as superseded**
> (disclosed, not dropped — and correcting an earlier claim: `workbench/` is
> gitignored, so there is *no* git history at the old path; the genesis full text
> is git-tracked verbatim as
> `docs/dev/0025-odm-discussion-bootstrap-rebuild-planning.md`): the requirements
> catalogue (§4 — now governed by ODD-0012/ODD-0013), the suggested first steps
> (§7 — the SDLC that was then run, producing ODD-0011…0015), and the carry-over
> artifact list (§8 — the research report is now **ODD-0011**).

### 8.1 Mission (as chartered)

`odm` ("our document manager") pre-existed as a crate inside the Oxur
language project; the charter was to split it into its own repo and grow it into
the planning/tracking substrate for the collaboration-framework. The end-state: a
markdown/git-native, dependency-ordered planning system that is
**self-documenting and self-tracking** — so the *mechanical* rules of the
framework (numbering, ordering, deferral-tracking, status discipline,
drift-watching) stop living as prose rules a human/LLM must remember and become
**tool-encoded checks**. When `odm` is done, framework docs/rules that are purely
mechanical get deleted and replaced by `odm` commands. Success test: *a fresh
session (human or LLM) reaches full situational awareness from `odm orient`
alone.*

### 8.2 The failure this fixes (the founding case)

Over a long pos-loyalty-svc session, work was driven slice-by-slice with
disciplined per-slice rigor — yet the *program-level* vision was lost and
deployed state drifted invisibly. The worked example: **production had a Cloud
SQL DB provisioned and migrated, but the service was never wired to it**
(`DB_HOST` missing from the prod overlay). The authenticated API was fail-closed
at 503 by design, and nobody saw it, because the fact "prod service wired to its
DB" existed only as the intersection of five scattered documents. No single
artifact tracked it; no check could fire.

**Root causes (the diagnosis the tool answers):**

1. **Identity conflated with order** — "Phase 9 / 8.5 / 10" use the number as
   both name and claimed sequence; deferrals make the numbers lie.
2. **No single source of truth for state** — state reconstructed by archaeology
   (git log + grep + scattered ledgers); drift invisible until tripped on.
3. **Plan is desired-state with no reconciliation loop against actual-state** —
   verification point-in-time, against the repo, never against live reality.
4. **Binary status hides integration-level truth** — `done/open/deferred` too
   coarse; "done at its layer" masked "not working when integrated."
5. **Dependencies are prose, not data** — nothing can mechanically warn "you are
   working out of order; dependency X is still open."
6. **The information architecture makes vision-loss the path of least
   resistance** — rebuilding the global picture each turn is expensive, so the
   cheap, recent, local artifact wins attention; context resets make an LLM pay
   the reconstruction tax repeatedly. *Fix: make global state cheap to load.*
7. **Vocabulary drift** — "phase" predates "project/arc/slice"; renumbering
   created stale links.

### 8.3 The convergent architecture (research finding)

Five independent literatures — WBS/CPM project management, Design Structure
Matrix engineering, build systems, infrastructure reconciliation, and
docs-as-code — all converge on one architecture:

> **Stable-identity nodes + an explicit dependency DAG + order *derived* by
> topological sort + per-edge staleness/reconciliation checks + a single
> *complete* graph as the source of truth.**

`odm` is therefore two things fused: **a build system for the plan** (ordering +
readiness + staleness) and **a reconciler for the plan's state** (desired vs.
actual drift). The formal backbone: *Build Systems à la Carte* proves
**correctness requires a complete dependency set — an incomplete graph silently
permits running a step before its inputs are satisfied.** That is the DB
failure, stated as a theorem. Paired with the IaC/Kubernetes lesson: **you can
only detect drift on what the source of truth claims to manage.**

*Evidence calibration:* trust the formal (WBS-is-scope-not-sequence,
identity-≠-order, topo-sort + cycle detection, the à-la-Carte theorem,
closed-loop control, Little's Law); treat as advisory lore (CCPM buffers, WSJF
scoring, most agile/SAFe ceremony). Dependency order gives a *correct* order,
not the *fastest* — priority is a separate, softer layer. Full citations:
**ODD-0011** (`docs/design/06-final/0011-research-a-markdowngit-native-dependency-ordered-planning-system.md`).

### 8.4 Principles & guardrails (standing)

- **Markdown/git-native; no ticketing system.** Files are the source; `odm`
  commands are "the build."
- **Identity ≠ order.** Stable IDs; order derived from the graph.
- **Complete graph or no detection.** Every real dependency must be an edge;
  bias toward over-declaring deps.
- **Track integration-level facts**, not just per-layer completion.
- **Supersede, don't delete.** History is preserved (git + supersession links).
- **Trust formal over lore**; keep advisory layers optional.
- **Self-documenting, self-tracking** is the success test (`odm orient` alone).

### 8.5 How this actualizes the collaboration-framework

- "number by dependency / don't work out of order" → the DAG + staleness guard.
- "disclosed deferral with named re-entry" → first-class deferred status with a
  checkable re-entry condition.
- "spec-keeping / no silent drops" → diff scope-as-delivered vs scope-as-declared
  in the rollup.
- "verify, don't assert" → the reconciler (desired-vs-actual probes).
- "cheap global state so vision isn't lost" → `orient` + the generated rollup.

Once tool-encoded, the corresponding prose rules retire (replaced by "run
`odm check`"). The framework keeps its *character/posture* layer; `odm` absorbs
the mechanical layer. (This is A6 slice06's charter.)
