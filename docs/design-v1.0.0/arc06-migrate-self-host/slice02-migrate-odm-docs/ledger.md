# Slice 02 (Arc 06): migrate odm's own docs

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Five-iteration cap. The importer **meets reality** — run on odm's own `docs/design`;
> the plan-set self-host is slice03.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| N-1 | `odm migrate docs/design` imports odm's real ODD corpus → `odd` nodes under `nodes/`; **every** legacy ODD is created or **reported-skipped** (none silently dropped); legacy `docs/design` files **intact** (never-delete) | `cargo test -p odm-migrate migrate_real_docs_all_accounted` → ok (created + skipped == ODD count; a byte-snapshot of the copied corpus is unchanged after) | serious | arc-plan slice02 / 0013 §9 | done | attested — `migrate_real_docs_all_accounted` → ok: created+skipped == `discover().len()` (12), 0 skipped, all type=odd, ODD-0013/0019 present, byte-snapshot of the copy unchanged. **Real import committed on the branch**: `odm migrate docs/design` → 12 `odd` nodes under `nodes/2026/07/`; legacy `docs/design` (14 files) intact. | Run over a snapshot copy of `docs/design` for a deterministic test. Real import is committed on the branch; legacy stays (supersede-not-delete). |
| N-2 | **`odm check` is green on the imported graph** — no orphans / dangling refs / integrity errors on the imported `odd` nodes (or every finding is a reported importer skip/warning, never a silent drop / fabricated edge) | `cargo test -p odm-cli check_green_on_migrated_odm_docs` → ok (`odm check` exit 0 on the imported corpus) | serious | arc-plan slice02 (real acceptance) | done | attested — `check_green_on_migrated_odm_docs` → ok (`odm check` exit 0). Real committed corpus: `odm check` → "ok (12 node(s), no problems)" exit 0. Green **naturally**: document nodes are orphan-exempt (`recompose::check_orphan`), have no edges/affects/deferred → no findings; no fabricated edge, no forced green. | The slice's acceptance bar. If not green: fix the importer bug or report the real graph issue — never force green by inventing edges. |
| N-3 | The **real `supersedes` value shape** is handled (whatever `docs/design` uses — `u32` / `"ODD-00NN"` string / `null` / ref), and the **`odd` numbering space** is documented as **distinct** (no collision with work-node numbers on the real corpus); **multi-supersession** is checked — handled (warn+single-edge) or an `odm-core` amendment raised if a real ODD supersedes >1 | `cargo test -p odm-migrate supersedes_real_shape_resolves` → ok AND the numbering-space note is in the importer docs AND multi-supersede observation recorded (present→decision / absent→noted) | serious | slice01 bubble-up (3 flags) | done | attested — real corpus uses `null` throughout; hardened parser also accepts a bare number and a `"ODD-00NN"` string ref (`de_opt_ref`), tested by `supersedes_real_shape_resolves` (string→ULID) + `de_opt_ref_accepts_null_number_and_string`. Numbering-space note in `odm-migrate` lib docs: key = `(type=odd, number)`, no collision (odd `2`,`9`–`19`). **Multi-supersession: none in the real corpus** → single-edge model kept, **no amendment**. | Settles slice01's flagged decisions against reality. A real multi-supersede that needs >1 edge is a model question → amendment, not a silent single-edge drop. |
| N-4 | The **`deferred` mapping** is settled against the real corpus: if a `07-deferred` ODD exists, its mapping is decided (A5 `deferred` marker vs retire vs distinct gate) + recorded; if none exists, deferred→retire is recorded as the interim (revisitable) | observation recorded in the closing-report + (if a real deferred ODD exists) a test asserting its mapping | correctness | slice01 flag #2 / A5 deferred marker | done | attested — **no `07-deferred` ODD exists** in `docs/design` (states present: Draft/Accepted/Final only). `deferred → retire` stands as the interim mapping (revisitable); recorded in the closing-report + the `odm-migrate` lib docs. No real instance forces the A5 marker → no amendment. | A5 built a first-class `deferred` marker — a real deferred ODD may map there rather than to retire. No real instance forces it; decide against what's actually in `docs/design`. |
| N-5 | **Idempotent + `--dry-run` hold on the real corpus** — a re-run creates 0 (skips all as `AlreadyExists`); `--dry-run` previews the real import and writes nothing | `cargo test -p odm-migrate migrate_real_docs_idempotent_and_dry_run` → ok | serious | slice01 M-3/M-4 / arc-plan | done | attested — `migrate_real_docs_idempotent_and_dry_run` → ok: re-run creates 0 / skips all as `AlreadyExists`, no duplicates; `--dry-run` plans all, writes nothing. Confirmed live: a 2nd `odm migrate docs/design` → "0 created, 12 skipped". | The slice01 guarantees must hold at real-corpus scale, not just on fixtures. |
| N-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) on new `odm-migrate` paths; **no regression** (full workspace green) | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-migrate/src` AND `cargo llvm-cov --summary-only -p odm-migrate` → **line** ≥ 90% AND `cargo test --workspace` green | serious | CLAUDE.md | done | attested — clippy `-D warnings` exit 0; no `unsafe`; line cov: lib.rs 99.56%, mapping.rs 99.05%, legacy.rs 93.86% (all ≥ 90); `cargo test --workspace` green (49 suites). Edge cases fixed in `odm-migrate` + fixtures (`test-data/legacy-refstr`), never in a legacy ODD. | Edge-case fixes land in `odm-migrate` + a fixture — never by hand-editing a legacy ODD. |

## What Worked

- **The importer met reality with zero surprises.** odm's real `docs/design`
  (12 ODDs, numbers `2`/`9`–`19`) imported clean: all progression states
  (Draft/Accepted/Final), all `supersedes`/`superseded-by` = `null`, no dustbin,
  no `07-deferred`. The three slice01 flags settled *against the corpus*, not in
  the abstract — and none forced a model amendment.
- **`odm check` is green by construction, not by force.** Document nodes are
  orphan-exempt (`recompose::check_orphan` requires a parent only for work nodes)
  and non-parent-capable (no decomposition checks); with no edges/affects/deferred,
  the imported `odd` graph produces zero findings. Green fell out of the model —
  no fabricated edge, no suppressed finding.
- **Snapshot-copy tests, live real import.** Every ledger test runs over a
  `TempDir` copy of `docs/design` (deterministic, read-only w.r.t. the live tree),
  while the *actual* import is committed on the branch — the two never interfere,
  and the never-delete guarantee is re-proven at real-corpus scale by byte-snapshot.
- **The `supersedes` shape hardening was cheap and future-proof.** A small
  `de_opt_ref` (null | number | `"ODD-00NN"`) means the human-facing ref string is
  handled now, so a future ODD that uses it resolves instead of loud-skipping —
  fixed in `odm-migrate` + a fixture, never by touching a legacy file.

## Deviations / decisions flagged

1. **The three slice01 flags — all settled without amendment.** (a) **Numbering
   space:** `odd` is a distinct space, key = `(type=odd, number)`; no collision on
   the real corpus. (b) **`supersedes` shape:** real = `null`; parser hardened to
   also accept a bare number and a `"ODD-00NN"` string. (c) **Multi-supersession:**
   none in the corpus → single-edge model kept (warn-on-multiple remains the
   guard). Recorded here + in the importer docs.
2. **`deferred → retire` stands (interim).** No `07-deferred` ODD exists, so the
   A5 first-class `deferred` marker is not exercised; the slice01 interim mapping
   is unchanged and revisitable if a real deferred ODD appears. No amendment.
3. **Real nodes committed on the branch.** The 12 `odd` nodes live under
   `nodes/2026/07/` (self-host begins); legacy `docs/design` stays intact
   (supersede-not-delete). ULIDs are freshly minted, so the *files* are not
   byte-reproducible across a fresh migrate — idempotence (re-run = no-op, keyed on
   `number`) is the reproducibility guarantee, and the deterministic tests use
   snapshot copies. Retiring the legacy files is a slice03/post-cutover call.
4. **oxur's `crates/design/docs` — out of scope** (arc-plan open Q), decided here:
   odm's own docs first; oxur's corpus is a separate later concern, not an A6 slice.
5. **No amendment to ODD-0013 §9 or the arc-plan mapping** was required — the
   real corpus fit the slice01 mapping as designed.

## Closure

Closed at commit `028d21d` on 2026-07-06. Verified by: CC (proposed-done, attested);
CDC to reproduce. Rows: 6. Done: 6. Deferred: 0. No-op: 0.
On close → bubble up to `arc-plan.md` (A-2) per LEDGER-DISCIPLINE v2.0 §A;
slice03 brings the plan-set in and the self-host loop closes.
