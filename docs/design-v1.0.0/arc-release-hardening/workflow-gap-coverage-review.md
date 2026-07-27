# Workflow-gap coverage review — does odm's plan cover what the workflow broke?

**By:** CDC · **Date:** 2026-07-25 · **Against:** `release/1.0.x` @ `9223fc1` +
uncommitted session docs · **Status:** NON-AUTHORITATIVE input to release
triage, same standing as the UAT batches.

**Method.** The lykn 0.6.0 sessions of 2026-07-24/25 produced a concrete
inventory of multi-stream PM failures (register untracked while cited ×5;
cross-branch citations; a plan that drifted while `check`-equivalents stayed
green; two conversation-only specs that evaporated; a stale directory that
misled a reader the same day). Each failure class below is mapped to its odm
home — or its absence. Every MISSING/MISROUTED verdict is backed by a search
run today, cited inline; absence claims name what was searched.

---

## Ownership — read this first (added same day, after the operator couldn't tell)

**Every G-numbered gap is odm work.** lykn appears in this document in exactly
two roles: as the *evidence corpus* (the left column of the matrix — incidents,
not work items) and as the *acceptance test* for G-5 (the importer
generalization is odm code; a lykn `--dry-run` is how it is verified). **This
review adds zero items to lykn's queue**, which remains: arc15 slice04 →
arc07 + arc16 → arc09, with `03-citation-repoint` held pending odm migration.

One straddler: **G-6** is authored in odm (PM-skill, A6 slice05) and consumed by
lykn when the operator decides where lykn's store lives — a decision gated on
odm shipping, queued nowhere yet. And one item is three-repo: **G-12**'s
enforcing *check* would be odm; the evidence-tier *convention* it enforces lives
in the collaboration-framework (`billosys/ai-engineering`), alongside the other
framework changes discussed 2026-07-25 (Supplement hard-gate, load manifest),
which belong to neither odm nor lykn.

## The matrix

| # | Workflow failure (evidence) | odm answer | Verdict |
|---|---|---|---|
| 1 | Semantic paths churn; citations dangle (lykn: 353 sites, 57 dead) | ULID-mirrored storage, files never move (ODD-0013 §6) | **COVERED — core design** |
| 2 | "Done" claims without evidence quality (arc06 gate history) | Evidence ladder + min-propagation + soft-satisfaction (§4.4) | **COVERED — core design** |
| 3 | Silent scope drops at slice close | `decomposed` assertion + recomposition checks (§4.5) | **PARTIAL → G-3.** Mechanism exists, **unenforced**: arc-release-hardening itself has no node, A6 slices 05/06 have no nodes, `check` green throughout `[ran: grep -ril release.hardening nodes/ → 0]`. Enforcement proposal lives only in a findings doc (UAT L-2) — by the routing rule, **not routed** |
| 4 | Stale plans while work moves (project-plan v1.6 vs arc06 reality) | Nodes are the plan; rollup regenerates | PARTIAL — holds only after prose stops being plan-of-record; no prose-vs-node drift check for the transition period. Accept as transition cost, note in PM-skill |
| 5 | Resuming a context cheaply | `orient` (17 ms) + proposed `diff <since>` (llm-surface slice04) | **COVERED** (pending that arc) |
| 6 | What changed / who verified what | A7 event log + structured CDC emission | COVERED — scoped, not started |
| 7 | Cycle-breaks as recorded decisions | `tear --because` required | **PARTIAL → G-2.** The rationale is validated then **dropped** — no schema field. Inventory routes the fix to "arc02 slice08", **which does not exist** `[ran: ls arc02…/ → slices 01–07 + 05.1]`. A data-loss bug routed to a phantom slice — the register's `D-2607-8HTN` shape exactly |
| 8 | Conversation-only specs evaporating (the command menu; twice more below) | Nothing — process, but the tool can't fix what has no node | **G-1, G-12** |
| 9 | Multi-branch store incoherence (the week's central pain) | **Nothing.** No branch-topology guidance anywhere `[ran: grep -rniE "branch topology|store.*branch|single branch" docs/ → 0 relevant]` | **MISSING → G-6** |
| 10 | Concurrent sessions racing on shared artifacts | **Nothing** `[ran: grep -rniE "concurren|race|lock|collision" docs/design/ → 0 relevant]`. Per-node ULID files structurally minimize it; sequential `number` allocation under two writers is untested `[inferred — not exercised]` | **G-9** (document as known limit) |
| 11 | Migrating a real foreign project (lykn is next) | `migrate` (legacy ODD) + `self-host` (own plan tree) | **MISSING → G-5.** The importer is hardcoded to odm's own shape: literal `design-v1.0.0`, `arcNN-*/sliceMM*` only, `MVP_MAX_ARC = 6`, number scheme 1100–1600 baked in `[read: crates/odm-migrate/src/selfhost.rs:1,8,15,33,61]`. lykn has **16 arcs, 3 standalone no-wrapper slices, two design trees (v0.6.0 + v0.7.0) on different branches**, and the Discovery Register |
| 12 | odm committing in a repo where others work | `commit_all` commits the **whole worktree** — recorded as a behavioral note, unresolved `[read: arc01/slice04 cdc-verification.md:36-39]` | **G-7.** Fine while odm owns the repo; in lykn (multi-session, operator-only commits) it would sweep unrelated work. Whether any command auto-commits today: `[unverified — must be settled before foreign-repo use]` |
| 13 | Evidence claims without evidence *artifacts* | `status` carries `reached/by/evidence/evidence_dates` — **level only, no pointer** `[read: odm-core/src/status.rs:53-79]`. The lykn ledger's Evidence column (SHAs, report paths) has no odm slot | **G-8** (additive field; A7 slice03 adjacent) |
| 14 | LLM can't read back what it can write (gate vectors) | llm-command-surface slice01 (UAT L-1) | COVERED as plan — **arc unscheduled** → **G-4** |
| 15 | The register as first-class data | Type taxonomy has no `discovery`/`finding` type (C-2 adds `research` only) | **G-10** — decide at lykn migration: `note` vs new type |
| 16 | Cross-repo projects (lykn = 3 repos) | `external` nodes are for foreign *tools*, not sibling stores; saga named, "no operational weight" `[ran: grep saga → 2 hits, both deferrals]` | **G-11** — post-1.0, record only |

## Two decisions made in conversation today with **no repo record** `[ran: greps cited]`

- **G-1 — the ID-system switch. ✅ RESOLVED 2026-07-27 — ODD-0024: ULID retained, register-style rejected, minting freeze lifted.** Operator, 2026-07-25: *"I like yours better,
  and I'm going to switch odm v2 to use it"* — the register's `D-YYMM-XXXX`
  scheme vs ULID. **Zero hits in the repo** for any such intention. This is
  identity — the one thing that cannot change after ship — and it is currently
  as recorded as the original command menu was. Needs an ODD **before** any
  further nodes are minted. CDC's input for that ODD: keep ULID as `id`
  (filenames, sortability, 80-bit collision margin are load-bearing);
  the D-style scheme's natural slot is the *human-facing handle* — i.e. it
  competes with `number`, not with `id`. 4 chars of entropy is a handle's
  budget, not an identity's.
- **G-12 — evidence tiers on prose claims** (+ an `odm check` rule for
  unmarked state claims in CDC documents). Proposed and discussed 2026-07-25;
  recorded nowhere. Post-1.0, but needs a row so it survives.

## Ship-blocking shortlist (criterion: lykn migrates in week one)

| Gap | Work | Size |
|---|---|---|
| **G-1** ✅ DONE — ID decision recorded as **ODD-0024** (ULID retained) | decision, not code | hours |
| **G-2** tear-rationale persisted + surfaced in `check`; re-route from the phantom "arc02 slice08" to a real chunk here (**C-6**) | schema field + plumbing | small |
| **G-3** `decomposed`/orphan enforcement in `check` (warn; `--strict` error) | one check rule | small |
| **G-4** status read-back on `show`/`--json` (llm-surface slice01, pulled forward alone if the arc waits) | print computed state | small |
| **G-5** foreign-project migration scoped: generalize the importer (tree name, arc cap, standalone `NN-slug`, per-tree runs) + **a lykn `--dry-run` as UAT pass 3** | the real work | **medium-large — the long pole** |
| **G-7** settle auto-commit behavior for shared repos (off / curated index / explicit `--commit`) | decision + small change | small |

**Explicitly not ship-blocking** (so ASAP stays ASAP): G-6 (PM-skill doc, A6
slice05's natural content), G-8/G-9/G-10 (fast-follow, additive),
G-11/G-12 (post-1.0, recorded), the rest of llm-command-surface (slices 02–07),
A7/A8, C-1…C-5 already triaged.

## What this review deliberately did not do

No priority order inside the shortlist beyond the table; no scoping of G-5's
slices (that is a real design task); no verdict on G-1's design (only that it
must be *recorded*). All three are the operator's.
