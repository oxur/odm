---
id: 01KYNDTQ6S1WWVD7SJR5B76S0V
number: 10842600
type: arc
schema: arc/v1.1
name: Arc — LLM command surface (plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-llm-command-surface/arc-plan.md
  class: arc-plan
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
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
# Arc — LLM command surface (plan-of-record)

**Opened:** 2026-07-25 · **Status:** shaped, not started · **Unnumbered**, per
the `arc-release-hardening` precedent (the numbering scheme is itself under
review in that arc).

## Provenance — corrected 2026-07-25

**Superseded within hours of being written.** This section originally reported
that the kickoff command specification *"was not recorded"*, based on a sweep of
the odm repo and `billosys/ai-engineering`. The operator then reconstructed it
from the four Cowork session transcripts and landed
**[`odm-command-inventory.md`](../odm-command-inventory.md)** — current surface,
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
| Ordered/typed `next` (02) | Absent. |
| `why` (02) | **Partly redundant — corrected.** `blocked <REF>` already claims to *"explain why a node is blocked or low-confidence."* See the amended slice 02 scope. |
| `search` (03) | **Convergent, and the inventory sharpens it:** legacy *had* `search`; it was dropped with the state-directory model, not deliberately as a capability. Under ID-mirrored storage it is needed *more* than it was, not less. |
| Flat rollup (03) | Compatible with C-4's `rollup --format={md,json}` + `--out` — flat is a third format, not a competing design. |
| `history` / `diff` (04) | Absent. Legacy `debug diff` is filesystem-vs-state, a different thing. |
| `export`, `viz` (05, 06) | Already in the inventory §3. Kept here only for sequencing. |

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
it is not yet met.

**Explicitly not in scope:** the surface polish and renames already chunked as
C-1…C-5 in `arc-release-hardening` (`context`→`project`, `path`→`chain`, list
overhaul, table styling, folding `self-host` into `migrate`). This arc assumes
those land first and does not restate them.

## Slice breakdown (shaped, plan-late)

| Slice | Scope | Source |
|---|---|---|
| **01 · read-back the status vector — CORE DONE (RH C-8)** | ✅ **Landed in C-8:** `show`/`--json` expose the per-gate vector (`reached`/`evidence`) + the normalized `status`; A-1 is satisfied. **Remainder:** add `by`/`evidence_dates` to the gates array; enumerate **unreached** gates in `--json` (the text `show` already lists them, so a consumer can see the *next* gate — a machine reading `--json` today sees only reached gates); the computed **satisfaction verdict**; and — for soft-satisfied deps — the **weakest link** (overlaps slice 02's readiness analysis). Optionally `odm status <ref>`. | UAT L-1 |
| **02 · explain the *ready* half; order `next`** | `blocked <ref>` already explains the blocked half. **Nothing explains readiness** — and that is the question an agent actually asks. On this store `blocked 1604` returns `nothing holding` and `--json` returns `[]`, which cannot distinguish *ready and well-supported* from *ready on an `asserted`-only dependency*. Extend `blocked` (or add the symmetric command) to report the supporting chain with evidence per hop and the weakest link named. Separately: `next` gains type labels, ordering by downstream fan-out, `unblocks: N`, `depth`, all in `--json`. | UAT L-4, L-5 (corrected) |
| **03 · `search` and flat rollup** | `odm search <text>` over names + bodies (`--type`, `--tag`, `--json`). Under ID-mirrored storage, topic-grep and `ls` stop working; search is their replacement, not a convenience. `odm rollup --format=flat` emits a greppable `id → number → type → name` map for consumers with **no odm binary**. | UAT L-7; the one `odm search` trace |
| **04 · `history` and `diff`** | `odm history <ref>` — gate transitions over time from `evidence_dates` + git. `odm diff <since>` — **what changed in the plan** since a commit/date: nodes added, gates advanced, edges rewired, evidence upgraded or downgraded. *This is the resume command:* `orient` says where things stand, `diff` says what moved while I was away, which is what a returning context actually needs. Feeds A7 telemetry directly. | UAT (new); A7 adjacency |
| **05 · `export`** | ODD-0017's projection-out: honestly-lossy, generated-do-not-edit, states any status-vector collapse and the rule used. Target set is ODD-0017 Q-1, still open. | ODD-0017 |
| **07 · `info`** | Restore legacy `info [states\|fields\|config\|stats\|dirs]`. Answers *"what are the gates for a slice"*, *"what fields are valid on this type"*, *"what is configured"* without reading source. Small, and it is the first thing a fresh context needs. | legacy surface (inventory §4) |
| **06 · `viz`** | Graph/decomposition rendering (`ascii-dag` first — terminal-native, no browser, LLM-readable). A renderer over existing state, never a source of truth. | `project-plan.md` §4; 0005 §5 |

**Sequencing:** 01 → 02 → 03 are the DoD-critical path and should not be split
across releases. **07 (`info`) is small enough to ride along with 01** and pays
for itself immediately. 04 is high value and pairs naturally with A7. 05 and 06 are
genuinely post-DoD and may belong to a later release entirely.

**Sizing note:** 01 and 02 are small (the data is already computed in memory and
simply not printed). 03 and 04 are medium. 05 and 06 are arcs' worth of work in
their own right if taken seriously, and are listed here only so they are not
lost again.

## Arc ledger

| Row | Criterion | Evidence | Sev | Status |
|---|---|---|---|---|
| **A-1** | A fresh context can answer *"what is the gate vector and evidence level of node X"* in **one** CLI call | **`odm node show 1600` / `--json` (RH C-8)** — per-gate `reached`/`evidence` + `status` | serious | **attested** (delivered by C-8; durable on push + CI) |
| **A-2** | A fresh context can answer *"why is X blocked"* in one call, naming the weakest link | transcript | serious | open |
| **A-3** | `next` output is ordered, typed, and carries the rationale for its order | transcript + `--json` shape test | serious | open |
| **A-4** | A consumer with **no odm binary** can resolve any ULID to a human-readable node | flat rollup artifact committed | correctness | open |
| **A-5** | A returning context can enumerate what changed in the plan since a named commit | transcript | serious | open |
| **A-6** | **The DoD is met and demonstrated**: a fresh LLM context reaches full situational awareness from `odm orient` (+ the commands above) alone, with no planning prose read. Verified by an LLM instance that has not seen this repository. | pass-3 UAT transcript | **serious** | open |
| **A-7** | No command added here introduces a per-actor metric or anything that could become a target | audit, mirroring A7's A-11 | correctness | open |

**A-6 is the arc's reason to exist.** It is also the only row that cannot be
closed by the party that implemented it — it requires a genuinely fresh context,
which is the same independence the ledger requires everywhere else.

## Open questions

- **Is the original command spec recoverable?** If yes, this plan is an input to
  reconciliation, not a replacement. Answer before scoping slice 01.
- **`status` as a field on `show`, or its own command?** UAT recommends the field
  (one call = full node picture); a dedicated command may still be warranted for
  scripting.
- **Does `diff` belong here or in A7?** It reads gate transitions from the same
  derived event log A7 builds. Building it here risks duplicating A7's
  derivation; deferring it to A7 delays the single most useful resume affordance.
  **Recommend: build the consumer here, and have A7 adopt it rather than
  re-derive.** Decide at slice 04 scoping.
- **Does `export` (05) belong to v1.0.0 at all?** ODD-0017 calls it the adoption
  engine; `project-plan.md` §5 scopes the v1.0.0 ledger to A1–A6 only. Likely a
  1.1 arc — flagged rather than assumed.

## Version History

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
