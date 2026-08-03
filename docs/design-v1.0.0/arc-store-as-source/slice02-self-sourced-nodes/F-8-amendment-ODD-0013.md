# Amendment stub — ODD-0013 (Architecture & Design) for arc-store-as-source slice02

> **Draft the amendment before the cc-prompt closes** (slice-doc §Scope-in item 6). This stub
> specifies the exact changes to fold into `docs/design/04-accepted/0013-odm-architecture-design.md`,
> plus the version-history entry to add. **Surfaced by:** ODD-0026 §3 (the accepted design basis).
> **Status:** proposed — implemented in code (`odm-core::Origin::Authored`,
> `odm-core::frontmatter::Source::authored`, `check::Violation::InconsistentAuthoredSource`); this
> stub is the schema-doc half.

## What changes

### §2.2 Node types — no new type, but the origin/provenance framing needs one new value

§2.2 already lists the full node-type set (`project`/`arc`/`slice`/`design`/`research`/`adr`/`note`/
`artifact`); this amendment adds **no new type**. "Authored" is not a node type — it is a value on
the existing `origin` field (§2.3), orthogonal to type. No change needed here beyond a forward
pointer to §2.3.

### §2.3 Frontmatter schema (normative) — `origin: authored` + the authored `source` shape

Current (§2.3):

```yaml
origin: planned                     # how this node AROSE: planned | discovered | amendment
...
source:                             # optional; every MIGRATED node carries one (v2.4, ODD-0025
                                     #   §2.0/§2.2) — work and document nodes alike; absent on a
                                     #   hand-created node. Distinct from `origin` (why a node
                                     #   exists) and from *provenance* below (derived, never
                                     #   stored) — `source` is stored because git cannot derive it.
  paths: [docs/design-v1.0.0/arc04-index-cache/slice07-early-cutoff/slice-doc.md]
  class: slice-doc                  #   the source doc's class
  normalization: trim+lf            #   what the body-hash gate stripped before comparing
  migrated_by: odm-migrate/1.0.0    #   tool + version
  migrated_on: 2026-07-27
```

Amended:

```yaml
origin: planned                     # how this node AROSE: planned | discovered | amendment | authored
...
source:                             # ODD-0025 §2.0/§2.2, extended ODD-0026 §2.1: `source` is
                                     #   ENDURING PROVENANCE, kept for every node that has ever had
                                     #   one — "authored" is a provenance VALUE, never the absence of
                                     #   the field. Two shapes share this one record:
  # --- migrated shape (origin: planned/discovered/amendment) — unchanged ---
  paths: [docs/design-v1.0.0/arc04-index-cache/slice07-early-cutoff/slice-doc.md]
  class: slice-doc                  #   the source doc's class
  normalization: trim+lf            #   what the body-hash gate stripped before comparing
  migrated_by: odm-migrate/1.0.0    #   tool + version
  migrated_on: 2026-07-27
  # --- authored shape (origin: authored) — ODD-0026 §2.1 ---
  # class: authored                 #   marks the provenance state explicitly (never an absent block)
  # paths: []                       #   no external path — the store IS the source
  # normalization: (absent)         #   nothing to hash-normalize; there was no migration
  # migrated_by: (absent)           #   no migrating tool
  # migrated_on: (absent)           #   no migration date
  # migrated_from: []               #   OPTIONAL historical marker (slice02 sub-decision (i)):
                                     #     the node's former `source.paths`, preserved when an
                                     #     already-migrated node is converted to authored, so its
                                     #     provenance is not lost — just no longer re-verified against
```

**The load-bearing invariant (repeat from ODD-0026 §2.1, stated here as schema law):** `source` is
never dropped for having "nothing to migrate." An authored node's `source.class` is the string
`"authored"`, and `check` (`Violation::InconsistentAuthoredSource`) enforces both directions of the
agreement: `origin: authored` requires `source.class == "authored"` with no `paths`/`migrated_by`/
`migrated_on`; `source.class == "authored"` requires `origin: authored`. Neither reading can drift
from the other silently.

### §2.3 — the author-vs-odm field boundary (ODD-0026 §2.5)

Add, immediately after the frontmatter example:

> **The author-owned vs odm-owned field boundary** (ODD-0026 §2.5). The metadata an author supplies
> when creating or editing a node is a strict subset of the schema: `name`/title, `type`,
> `edges.part_of`, status intent, and `tags`. Everything else is **odm-owned** and never
> author-settable: `id` (mint-time only), `number` (mint-time only), placement/path (derived from
> type + containment), and the entire `source`/provenance record (odm-set from the migration or
> authoring event, never hand-supplied). A future authoring surface (slice03) validates this
> boundary before any write — a partial that tries to set an odm-owned field is rejected, not
> silently accepted and overwritten.

## Version-history entry to add to ODD-0013

```
### vX.Y — 2026-08-03 — arc-store-as-source slice02: `origin: authored` + the author/odm boundary

Adds `authored` to the `origin` value set (§2.3): a node born in the store, never pulled from an
external file. `source` is reaffirmed as enduring provenance, kept for every node that has ever
carried one — "authored" is `source.class`'s explicit value for a store-born node (no `paths`, no
`migrated_by`/`migrated_on`), never the absence of the `source` block. An optional
`source.migrated_from` marker preserves a converted node's former `source.paths` without re-verifying
against it. `check` (`Violation::InconsistentAuthoredSource`) enforces the two-way agreement between
`origin` and `source.class`. Also records the author-owned vs odm-owned field boundary (ODD-0026
§2.5) that a future native-authoring surface (slice03) validates before every write. Surfaced by:
ODD-0026 §3 (design basis), implemented arc-store-as-source slice02.
```
