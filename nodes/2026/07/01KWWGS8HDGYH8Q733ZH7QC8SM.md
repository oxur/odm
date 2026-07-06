---
id: 01KWWGS8HDGYH8Q733ZH7QC8SM
number: 19
type: odd
schema: odd/v1.0
name: Incremental drift — probes as input-tracked rules over the stat-cache, with honest staleness
created: 2026-07-01
updated: 2026-07-01
tags:
- design
- architecture
- reconcile
- drift
- incremental
- stat-cache
- freshness
- honest-staleness
component: odm-reconcile / odm-index
origin: planned
reserved: false
status:
  accepted:
    reached: 2026-07-01
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-01
  draft:
    reached: 2026-07-01
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-01
  revised:
    reached: 2026-07-01
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-01
  under-review:
    reached: 2026-07-01
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-01
author: topological sort
---

# Incremental drift — probes as input-tracked rules over the stat-cache, with honest staleness

> **Status:** Accepted (Duncan + CDC, 2026-07-01). Realized by the reworked A5 tail
> (`arc05-reconciliation/arc-plan.md` v1.8, slices 07–08). This ODD records *why* and *how*;
> the arc-plan carries the slice breakdown.

## 1. Context

Arc 05 (Reconciliation) makes plans carry **checkable claims about the world**: nodes declare
`desired_facts`, probes diff *declared* desired state against *observed reality*, and drift
is reported. The marquee case is ODD-0001 C2 — the prod-DB-503: a plan that *said* the
service was healthy while reality had silently diverged. The whole point of A5 is to **catch
that divergence before it bites**.

Slices 01–04 built the model, both probes (`shell`, `file`), the runner, `odm reconcile`, and
drift in `rollup`/`orient`. But two things surfaced at slice04 close:

1. **Wiring drift into `orient` made bare `odm` run every probe on every invocation.**
   `orient` is the cheap, always-on "where am I" command (A3: bare `odm` orients). Running
   author-declared shell probes on every glance is both a latency cost and a **trust-surface
   change** — the shell probes we scoped as "on demand" would fire on every command.
2. **The original freshness plan — scheduled / manual `reconcile` — is the wrong mechanism.**
   A drift check that only runs on a cron or an explicit command is *laggy*, and lag means
   **continually missed divergences** — the exact failure A5 exists to kill. A stale "all
   clear" is worse than no claim.

The tension: drift must be **fresh on every command** (so nothing is missed), **cheap**
(so `orient` stays instant), and must **not** turn every command into an arbitrary-command
launcher.

## 2. Forces / constraints

- **Freshness on read, not on a clock.** odm already runs everything reconcile-then-read: the
  A4 index is freshened by a cheap `lstat` sweep before each command. Drift should ride the
  same rail, not a separate cadence.
- **Cost proportional to the change, not the corpus.** Re-running *every* probe on every
  command does not scale and re-introduces the `orient` regression. We want to re-process
  **only what changed** (the A4 early-cutoff lesson).
- **Not everything is stat-observable.** A `file` probe's truth is a filesystem fact
  (hashable). A `shell` probe against a prod DB is **not** — the DB going down leaves no
  filesystem trace. A stat-cache alone cannot detect external drift; pretending otherwise
  would manufacture false "all clear"s (the C2 failure, reintroduced).
- **Honesty over false freshness.** Where we cannot cheaply know, we say so ("last checked
  Xm ago") rather than imply fresh.
- **Layering.** `odm-core` (domain model) must not depend on `odm-reconcile`; `odm-reconcile`
  depends on `odm-core` and `odm-index`. Drift *shape* lives in `odm-core`; drift *compute*
  lives above.
- **No auto-spawn footgun.** Bare `odm` must never execute arbitrary author-declared commands
  as a side effect of merely orienting.

## 3. Decision

**Probes are input-tracked rules, reconciled incrementally over the A4 stat-cache, with two
freshness classes and honest staleness for the class the filesystem cannot observe.**

### 3.1 Two classes of fact

- **Input-derived** — a `file` probe, or a `shell` probe that is a **pure function of
  declared input files/globs** (`inputs:`). Its truth changes only when an input changes.
- **Environment / volatile** — external state (a service, the network) with **no filesystem
  signal**. Cannot be made fresh by stat. A `shell` probe with **no declared `inputs`** is
  **volatile by default** (conservative: honest staleness, never false-fresh).

### 3.2 Incremental reconcile over the stat-cache

Before every command, a cheap incremental drift pass runs (reconcile-then-read, as the index
already does):

1. For **input-derived** facts: fingerprint the declared inputs using the **A4 warm-path,
   racy-correct content-hash** (slice03) — `lstat` first, content-hash only on the racy `>=`
   window. If no input changed since the last snapshot, the **cached outcome stands** (early
   cutoff). If an input changed, **re-probe** and update the snapshot. → always fresh on every
   command, cost proportional to the change, near-zero when nothing moved.
2. For **volatile** facts: **do not run** them on a bare command. Carry the last outcome plus
   a **"last checked" timestamp** in the snapshot; they refresh only on explicit
   `odm reconcile` (or a declared cheap trigger — see §6).

### 3.3 Persisted drift snapshot

Drift state lives in a persisted **drift snapshot under `.odm/`** (a sibling of the index),
governed by the same versioned-header + atomic-write discipline. Per fact it records: the
last `ProbeOutcome`, the input fingerprint (input-derived) or the last-checked timestamp
(volatile). This is a **derived, time-varying** artifact — it is *not* the index (the index
mirrors file state; drift mirrors probe *results*), and drift is **never** folded into
`IndexRecord` (see §7, Alternatives).

### 3.4 Honest rendering

`orient`/`rollup` render **input-derived drift as fresh** and **volatile drift as "last
checked Xm ago"**. Clean is an honest "no drift", never fabricated. `orient` (bare `odm`)
runs **zero** volatile probes — the regression is dissolved.

### 3.5 Probe-model extension (additive)

`desired_facts` gains optional `inputs` (paths/globs → input-derived) and/or a `volatile`
marker (→ environment). Additive to the slice01 model — the `ProbeSpec` is internally tagged
and *not* `#[non_exhaustive]`, so new fields/kinds surface as compile errors at match sites
rather than silent wildcards. Absent `inputs` on a `shell` probe ⇒ volatile.

```yaml
# input-derived: fresh on every command, near-zero cost
- id: rollup-current
  describe: "ROLLUP.md matches the committed corpus"
  probe: { kind: file, path: ROLLUP.md, expect: { sha256: … } }

- id: schema-applied
  describe: "the migration produced the expected schema file"
  probe:
    kind: shell
    run: "sha256sum build/schema.sql"
    inputs: ["build/schema.sql"]      # ← makes it input-derived
    expect: { exit: 0, stdout_contains: "…" }

# volatile: no fs signal → honest staleness, explicit refresh only
- id: db-up
  describe: "the prod DB answers a trivial query"
  probe: { kind: shell, run: "pg_isready -h prod -t 2", expect: { exit: 0 } }
  # no `inputs:` ⇒ volatile ⇒ "last checked Xm ago", never auto-run by bare `odm`
```

## 4. Lineage (why this is recovery, not invention)

This is ODD-0014's caching research applied to probes:

- **Build-systems-à-la-carte** (verifying vs constructive traces): a *verifying trace* stores
  input hashes and skips work when they match — exactly the input-derived cutoff here.
- **Salsa backdating / Ninja `restat`**: re-running only when inputs actually changed; an
  unchanged result stops propagation. The drift snapshot is the memoized result.
- **Buck2 dep-files**: rules declaring the inputs they actually consumed → precise, minimal
  re-execution. `inputs:` is the same idea for probes.
- **git racy-git correctness** (ODD-0014 §): the input fingerprint must use the size +
  conditional content-hash discipline (never stat-only), which A4/slice03 already ships —
  drift reuses it rather than re-deriving it.

## 5. Consequences

- **Fresh on every command, cheap when nothing changed** — the freshness the whole arc
  needed, without a clock and without lag.
- **The `orient` regression is dissolved** — bare `odm` runs zero volatile probes and only
  re-hashes changed inputs.
- **Honest about what it cannot know** — volatile facts carry visible staleness instead of a
  manufactured "all clear"; the C2 failure mode is *named*, not hidden.
- **A5 composes onto A4** — the two arcs share the stat-cache spine; the index and the drift
  snapshot are siblings on the same reconcile-then-read discipline. This is the design paying
  the A4 investment forward.
- **The trust surface shrinks** — arbitrary command execution is confined to explicit
  `reconcile` (or a declared trigger), never a passive side effect of orienting.

## 6. Open questions / boundaries (settle in the slices)

- **Declared-input ergonomics.** Globs vs explicit paths; relative-to-repo-root (as the
  `file` probe already is). Keep it minimal in slice07.
- **The "declared cheap trigger" for volatile refresh.** Optional: allow a volatile probe to
  name a *cheap* liveness input (e.g. a heartbeat file a sidecar writes) that promotes it to
  input-derived. Deferred unless a real need appears — honest staleness is the floor.
- **Interaction with the `ROLLUP.md` early-cutoff** (slice04 finding 2). The cutoff keys on
  the corpus meta-fingerprint, which does not cover reality; the drift snapshot's own
  freshness dissolves this — `rollup` reads snapshot drift and no longer runs probes just to
  skip a write. Settle the exact seam in slice08.
- **Staleness display granularity** (exact/`Xm ago`/`>1h`) — a rendering detail for slice08.

## 7. Alternatives considered

- **Scheduled / manual `reconcile` (the original A5 plan).** Rejected: laggy → continually
  misses divergences (the failure A5 exists to kill). Demoted to an optional off-command
  refresh at most.
- **Always-run every probe on every command.** Rejected: re-introduces the `orient`
  regression (latency + auto-spawn trust surface). Retained only, at the user's option, for
  the volatile minority — and even there the default is honest staleness, not always-run
  (Duncan's call, 2026-07-01).
- **Cache drift *in the index* (`IndexRecord`).** Rejected: drift is a **derived,
  time-varying** fact; the index mirrors **file state**, not probe results. Embedding drift
  would bloat every snapshot, desync on every external change the index can't see, and trip
  the A4 adapter-fidelity invariant. The drift snapshot is a *separate* `.odm/` artifact.
- **TTL on volatile probes** (re-run if older than N). Considered; a reasonable middle path,
  but Duncan chose **honest staleness** (explicit refresh + visible timestamp) over
  auto-running-within-a-window, keeping bare `odm` side-effect-free. TTL remains a possible
  future opt-in per fact, not the default.

## 8. Impact on the plan

- **A5 arc-plan v1.8**: `## Freshness model` section added; `reconcile --schedule` demoted;
  reworked tail — **slice07** (probes-as-rules: `inputs`/`volatile` + persisted `.odm/` drift
  snapshot + incremental runner) and **slice08** (freshness-on-every-command + honest-staleness
  rendering), superseding the old scheduled-reconcile slice07.
- **Probe model** (slice01) gains the additive `inputs`/`volatile` extension in slice07.
- **`odm-reconcile`** gains the incremental snapshot mode; **`odm-index`** contributes its
  racy-correct input-fingerprint machinery (reuse, no new index fields).

## 9. References

- ODD-0001 §C2 (the prod-DB-503 — the drift the arc exists to catch), §C5 (stale-doc).
- ODD-0013 §5.2 (desired-state facts + probes).
- ODD-0014 (incremental indexing & caching — the stat-cache, racy-correctness, early-cutoff,
  verifying traces; the lineage in §4).
- `docs/design-v1.0.0/arc05-reconciliation/arc-plan.md` v1.8 (the slice breakdown).
- `docs/design-v1.0.0/arc05-reconciliation/slice04-drift-in-rollup-orient/cdc-verification.md`
  (the surfacing finding).
