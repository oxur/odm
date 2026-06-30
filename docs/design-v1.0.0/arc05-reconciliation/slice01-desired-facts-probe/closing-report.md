# Closing report — Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> strength `attested` (built + run locally on a 1.95 toolchain). CDC reproduces
> (CI / local 1.85+) to elevate to `reproduced`. Branch:
> `arc05-slice01-desired-facts-probe` (not `main`).

## Per-row walk

**F-1 — `desired_facts` field, round-trips on any node type — done (attested).**
Added `desired_facts: Vec<DesiredFact>` to `Frontmatter`
(`#[serde(default, skip_serializing_if = "Vec::is_empty")]`, placed after
`decomposed` in canonical order), with `with_desired_facts`/`desired_facts()`.
The model (`DesiredFact { id, describe, probe }`, `ProbeSpec`, `ShellExpect`)
lives in the new `odm_core::desired` module. `desired_facts_round_trip` exercises
a **project** node (two facts — exit-only and exit+stdout), a **slice** node, and
the **empty default**, asserting `parse ∘ emit == identity` on each, the accessor,
that empty is skipped on emit (no baseline churn), and that the slice-doc's
proposed YAML shape parses into the typed model. `cargo test -p odm-core
desired_facts_round_trip` → ok.

**F-2 — malformed entry → positioned error — done (attested).** Unknown probe
`kind` (internally-tagged enum), missing required field (`run`), and empty `id`
(a `deserialize_with` semantic check) each surface as `FrontmatterError::Yaml`
whose message carries `line`/`column` — verified against the existing positioned-
error channel, never a panic or silent drop. `desired_facts_malformed_errors_with_position`
asserts the position substrings on all three and that empty-`id` additionally
names the field. `cargo test -p odm-core desired_facts_malformed_errors_with_position` → ok.

**F-3 — `Probe` trait + three-way `ProbeOutcome` — done (attested).** In
`odm-reconcile`: `enum ProbeOutcome { Holds, Drifted { expected, observed },
Error { reason } }` (not a `bool`) and `trait Probe { fn evaluate(&self) ->
ProbeOutcome }` (object-safe, one method). `probe_outcome_has_three_variants`
constructs all three and asserts pairwise distinctness + carried data;
`grep -nE "enum ProbeOutcome|trait Probe"` locates both.

**F-4 — shell probe maps command → outcome — done (attested).** `ShellProbe`
runs the declared command and maps result to outcome. `cargo test -p
odm-reconcile` → 9 passed, including the three required real-command cases:
`true` → `Holds`, `false` (exit 1 vs expected 0) → `Drifted{exit 0, exit 1}`, a
non-existent binary → `Error`. Plus stdout-match holds/drift, empty-command
`Error`, and signal-termination `Error`. **Flagged design realization:** the
probe execs the command **directly** (whitespace-tokenized argv), not via
`sh -c` — see *Bubble-up* §2.

**F-5 — trust model documented — done (attested).** `ShellProbe`'s `# Trust
model` doc states: author-declared commands, run locally, with the user's own
privileges; **no MVP sandbox**; boundary = "a command written in your own node
files." Deferred guardrails (`--no-exec`/allowlist) are recorded in-comment as a
**no-op-with-rationale** (out of MVP scope — no code needed). `grep -nE
"trust|author-declared|no sandbox|privilege"` matches.

**F-6 — `odm-reconcile` is a workspace member — done (attested).** Manifest uses
`*.workspace = true` inheritance + `[lints] workspace = true`, **no version
literals** (`grep '= "[0-9]'` → none). Added to `[workspace] members`; CLAUDE.md
crate table + publish-order note updated (and the missing `odm-index` row, an A4
leftover, fixed in passing). `cargo build -p odm-reconcile` → ok.

**F-7 — gates — done (attested).** `cargo clippy --all-targets --all-features --
-D warnings` → exit 0; no `unsafe` in the new code (also denied workspace-wide);
`cargo llvm-cov --summary-only` → `odm-reconcile/src/lib.rs` **96.00% line**,
`odm-core/src/desired.rs` **100.00% line** — both ≥ 90. Full workspace
`cargo test` is green (no regression in odm-index/odm-cli baselines).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item of `slice-doc.md` shipped: the field (incl. project node),
the positioned error, the `Probe` trait + three-way outcome, the shell probe
(holds/drift/error), the crate as a clean workspace member, and the documented
trust model. Every "out" item stayed out — **no** probe-runner, **no** `file`
probe, **no** `reconcile` command, **no** rollup/orient wiring, **no**
index/`IndexRecord` change.

Two **disclosed** (non-silent) consequential edits, both in the diff:
1. The existing `unknown_keys_preserved_through_roundtrip` test used
   `desired_facts` as its example *unknown* key; since the key is now typed, its
   fixture was switched to a genuinely-unmodeled key (`provenance_note`). The test
   still proves typed-field + unknown-key co-survival.
2. `odm-core` frontmatter doc comments that named `desired_facts` as a
   "not-yet-modeled" example were updated (it is now modeled).

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice01 deliver A-1's piece?** Yes. A-1 ("slice01 closed") is the
arc-ledger row this slice discharges. The model + trait + first probe that
slices 02–04 compose on now exist and are gated. A-1 stays `attested` in the arc
ledger until CDC reproduces (`cdc-verification.md`).

**2. What the arc-plan didn't anticipate — the shell probe is exec-not-shell.**
The plan (and ODD-0013 §5.2) call this the "shell" probe and the proposed YAML
field is `run: "pg_isready -h prod -t 2"`. Realizing it surfaced a tension the
plan didn't name: if the command is run via `sh -c`, a **missing binary returns
exit 127** — a *divergence* — which collapses the constraint's non-negotiable
"couldn't check" (Error) ≠ "checked, drifted" (Drift) distinction. The slice
resolves this by **tokenizing `run` and exec-ing the program directly**, so a
spawn failure is a true `Error`. Cost: shell metacharacters (pipes, redirects,
`&&`, `$VAR`) are **not** interpreted in the MVP (documented on `ShellProbe`).
All proposed-shape examples (`program args`) work unchanged. **Recommendation for
the arc-plan:** if real probes need pipelines/redirects, add an explicit
shell-interpretation mode (or a `sh -c` escape hatch) as a *named* spec field in
a later slice rather than silently changing exec semantics — keep Error/Drift
honesty intact. Not blocking for slice02/03.

**3. Sharpening the slice02/03 index-integration decision (the carried
adapter-fidelity invariant).** slice01 added `desired_facts` to the **frontmatter
only** — no `IndexRecord`/adapter/fidelity-test change, exactly as the arc-plan's
fourth open question scopes it (the trigger is "the first *reader*", slice02/03).
Two concrete findings that sharpen that decision:
  - The field is `skip_serializing_if = "Vec::is_empty"` (correct for
    self-describing YAML; avoids baseline churn). **But** the A4 lesson is that
    `skip_serializing_if` *desyncs* postcard (the index's non-self-describing
    format). So when slice02/03 makes `reconcile` read facts via the index, the
    `IndexRecord` field for `desired_facts` must be **always-serialized** (or
    gated by a `FORMAT_VERSION` bump) — the frontmatter skip does **not** carry
    over. This is the precise shape of the obligation the arc-plan flagged.
  - `desired_facts` is a **structured, nested** field (a list of
    `{id, describe, probe-enum}`), not a scalar like the metadata A4 indexed.
    Whichever slice indexes it must extend the adapter **and** its fidelity test
    (`odm-index/tests/adapter.rs`) to compare the full nested value field-by-field
    — the CLI `*_matches_baseline` idempotence tests will pass *tautologically*
    and will **not** catch an incomplete adapter (the carried invariant). The
    decision to defer was correct; the deferred work is now concretely scoped.

**4. Other reusable findings.** (a) serde_norway positions custom
`deserialize_with` errors and nested missing-field errors — so the workspace's
"errors carry `Position`" convention extends cleanly to semantic validations, not
just structural ones. (b) The internally-tagged-enum + *not*-`#[non_exhaustive]`
choice means slice02's `file` kind will be a compile error at every dispatch
site until handled — a free completeness gate for the runner.

No row required an amendment; the YAML shape fit Rust/serde cleanly. The
exec-not-shell realization (§2) is flagged as a design decision, not a divergence
from any ledger criterion (F-4 requires only that a command map to the three
outcomes against real commands, which it does).
