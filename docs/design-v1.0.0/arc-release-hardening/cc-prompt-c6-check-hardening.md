# cc-prompt — RH C-6: `validate` / `check` hardening (G-2 + G-3 + L-3b)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-6 (the last RH chunk before compose) · **Covers:**
> **G-2** (tear-rationale persist + surface), **G-3** (decomposed/orphan rule), **L-3b** (no-vision
> finding) · **Kind:** model (one schema field) + static check rules. **One pass** over the graph
> checker so it churns once (the routing ratification). **Post-C-4 taxonomy:** all three are *static*
> (no probes) → they land in **`validate`**; `check` inherits them (it runs `validate` then reconcile).

## Why one pass, and where the rules live

C-4 split the checker: `validate` = the pure static pass (schema, links, cycles, recomposition,
order), `check` = `validate` then `reconcile`. G-2/G-3/L-3b are all **static** — they need only the
graph + node bodies, no reality probes — so they are **`validate` rules**, and `check` gets them for
free. Doing all three in one pass avoids three separate walks of the graph.

## Phase 0 — model first (G-2's schema half)

`node tear <X> depends_on <Y> --because <WHY>` **validates** the rationale today but the schema
**drops it** — there is no field, so the reason is lost the moment it's given (the G-2 finding: a
data-loss bug). **Amend ODD-0013's tear schema** to carry the rationale, backward-compatibly:

- Add a `rationale` (or `because`) string to each tear entry in the `tears` vector. **Absent = a legacy
  tear with no recorded reason** (the status quo; not an error, but see the warn rule below).
- Land the amendment (an ODD-0013 note / version bump) **before** the code, per the model-first rule
  C-2/C-5 followed.

## Phase 1 — the code

### G-2 — persist + surface the tear rationale
- **Persist:** `node tear … --because <WHY>` writes `<WHY>` into the tear entry (odm-store schema +
  the writer). Round-trips through save/load.
- **Surface:** `validate` (and thus `check`) lists each tear **with its rationale** — an assumed
  dependency is a deliberate, reviewable decision, so its reason belongs in the integrity output. A
  tear **without** a recorded rationale (legacy) → **warn** ("assumed dependency `X → Y` has no
  recorded reason; re-run `odm node tear … --because`"). `--strict` promotes.

### G-3 — decomposed / orphan rule (static)
- A **parent with children but no `decomposed` assertion** → **warn** (`--strict` errors): the parent's
  scope may not be fully accounted for (ODD-0013 §4.5).
- A **`decomposed` assertion whose child set disagrees with reverse-`part_of`** (asserts children that
  aren't its `part_of` children, or omits some) → **warn**/`--strict` error, naming the mismatch.
- **Heads-up (expected, not a defect):** odm's **own** self-hosted corpus has **no** `decomposed`
  assertions (the C-5 self-host didn't mint them), so enabling this rule will make `validate`/`check`
  on odm's corpus emit **warns** where they were clean. That is **correct and intended** — the rule is
  surfacing a real gap. `check` still exits 0 (warns don't fail without `--strict`). Adding the
  assertions (via `odm node decomposed`, or a future self-host enhancement) is **follow-up**, not a
  C-6 blocker — record it. *(If the warn volume is unwelcome pre-1.0, the fallback is `--strict`-only
  for this rule; recommend warn-by-default — the gap should be visible.)*

### L-3b — no-vision finding (static)
- A **`project` node with no `# Vision` section in its body** → **finding** (warn; the DoD depends on a
  stated vision — this is the enforcement half of L-3a's data fix, so it can't silently regress).
  odm's own project **has** a vision (L-3a), so this passes on the corpus — the rule guards against
  future loss.

## Scope boundary (out)

- **No reconcile/probe changes** — these are static `validate` rules only.
- **Not** minting `decomposed` assertions on odm's corpus (that's follow-up / self-host work).
- **Not** the `blocked`/readiness or satisfaction-verdict analysis (LLM arc slice 02).
- No node-model change beyond the one tear-rationale field.

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- **G-2:** `node tear X depends_on Y --because "cycle break: …"` → the rationale round-trips (read it
  back from the node file **and** `node show`/`--json`); `validate` lists tears with reasons; a
  rationale-less tear warns.
- **G-3:** a fixture parent with children but no `decomposed` → warn; `--strict` → error; a mismatched
  `decomposed` → warn. On odm's corpus, the new warns are the parents lacking assertions (expected;
  documented).
- **L-3b:** a fixture project with no `# Vision` → finding; odm's own project (has one) → clean.
- **Taxonomy:** the three rules fire under `validate` (verify `odm validate` shows them) and therefore
  under `check`; `reconcile` is untouched.
- Unit-test each rule directly (parent-without-decomposed, decomposed-mismatch, no-vision,
  tear-with/without-rationale) over fixtures; one integration pass on the self-hosted store.
- ODD-0013 tear-schema amendment landed **before** the code. **G-2/G-3/L-3b dispositioned** in the
  bubble-up.

## Decisions to confirm at kickoff

1. **Field name** — `rationale` vs `because` on the tear entry (recommend `rationale`; `--because` is
   the *flag*, `rationale` the *stored field*).
2. **G-3 default severity** — warn-by-default (recommended, so odm's own gap is visible) vs
   `--strict`-only.
3. **Tear-rationale-less** — warn (recommended) vs silent-accept for legacy tears.

## Method / housekeeping

- One branch (e.g. `rh-c6-check-hardening`, off the C-8 tip on `release/1.0.x`); one mergeable diff;
  five-iteration cap. CC on local 1.85+; cargo rows → CI. Reflexive check on the self-hosted store:
  `odm validate` and `odm check` behave, and the new decomposed-warns are exactly the parents without
  assertions (not spurious).
- **Ledger:** append a **new chunk-closed row** for C-6 (per the arc ledger's stable-id rule — the
  class-(a) rows currently stop at RH-5/C-5; C-8 and C-6 each want their own appended row, e.g.
  RH-9/RH-10 — reconcile the numbering at close so the ledger's class-(a) set matches the Chunks
  table). CDC verifies (`cdc-verification.md`) on the real store.
- **After C-6:** RH has no chunks left → **RH-6/RH-7 compose** (the self-hosted CLI is coherent +
  themed + UAT-validated end-to-end; re-run yields a `check`-green corpus) → **arc close** → **A6
  resumes at slice05** (PM-skill) against the settled surface.
