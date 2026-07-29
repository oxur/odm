---
id: 01KYP5FYP8NN14ZE3T66T5QX69
number: 568128400
type: artifact
schema: artifact/v1.1
name: Amendment stub — ODD-0013 (Architecture & Design) for RH C-2
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/C-2-amendment-ODD-0013.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# Amendment stub — ODD-0013 (Architecture & Design) for RH C-2

> **Draft the amendment before the cc-prompt** (RH arc-plan §Amendments). This stub specifies
> the exact changes to fold into `docs/design/01-draft/0013-odm-architecture-design.md`, plus
> the version-history entry to add. **Surfaced by:** F-2, F-3. **Status:** proposed.

## What changes

### §2.2 Node types — rename `odd` → `design`, add `research`

Current (§2.2): *"Document nodes: `odd` (design doc), `adr`/`rfc` (decision record), `note`."*

Amended:

> **Document nodes:** `design` (a design document — formerly `odd`), `research` (a research /
> literature-survey / investigation doc that *informs* decisions but does not govern like a
> design doc), `adr`/`rfc` (decision record), `note`.

Rationale (F-2): `odd` is odm-internal jargon that leaked into the UI; `design` is what the
artifact *is*. Rationale (F-3): several corpus docs are **research**, not design, and the type
should say so (they feed the PM-skill and the A7/A8 telemetry/forecasting line).

`type` remains fixed-at-creation; "new types are config + a gate-set" (unchanged principle) —
this amendment simply renames one type and adds one.

### §5 Gate-sets — rename `[gates.odd]` → `[gates.design]`, add `[gates.research]`

`[gates.odd]` becomes `[gates.design]` with the **same sequence** (it always was the design-doc
lifecycle):

```toml
[gates.design]
sequence = ["draft", "under-review", "revised", "accepted", "active", "final"]
```

Add a `research` gate-set. **Recommendation: mirror `design` initially** —

```toml
[gates.research]
sequence = ["draft", "under-review", "revised", "accepted", "active", "final"]
```

**Why mirror rather than a tighter set (the one open decision here).** The migrate importer maps
a source doc's **state directory** (`01-draft`…`06-final`) onto a gate reach. A shared full
sequence means research docs in *any* state dir map cleanly with zero special-casing. A
semantically tighter research lifecycle (e.g. `["draft", "under-review", "final"]`) is defensible
— research docs don't really go "active" — but it would also require constraining which state
dirs research docs may occupy (or a bespoke mapping), which is scope C-2 should not take on.
**Decision for the operator:** mirror now (recommended, zero-churn), tighten later if research
docs prove they need a distinct lifecycle.

### Classification rule (the durable design decision)

The migrate importer classifies a document node as **`research` iff its source frontmatter
`tags` include `research`**, else **`design`**. This is self-documenting and robust — preferred
over title-prefix or filename parsing.

**Corpus caveat (must be fixed as part of C-2):** node **0011** ("Research: A markdown/git-native
…") carries placeholder `tags: [change-me]` and would misclassify as `design` under this rule.
Its tags must be corrected to include `research`. The authoritative current research set is
**0011, 0014, 0016, 0018** (4 docs); the remaining 9 migrated docs are `design`. (Node 0012 also
carries `tags: [change-me]` — a design doc; fix its tags too, non-classification-critical.)

### Migration of the existing corpus

Existing on-disk `odd` nodes are re-stamped by **re-running `odm migrate`** with the amended
mapping (→ `design`/`research`), then `odm self-host`; `odm check` must stay green. See ODD-0020
amendment for the schema-marker half and the legacy-alias question.

## Version-history entry to add to ODD-0013

```
### vX.Y — 2026-07-26
Node-type taxonomy: renamed document node `odd` → `design` (§2.2); added a `research`
document type for investigation/literature docs. Gate-sets (§5): `[gates.odd]` → `[gates.design]`
(unchanged sequence); added `[gates.research]` (mirrors `design` initially — see the amendment
rationale). Classification: a document node is `research` iff source `tags` include `research`,
else `design`. Surfaced by: RH UAT F-2 (odd→design) + F-3 (add research). Realized in RH chunk C-2.
```
