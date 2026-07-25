# UAT — pass 2: LLM acceptance (batch 2 of this arc)

**By:** CDC (Cowork Claude), a fresh context with no prior odm exposure ·
**Date:** 2026-07-25 · **Against:** `release/1.0.x` @ `9223fc1`, the self-hosted
58-node store · **Method:** built `oxur-odm` from source in a clean sandbox
container (`cargo build --release -p oxur-odm`, **2m54s, exit 0**) and drove the
CLI as a user, not a reader.

**Status: NON-AUTHORITATIVE first pass**, same standing as
`arc-release-hardening/uat-punch-list.md`. Findings are `L-1…L-9` to avoid
colliding with that arc's `F-1…F-14`. Triage and disposition are the operator's.

> **Scope note.** Pass 1 (Duncan's punch list) asked *"is this usable by a
> human?"* — surface, naming, rendering. This pass asks *"can an LLM regain
> situational awareness and act from the tool alone?"*, which is the project's
> stated DoD. The two overlap less than expected: almost nothing below is a
> restatement of F-1…F-14.

---

## Headline

**The thesis holds and the plumbing is sound.** `check` is green on 58 nodes,
integrity and drift are clean, and the speed claim is not marketing:

| Command | Latency |
|---|---|
| `odm next` | **3 ms** |
| `odm check` | **8 ms** |
| `odm rollup --dry-run` | **12 ms** |
| `odm orient` | **17 ms** |

*"Project status is a CLI call, not an expensive LLM read-and-analyse"* is
**empirically true**. At 17 ms I will call `orient` reflexively; a 40k-token
document read I ration. That difference is the whole product.

**What does not yet hold is the DoD itself** — *"a fresh session reaches full
situational awareness from `odm orient` alone."* I was that fresh session. I
learned more from reading `project-plan.md`. Findings L-1…L-4 are why.

---

## Findings

### L-1 — Status is write-only *(severity: blocking for an LLM consumer)*

Multi-gate status vectors with evidence levels are odm's central innovation, and
**no command reads one back for a node.**

```
odm show 1600           → id, origin, created, updated, part_of, children.  No status.
odm show 1600 --json    → keys: id, number, type, name, origin, reserved, tags,
                          component, retired, part_of, supersedes
odm list --json         → same shape.  No status.
odm --help | grep gate  → only `set-gate` (a writer)
```

`set-gate` writes it. `ROLLUP.md` renders the whole tree. But to answer *"what
is the gate vector on slice 1604, and at what evidence level?"* I must grep the
markdown or parse `ROLLUP.md` — both of which defeat the reason to have a CLI.

This is the single highest-value fix in this report. It is also the one that
makes the evidence ladder *usable* rather than merely *stored*: soft-satisfaction
and min-propagation are the best ideas in the system, and today a consumer
cannot observe either.

**Proposal:** add `status` to `show` (text + `--json`), including per-gate
`reached` / `by` / `evidence` / `evidence_dates`, the computed satisfaction
verdict, and — where a dependency is soft-satisfied — the weakest link that made
it so. Optionally a dedicated `odm status <ref>`.

### L-2 — Nothing reconciles the plan against the node set, and two release-blocking bodies of work have no node at all *(severity: serious)*

`odm check` reports `ok (58 node(s), no problems)`. Meanwhile:

- **This entire arc has no node.** `arc-release-hardening/` has an arc-plan at
  v1.2, chunks C-1…C-5, findings F-1…F-14 and ledger rows RH-1…RH-5, and is
  described as **v1.0.0 release-blocking**. `grep -rl release.hardening nodes/`
  returns nothing. The store holds six arcs; this is not one of them.
- **A6 slice05 (PM-skill, ledger A-5) and slice06 (retire prose, A-11) have no
  nodes.** The store holds 1601–1604.
- Neither absence is detectable by any command. The `decomposed: {on, children}`
  mechanism that would catch it (ODD-0013 §4.5) exists and is not enforced.

**I reproduced the failure while writing this report, which is the useful part.**
I created a node for "A6 slice05 — UAT" after reading arc06's *directory
listing*. The `slice05-uat-cli-feedback/` directory is a **tombstone**: arc-plan
v1.8 briefly inserted UAT there, and **v1.9 reverted it** and extracted UAT to
this arc. I trusted the filesystem over the plan-of-record, exactly as a stale
path invites. The node is retired in place (#1605) with the reason recorded,
rather than deleted — it is a dated instance of the class.

So the finding has two halves. **Work with no node is invisible to the tool**,
and **directories that outlive the plan revision that abandoned them actively
mislead readers** — including readers who are checking for exactly this.

**Proposal:**
1. `check` warns — and `--strict` errors — when a parent has children but no
   `decomposed` assertion, and when the assertion disagrees with reverse-`part_of`.
2. Give `arc-release-hardening` a node before release; it gates the release and
   currently exists only as prose.
3. Tombstone directories should be removed rather than annotated. A file that
   says "safe to remove" and is not removed is a trap with a note on it.

### L-3 — `orient` degrades silently on the project that defines its DoD *(severity: serious)*

Run cold as a fresh session:

```
VISION  #1000 odm v1.0.0 — Project Plan (arc roadmap)
  _(no vision text yet — add a `# Vision` section to the project body)_
CURRENT FOCUS
  (no current arc — `odm use arc <ref>`)
```

The self-host cutover carried the plan's *structure* and not its *substance*.
Both hints are well-written and actionable, so this is not a UX failure — it is a
**data failure that only a human notices**. An LLM takes the output at face
value and concludes the project has no stated vision.

**Proposal:** two parts. (a) Backfill the vision/focus during self-host, or make
it a migration follow-up row. (b) `check` should treat *a project with no vision
body* as a finding, because the DoD depends on it. A DoD asserted in prose and
unchecked by the tool is the same shape as L-2.

### L-4 — `next` is an unordered, untyped set *(severity: serious)*

```
#17 Interop — projection out, reference-and-reconcile in
#15 odm — Arc/Slice Breakdown (build plan)
#19 Incremental drift — probes as input-tracked rules …
#18 Research — Forecasting under small, bursty, DAG-structured work
#13 odm — Architecture & Design (v-major rebuild)
#20 Versioned file-metadata schemas …
#1000 odm v1.0.0 — Project Plan (arc roadmap)
#1600 Arc 06 — Migrate, self-host & PM-skill (plan-of-record)
```

Three problems. **No type labels** — `#17` and `#1600` are a design doc and an
arc, indistinguishable here (`orient`'s READY block *does* show types, so the two
views of the same list disagree). **No ordering** — for a tool whose thesis is
dependency-derived ordering, the ordering command returns a set. **No rationale**
— nothing says which of these unblocks the most downstream work.

**Proposal:** type labels; sort by downstream fan-out (or topological rank); add
`unblocks: N` and `depth`. `--json` should carry all three as fields. This turns
`next` from a list into advice, which is what an agent actually needs.

### L-5 — `why` exists for the blocked half only *(severity: high value, low cost)*

**Corrected after `odm-command-inventory.md` landed.** The first draft claimed
there is no `why` command. There is: `blocked <ref>` is documented as
*"Explain why a node is blocked or low-confidence."* I had listed `blocked` in
this report's own disclosed-limits section as untested and wrote the finding
anyway. That is the failure this report's L-2 is about, committed inside the
report — logged rather than quietly fixed.

**The real asymmetry, which does survive:** nothing explains why a node is
*ready*, and that is the question an agent asks far more often. On this store:

```
odm blocked 1604          → blocked: nothing holding #1604 …
odm blocked 1604 --json   → []
```

An empty array cannot distinguish **ready and well-supported** from **ready on
an `asserted`-only dependency** — which is exactly the soft-satisfaction
distinction the evidence ladder exists to draw, and exactly what an agent needs
before starting work. `odm path 1604` likewise returns a single line (the node
itself) that is indistinguishable from an error.

**Proposal:** extend `blocked` — or add its symmetric partner — to report the
*supporting* chain: each hop, its evidence level, and the weakest link named.
Nothing new needs computing; min-propagation already produces it.

### L-6 — Distribution: the gap is reachability, not feasibility *(severity: medium)*

No design document states a position on prebuilt binaries; the README says build
from source or `cargo install oxur-odm`. But the empirical result is that a
clean container with a stock toolchain built the release binary in **2m54s** with
no special handling. The binary is 5,074,296 bytes, dynamically linked against
glibc.

So the barrier is not build difficulty — it is that a consumer without a Rust
toolchain has no path at all, and a consumer *with* one has no obvious signal
that this is expected to work.

**Proposal:** a CI matrix publishing to GitHub Releases (`cargo-dist` is the
cheap route), with **`x86_64-unknown-linux-musl` statically linked** for the
Linux artifact so it is a single portable file with no glibc match required.
Combined with the existing `cargo install`, that covers essentially every
consumer. This matters more than it looks: ODD-0017's adoption thesis is
legibility to teams who have not adopted odm, and those teams do not have cargo.

### L-7 — `ROLLUP.md` is the index, and it is missing one form *(severity: low, high leverage)*

`ROLLUP.md` is generated, fingerprinted, marked do-not-edit, and renders the
whole tree with gate states — it is already a better artifact than a naive
"index file" proposal. What it does not offer is a **flat, greppable
ID → number → type → name map**.

The use case is concrete and mine: a session with no odm binary — no cargo, no
network, a sandbox — that needs to resolve `01KWXMBBTKNA3A0QC3SWPHBNAX` to
something meaningful. Under ID-mirrored storage, `ls` and topic-grep stop
working; a flat map restores them at near-zero cost.

**Proposal:** `odm rollup --format=flat` (or a second generated artifact),
one line per node. Derived, regenerable, no new source of truth.

### L-8 — Authority has drifted out of the state directories *(severity: medium, and ironic)*

ODD-0013 is the normative architecture at v1.9 and lives in `01-draft/`, while
the two documents that amend it (0019, 0020) are in `04-accepted/`. ODD-0017 and
ODD-0018 are load-bearing for scoped arcs and also `01-draft`.

The directories no longer report the documents' actual authority — which is
precisely the *truth-encoded-in-directory-position* problem ODD-0013 §9 retires
for migrated nodes. The `odd` corpus kept the disease the node model cured.

**Proposal:** move the gate vector to be authoritative for design docs too (it
already is, in the node), and treat the directory as display. At minimum,
reconcile the four documents before release.

### L-9 — `number` looks positional; say whether it is *(severity: low, but cheap to get wrong)*

The scheme reads `1_A_SS`: #1000 project, #1600 arc 06, #1604 its slice 04. If
arcs are ever resequenced, either the numbers churn — breaking the locked-handle
property for the human-facing identifier — or they keep a shape that no longer
means what it looks like.

This is a solved problem elsewhere in the family: lykn hit it and settled on
*"NN = creation order, NOT dependency order; stop renumbering on inserts."*

**Proposal:** state the invariant explicitly in ODD-0013 §2.3. One sentence,
and it prevents a class.

---

## Two things worth recording as strengths

**The research constrains the design.** ODD-0016 rejects the Standish/CHAOS
figures with the Eveleens & Verhoef sign-flip, demotes DORA's multipliers to
`[P]`, notes Little's Law is a theorem only for stable systems — and then the
design *refuses to build* the estimation layer the evidence does not support.
Tools normally do the research and ship the feature anyway. Letting evidence
delete scope is the rarest thing in this repository.

**Arc 08 builds the control arm.** The two-clock split is named as *"an untested
hypothesis, not an established technique"*, the literature sweep that found no
supporting evidence is reported as an absence rather than buried, and the plan
builds total-cycle Monte-Carlo as the control so the event log adjudicates on
held-out calibration. That is a real experiment inside a planning tool.

---

## Disclosed limits of this pass

- **L-5 was wrong on the first pass** — I asserted a missing command while
  `blocked` was sitting in my own untested list. Corrected above; the surviving
  half is narrower and better.
- **I got L-2's evidence wrong on the first pass** and corrected it in place —
  the original draft cited the tombstoned slice05 directory as a missing node.
  The corrected finding is stronger, and the error is left visible above because
  it is the finding.
- **Read-mostly.** I exercised `orient`, `check --strict`, `next`, `list`,
  `show`, `rollup --dry-run`, and `--json` on each. I did **not** exercise
  `reconcile`, `migrate`, `self-host`, `tear`, `supersede`, `retire`, `link`,
  `unlink`, `decomposed`, `use`, `path`, or `blocked` against real state. A
  second batch should.
- **One store, and it is odm's own** — 58 nodes, self-hosted, homogeneous. Scale
  behaviour is untested here, and ODD-0013 Q-5 (rollup granularity/perf at
  scale) remains open on its own terms.
- **The container clone is `9223fc1`**, matching the device tip at the time of
  writing. Nothing here was measured against uncommitted work.
