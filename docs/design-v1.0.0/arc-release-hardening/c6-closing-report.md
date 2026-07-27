# C-6 closing report — `validate` hardening (G-2 + G-3 + L-3b)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-6 (the last RH chunk before compose)
> **Covers:** G-2, G-3, L-3b · **Assignment:** `cc-prompt-c6-check-hardening.md`
> **Implemented by:** CC · **Date:** 2026-07-27 · **Branch:** `rh-c6-check-hardening`
> (off the C-8 tip on `release/1.0.x`) · **Evidence class:** attested-by-CC (local 1.85+);
> cargo rows reproduce on CI.

## Summary

Three static rules were scoped. **One already existed**, so this is substantially two — and the two
premises the brief got wrong are the most useful part of the record.

```
$ odm validate                     # on odm's own store
⚠ validate: 0 error(s), 2 warning(s)
  [warning] #1000 "odm v1.0.0 — Project Plan (arc roadmap)": [undecomposed-parent]
      has 6 child(ren) but has never affirmed that they account for its scope
    fix: affirm it with `odm node decomposed 1000`
  [warning] #1600 "Migrate, self-host & PM-skill": [undecomposed-parent]
      has 5 child(ren) but has never affirmed that they account for its scope
(warnings do not fail; run with --strict to enforce)                     exit 0
```

## G-2 — already done, and the brief said otherwise

The chunk describes the tear rationale as a **data-loss bug**: *"validates the rationale today but the
schema drops it — there is no field, so the reason is lost the moment it's given."*

Checked against a real node before writing anything:

| Claim | Reality |
|-------|---------|
| no schema field | `TornEdge.because: String` — **required**, not optional |
| the reason is lost | round-trips to disk: `because: 'cycle break: …'` in the node file |
| `validate` doesn't surface it | `active tears (1): #2 A depends_on #3 B (because: …)` |
| `--json` doesn't carry it | `tears: [{from, to, because}]` |
| ODD-0013 needs amending | §4.3 already says *"required `because` rationale … never dropped"* |

So **Phase 0's schema amendment had nothing to amend** — the model and the code already agreed, and
editing ODD-0013 would have been writing down what it already said.

**The one real gap** was that `node show` never mentioned tears at all, which the acceptance criteria
do ask for ("read it back from the node file **and** `node show`/`--json`"). A tear is a deliberate,
reviewable decision; a reader looking at the node should see that one was made and why, not have to
run the integrity report to find out.

```
$ odm node show 2
  assumed (torn) dependencies:
    depends_on 01KYGRKFACJ30D44JRSP20V5NT — cycle break: B's API is stable
```

`--json` gains a matching `tears` array.

### The rationale-less warn: declined, with reason

The brief asks to warn on a tear with no recorded reason ("absent = a legacy tear … the status quo").
**That state cannot exist.** `because` is required, and a node missing it does not load:

```
Error: invalid frontmatter YAML: edges.tears[0]: missing field `because` at line 15 column 5
```

Implementing the warn would mean making `because` **optional** — trading a parse-time impossibility
(an unexplained assumed dependency cannot be written down at all) for a runtime nag. That is strictly
weaker, so it was declined and the existing guarantee pinned by
`a_tear_without_a_rationale_is_rejected_at_parse`, whose comment records why, so nobody loosens the
field later thinking they are implementing this.

## G-3 — broadened, warn-by-default

The requirement moves from *"affirm before you call it done"* to *"affirm once you have decomposed at
all"*. `undecomposed-parent`: warn by default, `--strict` → exit 1.

**Exclusive with the existing rule.** A done parent reports `advanced-without-decomposition` (the
stronger case) and *not* also the broader one — one gap, one finding. Pinned by
`a_done_parent_reports_the_stronger_finding_only`.

**The brief's heads-up was wrong**, and in the reassuring direction:

> *"odm's own self-hosted corpus has **no** `decomposed` assertions (the C-5 self-host didn't mint
> them), so enabling this rule will make `validate`/`check` on odm's corpus emit **warns** where they
> were clean."*

Arcs **1100–1500 all carry assertions** — 5 of 7 parents. The real gap is **2 nodes**. Predicted from
the corpus *before* writing the rule and confirmed after:

```
warned:                    [1000, 1600]
predicted (pre-analysis):  [1000, 1600]     MATCH
```

That also makes the brief's "if the warn volume is unwelcome, fall back to `--strict`-only" contingency
moot.

### A dynamic worth recording

**#1600 is still in progress.** Affirming its decomposition now is premature, and if a slice is added
later, `decomposition-drift` fires — which is an **error**, not a warning. So the rule nudges toward
an assertion that can become a defect.

Raised at kickoff alongside a narrower alternative (warn only once a parent's children are all done).
**Warn-by-default was chosen deliberately**, with that trade-off on the table: the gap should be
visible. Recorded here so the choice is legible later, not rediscovered.

## L-3b — the no-vision finding

A `project` with no `# Vision` section is reported (`no-vision`, warning). This is the enforcement
half of L-3a's data fix — odm's project *has* a vision, so the corpus passes and the rule exists to
stop that regressing silently.

Bodies are deliberately out of the index (ODD-0014 §3.5), so this is a **targeted load of project
nodes only** — the same shape `orient` uses for the one body it needs, not a corpus-wide body read.

The predicate accepts **any heading level and any case**: a body that renders a vision to a reader
must not be reported as lacking one on a technicality. An **empty** section does not count — that is
the same absence with extra steps. Prose containing the word, or a `# Vision statement process`
heading, does not count either.

## Taxonomy

All three are static, so they are `validate` rules and `check` inherits them. Asserted rather than
assumed, by `c6_rules_are_static_so_check_inherits_them`, which runs both verbs and requires each code
in both outputs.

## Tests

**Unit (odm-core, `recompose`)** — 6 new: the rule fires with its child count; a childless parent is
exempt; an affirmed parent is silent; a done parent reports only the stronger finding; a slice is
never a parent; a *mismatched* affirmation still reports drift and is not masked by the new rule.

**Unit (odm-cli, `has_vision`)** — 5: accepts a stated vision; indifferent to heading level and case;
rejects absent, empty, and word-elsewhere cases. Plus `dependency_label` keeping the qualifying gate.

**Schema (odm-core)** — the rationale-less tear is rejected at parse.

**Integration (odm-cli)** — 7: G-3 warns / clears on affirmation / promotes under `--strict`; L-3b
reports and clears; the taxonomy check; and G-2's rationale reaching `validate`, `show`, and
`show --json`.

**Three existing tests changed, because the definition of "clean" grew** — the intended effect, not
collateral. `check_clean_passes` now states a vision and affirms both parents' decompositions: that
test says what a clean corpus *is*, and the answer expanded. Two store-resolution tests asserted
`"3 node(s)"`, a string that only appears in the all-clear line; they now ask `node list --json` which
corpus was read, because **resolution** is their claim and it should not be coupled to how clean the
fixture happens to be.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --all-features --workspace` | **58 binaries ok, 0 failed** (+19 tests) |
| `cargo clippy --all-targets --workspace --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none |
| Reflexive check on the self-hosted store | 2 warns, **exactly** #1000 and #1600; 0 `no-vision`; exit 0 |
| `--strict` | exit 1 on the same corpus |
| Model change | none — no new schema field was needed |

## Deviations from the brief

| Brief | What happened | Why |
|-------|---------------|-----|
| Phase 0: amend ODD-0013's tear schema before the code | **No amendment** | The field already exists and §4.3 already documents it. Nothing to change. |
| G-2: persist the rationale (data-loss bug) | **Already done**; only `node show` was missing it | Verified against a real node, not assumed. |
| G-2: warn on a rationale-less tear | **Declined** | Would require making `because` optional — weaker than the parse-time guarantee it would replace. Pinned by test. |
| G-3: expect widespread new warns on odm's corpus | **2 nodes** | Arcs 1100–1500 already carry assertions; the brief's premise was wrong. |
| G-3: `decomposed` mismatch → warn / `--strict` error | **Left as an error** | `DecompositionDrift` is already an unconditional error. Downgrading it to a warning would weaken a live rule; a wrong assertion is worse than an absent one. |

## Silent-drop diff

None. Scoped out and absent: probe/reconcile changes, minting `decomposed` assertions on odm's corpus
(follow-up), readiness/satisfaction analysis (LLM arc slice 02).

## Ledger

- **RH-10** (new class-(a) row) — C-6 closed, **attested**; **RH-9** appended for C-8 in the same pass,
  so the class-(a) set now matches the Chunks table (RH-1…RH-5, RH-9, RH-10).
- **G-2 / G-3 / L-3b** — dispositioned; the two wrong premises and the declined sub-decision recorded
  in RH-8's change-log.
- **Follow-up, unassigned:** affirm `decomposed` on #1000 (its 6 arcs are settled) — a one-command fix
  the rule now surfaces. #1600 should wait until its slices are final, by the dynamic above.
- **After C-6:** RH has no chunks left → **RH-6/RH-7 compose** → arc close → A6 resumes at slice05.
