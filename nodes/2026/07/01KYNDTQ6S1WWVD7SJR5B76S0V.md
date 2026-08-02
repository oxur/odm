---
id: 01KYNDTQ6S1WWVD7SJR5B76S0V
number: 10842600
type: arc
schema: arc/v1.1
name: Arc — LLM command surface (plan-of-record)
created: 2026-07-28
updated: 2026-08-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-llm-command-surface/arc-plan.md
  class: arc-plan
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-02
edges:
  part_of: 01KWXMBBTJCJ30F3TE4BJJV2CB
status:
  in-progress:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Arc — LLM command surface

<!-- Title de-labelled per ODD-0013 §2.1 ("names embed no metadata") — see slice 10 / B2-3. -->

**Opened:** 2026-07-25 · **Status:** shaped → reconciled 2026-08-02 →
**v1.4 2026-08-02: B2-3 folded in as slice 10** (Batch-2 UAT items +
Migration-Fidelity carry-ins folded in; re-grounded 60 → 400 nodes).
**Unnumbered**, per the `arc-release-hardening` precedent (the numbering scheme is
itself under review in that arc).

> **Grounding note (2026-08-02).** This plan and
> [`odm-command-inventory.md`](./odm-command-inventory.md) were both first
> grounded at **60 nodes** (`release/1.0.x` @ `8de6d0d`). The live self-hosted
> store is now **400 nodes** (CDC ground-truth by direct file read,
> `.worktrees/odm/nodes/`, 2026-08-02) — so the read-command outputs this arc
> reshapes (`next`, `orient`, `rollup`) are **materially larger** than the
> traces the slices below quote. A full re-ground of the
> [`command-surface-uat-checklist.md`](../command-surface-uat-checklist.md) §1
> against the 400-node store is its own punch item, tracked there, not here.

## Provenance — corrected 2026-07-25

**Superseded within hours of being written.** This section originally reported
that the kickoff command specification *"was not recorded"*, based on a sweep of
the odm repo and `billosys/ai-engineering`. The operator then reconstructed it
from the four Cowork session transcripts and landed
**[`odm-command-inventory.md`](./odm-command-inventory.md)** — current surface,
UAT-decided proposals, future-arc proposals, **and the full legacy surface**.

**The corrected finding is narrower and still worth keeping:** the specification
was never *in the repository*. It existed only in conversation transcripts, and
recovering it required the operator to go back to those transcripts by hand. No
artifact, no row, nothing watching. That is the same class as `D-2607-8HTN`
(*"a routing row must name a home that can be opened"*) — the work was real, the
home was not. It is now fixed, and the fix is the inventory.

**`odm-command-inventory.md` is the authority for the command surface.** This
arc plan is scoped to what the inventory does *not* already cover, plus the
places where driving the tool disagreed with it.

### Reconciliation against the inventory

| Proposed here | Status against the inventory |
|---|---|
| Status read-back (slice 01) | **DELIVERED by RH C-8 (2026-07-27).** `odm node show` (text + `--json`) now exposes the per-gate vector (`reached`/`evidence` + the normalized `status`) — the L-1 blocking gap, closed ahead of schedule as a side effect of the F-19 status work. Slice 01 shrinks to the *remainder* (see the slice row). |
| Ordered/typed `next` (02) | Absent — **and upgraded by Batch-2 B2-1** (grouped tree + ready-frontier *correctness*, not just typing/ordering). See slice 02. |
| `why` (02) | **Partly redundant — corrected.** `blocked <REF>` already claims to *"explain why a node is blocked or low-confidence."* See the amended slice 02 scope. |
| `search` (03) | **Convergent, and the inventory sharpens it:** legacy *had* `search`; it was dropped with the state-directory model, not deliberately as a capability. Under ID-mirrored storage it is needed *more* than it was, not less. |
| Flat rollup (03) | Compatible with C-4's `rollup --format={md,json}` + `--out` — flat is a third format, not a competing design. |
| `history` / `diff` (04) | Absent. Legacy `debug diff` is filesystem-vs-state, a different thing. |
| `export`, `viz` (05, 06) | Already in the inventory §3. Kept here only for sequencing. |

### Batch-2 & carry-in reconciliation (added 2026-08-02)

The 2026-08-01 Batch-2 UAT additions (`command-surface-uat-checklist.md` §6) and
the Migration-Fidelity close carry-ins (`arc-migration-fidelity/closing-report.md`)
are folded in here so nothing waits in a checklist the arc doesn't read:

| Item | Source | Disposition in this arc |
|---|---|---|
| **B2-1** — `next` dumps a huge flat ungrouped list; (a) *correctness*: should return the **ready frontier**, not every unblocked node; (b) *presentation*: typed + **grouped + tree** (matching `node list`). | UAT checklist §6 | **Upgrades slice 02** (supersedes the terse L-4 "typed + ordered"). |
| **B2-2** — retire numeric handles from **display AND input**, surface-wide; `node show` takes a ULID; accept a ULID **prefix**. `number` becomes internal-only (coverage match-by-number fallback). | UAT checklist §6; operator decision 2026-08-01 | **New slice 08** — split out of slice 02 on sizing grounds (surface-wide sweep across ~13 commands + prefix resolution is more than rides on `next`). **Operator-confirmed 2026-08-02** (own slice). |
| **B2-3** — `(plan-of-record)`/role-restating parentheticals leaked into **24 live store node names** (CLI-facing) + **57 plan-tree H1 titles**. | UAT checklist §6; **ODD-0013 §2.1** | **New slice 10 — in scope** (corrected 2026-08-02). *Not* doc hygiene: it is a **regression against an accepted decision.** ODD-0013 §2.1 v2.3 ("names embed no metadata", naming `(plan-of-record)` explicitly) specified **mint-time normalization** in `migrate`/`self-host`; that regressed, so 24 store names carry the label and render in `next`/`list`/`orient`/`show`. CDC stops adding it now (the going-forward half). |
| **F-21** — `--json` omits `created`/`updated` (`node list --json`). | UAT checklist §5 | **Folded into slice 01** (the `--json`-contract remainder); also asserted as ledger row A-10. |
| **F-22** — retired/superseded nodes leak into `next` and `orient` READY. | UAT checklist §5 | **Folded into slice 02** (the readiness-set fix); ledger row A-9. Ground-truthed: retired nodes exist in the live store. |
| **CDC-ARC-1 / L-8** — 14 RH-era design nodes dropped `source.version`; needs a design-corpus migrate + a **standing frontmatter-fidelity check**. | MF-4 → routed into this arc (§0e resume) | **New slice 09** — the check lands in `validate`/`check` (command surface); the corpus repair is a one-time migrate run. Ledger row A-11. |

**Recovered from the legacy surface — worth restoring, and not currently
proposed anywhere:**

- **`info [states|fields|config|stats|dirs]`** — self-documentation. For an LLM
  this is disproportionately valuable: `odm info fields` answers *"what does
  frontmatter look like for this node type"* without reading source or docs, and
  it is the kind of question a fresh context asks first, every time.
- **`debug orphans`** — surfaces unreachable entries. That is a near-relative of
  UAT L-2 (work with no node, nodes with no decomposition assertion) and should
  be considered when scoping that check rather than built twice.

**Already routed elsewhere, do not duplicate:** the `tear --because` persistence
bug (rationale validated by `Tear::new`, never written to the `tears` vector) is
recorded in the inventory's notes and belongs to arc02 slice08. It is a genuine
correctness defect and should be release-blocking.

## Capability

Make the CLI sufficient for an LLM to regain full situational awareness, decide
what to do next, and verify what it did — **without reading the planning prose.**
That is the project's stated DoD (`project-plan.md`: *"a fresh session reaches
full situational awareness from `odm orient` alone"*), and the pass-2 UAT found
it is not yet met. The Batch-2 additions sharpen this: at 400 nodes an
LLM-usable surface is not just *complete* but **legible at scale** — the ready
frontier, not every unblocked node; ULIDs as the one identity; grouped, typed,
tree-rendered output the size of the real store.

**Explicitly not in scope:** the surface polish and renames already chunked as
C-1…C-5 in `arc-release-hardening` (`context`→`project`, `path`→`chain`, list
overhaul, table styling, folding `self-host` into `migrate`). This arc assumes
those land first and does not restate them.

## Slice breakdown (shaped, plan-late)

| Slice | Scope | Source |
|---|---|---|
| **01 · read-back the status vector — CORE DONE (RH C-8)** | ✅ **Landed in C-8:** `show`/`--json` expose the per-gate vector (`reached`/`evidence`) + the normalized `status`; A-1 is satisfied. **Remainder:** add `by`/`evidence_dates` to the gates array; enumerate **unreached** gates in `--json` (the text `show` already lists them, so a consumer can see the *next* gate — a machine reading `--json` today sees only reached gates); the computed **satisfaction verdict**; the **weakest link** for soft-satisfied deps (overlaps slice 02); **and `created`/`updated` on `--json` payloads (F-21).** Optionally `odm status <ref>`. | UAT L-1; **F-21** |
| **02 · explain the *ready* half; fix + reshape `next`** | `blocked <ref>` already explains the blocked half. **Nothing explains readiness** — extend `blocked` (or add the symmetric command) to report the supporting chain with evidence per hop and the **weakest link** named; on this store `blocked 1604` returns `nothing holding` / `[]`, which cannot distinguish *ready-and-well-supported* from *ready-on-an-`asserted`-only dep*. **`next` gets two fixes (B2-1):** (a) *correctness* — return the **ready frontier** (dependency-satisfied and not withdrawn), **not every unblocked node**; the **F-22** retired/superseded leak is one symptom the readiness set is too loose. (b) *presentation* — **typed + grouped + tree**, matching `node list` (the reference surface), with `unblocks: N`, `depth`, ordering by downstream fan-out, all in `--json` (name included). | UAT L-4↑ (B2-1), L-5; **F-22** |
| **03 · `search` and flat rollup** | `odm search <text>` over names + bodies (`--type`, `--tag`, `--json`). Under ID-mirrored storage, topic-grep and `ls` stop working; search is their replacement, not a convenience. `odm rollup --format=flat` emits a greppable `id → number → type → name` map for consumers with **no odm binary**. | UAT L-7; the one `odm search` trace |
| **04 · `history` and `diff`** | `odm history <ref>` — gate transitions over time from `evidence_dates` + git. `odm diff <since>` — **what changed in the plan** since a commit/date: nodes added, gates advanced, edges rewired, evidence upgraded or downgraded. *This is the resume command.* Feeds A7 telemetry directly. | UAT (new); A7 adjacency |
| **07 · `info`** | Restore legacy `info [states\|fields\|config\|stats\|dirs]`. Answers *"what are the gates for a slice"*, *"what fields are valid on this type"*, *"what is configured"* without reading source. Small, and the first thing a fresh context needs — **rides with slice 01.** | legacy surface (inventory §4) |
| **08 · surface-wide ULID cutover (NEW — B2-2)** | Retire the numeric `number` from **display everywhere** a `#NN` appears today (`next`, `orient` READY/BLOCKED, `rollup`, `list`, `show`, …) and from **input** (`<ref>` args resolve by ULID; `node show` takes a ULID). Accept a **ULID prefix** (git-short-SHA style) so `node show 01KYSX` resolves, erroring on an ambiguous prefix. `number` survives **only** as the internal migration-time coverage match-by-number fallback — invisible to the user; its eventual removal is a separate model question. **Foundational for display:** should land **before/with slice 02** so the reshaped `next` renders ULIDs from the start. | UAT checklist §6 B2-2; operator 2026-08-01 |
| **09 · frontmatter-fidelity check + design-corpus repair (NEW — CDC-ARC-1/L-8)** | Add a **standing frontmatter-fidelity check** to `validate`/`check` that flags a node whose stored `source` frontmatter has silently dropped fields (the RH-era `source.version` drop). Then the **one-time design-corpus migrate** repairing the 14 affected design nodes. The check is the durable half (catches the next drift); the migrate is the repair. | MF-4 carry (CDC-ARC-1/L-8) |
| **10 · name/title de-redundancy — restore ODD-0013 §2.1 (NEW — B2-3)** | Names and titles must embed **no document-role metadata** (ODD-0013 §2.1 v2.3: no `(plan-of-record)`, `(ledger)`, `(build plan)`, …). Three parts: **(a) repair** the 24 store node names + 57 plan-tree H1 titles carrying the label; **(b) fix the mint-time normalizer** in `migrate`/`self-host` — the ODD *specified* it normalizes derived names; it regressed, which is *why* the labels came back after RH C-3/C-5 cleaned them once; **(c) a standing `check` rule** flagging any name/title that embeds role metadata, so it cannot regress a third time. Pairs with slice 08 (both surface-wide legibility sweeps over names/IDs). This is a **regression against an accepted decision**, not polish. | UAT B2-3; **ODD-0013 §2.1** |
| **05 · `export`** | ODD-0017's projection-out: honestly-lossy, generated-do-not-edit, states any status-vector collapse and the rule used. Target set is ODD-0017 Q-1, still open. **Genuinely post-DoD** — may belong to a later release. | ODD-0017 |
| **06 · `viz`** | Graph/decomposition rendering (`ascii-dag` first — terminal-native, no browser, LLM-readable). A renderer over existing state, never a source of truth. **Genuinely post-DoD.** | `project-plan.md` §4; 0005 §5 |

**Sequencing (updated 2026-08-02):** **01 (+07) → 08 → 02 → 03** is the
DoD-critical path and should not be split across releases. **08 (ULID cutover)
moves ahead of 02** so every display slice emits ULIDs from the start rather than
numbers-then-rewritten. **10 (name/title de-redundancy) pairs with 08** — the same
surface-wide sweep over names/IDs; do them together. **07 (`info`) rides with 01.**
**04** is high value and pairs naturally with A7. **09** (fidelity check + repair)
is off the read-command critical path — schedule after the DoD path or in parallel;
it is data-integrity, not situational-awareness. **05 and 06** are genuinely
post-DoD and may belong to a later release entirely.

**Sizing note:** 01, 02, and 08 are each small-to-medium (01/02's data is already
computed in memory and simply not printed; 08 is broad but mechanical, its only
real logic the prefix-disambiguation). 03, 04, 09, and 10 are medium (10's repair
is mechanical; its normalizer fix + `check` rule are the real work). 05 and 06 are
arcs' worth of work in their own right if taken seriously, and are listed here
only so they are not lost again.

## Arc ledger

| Row | Criterion | Evidence | Sev | Status |
|---|---|---|---|---|
| **A-1** | A fresh context can answer *"what is the gate vector and evidence level of node X"* in **one** CLI call | **`odm node show 1600` / `--json` (RH C-8)** — per-gate `reached`/`evidence` + `status` | serious | **attested** (delivered by C-8; durable on push + CI) |
| **A-2** | A fresh context can answer *"why is X blocked"* in one call, naming the weakest link | transcript | serious | open |
| **A-3** | `next` returns the **ready frontier** (not every unblocked node) and is **typed, grouped, tree-rendered, and ordered**, with the rationale for its order — in text and `--json` | transcript + `--json` shape test | serious | open (upgraded by B2-1) |
| **A-4** | A consumer with **no odm binary** can resolve any ULID to a human-readable node | flat rollup artifact committed | correctness | open |
| **A-5** | A returning context can enumerate what changed in the plan since a named commit | transcript | serious | open |
| **A-6** | **The DoD is met and demonstrated**: a fresh LLM context reaches full situational awareness from `odm orient` (+ the commands above) alone, with no planning prose read. Verified by an LLM instance that has not seen this repository. | pass-3 UAT transcript | **serious** | open |
| **A-7** | No command added here introduces a per-actor metric or anything that could become a target | audit, mirroring A7's A-11 | correctness | open |
| **A-8** | Numeric handles are gone from **display** surface-wide and `<ref>` args resolve by **ULID or unambiguous prefix**; `number` is internal-only | grep the render paths + a resolution test (ULID, prefix, ambiguous-prefix error) | serious | open (B2-2) |
| **A-9** | Retired/superseded nodes do **not** appear in `next` or `orient` READY | fixture with a retired node; `next`/`orient` exclude it | correctness | open (F-22) |
| **A-10** | `--json` payloads carry `created`/`updated` wherever the human table shows a date | `node list --json` shape test | correctness | open (F-21) |
| **A-11** | A standing frontmatter-fidelity check flags dropped `source` fields; the design corpus is repaired (14 RH-era nodes re-migrated, `check` clean) | check output on a drift fixture + post-repair `check` green | serious | open (CDC-ARC-1/L-8) |
| **A-12** | No node name or title embeds document-role metadata (ODD-0013 §2.1); the mint normalizer enforces it and a `check` rule guards against regression | grep store names + plan-tree H1 titles for `(plan-of-record)`/`(ledger)`/… → 0; a re-migrate re-introduces 0; the `check` rule fires on a drift fixture | serious (regression vs accepted decision) | open (B2-3) |

**A-6 is the arc's reason to exist.** It is also the only row that cannot be
closed by the party that implemented it — it requires a genuinely fresh context,
which is the same independence the ledger requires everywhere else.

> **Ledger note.** These are the arc ledger's **opening capability rows** (the
> class-(b) "slices compose into the capability" criteria, stated up front per
> `LEDGER-DISCIPLINE.md` §B). Class-(a) per-slice-closed and class-(c)
> bubble-up-dispositioned rows accrue as slices close; A-6 is reproduced at arc
> scale in the `closing-report.md`.

## Open questions

- **`status` as a field on `show`, or its own command?** UAT recommends the field
  (one call = full node picture); a dedicated command may still be warranted for
  scripting.
- **Does `diff` belong here or in A7?** It reads gate transitions from the same
  derived event log A7 builds. **Recommend: build the consumer here, and have A7
  adopt it rather than re-derive.** Decide at slice 04 scoping.
- **Does `export` (05) belong to v1.0.0 at all?** ODD-0017 calls it the adoption
  engine; `project-plan.md` §5 scopes the v1.0.0 ledger to A1–A6 only. Likely a
  1.1 arc — flagged rather than assumed.
- ~~**Is B2-2's ULID cutover one slice (08) or a rider on slice 02?**~~ **Resolved
  2026-08-02:** its own slice 08 (operator-confirmed), sequenced ahead of 02.
- **Does the frontmatter-fidelity check (09) overlap `reconcile`/`validate`
  enough to fold in rather than add?** Decide at slice 09 scoping — the s11 MF
  carry (a legitimate source change is update-to-re-snapshot, not drift-to-reject)
  is the adjacent precedent.

## Version History

### v1.4 — 2026-08-02 (B2-3 folded in as slice 10, grounded in ODD-0013 §2.1)

Reversed v1.3's out-of-scope routing of B2-3 after the operator pushed back and CDC
ground-truthed. The redundancy is **not** doc-hygiene:

- **It is CLI-facing.** 24 live store node `name` fields carry `(plan-of-record)` /
  `(ledger)` / `(slice-doc / …)` — and `name` is what `next`/`list`/`orient`/`show`
  render. At 400 nodes that is a legibility defect on the command surface, which is
  this arc's territory.
- **It is a regression against an accepted decision.** ODD-0013 §2.1 v2.3 ("names
  embed no metadata") names `(plan-of-record)` explicitly and specifies that the
  `migrate`/`self-host` importer **normalizes** derived names at mint time. Git
  shows the decision landed at `c95da8c` (RH C-5 phase 0); the labels returned
  because the MF `migrate --all` re-derived names from still-labelled doc H1 titles
  and the normalizer did not strip them. A finding that survived RH C-3/C-5 and
  returned is a **systemic/trending** signal — the rule lived as prose with no
  mechanical guard.

New **slice 10** restores it (repair 24 names + 57 titles, fix the normalizer, add
a standing `check` rule) — ledger row **A-12**. The going-forward half is in effect:
CDC stops adding the parenthetical, and this doc's own H1 was de-labelled as the
first instance. B2-2 confirmed as its own slice 08. Surfaced by: operator
(2026-08-02); ground-truthed by CDC (store read: 24 names; git `c95da8c`;
ODD-0013 §2.1 + `odm-cli/src/lib.rs`). Slice 10 sequenced with slice 08.

### v1.3 — 2026-08-02 (reconciled for start: Batch-2 + MF carry-ins folded, re-grounded 60→400)

The arc was cleared to start after Migration Fidelity closed (the release-blocker).
This rev folds every open command-surface finding into the plan so none waits in a
checklist the arc doesn't read, and re-grounds the plan on the live store.

- **Re-grounded 60 → 400 nodes.** Added the grounding note: the plan and inventory
  were written at 60 nodes (`@8de6d0d`); CDC ground-truthed **400** in
  `.worktrees/odm/nodes/` on 2026-08-02. Read-command outputs are materially
  larger; the UAT §1 re-ground is tracked as its own punch item.
- **B2-1 → slice 02 (upgrade).** `next` gains a *correctness* fix (return the ready
  frontier, not every unblocked node) on top of the presentation fix (typed +
  grouped + tree). Supersedes the terse L-4 "typed + ordered." Surfaced by: UAT
  checklist §6 (Duncan, 2026-08-01).
- **B2-2 → new slice 08.** Surface-wide ULID display+input cutover + prefix
  resolution; `number` becomes internal-only. Split out of slice 02 on sizing
  grounds and sequenced ahead of it (display slices should emit ULIDs from the
  start). *Flagged for operator confirmation* (§Open questions). Surfaced by: UAT
  checklist §6 + operator decision, 2026-08-01. Ground-truth: every edge target in
  the live store is a ULID (`part_of: 01K…` confirmed), so the cutover has no
  relational blocker.
- **B2-3 → out of scope (disclosed).** Doc-title parenthetical redundancy is doc
  hygiene, not command surface; routed to the pre-close-out doc-hygiene sweep, and
  CDC stops adding the parenthetical now. Recorded so it is disclosed, not dropped.
- **F-21 → slice 01 + row A-10.** `--json` `created`/`updated` folded into the
  `--json`-contract remainder.
- **F-22 → slice 02 + row A-9.** Retired/superseded leak into `next`/`orient`
  READY folded into the readiness-set fix. Ground-truth: retired nodes exist in
  the live store.
- **CDC-ARC-1/L-8 → new slice 09 + row A-11.** The frontmatter-fidelity check
  (durable, in `validate`/`check`) + the one-time design-corpus repair migrate for
  the 14 RH-era `source.version` drops. Carried from the MF close.
- **Ledger:** added A-8…A-11 (append, not renumber); A-3 reworded for the B2-1
  upgrade; added the opening/accruing ledger note. Slice identities 01–07 preserved;
  new work took new numbers 08/09 (append, don't disturb).

Surfaced by: the MF close (§0e resume) clearing this arc to start, plus the
2026-08-01 Batch-2 UAT. **Sequencing:** still runs after RH; internal order now
01(+07) → 08 → 02 → 03.

### v1.2 — 2026-07-27 (slice 01 core delivered by RH C-8)
**The blocking gap closed early, as a side effect.** RH C-8 (normalized display status, F-19) had to
expose the gate vector in `odm node show` / `--json` — because `list`'s STATUS column was the *only*
place the ladder surfaced, and normalizing it alone would have hidden the ladder from the CLI entirely
(a flaw CDC's own C-8 brief carried; CC checked the running tool, found it, and made the acceptance
text true). That exposure **is L-1** — this arc's slice 01, "the blocking gap." So **A-1 is attested**
and slice 01 shrinks to its remainder: `by`/`evidence_dates` fields, enumerating **unreached** gates in
`--json`, the satisfaction verdict, and the soft-satisfaction weakest link (the last two overlap slice
02's readiness work). Recorded here rather than left to be rediscovered when the arc is picked up.
Surfaced by: the RH C-8 CDC verification. **Sequencing unchanged** — this arc still runs after RH.

### v1.1 — 2026-07-25 (reconciled against `odm-command-inventory.md`)

The operator reconstructed the kickoff command spec from session transcripts the
same day this arc was shaped. Provenance section rewritten: the finding is not
*"the spec was never recorded"* but *"the spec never had a home in the
repository"* — narrower, still real, and now fixed. Slice 02 **corrected**:
`blocked` already covers the blocked half, so the gap is explaining *readiness*
and its evidence quality, which is the question that actually gets asked. Slice
07 (`info`) added from the legacy surface. `debug orphans` flagged as prior art
for the L-2 check. `search` re-framed: dropped with the legacy model rather than
deliberately, and needed *more* under ID-mirrored storage. Tear-rationale
persistence noted as already-routed (arc02 slice08) — not duplicated here.

### v1.0 — 2026-07-25 (arc shaped)

Opened after the pass-2 (LLM) UAT. Records the **loss of the original command
specification** as a finding in its own right, reconstructs a candidate surface
from the four surviving traces plus observed LLM need, and separates the
DoD-critical path (slices 01–03) from the genuinely post-DoD items (05–06) so
the second does not delay the first. Deliberately does not restate
`arc-release-hardening`'s C-1…C-5.
