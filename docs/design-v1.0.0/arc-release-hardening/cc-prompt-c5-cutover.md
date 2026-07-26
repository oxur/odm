# cc-prompt — RH C-5 (cutover): the dogfood relocation + derivation overhaul

> **Arc:** Release Hardening (UAT) · **Chunk:** C-5 (the payoff) · **Covers:** `F-14` (fold),
> `F-18` (names), `F-20` (dates), **`L-3a`** (vision backfill), **`SH-6`** (arc-store-home dogfood
> cutover) · **Kind:** model + data + the store relocation. **Builds on** and **completes**
> `cc-prompt-c5-selfhost-derivation.md` — that doc is still the spec for the *derivation* half
> (fold/names/dates); **this doc adds the store cutover, L-3a, and the integrated ordering.** One
> corpus rewrite does all of it, per the arc-store-home coupling (SH-6) and F-20's disposition.

## The one-sentence goal

In a **single corpus rewrite**, move odm's own plan corpus **onto its new store home** (the orphan
`odm` branch in `.worktrees/odm`), regenerating it with **clean names**, **real git dates**, and a
**vision body** — so `odm orient` on odm's own repo, run cold, tells the truth; the working branch no
longer carries `nodes/`; and odm finally dogfoods the store home it just built.

## Why now, and why one pass

arc-store-home is code-complete: `odm store init` (bootstrap/attach/ff-sync) and `odm store rename`
all ship. **SH-6 — odm living in its own store — was deliberately deferred to here**, because C-5
already rewrites the corpus (fold + names + dates), and doing the relocation in the same pass means
the corpus is written **once**, not twice. F-20's disposition says the same: the fix needs an
**in-place re-stamp** (re-running derivation skips existing nodes, keyed on `(type, number)`), so the
46 work nodes must be rewritten explicitly — and that rewrite is the natural moment to relocate them.

## The G-1 gate — cleared for this chunk (important)

CLAUDE.md says *"do not mint new nodes before the G-1 (ID scheme) decision."* **C-5 does not mint.**
The cutover **preserves every ULID** — it *moves and re-stamps existing* node files (names, dates,
vision, and the `nodes/YYYY/MM/` path that the corrected `created` implies), keeping ids, edges
(`part_of` etc.), gates, and numbers exactly as they are. No new node is created; nothing keys on the
unresolved id scheme. So **C-5 is G-1-safe and can proceed now.** (G-1 still gates *future* new nodes —
unchanged.) Confirm this holds in review: if any step would mint a fresh id, stop — that step is out of
scope until G-1.

## Ordering (one reworked path, one rewrite, one verify)

### Phase 0 — model first (no code)
Land **`C-7-amendment-ODD-0013.md`** — §2.1 "names embed no metadata" (v2.2) — first. Dates need **no**
schema change (`created`/`updated` exist; F-20 is a derivation fix). Already drafted; commit it ahead
of the code.

### Phase 1 — the reworked derivation (code)
Per `cc-prompt-c5-selfhost-derivation.md`, in **one** derivation path:
- **Fold (F-14):** `self-host` becomes a case of `migrate` (autodetect plan-set vs legacy, or
  `--plan`); one verb.
- **Names (F-18):** `normalize_name` strips the `"<Type> NN —"` number-prefix and known role-suffixes
  (`(plan-of-record)`, `(build plan)`, …); genuine descriptive parentheticals preserved.
- **Dates (F-20):** `created` = earliest git add-date of the node's plan directory
  (`git log --diff-filter=A --reverse -- <path>`); `updated` = latest commit date; today() only for an
  untracked path (disclosed).
- **Vision (L-3a — new):** the derivation writes a **`# Vision`** section into the **project node body**,
  sourced from `project-plan.md` §1 (Definition of done). This is the L-3 fix: today `orient` prints
  *"no vision text yet — add a `# Vision` section"* because self-host carried the plan's structure but
  not its substance. Also set the **current-focus** context so `orient`'s CURRENT FOCUS block resolves
  (the active arc — e.g. run `odm use arc <RH-or-store-home>` as the cutover's last step, or derive it);
  L-3 showed both halves blank. Vision is the substantive data fix; the focus context is the one-liner
  that finishes it.

### Phase 2 — the cutover (SH-6), on odm's own repo
1. **`odm store init`** → orphan `odm` branch in `.worktrees/odm`, scaffolded `config.toml`, `[store]`
   appended to `odm.toml`, `/.worktrees/` gitignored.
2. **Split the config** (ODD-0022 §4.6): move `odm.toml`'s operational keys (`[gates.*]`,
   `[display]`, `docs_directory`, author) **into** the store's `config.toml` (replacing the scaffolded
   defaults with odm's real settings); strip them from `odm.toml`, leaving it a **locator only**
   (`[store]`). Verify resolution still loads odm's real gate-sets from the store.
3. **Relocate + re-stamp**, id-preserving: move the existing corpus into `.worktrees/odm/nodes/`,
   re-stamped — names, dates, and the vision body — with **ULIDs, edges, gates, numbers unchanged**.
   The corrected `created` moves each file to its real `nodes/<year>/<month>/<ULID>.md`. (Mechanically:
   a git-move of the existing tree into the home + the in-place re-stamp pass — **not** a fresh
   derivation into an empty store, which would mint new ids and break every edge.)
4. **Retire the working-branch `nodes/`**: it has moved to the orphan branch, so the code branch no
   longer carries the corpus. Confirm it's gone from the working tree (and that git history still holds
   the pre-cutover state for recovery).

### Phase 3 — verify, once
See acceptance below. `odm check` green in the new home; `orient` renders vision + focus; names clean;
dates real; `odm.toml` locator-only; `config.toml` carries odm's config; old `nodes/` gone.

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- **Store home:** after the cutover, `.worktrees/odm` is the store; `odm.toml` has **only** `[store]`
  (grep: no `[gates.` / `[display]` / `docs_directory` in `odm.toml`); `config.toml` in the store
  carries odm's real gate-sets/display; `odm check` green at **60 nodes** *in the home*; the repo-root
  `nodes/` is gone.
- **Ids preserved (G-1-safe):** every pre-cutover ULID is present post-cutover; **no new ids**; `part_of`
  and all edges resolve; `odm check` reports the same node/edge counts (60), zero dangling.
- **Names (F-18):** `grep -rE "^name:.*\((plan-of-record|build plan)\)" <store>/nodes` → empty; no
  number-prefix.
- **Dates (F-20):** work-node `created` spans **2026-06-20 → 2026-07-25** (not 45× `2026-07-07`);
  `updated` real; document nodes' legacy dates unchanged (#13 still `2026-06-20`/`2026-06-26`).
- **Vision (L-3a):** the project node body has a `# Vision` section; `odm orient` run **cold** prints a
  vision (not the "no vision text yet" hint) and a resolved CURRENT FOCUS (not "no current arc").
- **Fold (F-14):** the self-host case is reachable through `migrate`; one verb.
- **Single rewrite:** the re-stamp + relocation is one coherent change; `odm check` green; counts
  unchanged.
- ODD-0013 §2.1 at v2.2 landed **before** the code. **SH-6, F-14/F-18/F-20/L-3a dispositioned** in the
  bubble-up (arc-plan RH-5/RH-8 **and** arc-store-home SH-6 + project-plan P-14 — the cutover closes SH-6).

## Decisions to confirm at kickoff

1. **Relocate mechanics** — git-move the existing tree into the home + re-stamp in place (id-preserving,
   recommended) vs. any approach that re-derives into an empty store (rejected: mints ids, breaks edges,
   hits G-1). Confirm the id-preserving path.
2. **Vision source & focus** — vision from `project-plan.md` §1 (recommended); how to set CURRENT FOCUS
   (a cutover-final `odm use arc <active>` vs. deriving it). L-3a's substantive half is the vision body.
3. **Fold shape** — `migrate --plan` explicit vs. autodetect (F-14), and the name strip-list (F-18), per
   the derivation prompt's open decisions.
4. **Config-split ownership** — does `odm store init` grow a `--adopt`/`--split` mode that moves an
   existing `odm.toml`'s operational keys into `config.toml` automatically, or is the split a C-5 step
   done by hand/here? (Recommend: a small, tested split step in C-5, since this is the first real
   adopter; generalize later only if a second repo needs it — YAGNI.)

## Safety (this chunk touches the live corpus + relocates it)

- **Dry-run first.** Prove the reworked derivation + re-stamp on a `--dry-run` (names/dates/vision diff)
  **before** the live relocation commit, so the destructive move happens only once the rewrite is
  verified. The old `nodes/` stays recoverable from git history on the working branch regardless.
- **The diff should touch only** `name:` / `created:` / `updated:` / the vision body and the file
  *locations* (relocation) — **ids, numbers, gates, edges, non-vision body unchanged.** Anything else in
  the diff is a red flag.
- **`odm.toml` split is reversible** in principle (the operational keys move, not vanish); keep them in
  `config.toml`, don't drop any.

## Reflexive validation (RH-7) & the arc-store-home close

- **RH-7** ("re-running the derivation yields a `check`-green corpus with `design`/`research` +
  de-numbered names") is satisfied *in the home* after the cutover. Note the nuance for the record: the
  corpus is **re-stamped in place**, not minted fresh — the reflexive loop validates the *derivation
  logic* via the re-stamp, and a truly-fresh re-derivation (new ids) is **out of scope** pending G-1.
- **This chunk closes SH-6** on the arc-store-home ledger and completes P-14's dogfood row — the
  arc-store-home *true close* (its deferred fresh-context arc-gate) rides this cutover. Bubble up to
  **both** arcs.

## Method / housekeeping

- One branch (e.g. `rh-c5-cutover`, off the latest RH tip / `release/1.0.x` — settle at start); one
  mergeable change; **one** re-stamp+relocation; five-iteration cap. CC implements on local 1.85+ **with
  a real git** (the dates + the cutover shell out via the store's worktree module); cargo rows → CI.
- Arc-plan reconciliation (when the tree is clear): C-5's row/scope grows to include SH-6 + L-3a; the
  standalone C-7 retires into C-5 (already noted); F-14/F-18/F-20/L-3a and SH-6 point here. CDC verifies
  (`cdc-verification.md`) — including an independent re-run of `orient`/`check` **in the relocated home**.
- **Not in this chunk:** C-4 (command renames/reorg + ODD-0023), C-6 (G-2 tear-rationale + the routed
  G-3/L-3b check-hardening), C-8 (F-19 normalized status), F-21 (`--json` dates → LLM arc). Those keep
  their homes.
