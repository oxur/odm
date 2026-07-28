# Slice 06 CDC verification — Live-run capability

> **Arc:** Migration Fidelity · **Slice:** 06 · **Feeds:** MF-2, MF-3, MF-5 · **Implemented by:** CC
> **Commit:** `d589aa1` on `release/1.0.x` (impl + close) · **Verified by:** CDC (Opus session) · **Date:** 2026-07-28
> **Verdict: PASS — CDC-verified. 10/10 rows done; both v2.1/v2.2 findings resolved at their root, plus
> a self-identified retired-node gap closed in the same pass. No iterations. One CDC finding — an
> internal contradiction between the `self_host_inner` doc comment and CC's own closing-report F-3 (the
> comment asserts a correctness invariant CC elsewhere disproves) — is doc-only, fails no row, and is
> recommended for correction in/before s07 (below).**

## Method & evidence access

CC committed (`make check` green, self-attested); I read the committed artifacts and **reproduced the
structural rows by close code-read on the live machine**. Git unreachable in the CDC VM and no 1.85+
cargo in the sandbox → SHAs/`git status` and the cargo-run rows (F-4/F-6/F-7 behaviors, clippy,
coverage) are **attested-by-CC → reproduced-on-CI**; the policy/wiring/exclusion rows I reproduced
directly.

**Independent reproductions (CDC, live machine):**

| Check | Row | Result |
|-------|-----|--------|
| `reconcile_source()` is the **one** policy fn; both `repair()` (L553) and `self_host`'s `to_populate` loop (L314) call it | F-1/F-10 | **reproduced (read)** — no second copy of the gate/exclusion logic exists |
| Gate fires inside `reconcile_source`: `verify_body_hash(source_body, new_body, …)` before `Ok(Some(..))`; drift → `BodyHashMismatch`, never stamped over | F-1 | **reproduced (read)** — L505–512 |
| Project **and** retired excluded — both at the `to_populate` pre-check (L274–282) **and** inside `reconcile_source` (L479, `node_type == Project \|\| retired().is_some()` → `Ok(None)`) | F-1/F-5 | **reproduced (read)** — defense in depth; the transition path specifically cannot stamp the project node |
| Gate runs under `--dry-run` (only `store.persist` is skipped) | F-6 | **reproduced (read)** — L313–320: `if !mode.is_dry_run()` guards the write only |
| `self_host_inner` calls `repair()` **then** `self_host()`, both `Mode`-aware; default-on, no new flag; both reports rendered | F-2/F-3 | **reproduced (read)** — `odm-cli/src/migrate.rs` L128–142 |
| New tests exist (4 unit + 1 `source_identity` + 3 `odm-cli`) | F-4/F-5/F-6/F-7 | **reproduced (grep)** — all seven `fn`s present; behavior attested → CI |
| No `unsafe` in changed files | F-9 | **reproduced** — `grep -RnE '\bunsafe\b'` on `selfhost.rs` + `migrate.rs` → none |
| Live store untouched | F-8 | **reproduced** — `.worktrees/odm` not checked out in this tree |
| No model drift — no stored hash (`build_source` unchanged), §2.3 exclusion explicit, update-in-place (id/edges preserved) | F-10 | **reproduced (read)** |

## Per-row disposition

All ten **done**. Reproduced-by-CDC where marked above; **attested → CI** for the cargo-run behaviors
(the drift/dry-run/idempotence assertions, clippy, `selfhost.rs` 96.26% / `migrate.rs` new-code-fully-
covered). The unification is genuinely good work: rather than adding a *second* gate check to the
transition, CC lifted `repair()`'s per-node logic into one `reconcile_source(document, plan_node,
today)` that **both** call sites route through — so the two paths now *cannot* drift out of sync a
second time (there is no second copy of the policy to forget to update). That is the correct structural
answer to a "two paths, inconsistent" finding: collapse to one, don't patch both. Endorsed.

**The retired-node extension (F-1, self-identified) is a real catch, not gold-plating.** The live
corpus documents an actual tombstone (`design-notes.md` §1), and `self_host`'s `by_coordinate` map
carried no retired guard — `repair()` only excluded retired nodes as an accident of its own pre-check.
CC found it by asking "what does the *other*, already-correct path guard that this one doesn't?" — the
same question that would have caught the original project-node gap proactively. Both the pre-check and
`reconcile_source` now exclude retired nodes for both callers. Correctly scoped as within "unify the
`source`-backfill policy," disclosed rather than folded in silently. Concur.

**F-5 reproduced structurally, not just attested:** the project exclusion exists on *both* the
transition pre-check and inside `reconcile_source`, and `selfhost_transition_excludes_the_project_node_from_source`
drives it through the transition path specifically — the exact path my v2.1 finding said F-6 didn't
cover. The gap I flagged is closed by the path I flagged it on.

## Finding — the `self_host_inner` doc comment contradicts CC's own F-3 (doc-only; correct in/before s07)

`odm-cli/src/migrate.rs`'s doc comment on `self_host_inner` (≈L111–120) asserts the call order is a
**load-bearing correctness invariant**:

> "Runs the full migration-fidelity flow in **load-bearing order** … Reordering these would reintroduce
> exactly the gap s06 closed — a reconciled-before-import invariant, not an incidental sequence."

CC's **own** closing-report F-3 (and arc-plan v2.4 point 1) prove the opposite, and I reproduce that
conclusion: because the **Preferred** unification routes `self_host`'s `to_populate` transition
through the *same* `reconcile_source` gate, `self_host()` **alone** reconciles every
coordinate-matched node identically to `repair()`-then-`self_host()`. Both orderings converge on an
identical final store state; the gap s06 actually closed (ungated / project-stamped `source`) lives
*inside* `reconcile_source`, which runs regardless of the repair/import order. So **reordering would
not reintroduce that gap** — the comment's central claim is false, and CC states as much two documents
over in the same commit.

This is **doc-only**: the implementation runs in the specified order, which is correct, safe, and the
right shape for API composability (it gives `repair()` the real, independent caller the v2.2 finding
demanded). It **fails no row** — F-3's criterion is "runs the flow in the correct order," which it
does. But a comment that asserts a falsified invariant is a latent trap: a future maintainer reads
"reordering reintroduces the gap," believes a `repair()`-first ordering is load-bearing for
correctness when it is not, and is steered away from the *actual* dependency — **correctness rests on
`reconcile_source` being correct, exercised both ways, not on the call sequence.** (There *is* an
ordering-sensitive world — the "remove the transition's backfill entirely, make `repair` the sole
populator" alternative the cc-prompt also offered — but CC did not choose it, so the comment describes
a dependency this implementation doesn't have.)

**Recommendation:** correct the `self_host_inner` comment to say what's true — the order is chosen for
composability and report attribution; per-node correctness rests on `reconcile_source`, which both
call sites exercise. Cheap (a comment edit), and s07 works in exactly this code path, so fold it into
s07's opening rather than reopening s06. I did **not** edit the code myself (CDC verifies; CC owns the
diff). Recorded so s07 starts from an accurate comment.

*(Minor, concurred-not-actioned: CC's `context.json` note — `slice-doc.md`'s Goal prose lists it but
the operative `cc-prompt.md` Task list and the arc-plan slice table both assign it to s07, and it's a
live-store operator statement, not a migration artifact. CC correctly followed the assignment and
flagged the doc inconsistency. Concur — s07's job.)*

## Bubble-up ratification (MF-2, MF-3, MF-5)

**CONCUR** — all three stay **planned**. s06 makes the flow that a safe live re-run needs exist,
one-policy, invocable via `odm migrate`, and fixture-proven end-to-end; the live-corpus outcomes
(zero stubs, every non-project node source-bearing, all arcs represented) are **s07**'s to reproduce
at arc scale (LEDGER-DISCIPLINE v2.0 §B — never inherited from a slice's fixture-only attestation).
Correct §B disposition; the closing report's pointers from MF-2/MF-3/MF-5 to slice04/05/06 as baseline
evidence are right.

**Disposition:** s06 ledger 10/10 done; both CDC findings (v2.1 two-path, v2.2 no-entry-point) resolved
at root; retired-node gap closed as a self-identified extension. **s07 (live repair run) is next** and
opens by (a) correcting the `self_host_inner` comment above, then (b) snapshot → `odm migrate
--dry-run` → inspect → fire on the real `.worktrees/odm` corpus, with this capability CDC-verified
behind it. Recommended arc-plan touch (v2.5): flip s06 to **CDC-verified PASS**; carry the comment
finding onto s07's opening. **Closed.**
