# Slice 05 — CDC verification

**Method:** LEDGER-DISCIPLINE v2.0 §A. **Verdict: PASS — with notes.** Structural
rows reproduced by direct read of the working tree + the live store; cargo/`check`
rows attested-by-CC → CI (no Rust 1.85+/macOS binary in the CDC sandbox). Verified
2026-08-02.

## Verdict

**PASS — CDC-verified, with two disclosed notes and one separate operational
finding.** The consistency fix (a) is real and clears the MF error; the seam (F-5)
is correctly implemented. The auto-recompose (b) is correct-by-design but **inert on
real data** (disclosed, not a defect). The **store-worktree state** is a serious
finding surfaced during verification — *not* a slice-05 defect, but it must be
resolved before more commits/migrates.

## Ledger walk (F-1…F-6)

- **F-1 — reproduced.** `odm_core::recompose::decomposition_children` (recompose.rs:218)
  is the single work-typed definition (`filter(is_work)`). Both callers use it:
  `check_decomposition` (recompose.rs:245) and `commands::decomposed`
  (commands.rs:1242, with the intent comment at 1234). The divergence is closed at
  the root. Unit test `decomposition_children_is_the_single_work_typed_definition`
  (odm-core/tests/recompose.rs:413) asserts it. Strength: **reproduced** (code read).
- **F-2 — reproduced (acceptance anchor).** MF `#58837400`'s `decomposed.children`
  now lists **exactly the 16 work slices** (s01–s16 ULIDs), with the two non-work
  children — `536513400` (`01KYP5FSQ6…`, provenance/synthesis) and `560811200`
  (`01KYZTRECS…`, closing-report) — **removed**. Affirmed(16) = `check`'s
  work-children(16) → **0 drift** by direct arithmetic. I cannot run `odm check`
  (macOS binary), but the drift is now structurally zero. Strength: **reproduced**
  (store read). **Caveat:** the affirmation is uncommitted (see the store finding).
- **F-3 — attested → CI.** Tests `decomposed_drift_guard_ignores_document_family_children`
  (recompose.rs:223) and `decomposed_with_no_children_affirms_only_work_typed_children`
  (cli.rs:1497) cover the artifact-child case at both layers. Verified by test name +
  presence; execution attested by CC (`make check` green) → CI.
- **F-4 — attested (fixture); real-data behavior reproduced as INERT.**
  `odm_migrate::decompose::auto_recompose` is implemented and wired into `migrate --all`,
  with the correct `ReAffirmed`/`LeftAsDrift` outcomes (decompose.rs). **But it is
  inert on real data:** it re-affirms only when an id-remap the run performed maps a
  removed affirmed child to an added child, and today's `self_host`/`reconcile_source`
  **never reassign ids** — CC disclosed this, and the live store confirms it: the
  `2026/08/` churn deletes and re-creates nodes under the **same ULIDs**
  (`01KYZTR…` deleted == `01KYZTR…` present), never new ones. So `id_remap` is empty
  and `auto_recompose` produces no `ReAffirmed` outcomes in practice. Fixture-attested
  → CI; real-data inertness **reproduced**. See Finding SS5-1.
- **F-5 — attested → CI, design-reproduced.** The seam holds: test
  `migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed`
  (cli/tests/migrate.rs:501), and `decompose.rs`'s module doc states it explicitly —
  *"must never silence [DecompositionDrift] … `decomposed: complete` is a human
  completeness judgment; only the mechanical child-set half auto-heals."* The design
  matches the slice-doc's seam requirement exactly. Execution attested → CI.
- **F-6 — attested → CI.** `make check` green per CC; not reproducible in the CDC
  sandbox (no toolchain).

**Rows: 6. done (reproduced/attested): 6.** No silent drops.

## Findings

### SS5-1 — auto-recompose is correct but inert; the operator's real friction is on the other side of the seam (elevate to slice 01 / ODD-0026)

CC's bubble-up, confirmed. `auto_recompose` targets *identity churn* (same logical
child, re-minted id). The live store proves migrate **preserves ids** — so that case
does not occur, and (b) never fires. Meanwhile the friction the operator actually
feels ("don't make me re-affirm after migrate") is the **genuine-addition** case
(migrate mints a *new* slice under an affirmed parent), which the seam **deliberately
leaves as drift** requiring a human affirm. So slice 05 fixed the real bug (a) but its
(b) does not reduce operator friction as hoped.

**The design question for slice 01 (ODD-0026):** under a plan-tree/store-as-source
model, *authoring a slice under an arc is itself the scope decision.* If so, migrate
(or authoring) reflecting that child is not an unplanned addition needing re-judgment
— the completeness call was made at authoring time. That would let genuine additions
auto-extend the affirmation **without** crossing the spec-keeping seam, because the
human already decided scope when they wrote the child. Recommend deciding this in
ODD-0026 rather than expanding `auto_recompose` speculatively. **Severity: design /
non-blocking.**

### SS5-2 — the odm store worktree is in a large uncommitted, inconsistent state (SERIOUS; predates slice 05; blocks clean progress)

Surfaced by CC and reproduced. `.worktrees/odm` (branch `odm`, last commit
**`6225d1f`**) carries, uncommitted:

- `config.toml` modified;
- **9 tracked node files modified** — incl. the project `#1000`
  (`01KWXMBBTJ…`), `#1001` (`01KYSX4RGC…`), MF `#58837400`, the LLM-arc node
  (`01KYNDTQ6S1WWVD…`), and several `01KYP5H…`;
- the **entire `nodes/2026/08/` in a broken index state**: 13 files staged as
  **deleted** while **still present in the working tree as untracked** (same ULIDs
  on both sides — a `git rm --cached`-style index/worktree split, not a real
  deletion), *plus* this session's newly-minted nodes (slice16 `01KZ0FQ6JKZT…`,
  the Store-Lifecycle arc/slice, and 8 artifacts `01KZ0FQ7…`) sitting untracked.

Nothing is lost (all files exist), but the git state is internally inconsistent —
staged deletions of files that are present — and it is the accumulated,
never-committed residue of this session's `migrate --all` runs (and possibly
earlier). This is exactly the *invisible drift in the source of truth* odm exists to
prevent, now sitting in odm's own store. **CC's MF re-affirmation (F-2) and the
whole slice-05 code change are uncommitted on top of it.**

**Recommendation (operator):** before any further commit or migrate on the `odm`
branch, deliberately reconcile it — on `odm`, `git add -A`, confirm `odm check` is
green, and commit a single clean state; *or* reset to `6225d1f` and re-run one clean
`migrate --all` to regenerate. Do **not** let it accumulate or get swept into an
unrelated commit. **Severity: serious. Not a slice-05 defect** — it predates and
spans it; flagged here because verification cannot honestly bless the store while it
is in this state.

## Evidence-strength summary

- Reproduced (direct read): F-1, F-2, F-4-inertness, SS5-2.
- Attested → CI (no toolchain): F-3, F-5 execution, F-6, F-4 fixture.
- The slice's substance (a + the seam) is sound. Commit + CI flips the attested rows
  to reproduced/reconciled.

## Arc-plan disposition

Flip slice 05 → **CDC-verified PASS (with notes)**. Carry **SS5-1** to slice 01
(ODD-0026 model decision) and **SS5-2** to the operator as a store-reconcile action
before proceeding. Arc ledger row **SS-8**: (a) satisfied (MF clears); (b) delivered
but inert — record as such, don't claim active.
