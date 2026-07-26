# UAT feedback coverage audit — F / L / G rows

> **Question answered:** is every piece of UAT feedback captured, and where does each live? Prompted
> by the retirement of node #1605 ("UAT — CLI feedback"). **Date:** 2026-07-26 (CDC).
> **Headline: nothing is lost.** All three feedback corpora are present and tracked
> (`uat-punch-list.md` → F-rows; `uat-report-llm-pass-batch2.md` → L-rows;
> `workflow-gap-coverage-review.md` → G-rows). Node #1605 held **no** feedback — it was a tombstone
> pointing at `uat-report-llm-pass-batch2.md` L-2.

## F-rows — Duncan's punch list, triaged in the RH arc-plan (21 rows)

| Row | Chunk | Status |
|-----|-------|--------|
| F-1 (colours/theming) | C-1 | **done** (RH-1) |
| F-2 (`odd`→`design`), F-3 (add `research`) | C-2 | **done** |
| F-4…F-9 (`list`: number col, date, status col, tree, de-number, width) | C-3 | **done** |
| F-15 (retired nodes shown in `list`) | C-3 | **done** |
| F-10…F-13 (`new` warn, `context`→`project`, `path`→`chain`, `rollup` md/json) | **C-4** | open |
| F-14 (`self-host`→`migrate`) | **C-5** | open |
| F-18 (names embed metadata) | **C-5** (was C-7) | open |
| F-19 (normalized status) | **C-8** | open |
| F-20 (work-node dates) | **C-5** | open |
| F-16 (unconditional table ANSI) | upstream `oxur-term` (unassigned) | open — decide |
| F-17 (body-snapshot drift) | migrate/self-host (unassigned) | open — decide |
| F-21 (`--json` omits dates) | **LLM-command-surface arc** | open |

**10 done, 11 routed-and-open** — every one has a home.

## L-rows — pass-2 LLM UAT (9 rows). The arc-llm-command-surface arc was shaped *from* this report.

| Row | Home | Status |
|-----|------|--------|
| L-1 (status is write-only) | LLM arc **slice 01** (= **G-4**) | planned (the blocking gap) |
| L-2 (nothing reconciles plan vs nodes; work with no node) | **F-15** (retired-node half, **done**) + G-1/G-3 (check rules) | partly done |
| L-4 (`next` unordered/untyped), L-5 (`why` blocked-half only) | LLM arc **slice 02** | planned |
| L-7 (`ROLLUP` missing flat form) | LLM arc **slice 03** | planned |
| L-9 (`number` looks positional) | ODD-0013 **§2.1** (number = metadata, no ordering claim) | doc-addressed |
| **L-3 (orient degrades silently on the project)** | **report-only — no chunk found** | ⚠ un-routed |
| **L-6 (distribution: reachability not feasibility)** | **report-only** (adjacent to LLM export, slice 05) | ⚠ un-routed |
| **L-8 (authority drifted out of state dirs)** | **report-only** (adjacent to store-home / self-host) | ⚠ un-routed |

## G-rows — workflow gap review (12 rows). All tracked in the review with routing.

| Row | Route | Status |
|-----|-------|--------|
| G-1 (ID-scheme decision) | its own **ODD** — ⚠ gates minting new nodes | pending decision |
| G-2 (tear-rationale in `check`) | RH **C-6** (reserved) | queued |
| G-3 (`decomposed`/orphan check rule) | a `check` rule | queued (small) |
| G-4 (status read-back) | = L-1, LLM arc slice 01 (pull-forward candidate) | queued |
| G-5 (foreign-project migration + lykn `--dry-run` = UAT pass 3) | **the long pole** | queued (medium-large) |
| G-6 (PM-skill straddler) | A6 slice05 | queued (A6) |
| G-7 (auto-commit for shared repos) | decision + small change — **overlaps arc-store-home sharing** | queued |
| G-8 (evidence-artifact field) | additive field; A7 slice03-adjacent | queued (post-MVP) |
| G-9 (concurrent-session races) | document as a known limit | queued (doc) |
| G-10 (register/`finding` as first-class type) | decide at lykn migration (`note` vs new type) | queued |
| G-11 (cross-repo / saga), G-12 (three-repo lykn) | post-1.0, record only | deferred |

## Actionable finding — the un-routed tail

Nothing is lost, but **four items are captured-but-not-yet-chunked** and want an explicit decision
(chunk it, or accept as known/deferred), rather than sitting in a report indefinitely:

- **L-3** (orient degrades silently on the project that defines the DoD) — this one is *serious* and
  touches the DoD directly; I'd route it, not leave it report-only.
- **L-6** (distribution/reachability) — naturally folds into the LLM arc's export slice.
- **L-8** (authority drifted from the state directories) — resonates with arc-store-home + self-host;
  worth a deliberate call.
- **G-7** (auto-commit for shared repos) — **should be settled inside arc-store-home**, since the
  `odm` branch is the shared DB and auto-commit/push semantics are exactly that arc's concern.

Everything else is either done, routed to a named chunk/arc, or a recorded post-1.0 deferral.
