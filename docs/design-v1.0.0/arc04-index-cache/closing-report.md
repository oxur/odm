# Arc 04 — Index & cache — closing report

> The arc-level close for A4, per PROJECT-MANAGEMENT.md Part V + LEDGER-DISCIPLINE
> v2.0 §B. Distinct from the eight per-slice closing-reports one level down. Written
> by CDC (who ran the per-slice verifications); the independent **arc-gate review** is
> recorded at the foot (a fresh-context subagent, per §B's doer ≠ gatekeeper rule).
>
> **Evidence register (disclosed once, applies throughout).** The sandbox has no 1.85+
> toolchain. Two reproduction modes run in this report: **(i) structural reproduction
> by CDC in-session** — the end-to-end integration tests *exist and exercise the
> composed path* across slices (read line-by-line below), the wiring is in place, the
> diff scope is honest, the ODD-0014 promotion is real; **(ii) executable reproduction**
> (actually running cargo over those demos + the benchmark) routes to CI / a local 1.85+
> run and is held **attested pending CI** — exactly as every slice's cargo rows have been
> through the whole arc. Per §B, class-(b) composition is *reproduced at arc scale*
> structurally here; the executable pass is the CI gate. This register is the honest
> floor: "the demonstrations are built and span the slices; the green checkmark is CI's."

## 1. Capability — restated, and the verdict

**Capability (from `arc-plan.md`):** a read-acceleration mini-infra — the `odm-index`
crate — that makes "which files define which nodes?" and metadata filter/sort fast at
scale **without a database, an FTS engine, or a daemon**. A persisted, sorted, derived
**stat-cache** under `.odm/` (gitignored, never truth): the first run pays a full scan +
hash + parse and persists; subsequent runs `lstat`-compare and touch only the delta. It
is **racy-git-correct** (size + conditional content-hash, never stat-only),
**self-healing** (corrupt/missing ⇒ rebuild from the node files), and it feeds
`list` / `orient` / the graph build / `check` / `rollup` so they stop re-walking the tree.

**Verdict: delivered.** All five arc exit criteria are met by composed, end-to-end tests
that span the slices, and the 100k benchmark promoted ODD-0014's index-engine `[P]` claims
to `[E]` with the scaling as the durable claim. The single honest caveat is the evidence
register above (executable reproduction is CI's, not the sandbox's) — no capability gap.

## 2. Slice walk (8 slices — matches the `arc-plan.md` breakdown; no arc-scale silent drop)

| Slice | Scope delivered | Outcome | Pointer |
|-------|-----------------|---------|---------|
| 01 record + persistence | `IndexRecord` + versioned `Snapshot` (magic/format-version/hash-algo/checksum); atomic write (temp+rename+fsync); self-heal load | **delivered** | `slice01-…/cdc-verification.md` |
| 02 cold-path build | walk → lstat+read+hash+parse → records → persist; `meta_hash` semantic field set | **delivered** | `slice02-…/cdc-verification.md` |
| 03 warm-path detection | load+verify → lstat-classify → racy `>=` content-hash fallback → delta → persist-on-change | **delivered** (correctness core) | `slice03-…/cdc-verification.md` |
| 04 enrich + maps + wire `list` | per-gate evidence + number/component; `FORMAT_VERSION 1→2`; in-memory maps; index-backed `list` | **delivered** (seam a) | `slice04-…/cdc-verification.md` |
| 05 index→graph adapter + derived readers | adapter synthesizes frontmatters → unchanged `NodeGraph`/`Satisfaction`; `next`/`blocked`/`path` index-backed | **delivered** (seams b; c→06) | `slice05-…/cdc-verification.md` |
| 06 enrich origin+decomposed + wire check/rollup/orient | `FORMAT_VERSION 2→3`; `aggregate(&[Frontmatter])`; shared `index_frontmatters`; one targeted body load for `orient` | **delivered** (closes A-4+A-5) | `slice06-…/cdc-verification.md` |
| 07 early-cutoff invalidation | `Delta.meta_changed`; `Snapshot::meta_fingerprint` stamps `ROLLUP.md`; body-only skips, meaning-change regenerates; readers stay eager (§2.4) | **delivered** | `slice07-…/cdc-verification.md` |
| 08 benchmark harness | seeded `synth` generator + `harness=false` bench; 1k/10k/100k recorded; `[P]`→`[E]` | **delivered** (capstone) | `slice08-…/cdc-verification.md` |

**Arc-scale silent-drop check:** 8 slices specified, 8 delivered, 8 CDC-verifications on
disk. No slice dropped. The two mid-arc *splits* (slice04→05, slice05→06) were tracked
plan-changes (arc-plan v1.7–v1.12), not drops — each carried its deferred seam forward and
closed it; the arc was then frozen at 8 (grep-verified the remaining field gap was exactly
`origin`+`decomposed`, no third surprise).

## 3. Arc Ledger — per-row walk (the close of the rows opened in `arc-plan.md`)

### Class-(a): slices closed (attested by pointer to closed slice ledgers — §B spot-check)

- **A-1…A-8** — each points to a closed `cdc-verification.md` (slices 01–08). All eight
  present and verified on structure; their cargo/coverage/number rows are attested pending
  CI. Strength: `attested` (children-closed; the §B-correct strength — composition is
  *not* inherited from these, it is reproduced below). ✔

### Class-(b): slices compose — **reproduced at arc scale** (the load-bearing rows)

Per §B these are never inherited from the slices. CDC reproduced each *structurally* by
reading the end-to-end test that spans the slices and confirming it exercises the composed
path (not a slice-local unit); executable re-run routes to CI.

- **A-9 — first run builds+persists; a subsequent run touches only the delta.**
  Reproduced via `odm-index/tests/warm.rs`: `warm_clean_file_skipped_not_reparsed`
  (clean files are not re-parsed), `warm_no_change_no_rewrite` (an unchanged corpus
  re-stamps nothing — the proxy for "no rewrite"), `warm_returns_delta` (a mixed mutation
  yields `{new, changed, deleted, clean}` touching only the changed set). The cold→warm
  composition (build persists; reconcile reads + diffs) is exactly slices 02∘03. The
  benchmark (P-2…P-4) confirms the *cost* claim: warm 7.6× cold at 100k, delta marginal
  cost ≈ 0. ✔ (structural; numbers pending CI)
- **A-10 — change detection is racy-git-correct end-to-end (would fail under stat-only).**
  Reproduced via `warm_racy_same_size_edit_caught`: a same-byte-length in-place edit with
  `mtime` reset so size + mtime_secs *match* the prior (mode unchanged by the in-place
  rewrite) — the cheap signal says CLEAN
  — yet the `mtime_secs >= index_timestamp` content-hash fallback catches it: the test
  asserts `delta.changed == [id]` **and** `delta.clean == 0` with the comment "would be,
  under stat-only." Its complement `warm_racy_unchanged_stays_clean` proves the fallback
  doesn't cry wolf (identical content under a racy stat stays clean), and
  `warm_racy_entries_size_zeroed_on_write` reproduces git's same-size-edit defense. This is
  the arc's correctness core, and the test is *constructed to fail* under the shortcut the
  criterion forbids. ✔ (structural; execution pending CI)
- **A-11 — a missing/corrupt index self-heals; results identical.**
  Reproduced via `snapshot.rs::corrupt_or_version_mismatch_signals_rebuild_through_load`
  + `warm.rs::warm_rebuild_on_load_failure` (a bad/absent snapshot → `RebuildNeeded` → cold
  rebuild on the warm path) and `enrich.rs::v2_index_triggers_rebuild` (an on-disk *older
  format version* self-heals — the FORMAT_VERSION 2→3 path, no migration code). The index
  carries no authority: rebuilt from the node files. ✔ (structural; execution pending CI)
- **A-12 — `list`/`orient`/graph-build/`check`/`rollup` read the index and match the
  full-scan baseline.** Reproduced via the **adapter-fidelity chain ∘ unchanged
  aggregate/assemble/integrity + per-consumer idempotence** (the verify method corrected in
  arc-plan v1.12, because wiring a consumer *removes* its `load_all` path, so a live
  full-scan-vs-index A/B no longer exists). Concretely:
  (1) `odm-index/tests/adapter.rs`: `index_graph_adapter_equals_frontmatter_graph` +
  `adapter_reconstructs_origin_decomposed` prove the synthesized frontmatters equal the
  parsed ones for **every field each consumer reads** (graph, origin, decomposed);
  (2) `odm-cli/tests/index_backed.rs`: `derived_order_…`, `list_…`, `check_…`, `rollup_…`,
  `orient_index_backed_matches_baseline` each assert the warm read == the cold-built read
  end-to-end, and surface the field-driven findings (decomposition-drift, orphan,
  provenance grouping, the vision body via the one targeted load);
  (3) the reconcile-before-read freshness wrappers (`graph_consumers_…`,
  `view_consumers_reconcile_before_read`). **Honest scope note:** equivalence rests on the
  adapter-fidelity tests, not an A/B demo — so any *future* consumer reading a *new* field
  must extend the adapter **and** its fidelity test, or the guarantee silently weakens.
  This standing note is the load-bearing maintenance obligation A-12 leaves behind. ✔
  (structural; execution pending CI)
- **A-13 — the 100k benchmark promotes ODD-0014's `[P]` perf claims to `[E]`.**
  Reproduced via slice08: `benchmark-results.md` records 1k/10k/100k (cold/warm/delta/load/
  size/consumer) + environment; ODD-0014 promoted to `[E]` (single snapshot fine to tens of
  MB / sub-second load = 24 MB / 128 ms; warm avoids re-parse / 7.6× cold at 100k, with the
  `lstat`-sweep-is-O(corpus) nuance carried *inside* the `[E]` claim; eager recompute
  acceptable). **Text-search "linear scan fast enough" deliberately left `[P]`** (not
  measured — promote only what was measured). The seeded generator's determinism is
  reproduced structurally (the two-way same-seed test). ✔ (numbers pending CI)

### Class-(c): bubble-up findings dispositioned

- **A-14** — every slice bubble-up was routed via a dated `arc-plan.md` version-history
  entry (v1.2 slice01 … v1.14 slice08, plus the CDC plan-keeping revs v1.3/v1.10/v1.12).
  The two scope corrections (the consumer-wiring is ~3 slices not 1; the adapter field gap
  = origin+decomposed) were dispositioned as tracked splits, not deferrals. No finding is
  open or undisposed. ✔

## 4. Accumulated arc-plan change log (drift from the original plan, in one place)

The arc shipped at `arc-plan.md` **v1.14**, from v1.0. The substantive drift: the original
"wire all consumers in one slice (slice04)" decomposed into three (04 `list` · 05 graph +
derived readers · 06 composed views) once it became clear **each consumer reads a different
frontmatter projection, so the record grows per consumer** (v1.7/v1.9) — the single most
important arc-level finding, and the one that drove `FORMAT_VERSION` 1→2→3. Two clean
sequential renumbers (v1.8, v1.10) absorbed the splits; v1.10 froze the arc at 8 slices and
flagged that a *third* insert would switch to stable row IDs (the number-as-identity
brittleness odm itself exists to kill — a fitting finding for this project). v1.12 corrected
A-12's verify method once `load_all` removal made the A/B impossible. No finding was lost;
the change log is the audit trail.

## 5. Bubble-up to the project

1. **Did A4 deliver its capability as `project-plan.md` defined it?** **Yes** — "Incremental,
   DB-free/FTS-free stat-cache under `.odm/`; replaces full-scan in `list`/`orient`/
   graph-build; racy-git-correct; self-healing; 100k-node benchmark" (project-plan row A4)
   is delivered, with the consumer set *broader* than the roadmap line named (also `check`
   and `rollup`).
2. **What did A4 reveal that the project plan did not anticipate?** Nothing that re-scopes
   the roadmap. Two findings worth carrying forward, neither forcing a project-plan change:
   (a) **the per-consumer projection growth** means A5 (reconciliation) and any future
   index reader must extend the adapter + its fidelity test when they read a new field — a
   standing maintenance invariant, recorded for A5's planning; (b) **the scaling lever, if
   a corpus ever outgrows the O(corpus) `lstat` sweep, is a watcher / dir-mtime shortcut,
   NOT derived-artifact caching** (the consumer-read benchmark shows caching saves ~nothing)
   — already out of scope per ODD-0014 §2.4 / §5, now backed by measurement.
3. **Silent-drop diff at arc scale, rolled to the project:** none. Everything the roadmap
   expected from A4 landed; the only deferral is the *unmeasured* text-search claim, which
   was never in A4's scope (it's a body-search concern, flagged for its own future
   benchmark).

**Standing arc→project status:** A4 is independently shippable (a performance arc; it
changed no semantics — every consumer matches baseline). It does not block A5/A6. The
project DoD row **P-4 ("A4 closed + composed")** flips to `done` on CI-green reproduction of
the arc's attested rows; structurally it is closed now.

## 6. Cross-scale trending (what recurred across slices — §B)

One pattern recurred and is therefore an arc-level signal, not a slice quirk: **the index
record had to grow each time a new consumer was wired** (evidence for the graph; number/
component for `list`; origin/decomposed for the views). It surfaced as a slice-05 "miss"
but was structural — a consequence of metadata-in/bodies-out (§3.5) meeting per-consumer
projections. It is now closed (the record carries every field its consumers read, frozen at
FORMAT_VERSION 3) and recorded here so A5 plans for it rather than rediscovering it.

## 7. Closure

**Composition verdict: delivered — the slices compose into the index/cache capability.**
Slices: 8 (matches the arc-plan breakdown). Class-(b) composition rows A-9…A-13: reproduced
at arc scale on structure (end-to-end tests that span the slices, read line-by-line);
executable reproduction + the benchmark numbers route to CI / local 1.85+ (attested pending
CI). Findings dispositioned: all (A-14). No arc-scale silent drop. A failed class-(b) row
would spawn a remediation slice (none needed).

Gate reviewed by: **independent fresh-context subagent** (recorded below) — per LEDGER-
DISCIPLINE v2.0 §B, the one who performed the composition does not sign it off.

CDC: planning thread, 2026-06-30.

---

## Independent arc-gate review

**Reviewer:** fresh-context subagent (read-only; no role in the composition). **Verdict:
PASS-WITH-NOTES.**

The gate verified each cited test against the source, not against this report's summary. Its
findings:

- **Slice count (PASS).** 8 in the walk, 8 in the arc-plan breakdown, 8 `cdc-verification.md`
  on disk. No gap.
- **A-10 racy core (PASS).** `warm.rs:166-192` is genuinely a negative-control: same-length
  write + `set_mtime_secs` to the prior, framed racy (`mtime_secs >= index_timestamp`),
  asserting `delta.changed == [id]` **and** `delta.clean == 0` ("would be, under stat-only").
  Constructed to fail under the shortcut the criterion forbids.
- **A-12 (PASS — the one place a tautology *could* hide, and it doesn't).** The CLI
  `*_index_backed_matches_baseline` `first.out == second.out` asserts are idempotence
  (both runs index-backed, routed through `index_frontmatters`/`Derived`→`frontmatters_from_records`,
  never `load_all` — confirmed at `commands.rs:1382-1403`) and **alone prove nothing about
  equivalence-to-full-scan** — but this report *explicitly disowns* them and rests
  equivalence on the adapter-fidelity tests (`adapter.rs::index_graph_adapter_equals_frontmatter_graph`
  field-for-field over every edge kind; `adapter_reconstructs_origin_decomposed` asserting a
  *positive* engineered drift, not empty==empty). Honest account, not theater.
- **A-9 / A-11 / A-13 (PASS).** Cited tests exist and match the claims; ODD-0014's promotion
  is real in the source and correctly leaves text-search `[P]`.
- **Overclaim / CI-caveat (PASS).** The structural-vs-executable register is applied
  consistently; no row claims executable green.
- **Bubble-up (PASS).** Silent-drop diff complete; the delivered consumer set is *broader*
  than the roadmap line (over-delivery, not a drop); the two mid-arc splits are tracked
  plan-changes (arc-plan v1.7–v1.10), CDC owning the proximate miss.

**Gate's corrections, dispositioned:**

1. **A-10 wording** (the report implied a deliberate three-field match; mode is merely
   preserved by the in-place rewrite). **Applied** — softened above to "size + mtime_secs
   match (mode unchanged)."
2. **A-12 standing risk should be a hard gate in A5, not just a footnote here.** *The single
   largest latent risk in the arc:* a future consumer that reads a new field but forgets to
   extend the adapter-fidelity test would silently weaken the equivalence guarantee — and the
   CLI idempotence tests would still pass green (tautologically), hiding the regression.
   **Accepted and carried forward** — recorded as an explicit invariant in `arc05` /
   `arc06`'s open questions (the adapter-fidelity obligation), not left only in this report's
   prose. See §5 bubble-up finding (a).
3. **Pointer elision** (cosmetic — `slice06-…/` vs the on-disk `slice06-views-and-check/`).
   Noted; harmless, no broken link.

The gate's independence did real work: it confirmed the A-12 tautology trap was *named and
avoided* rather than walked into, which is the exact failure mode (inherited / vacuous
composition) §B exists to catch. Arc close stands.
