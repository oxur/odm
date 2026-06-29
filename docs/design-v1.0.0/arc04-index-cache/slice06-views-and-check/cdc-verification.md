# CDC Verification — Arc 04 / Slice 06: enrich `origin`+`decomposed` + wire `check`/`rollup`/`orient`

> Independent verification of CC's closed ledger (impl + close on
> `arc04-slice06-views-and-check`), per LEDGER-DISCIPLINE v2.0 (slice scale, §A). This
> finishes Arc 04's consumer-wiring (slice05's deferred G-3/G-4/G-5). CDC reproduces
> structural rows here; cargo rows route to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows on the branch.

## Row dispositions

**Row count:** 8 opened, 8 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **V-1** — `IndexRecord.origin: Origin` + `decomposed: Option<Decomposition>`
  (`record.rs:79/97`); `FORMAT_VERSION = 3` (`snapshot.rs:41`); v2 index → version
  mismatch → cold rebuild (slice01 self-heal). **`Decomposition` is an index-owned
  mirror** (`record.rs:112`), *not* odm_core's — because the core type's `children`
  carries `skip_serializing_if`, which would desync postcard. That is the slice02
  `EdgeRef` lesson applied **prophylactically** — a genuinely good catch. ✔
- **V-3** — `adapter.rs` reconstructs `origin` (from the record) and re-affirms
  `decomposed` (`affirm_decomposed(children, on)`), so a synthesized `Frontmatter` is
  faithful for `Rollup::assemble` provenance + `recompose::integrity`. ✔
- **V-4** — `aggregate` takes `&[Frontmatter]` (`commands.rs:1076`); the shared
  `index_frontmatters(store, gates)` (`:1382`, reconcile→adapter) feeds `check`
  (`:1240`), `rollup`, `orient`, and folds in `Derived`. ✔
- **V-6** — `orient` resolves the project off the index then does one targeted
  `store.load` for the vision body (`orient.rs:59` + `extract_vision(project.body())`). ✔
- **V-8 (no `unsafe`)** — grep empty. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** clippy `-D warnings` → exit 0;
`fmt` clean; line coverage **odm-index 97.49% / odm-cli 93.63%**; full workspace green.
→ **PENDING CI**.

## Rulings on CC's flagged items

1. **"Identical to baseline" is structural, not a literal byte-diff** (the `load_all`
   path is removed once a consumer is wired, so there's no live A/B to diff).
   **Accepted — and it is the strongest evidence the architecture allows.** The
   equivalence chain is sound: the adapter-fidelity tests prove the *synthesized*
   frontmatter equals the *parsed* one for exactly the fields each consumer reads
   (graph/satisfaction: slice05 G-1; provenance + recomposition: V-3), and both the
   index path and the old path call the **same unchanged** `aggregate`/`Rollup::assemble`/
   `recompose::integrity` — so the only variable is the (proven-equal) frontmatter
   source. The CLI tests then assert correctness (origin/decomposed-driven findings
   surface — drift, provenance grouping) + idempotence (warm == cold-built). A live A/B
   diff would be tautological given the fidelity tests. Consistent with how slice04/05
   established "== baseline."
   - **Standing consequence (recorded):** with no A/B safety net, the **adapter-fidelity
     tests are now the load-bearing guarantee**. Any *future* consumer that reads a new
     record/frontmatter field must extend the adapter **and** its fidelity test — the
     same projection-per-consumer discipline this arc surfaced.
   - **Plan-keeping fix applied:** the arc-ledger compose row **A-12** ("consumers read
     the index and match the full-scan baseline") had a Verify of *"index-backed vs.
     forced full-scan"* — now impossible (no full-scan path). Updated A-12's Verify to
     the real reproduction: the adapter-fidelity chain + per-consumer correctness +
     idempotence (arc-plan v1.12).
2. **slice04's I-2 `FORMAT_VERSION == 2` grep is now stale** (this slice bumped to 3; CC
   moved the version-guard to a `v2_index_triggers_rebuild` test). **Noted — disclosed,
   expected evolution, not a defect.** slice04's ledger is a point-in-time record (true
   at `2dafaa1`); rebuild-on-version-mismatch is continuously tested across the v1→ and
   v2→ rebuild tests.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — the two-field enrichment + all three remaining consumers,
  exactly the bounded gap (no third field appeared — the grep-verified prediction held).
- **Silent-drop diff honest?** ✔ — 8/8; the structural-equivalence caveat is flagged, not
  buried.
- **Findings + arc-plan?** ✔ — arc-plan **v1.11** marks A-6 attested and **A-4 + A-5
  close** (their deferred seams delivered). On CI green, A-4/A-5/A-6 → `done`.

## Verdict

**Arc 04 / Slice 06 CDC-verified on structure; both flags ruled; cargo rows pending CI.**
Arc 04's **consumer-wiring is complete** — `list`, the derived-order queries, `check`,
`rollup`, and `orient` all read the index (reconcile-then-read), with equivalence proven
through the adapter-fidelity chain. On CI green, **A-4, A-5, A-6 all close** and the
compose row A-12 is satisfiable. Remaining: **slice07 (early-cutoff)** and **slice08 (the
100k benchmark)** — then the arc closes.

CDC: planning thread, 2026-06-29. Iterations used: 1.
