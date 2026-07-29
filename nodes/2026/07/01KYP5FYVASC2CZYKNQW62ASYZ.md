---
id: 01KYP5FYVASC2CZYKNQW62ASYZ
number: 520951000
type: artifact
schema: artifact/v1.1
name: Amendment stub — ODD-0020 (per-type schema markers) for RH C-2
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/C-2-amendment-ODD-0020.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# Amendment stub — ODD-0020 (per-type schema markers) for RH C-2

> Changes to fold into `docs/design/04-accepted/0020-versioned-file-metadata-schemas-per-type-schema-markers.md`,
> plus the version-history entry. **Surfaced by:** F-2, F-3. **Status:** proposed. Pairs with the
> ODD-0013 amendment (which owns the type rename + gate-sets); this ODD owns the **schema-marker**
> half and the **re-stamp path**.

## What changes

### The per-type marker set

The marker set (ODD-0020 §"Approach", the `schema: <type>/vMAJOR.MINOR` list) changes:

- **Remove** `odd/v1.0`.
- **Add** `design/v1.0` (the renamed design-doc contract — identical frontmatter shape) and
  `research/v1.0`.

New set: `project/v1.0`, `arc/v1.0`, `slice/v1.0`, **`design/v1.0`**, **`research/v1.0`**,
`adr/v1.0`, `note/v1.0`. (`saga` still gets none, unchanged.)

`research/v1.0` shares the design frontmatter contract initially (same required fields); it is a
*distinct marker* so the two can version independently later.

### The re-stamp path (migrate is the schema-upgrade path — unchanged principle)

ODD-0020 §"migrate is the schema-upgrade path" already frames migrate as the upgrade mechanism.
Extend it for the rename:

- On re-run, `odm migrate` re-stamps each migrated doc node: `schema: odd/v1.0` →
  `design/v1.0` **or** `research/v1.0` per the ODD-0013 classification rule (tags include
  `research`).
- **Legacy-read alias (decision to record).** After the rename, an on-disk `type: odd` /
  `schema: odd/v1.0` no longer parses (the enum variant is gone). Two clean options:
  **(A, recommended)** hard re-stamp the corpus in the same change (odm self-hosts — we own every
  node), and `check` proves zero `odd` markers remain; **(B)** additionally accept `odd`(`/vX`)
  as a **read-time alias** for one migration cycle, then drop it. Recommend **A** (no lingering
  alias), with B available only if a non-self-hosted consumer needs the grace window.

## Version-history entry to add to ODD-0020

```
### vX.Y — 2026-07-26
Marker set updated for the node-type rename (ODD-0013 amendment): removed `odd/v1.0`; added
`design/v1.0` (renamed, same contract) and `research/v1.0` (distinct marker, shared contract
initially). `odm migrate` re-stamps `odd/v1.0` → `design/v1.0`/`research/v1.0` on re-run; the
corpus is hard re-stamped (no lingering `odd` marker) rather than kept behind a read alias.
Surfaced by: RH UAT F-2/F-3. Realized in RH chunk C-2.
```
