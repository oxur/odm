---
id: 01KYP5FZB2FNNJ050FRVMS4P63
number: 569646700
type: artifact
schema: artifact/v1.1
name: Amendment stub — ODD-0013 §2.1 for RH C-7 (names embed no metadata)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/C-7-amendment-ODD-0013.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# Amendment stub — ODD-0013 §2.1 for RH C-7 (names embed no metadata)

> Changes to fold into `docs/design/…/0013-odm-architecture-design.md` §2.1, plus the
> version-history entry. **Surfaced by:** F-18. **Status:** proposed. This **generalizes** the
> v2.1 rule ("names don't embed numbers") — it does not replace it.

## What changes

### §2.1 — the `name` / `title` bullet

Current (v2.1):

> `name` / `title` — human label; freely editable; never affects identity or file location.
> **Names do not embed numbers** (v2.1): write `"Workspace scaffolding"`, not
> `"Slice 01 — Workspace scaffolding"`. … `odm list` strips such prefixes on display, but the
> convention is that they are not written in the first place.

Amended (v2.2):

> `name` / `title` — human label; freely editable; never affects identity or file location.
> **Names embed no metadata** (v2.2, generalizing v2.1). A name is the label for *the thing
> itself* — never its coordinates or its source document's role. Specifically, names carry:
> - **no numbers / positional refs** — `"Workspace scaffolding"`, not `"Slice 01 — Workspace
>   scaffolding"` or `"… (Arc 06)"` (that's `number` + the `part_of` tree);
> - **no document-role labels** — not `"… (plan-of-record)"`, `"… (build plan)"`, etc. (that's
>   `type` + which plan doc it came from).
>
> Everything in that list is *already* carried by `number`, the `part_of` containment tree, and
> `type`/gates; embedding it in the name duplicates it and goes stale on any renumber or re-role.
> **Enforcement is at mint time:** the `self-host` / `migrate` importer **normalizes** a derived
> name to this rule (strips the number-prefix and role-suffix) rather than copying a plan-doc H1
> verbatim. `odm list`'s display-stripping (F-6) is then belt-and-suspenders, not the mechanism.
>
> *Scope note:* the normalizer targets the **mechanical, unambiguous** cases (leading
> `"<Type> NN —"` / `"… (Arc NN)"` prefixes; the known role-suffixes `(plan-of-record)`,
> `(build plan)`). Genuinely descriptive parentheticals that are part of a name's meaning
> (e.g. `"(v-major rebuild)"`) are left to human judgment — the rule prohibits *metadata*, not
> *qualifiers*.

## Version-history entry to add to ODD-0013

```
### v2.2 — 2026-07-26
Generalized the §2.1 naming rule from "names don't embed numbers" (v2.1) to "names embed no
metadata": names also carry no document-role labels ("(plan-of-record)", "(build plan)").
Enforcement moves to mint time — the self-host/migrate importer normalizes derived names instead
of copying plan-doc headings verbatim. Surfaced by: RH UAT F-18 (32/47 work-node names carried a
"(plan-of-record)" suffix inherited from plan-doc H1s). Realized in RH chunk C-7.
```
