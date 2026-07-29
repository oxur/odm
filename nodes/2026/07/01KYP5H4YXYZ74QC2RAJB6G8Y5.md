---
id: 01KYP5H4YXYZ74QC2RAJB6G8Y5
number: 24
type: design
schema: design/v1.1
name: ID scheme — retain ULID identity; register-style handle rejected
created: 2026-07-27
updated: 2026-07-27
tags:
- identity
- id-scheme
- ulid
- decision
- g-1
component: All
author: Duncan McGreggor
version: '1.0'
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design/04-accepted/0024-id-scheme-retain-ulid.md
  class: odd
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
status:
  accepted:
    reached: 2026-07-27
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-27
  draft:
    reached: 2026-07-27
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-27
  revised:
    reached: 2026-07-27
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-27
  under-review:
    reached: 2026-07-27
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-27
---

# ID scheme — retain ULID identity; register-style handle rejected

> **Drafting note.** Recorded by CDC 2026-07-27 from Duncan's decision, to **close the
> standing G-1 gate** (`arc-release-hardening/workflow-gap-coverage-review.md`). G-1 has
> blocked node minting and resurfaced repeatedly for one reason: only the *open question*
> was ever written down, never a *verdict*. This ODD is that verdict of record.

## 1. Decision

odm **retains ULID** as node identity (`id`). The proposed register-style **`D-YYMM-XXXX`**
scheme is **rejected**. The human-facing **`number`** field is **unchanged** (project = 1000,
arc = 1000 + 100·N, slice = arc + position; design/research keep their legacy ODD numbers).
No existing id is re-minted or re-stamped. **Identity is locked for v1.0.0 and beyond.**

**Direct consequence:** the standing "**do not mint new nodes before the G-1 decision**" freeze
is **LIFTED**. Minting resumes.

## 2. Context — why G-1 existed

G-1 began as a conversational intent (operator, 2026-07-25: *"I like yours better, and I'm
going to switch odm v2 to use it"*) to replace ULID with the register's `D-YYMM-XXXX`. It was
recorded nowhere in the repo — yet identity is the one thing that **cannot change after ship**,
so the gap review froze minting pending an ODD. Because only the open question was recorded and
never a verdict, every new session re-encountered the *freeze* rather than a *decision*, and the
question was re-litigated three separate times. This ODD ends that.

## 3. Rationale

- **ULID is load-bearing as identity.** It is the node filename, it sorts chronologically, and
  its ~80-bit entropy is the collision margin an identity needs. The store, the index, and every
  edge rely on these properties.
- **`D-YYMM-XXXX` is a handle's budget, not an identity's.** Four characters of entropy suits a
  human-facing label, not the thing every edge points at. Its natural slot competes with
  `number`, not with `id`.
- **A switch is maximally costly at the worst time.** Re-scheming identity re-mints every id and
  breaks every edge (`part_of`, `depends_on`, `supersedes`, `decomposed`, `consumes`, `affects`)
  — precisely the failure the freeze exists to prevent — for no capability gain.
- **Stability over churn, pre-ship.** With v1.0.0 imminent, locking the status quo removes a
  standing release-gate at zero cost.

## 4. Alternatives considered

- **Switch `id` to `D-YYMM-XXXX`** — *rejected*: identity churn, edge breakage, entropy too low
  for identity.
- **Adopt `D-YYMM-XXXX` as the `number`/handle (keep ULID `id`)** — *out of scope; not adopted.*
  The current `number` works and nothing in v1.0.0 requires a register-style handle. If a
  register-style *display handle* is ever wanted, it is a post-1.0 presentation concern touching
  `number` only — never `id` — and earns its own ODD. This ODD does not commit to it.
- **Retain ULID (status quo)** — **accepted.**

## 5. Consequences

- The G-1 minting freeze is lifted; `arc-migration-fidelity` and all other minting work may
  proceed.
- No migration, re-stamp, or re-numbering of existing nodes.
- Standing-gate references are updated to point here: `project-plan.md` §3 (pre-ship gates) +
  Version History, `CDC-SESSION-BOOTSTRAP.md` (§0/§0a resume), and the gap review's G-1 row.

## 6. Success criteria

- No session re-opens the id-scheme question without first **superseding this ODD**.
- `check` and minting operate with no id-scheme freeze in force.
