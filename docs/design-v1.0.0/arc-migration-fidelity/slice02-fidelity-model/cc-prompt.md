# CC Prompt — Slice 02 (Migration Fidelity): The fidelity model — STAND DOWN (superseded)

> **Status 2026-07-27 — do not act on this file.** Slice 02 (the model) was **authored in the
> CDC seat** (operator decision) and its model is **decided and Accepted**. This prompt's original
> assignment — for CC to author the ODD and confirm the open forks — is **superseded**. The forks
> you were about to ask about are all resolved (below). Await the **s03** prompt; do not start from
> here. *(Prior assignment is in git history — supersede-don't-delete.)*

## The deliverable already exists
**ODD-0025 — Migration Fidelity** is **Accepted** at
`docs/design/04-accepted/0025-migration-fidelity-model.md`. It is the authoritative model the
implementation slices build against. **You are not authoring or verifying it.**

## The forks you were about to ask about — all decided
- **F4 normalization:** `trim` leading/trailing whitespace **+ CRLF→LF**, nothing else.
- **F9 artifact class:** a new **`artifact`** document node type (American spelling); `part_of`
  its nearest *modeled* scale (per-slice → slice; arc/chunk → arc); **no `chunk` node scale**.
- **F10 report coverage:** mint an `artifact` node for **every** report, `coverage-report.md`
  included — **no exemption**.
- **`author` / `version`:** preserved as **new typed document-node fields** (not dropped, not
  git-derived — git returns the migrator, not the source author).
- **The stored migration record is named `source`** (not `provenance` — 0013 reserves
  `provenance` for *derived* lineage). See ODD-0025 **§2.0** for the origin / source / provenance
  split.

## Your next assignment: s03 — migration-fidelity core
The first implementation slice: 1:1 verbatim body import + **hard body-hash gate** + `source`
persistence + **no-transform** (kill the stub synthesis), across both importers (`mapping.rs`,
`selfhost.rs`), typing `source`/`author`/`version` at creation. Its open set
(`slice03-<slug>/{slice-doc,ledger,cc-prompt}.md`) is being drawn against ODD-0025. **Wait for
that prompt.**
