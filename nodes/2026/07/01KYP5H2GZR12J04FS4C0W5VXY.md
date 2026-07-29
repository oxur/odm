---
id: 01KYP5H2GZR12J04FS4C0W5VXY
number: 519434700
type: artifact
schema: artifact/v1.1
name: Doc ↔ node reconciliation audit — odm self-host corpus (2026-07-27)
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/reconciliation-audit-2026-07-27.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
---
# Doc ↔ node reconciliation audit — odm self-host corpus (2026-07-27)

> **Scope:** the `1.0.x` design docs (`docs/design-v1.0.0/` + `docs/design/` ODDs) vs. the odm store
> corpus (`odm` branch, 60 nodes @ `e2ab628`). **Purpose:** ground the model decision + the
> reconciliation. **Method:** static walk of both trees + frontmatter parse (clone at `e2ab628`,
> node-identical to the live store). No binary run, no writes. **Verdict: the corpus captures the plan
> *skeleton* only** — 6 of 11 arcs, 39 of 43 slices, and *one* primary doc per unit; every supporting
> doc and every body is absent.

## 1. Node inventory (what's in the graph)

| type | nodes | stub (no body) | full body | no `part_of` |
|---|---|---|---|---|
| arc | 6 | **6** | 0 | 0 |
| slice | 39 | **38** | 1 | 0 |
| design | 9 | 0 | 9 | **9** |
| research | 5 | 0 | 5 | **5** |
| project | 1 | 0 | 1 | 1 (root) |
| **TOTAL** | **60** | **44** | **16** | **15** |

Two findings here, not one:

- **44 of 60 nodes are bodyless stubs** — every arc and all but one slice. `self_host` writes
  `Document::new(fm, "# {name}\n")` (structure, no substance); the design/research/project nodes have
  bodies because they came through the *file-importing* path (`migrate --legacy`) or a hand backfill
  (the project vision, L-3a).
- **14 non-root nodes have no `part_of`** — all 9 `design` + all 5 `research`. They are **not in the
  containment tree at all**: they float beside the project→arc→slice skeleton rather than hanging under
  it. So "100% revealed in the graph" isn't true today even for the nodes that *do* have bodies. (Worth
  confirming whether odm's `orphan` error currently exempts these types or simply hasn't been run against
  them — either way, they're unattached.)

## 2. Plan-doc inventory (what's on disk)

- **arc directories: 11** on disk vs **6 arc nodes** → **5 arcs unrepresented.**
- **slice directories: 43** on disk vs **39 slice nodes** → **4 slices unrepresented.**
- **266 total plan `.md` files**; only ~**55** are "primary" (project-plan / arc-plan / slice-doc, the
  ones self-host maps to a node). **~211 docs are invisible to the graph.**

| doc class | count | has a node? |
|---|---|---|
| primary (project-plan / arc-plan / slice-doc) | 55 | yes |
| canonical support (ledger / cc-prompt / cdc-verification / closing-report) | 173 | **NO** |
| ad-hoc / other | 38 | **NO** |

### The 5 unrepresented arcs — triage needed (not all the same)

`arc01…arc06` (Substrate, Graph, Rollup, Index, Reconciliation, Migrate) **are** the 6 arc nodes.
Missing: `arc-store-home`, `arc-release-hardening`, `arc-llm-command-surface` — **real, post-cutover
arcs that should be nodes** (genuine drift) — plus `arc07-two-clock-telemetry`, `arc08-forecasting`,
which may be **future/planned** arcs that legitimately aren't nodes yet. Worth confirming which is which.

## 3. ODD docs vs `design` nodes

16 ODD files on disk (numbers 2, 9–23); 9 `design` nodes. **ODD-0022 and ODD-0023 have no node**
(verified by store grep — no node names "store home" / "command surface"), and the design-node ↔ ODD
mapping is incomplete besides. The amendment I drafted is a third doc in this same missing-from-graph
class.

## 4. Supporting-doc taxonomy — what the flexible model must hold

Nodeless supporting docs, by kind (this is the shape the model has to accommodate — at **slice, arc,
and project** scale):

**High-volume, canonical (per slice):** `closing-report.md` ×46 · `ledger.md` ×43 ·
`cdc-verification.md` ×42 · `cc-prompt.md` ×42.

**Ad-hoc / cross-scale:** `adr-c1-*` (an ADR) · `C-2/C-7-amendment-ODD-00NN` (design-doc **amendments** —
same class as the ODD-0022 amendment) · `benchmark-results` · `cc-prompt-amendment` ·
`workflow-gap-coverage-review` · `uat-punch-list` / `uat-coverage-audit` / `uat-report-*` ·
`arc-close` · `odm-command-inventory` · `CDC-SESSION-BOOTSTRAP`.

So the primary/supporting split is real and open-ended: a unit has **one** plan-of-record plus **N**
supporting artifacts of varied, growing kinds — exactly the "flexible, but fully revealed" model.

## 5. What this implies for the model (the decisions to make)

1. **Supporting docs become nodes, `part_of` their owner.** A slice node's body = `slice-doc.md`; its
   `ledger` / `cc-prompt` / `cdc-verification` / `closing-report` / benchmarks / amendments become
   **child nodes** (`part_of` the slice), each carrying its own doc as its body. Same at arc scale
   (arc-plan is the arc node's body; cc-prompts/closing-reports/adrs/amendments are children) and
   project scale. This answers the earlier "what is a slice's body?" question cleanly: the plan-of-record
   is the body; everything else is a contained child.
2. **A node *class* distinction, likely.** A `cc-prompt` or `closing-report` is an **artifact**, not
   **work** — it shouldn't get work-gates, ordering, or readiness. odm already has non-work doc nodes
   (`design`/`research`, with doc-gates draft→final), so there's precedent: supporting docs want a
   documentation/artifact class that is *contained and revealed* but *excluded from the work-graph*
   computations. What **type(s)** they take (`note`/`adr` + a `role` attribute, vs. new per-kind types)
   is your call.
3. **Two checks are missing and needed for "100% revealed":**
   - **Doc-coverage (the inverse of `orphan`):** every `.md` in the plan/ODD trees has a node — flag any
     doc with none. This is the guard that would have caught all 211. It **does not exist** today.
   - **Attach design/research to the tree:** give the 14 floating design/research nodes a `part_of` so
     they're revealed, and confirm the `orphan` error actually covers them.
4. **Re-import is a real derivation, not a re-run.** `self-host` as built imports skeleton-only; carrying
   bodies + minting supporting-doc children + wiring the class distinction is **new capability**, not a
   flag. It wants its own arc.

## 6. Recommendation

Scope this as an **arc on `1.0.x`** — call it the corpus-reconciliation arc — that **subsumes L-8b**
(the ODD drift is one slice of it). Rough slice shape, for your adjustment:

- **model** — the supporting-doc node class + `part_of` containment + the body-is-plan-of-record rule
  (an ODD; this is where the type/class decision lands).
- **checks** — the doc-coverage guard + design/research attachment (so the model is enforced, not just
  intended).
- **derivation** — extend self-host/migrate to carry bodies and mint supporting-doc children, idempotent
  and re-runnable.
- **reconcile** — run it against odm's own corpus; the 3 real missing arcs, the 4 slices, ODD-0022/0023 +
  the amendment, and the ~211 supporting docs all come in; verify no doc left uncovered and no orphan.

**Not to run before that lands:** any `migrate`/`self-host` against the live corpus — the current
derivation would re-mint skeleton stubs, not fix them.
