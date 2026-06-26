# CDC Verification — Arc 03 / Slice 04: `--json` + polish

> Independent verification of CC's closed ledger (impl `60f4d57`; closed `893a34f`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran in
> this session; **attested** = CC's evidence (incl. CC's local 1.95.0 run), not
> independently reproduced here.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0
run (held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc03-slice04-json-and-polish` at `60f4d57`.

## Row dispositions

**Row count:** 9 opened, 9 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `60f4d57`):**

- **J-1 / J-2 / J-4** — `crates/odm-cli/src/json.rs`: `#[derive(Serialize)]` views with
  `From<&Model>` impls (`impl From<&Rollup> for RollupJson`, etc.) — a 1:1 projection
  of the slice02 model, so JSON and Markdown read the *same* `&Rollup` and cannot drift
  in data (D-3). `rollup --json` / `orient --json` / `brief --json` serialize over it. ✔
- **J-3 / J-5** — schema markers `ROLLUP_SCHEMA = "rollup/v1"`, `ORIENT_SCHEMA =
  "orient/v1"`, and `check/v1` (0013 §7.1); additive top-level `schema` key on each
  envelope. The `check` envelope shape-lock now includes `schema`. ✔ *(version form —
  see ruling 1)*
- **J-6** — 0013 gains `### 7.1 JSON output schemas (canonical)` documenting the
  `check`/`rollup`/`orient` envelopes; doc bumped `1.8 → 1.9`; `list`/`show`/`context`
  noted unchanged. ✔
- **J-9 (no `unsafe`)** — grep over odm-core/odm-cli src → no matches. **Hygiene:** the
  diff is clean — no stray `nodes/` dir or `ROLLUP.md` tracked (CC caught + removed a
  smoke-test artifact; confirmed absent from `git ls-files`). ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test --workspace` → 227
passed, 0 failed; clippy `--all-targets -- -D warnings` → exit 0; `fmt --check` clean;
line coverage **odm-core 98.69% / odm-cli 94.40%** (json.rs 100%, orient.rs 98.22%,
rollup.rs 98.82%). → **PENDING CI.**

## Rulings on CC's flagged items

1. **Schema marker `check/v1` (not `check/v2`).** **Ratified (CDC recommendation;
   Duncan to confirm — J-5 was flagged for ratification).** The marker is a *forward*
   contract: it begins counting when there is a marker to pin. The two prior `check`
   envelope evolutions were unmarked and unpinnable — pre-history, not discoverable
   versions — so a uniform `<command>/v1` across check/rollup/orient is the honest,
   consistent choice. "v2" would imply a marked v1 that never existed. The shape-lock
   tests now prevent further silent drift, so the contract is real from v1 forward.
2. **Adding `schema` changed the `check` top-level key set; three shape-lock tests
   updated.** **Accepted** — additive (existing consumers keep their keys), and the
   test updates are intentional re-locks, not drift.
3. **`rollup --json` is non-writing; `orient --json` integrity carries errors only.**
   **Accepted** — `--json` is an output mode (stdout), not a file-write; mirroring the
   human view (which surfaces Errors) keeps the two renders consistent.
4. **`orient --json` many-projects fallback omits the candidate list; `list`/`show`/
   `context` left unmarked.** **Accepted, with a minor follow-up.** The fixed orient
   key set (`project: null` + `hint` across all no-project states) is a clean contract.
   *Minor:* if 0017/automation later needs to *act* on the many-projects state, add a
   `candidates: []` field (empty in other states, preserving the fixed key set) rather
   than a variable shape. `list`/`show`/`context` staying unmarked is consistent with
   scope; a future pass may mark them for uniformity. Not a blocker.

## Verdict

**Arc 03 / Slice 04 CDC-verified on structure; all flags accepted; cargo rows pending
CI.** `rollup`/`orient`/`brief` have a stable, documented, shape-locked `--json`
serialized 1:1 from the model (D-3), and the canonical schemas are pinned in 0013 §7.1 —
the contract ODD-0017 export will target. On CI green, the slice closes.

**This is the last capability slice of Arc 03.** See `../arc-close.md` for the
arc-level recomposition / silent-drop check.

CDC: planning thread, 2026-06-26. Iterations used: 1.
