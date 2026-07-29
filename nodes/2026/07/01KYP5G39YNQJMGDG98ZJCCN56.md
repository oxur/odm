---
id: 01KYP5G39YNQJMGDG98ZJCCN56
number: 509948700
type: artifact
schema: artifact/v1.1
name: UAT punch list — batch 1 (first-pass, NON-AUTHORITATIVE)
created: 2026-07-25
updated: 2026-07-25
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/uat-punch-list.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# UAT punch list — batch 1 (first-pass, NON-AUTHORITATIVE)

> **Duncan's raw hands-on-UAT feedback, verbatim.** Captured 2026-07-07. This is a
> **first-pass, non-authoritative** dump from driving the self-hosted `odm` — the source
> record, *not* a settled spec. The CDC triage (surface vs model) + chunking lives in
> `arc-plan.md` → **Findings Log (F-1…F-14)** and the **Chunks table (C-1…C-5)**; where this
> list and the triage disagree, this list is the *input* and the arc-plan is the *working
> disposition*. More batches expected (~3–4 total). Nothing here is final until dispositioned.

---

## Output / global

- The output is a table for me, **no colours**; when we'd originally spec'ed this out, we'd
  talked about using the **oxur-cli/table styling/theming** — this is **VERY important** to me
  as part of the first release (since there's already an established pattern, etc.). If we need
  to **split the table/terminal code out of oxur-cli, that's fine**.

## `odm list`

- In the UI, we don't want to use **"odd" as a type**; this needs to be **"design"** instead.
  Do we have one for **"research"**? Will need that, too — I see various research docs that
  should be switched from odd/design to a **research** type.
- The **number column** needs to go.
- The **first column should be "date" (creation)**; there should be a flag like
  **`--date=updated`** which would cause updated to be used instead of created.
- **References to numbers in titles** should be removed (that will be very confusing as time
  moves on).
- We should add a **"status" column after "type"**.
- The **arc/slice/etc. prefixing** in the listing as part of "name" should go; instead,
  starting at project and going down for a project's arcs and slices, we should use
  **branch-and-leaf ASCII/table parts** (with indents, etc.) to show relationship/placement.
- We should have a **max display width** config option + flag; anything past that should have
  the remainder **elided with ` ...`** (or rather anything past that minus the 4 char spaces
  needed for ` ...`).

## `odm --help` / command surface

- **`odm new`** is good to have idempotent; but having it **display information upon
  subsequent calls** is a confusing UX antipattern; instead, we should **warn** the user:
  *"project exists; for details run 'odm project --name=<name>'"*.
- **`odm context`** is not a good name, as it's too confusing/general; **`odm project`** would
  be better, with the current project being the default, supporting others with
  `odm project --name=...`.
- **`odm path`** is going to be confusing; people will think "filepath"; instead, we should
  rename to something like **`odm dep`** (other better name options?). *(→ decided:
  **`odm chain`**.)*
- **`odm rollup`** has help text that says what the output is (`ROLLUP.md`); that should **not**
  be part of the help text, since this should support **both md and json** output, with the
  option of also providing the **output name** (defaults: "md" and "ROLLUP").
- **`odm self-host`** just seems like a special case of **`odm migrate`**; let's **combine**
  and support the case that `odm self-host` currently offers as part of `migrate`.

---

*"I haven't tried using all of the commands yet, but this feels like a good enough place to
start; there will be more feedback for the other commands, though … this is definitely feeling
like its own arc."* — Duncan, 2026-07-07
