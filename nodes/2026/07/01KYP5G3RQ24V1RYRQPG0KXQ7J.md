---
id: 01KYP5G3RQ24V1RYRQPG0KXQ7J
number: 595198000
type: artifact
schema: artifact/v1.1
name: CDC session handoff — arc-store-home reopening (2026-07-27)
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/SESSION-HANDOFF-2026-07-27.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SHYYMCPX4JN86ZJ6F
---
# CDC session handoff — arc-store-home reopening (2026-07-27)

> Pass this to the next CDC session alongside `amendment-worktree-discovery-ODD-0022.md`.
> The amendment is the *design*; this is the *state, the gotcha, and the pending edits*.
> First moves in the new session: load the **collaboration-framework** skill (standing
> instruction) and **read `PROJECT-MANAGEMENT.md` in full before touching `arc-plan.md`**
> (the reopening is a planning action). Then read `CDC-SESSION-BOOTSTRAP.md` §0.

## Resume pointer — the next concrete action

The ODD-0022 amendment is **drafted and operator-approved**. Next:

1. **Place the amendment** in the repo — it currently lives only in chat/handoff, not on disk.
   Save it to `docs/design-v1.0.0/arc-store-home/amendment-worktree-discovery-ODD-0022.md` on the
   **`release/1.0.x`** worktree (`.worktrees/1.0.x/`).
2. **Reopen `arc-store-home/arc-plan.md`** by the plan-change discipline (PROJECT-MANAGEMENT.md Part V):
   version bump; a Version-History entry naming *what* expanded (worktree-general store discovery),
   *which* work surfaced it (the multi-project / per-line-worktree setup, 2026-07-27), and *why*; mark
   the existing `closing-report.md` **reopened-for-expansion, not erased**; add new **arc-ledger rows**
   for slices 05–07; and **bubble up to `project-plan.md` §2a** (SH: closed → reopened).
3. **Write the slice05 open set** (`slice-doc.md` / `ledger.md` / `cc-prompt.md`) and hand to CC.
4. Ship on **`release/1.0.x`** throughout (it's pushed to origin over SSH).

## The operational gotcha (read this before running `odm`)

**The corpus is currently unreachable by the binary.** Verified 2026-07-27: top = `main` @ `bf86593`,
whose `odm.toml` is the 0.3.5 format with **no `[store]` section** (falls back to repo-root → no nodes);
the linked worktrees still hit the nested-`.worktrees/odm` resolution bug the amendment fixes. The corpus
is safe on disk at `.worktrees/odm` (branch `odm`). Until **slice06** (common-dir discovery) lands, or a
stopgap `[store]` locator is added to `main`, `odm orient`/`list`/`node new` will NOT resolve the corpus.
**Consequence:** Bitubardos plan-minting and any corpus planning command are blocked until then. Don't be
fooled by an empty `orient` — the data is fine; discovery is the gap.

## Verified repo state (2026-07-27)

- **top = `main` @ `bf86593`** — now carries the `.worktrees/` gitignore (Duncan's structural commit).
  Frozen on 0.3.5 code otherwise; updates to release code at the 1.0.0 cutover.
- **Worktrees** (all siblings under `.worktrees/`): `1.0.x`→`release/1.0.x` @ `b878d7a` (pushed);
  `1.1.x`→`release/1.1.x` @ `6f0f9b0`; `bitubardos`→`project/bitubardos` @ `6f0f9b0` (branched off 1.1.x,
  merges back into it); `odm`→ store branch `odm` @ `e2ab628`.
  *(Over the device bridge these show `prunable` — a mount artifact, the gitlinks' absolute paths don't
  resolve inside the VM. On Duncan's machine they're fine; don't "repair" them from a bridge view.)*
- **`release/1.0.x` and `release/1.1.x`** were content-identical at the fork; propagation between them is
  what the future propagation-queue arc tracks.
- **Store branch is still `odm`** — the rename to `odm/store` is slice05, not yet done.

## Slice plan (from amendment §5; sizing is the arc-plan's call)

- **slice05** — `odm/store` rename + split the shared `worktree_name`/`branch_name` default.
- **slice06** — common-dir discovery + `top_level_branch` + the main-anchor commit. *(This is the slice
  that unblocks corpus reachability — consider sequencing its main-anchor bit early.)*
- **slice07** — project scoping: `git worktree list` mapping, `--project`/`--repo` surface, the hub
  read-vs-mutate default, and the derived registry + reconcile probe.

## Pending repo edits not yet applied

1. **Place the amendment** (above).
2. **CLAUDE.md** — remove the stale *"Do not mint new nodes before the G-1 (ID scheme) decision"* note.
   **G-1 is dropped** (operator decision: keep the current ULID scheme — dogfooding showed it's better);
   the mint-gate is lifted. Also update the "corpus lives on the `odm` branch" note when the branch
   becomes `odm/store` (slice05).
3. **Main-anchor `[store]`** — either as slice06's proper step, or a manual stopgap now to unblock
   planning/minting before slice06.

## Corpus modeling (do once the corpus is reachable)

- Under "each line is a **peer project** of equal standing" (git ancestry ≠ project standing): the store's
  **existing single project node becomes the `release/1.0.x` project** (give it a `branch` attribute) —
  nothing to reslot, no parent.
- **Mint `project/bitubardos` as a peer.** Duncan's node-new/link script (project + 6 arcs in a linear
  `depends_on` chain) is **reviewed and approved** — it runs unchanged once discovery resolves the corpus.
  Flags verified against the CLI: `--parent <ref>`, `--yes`, and name-prefix `resolve` all exist.

## Separate next arc (not part of SH)

**Propagation queue** — its own `release/1.0.x` arc, after SH's expansion lands. It formalizes the
rootstock "port lane" natively: **typed edges between project nodes** + a **node per undispositioned
cross-branch commit** + a **lane-drained `odm-reconcile` probe** (`git rev-list base..tip` minus
dispositioned == 0) + a status render. Name chosen: **propagation queue**. It consumes SH's branch-map
substrate (project nodes + `branch` attribute + inter-project edges).

## Where to watch me (named pulls, per the posture)

- **Corpus pull → "versions as sub-scopes."** I framed 1.0.x/1.1.x as maintenance branches of one product;
  Duncan corrected to **peer projects of equal standing**. Watch for the conventional release-management
  shape overriding the stated model.
- **Current-state drift in briefs (recurring — 4 RH instances).** cc-prompts have repeatedly asserted a
  false claim about the tool's *current* state. **Mechanical control: run the binary / read the code
  against every current-state claim before a brief ships**, and note which claims were verified. The
  amendment above was written this way (resolution behavior verified against real git + the source).

## Broader project state

- **RH arc closed** (PASS-WITH-NOTES); F-22 (retired-node leak in `orient`/`next` READY) routed to the
  LLM-command-surface arc slice 02.
- **Dual UAT sign-off still gates close-out** (SKILL writing): Duncan's human column (theme/table/naming
  in his TTY) + my LLM column (helpful/informative/correct/optimised). Worksheet:
  `docs/design-v1.0.0/command-surface-uat-checklist.md`. My LLM half is a first pass; the ⚠ rows are the
  LLM-command-surface arc's slices.
