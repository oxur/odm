---
id: 01KYP5FV2Y8QCXTN84Y6S1JR6R
number: 527529100
type: artifact
schema: artifact/v1.1
name: Slice 03 CDC verification — Fidelity core
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice03-fidelity-core/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SEHW16DCQZB9ZSBQP
---
# Slice 03 CDC verification — Fidelity core

> **Arc:** Migration Fidelity · **Slice:** 03 · **Feeds:** MF-2, MF-3 · **Implemented by:** CC
> **Commits:** `39eaf66` (impl) + `14173ee` (docs/ledger) on `arc-migfidelity-slice01-coverage`
> **Verified by:** CDC (Opus session) · **Date:** 2026-07-27
> **Verdict: PASS — CDC-verified. 12/12 rows done (F-3 with a criterion CC correctly fixed against
> the Accepted ODD — the error was mine, see below). No iterations required. Three findings, all
> minor/carry-forward — none blocks the close.**

## Method & evidence access

CC committed (working tree clean), so this pass read the actual committed artifacts and
independently reproduced the structural and count rows on the live machine. **Git is unreachable in
the CDC VM** (the `1.0.x` worktree's gitdir points at the macOS path), so the commit SHAs and the
`git status` cleanliness are **attested-by-CC**; I substituted a store-content check for F-10.
Cargo test/clippy/coverage rows are **attested → reproduced-on-CI** (no 1.85+ cargo here).

**Independent reproductions (CDC, on the live machine):**

| Check | Row | Result |
|-------|-----|--------|
| `selfhost.rs` stub synthesis removed | F-4 | **reproduced** — `grep 'format!("# {}'` → no match |
| Live store untouched | F-10 | **reproduced** — 0 live nodes carry a `source:` field; still 60 nodes (unchanged from s01) |
| Fields typed on `Frontmatter` | F-1 | **reproduced** — `source: Option<Source>`, `author`/`version: Option<String>` + builders; `Source` struct present |
| `normalize = trim+lf`, `source` not `provenance`, no stored hash | F-12 | **reproduced** — read `fidelity.rs`: `NORMALIZATION="trim+lf"`; field is `Source`; `verify_body_hash` computes + discards |
| ODD-0013 §2.3 gained the fields; 0020 amended | F-9 | **reproduced** — §2.3 shows `author`/`version` (document-only) + `source` (every migrated node); 0020 v2.1 entry present |
| `de_opt_version` fix | F-8 | **reproduced (read)** — untagged Int/Float/Str; `{f:?}` preserves `1.0`; unit test covers `1.0`/`2.3`/`3`/`"1.0.0"`/absent |

## Per-row disposition

All twelve **done**. Reproduced-by-CDC where marked above (F-1, F-4, F-8, F-9, F-10, F-12);
**attested → CI** for the cargo-dependent rows (F-2 round-trip proptest, F-5 mapping test, F-6 gate
pass/fail, F-11 clippy/coverage — CC reports all green, `fidelity.rs` at 100%, all changed modules
≥ 94.5%). F-7 (`source` populated, both importers) is reproduced structurally via `build_source` +
the fixture tests CC cites; the field's live population is s04. **F-3 — done with a corrected
criterion (see below).**

## The F-3 correction — my error, correctly caught

**CC is right and my ledger was wrong.** My s03 ledger F-3 read "`author`/`version`/`source` valid on
document nodes; a `check` finding on a work node" — bundling all three. But ODD-0025 §2.2 (which I
authored, Accepted) says "**every migrated node** carries a `source` sub-map" (no type restriction),
and a self-hosted arc/slice/project **is** a migrated node — so `source` is valid on work nodes.
Only `author`/`version` are the "document-node fields." My F-3 therefore contradicted my own **F-7**
(which requires `selfhost.rs` to populate `source` on those same work nodes): a work node cannot be
both "source-populated" and "source-invalid." CC hit the contradiction as a failing
`odm-cli::check_green_on_self_hosted_corpus`, re-read the ODD's actual prose rather than my ledger
paraphrase, and corrected `check.rs` to flag only `author`/`version` on work nodes — a fix *toward*
the Accepted spec, needing no ODD change. I verified `check_field_validity`: `source` is in no
work-invalid branch; `author`/`version`/`supersedes` are. **Correct.** The lesson (CC's bubble-up
point 1, which I endorse): encode per-type rules from the ODD's normative text, not a ledger's
compressed restatement. Owned.

## Findings (none blocks the close)

**1 — ODD version-field drift (minor / polish; recommend fixing).** The amendment bumped the
version *histories* but not the docs' own SoT `version:` fields — and this arc is precisely about
version-as-SoT, so it should be clean:
- **ODD-0013** frontmatter `version: 2.3` but carries a `### v2.4` entry → should be `version: "2.4"`.
- **ODD-0020** frontmatter `version: 1.1` and its new entry is numbered **`v2.1`** while the
  sequence was v1.0 → v1.1 → (next is **v1.2**, not v2.1). → renumber the entry `v1.2` and set
  frontmatter `version: "1.2"`.

**2 — the schema-minor bump is on s04's critical path (carry-forward; CC disclosed).** CC recorded
in ODD-0020 v2.1 that actually bumping `SchemaMarker::current()` for the new fields is bigger than a
typing rider (it touches the `check`-level unsupported-newer-schema contract) and **deferred
executing it**. I accept the deferral as disclosed — but flag it is **not deferrable past s04**: the
moment s04 re-migrates the live corpus, it will stamp nodes carrying `source`/`author`/`version`
under an **un-bumped** marker, which *redefines* the current schema version instead of versioning the
change — exactly the drift 0020 exists to prevent. Whichever slice runs the live migration (s04) must
own the bump as its own ledger row, before or with the mint.

**3 — branch deviation (process; for the operator).** This landed on
`arc-migfidelity-slice01-coverage` (the continued session branch), not the named
`arc-migfidelity-slice03-fidelity-core` — CC disclosed it, and notes s01/s02 also landed there. Not
a correctness issue, but it means the arc's slices are accreting on one branch rather than
per-slice branches, which affects how they're reviewed/merged. An operator call on whether to keep
the running-branch pattern or enforce per-slice branches from s04 on.

## On the hash gate (context, not a finding)

CC honestly discloses that today the gate cannot fail live — both importers build `node_body` from
the same read as `source_body`, so `verify_body_hash` is a **regression guard** (it fires only if a
future change reintroduces a transform), not a live-diverging comparison. That is the correct shape
for a no-transform importer and it is disclosed, not hidden. It becomes live-meaningful the instant
anyone reintroduces body synthesis — which is the whole point.

## Bubble-up ratification (MF-2, MF-3)

**CONCUR** — both stay **planned**. s03 built and fixture-proved the *capability* (the gate, the
`source` record, the no-transform import, the typed fields); MF-2 ("zero stub bodies remain") and
MF-3 ("every migrated node carries a `source` sub-map") assert a **live-corpus outcome** that only
s04's re-migration produces. Recording MF-2/MF-3 as pointers to this slice as their design/mechanism
baseline is the correct LEDGER-DISCIPLINE v2.0 §B disposition (composition reproduced at arc scale,
at arc close — never inherited from a fixture-only attestation).

**Disposition:** s03 ledger 12/12 done; capability fixture-proven; MF-2/MF-3 planned. Recommended
arc-plan touch (v1.4): mark s03 closed; carry findings 1–2 forward (the schema-bump onto s04's row,
the ODD version-hygiene fix). **s04 (scope + repair) is next** and is where the capability meets the
live corpus. **Closed.**
