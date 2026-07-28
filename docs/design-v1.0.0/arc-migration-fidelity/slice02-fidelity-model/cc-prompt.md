# CC Prompt — Slice 02 (Migration Fidelity): The fidelity model (ODD-0025)

Author **ODD-0025**, the single authoritative model the rest of this arc implements against. This
slice writes **one design document** — it mints no nodes, changes no store schema, and writes **no
code**. It only *decides and records*.

> **Seat.** This is a **model / design slice**. The design **decisions are already made** —
> `design-notes.md` §3 (forks F1–F10) plus the three operator-confirmed rulings below. Your job is
> to **author the ODD faithfully** from those decisions, not to make new ones. Where a decision
> feels wrong or underspecified, **raise an amendment — do not work around it, and do not
> improvise a new design call.** (If the operator has assigned this slice to the CDC seat instead,
> the same spec applies to whoever authors it.)
>
> **Start condition:** on `release/1.0.x`. **G-1 is closed** (ODD-0024, ULID retained) — but this
> slice mints nothing regardless. **Do not proceed until the operator has confirmed the three
> open-fork rulings (F4, F9, F10 below);** they change what the ODD records.

## Read first
1. `slice02-fidelity-model/ledger.md` (14 rows) — the spec of "done."
2. `slice-doc.md` (same dir), `../design-notes.md` **§2–§3** (the decisions + the open forks —
   this is the source of truth for the model), and `../arc-plan.md` (the capability's four
   properties + the s02 row).
3. `slice01-coverage-discovery/coverage-report.md` — the exact inventory the model must be
   adequate to repair.
4. **A sibling ODD for format** — `docs/design/04-accepted/0024-id-scheme-retain-ulid.md`
   (frontmatter shape, section conventions). New number = **25**, `state: Draft`, under
   `docs/design/01-draft/`.
5. **`odm-core/src/frontmatter.rs`** — the node field set (the mapping's target side) and
   `Frontmatter.extra` (`#[serde(flatten)]`, where provenance lands additively).

## Load skills (via `/<name>`)
- `/collaboration-framework` → LEDGER-DISCIPLINE (v2.0 — fill evidence at `attested` per commit;
  a model row's evidence is a pointer to the ODD section that records the decision).

## Task — author ODD-0025 recording, unambiguously, each of:
1. **Provenance sub-map** (F5): `source_paths` (list) · `source_class` · `normalization` ·
   `migrated_by` · `migrated_on`; **computed at migration, no stored hashes**; lands via
   `Frontmatter.extra` before formal typing.
2. **No-transform body rule** (F3): body = post-frontmatter (FM docs) / whole file (FM-less); no
   synthesized H1, no header injection.
3. **Body-hash gate** (F2/F5): hard-fail on `sha256(normalize(source)) != sha256(normalize(node))`,
   migration-time only.
4. **Normalization** (**F4 — CONFIRM**): `trim + CRLF→LF`, nothing else; record the rationale and
   `normalization: trim+lf`.
5. **Synthesis model** (F1/F2): `supersedes`→`Vec`; `superseded_by` **derived + tooling-guaranteed
   + checked**; synthesis-type {`concatenation` hash-gated (deterministic join) / `editorial-merge`
   attested / `other`}; keep the two axes vs `SupersedesKind`.
6. **`artifact` node class** (**F9 — CONFIRM**): one supporting-doc class, `part_of` nearest
   **modeled** scale (per-slice→slice; arc/chunk→**arc**); **introduce no "chunk" node scale**.
7. **Report self-coverage** (**F10 — CONFIRM**): authored reports = `artifact` nodes;
   regenerable `coverage-report.md` = a coverage-exemption/ignore rule.
8. **Containment-optional** for design/research doc nodes (F7); orphan check must not flag a
   top-level doc node.
9. **Update-in-place fix vector** (F8): match node→source by coordinate; rewrite body+provenance
   in place; hard-fail gate; preserve id/edges/status; stub = lone-H1.
10. **Frontmatter-fidelity schema mapping** — a versioned table over **originally-present fields
    only**. Cover the legacy ODD field set; `state`→cumulative gate reach (already faithful,
    `mapping.rs::reach_cumulative`); `supersedes`→`edges.supersedes`; `superseded-by`→derived.
    **Resolve the `author` and `version` orphan cells** (no node field today) — pick a target or
    a dropped-with-rationale, and say which.
11. **Naming disambiguation** — one line separating `docs/dev/research/`, `docs/dev/`, and the
    `research` node type.
12. **Amendments-required section** — name, by section, what **ODD-0013** (add `provenance`; add
    the `artifact` node type/class) and **ODD-0020** (schema versions for `provenance` + `artifact`)
    must gain. **Specify only** — do not edit 0013/0020 (that is s03).

## Constraints (flag, don't silently change)
- **No code. No minting. No store-schema change. No edits to 0013/0020** — this slice writes one
  new Draft ODD and nothing else.
- **Amend, don't work around.** If a `design-notes.md` §3 decision is wrong/impossible, or a
  mapping cell has no good answer, raise it — do not invent a new design call.
- **Spec-keeping:** every DECIDED fork must be reflected without contradiction (F-14). Any
  intentional divergence is a tracked `design-notes.md` update, never silent.
- Match the existing ODD format exactly (frontmatter fields, section style) — see 0024.

## Deliverables
`docs/design/01-draft/0025-migration-fidelity-model.md` (Draft); `ledger.md` evidence per row (at
`attested`, each a pointer to the ODD section); `closing-report.md` — the per-row walk **plus the
v2.0 Bubble-up to the arc** (did s02 deliver the model MF-2/MF-3/MF-4/MF-6/MF-7 plan against; what
did authoring reveal the arc-plan didn't anticipate — e.g. a mapping cell that forces a new fork;
the slice-scale silent-drop diff). Feature branch (`arc-migfidelity-slice02-model`), not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is *proposed-done*
(`attested`) → CDC reproduces by independent read. On close, bubble up to `../arc-plan.md` per
LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
