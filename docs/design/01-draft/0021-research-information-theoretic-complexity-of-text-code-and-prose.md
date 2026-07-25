---
number: 21
title: "Research — Information-Theoretic Complexity of Text (Code and Prose)"
author: "the evidence"
component: All
tags: [change-me]
created: 2026-07-25
updated: 2026-07-25
state: Draft
supersedes: null
superseded-by: null
version: 1.1
---

# Research — Information-Theoretic Complexity of Text (Code and Prose)

> **Status:** Draft research ODD. Promoted from `workbench/complexity-research.md`
> (2026-07-25, operator request) alongside the scope **decision of record**:
> complexity analysis enters odm as a **covariate feed for A7/A8** (telemetry ×
> forecasting), *not* as a first-class capability arc — no arc09. Tier-1
> implementation folds into **arc07 slice05** (covariate collection); the
> experiments join the **A8 research-gate** beside ODD-0018.
>
> **Question of record.** Can we compute a defensible complexity measure for
> *any* text a slice touches — prose (Markdown design docs) and code (Rust,
> Erlang, Lisp, Shell, Make…) alike — to serve as covariates in the
> empirical-velocity model? A typo fix and an architecture rewrite must land
> at opposite ends of the scale, in either format.
>
> **Sources.** Three raw research sweeps (2026-07-25, parallel web research
> with primary-source fetches, unverified items flagged inline):
> `docs/dev/research/0001-measuring-software-code-and-code-change-complexity-a-literature-review.md`
> (theory + empirical validity of code metrics),
> `docs/dev/research/0002-measuring-the-conceptualsemantic-complexity-of-written-prose-a-literature-survey.md`
> (prose/conceptual complexity), and
> `docs/dev/research/0003-code-text-complexity-tooling-survey-july-2026.md`
> (implemented tools, crate-level verification). Plus the standing design record:
> `docs/dev/research/0004-odm-telemetry-forecasting-post-arc6-thread.md` §5, ODD-0016, ODD-0018. Crate facts
> spot-re-verified 2026-07-25 (`complexity` v0.2.0 Apr-2020 stale; `tokei`
> v14.0.0 active; `candle-core` v0.10.2 active).
>
> **Calibration up front.** The *theory* here is solid (Kolmogorov →
> compression is textbook; Hassan's change-entropy is defect-validated at
> scale). The *bet* — that any of these measures predicts slice effort in our
> small-N, LLM-executed regime — is unvalidated anywhere in the literature.
> Nobody has published that link. We will be generating the evidence
> ourselves, which is exactly what the A7 telemetry corpus is for.

---

## 1. The prior record (answering "did we write this down?")

Yes — and the record says the opposite of the half-memory that prompted this
document. `docs/dev/research/0004-odm-telemetry-forecasting-post-arc6-thread.md` §5 (2026-06-24, confirmed in
the session memory of the same date):

> "no code-AST / language-specific metrics (no cyclomatic/cognitive parsing —
> those can't span Erlang/Lisp/Makefile, and reaching into language toolchains
> would couple the planning substrate to them) … rejected rust-code-analysis
> et al."

The ambiguity that survived in memory, resolved: **rust-code-analysis is a
Rust-implemented library that analyzes seven *other* languages** (C, C++,
Java, JS, TS, Python, Rust — via tree-sitter), not a Rust-code-only tool. It
was rejected on architectural grounds (language coverage, toolchain
coupling), not capability grounds. Its status since retroactively strengthens
the call: last real release v0.0.25 (Jan 2023), commits since are dependabot
only, no successor named by Mozilla. It has never supported Markdown or any
prose format.

What §5 *kept* was one information-theoretic signal: **diff entropy ("real vs
repetitive change")** among the git covariates. This document is the research
that gives that line formal footing and siblings.

## 2. The reframe that makes the problem tractable

Two moves, both already implicit in the §5 design, made explicit here:

**We do not need to solve "conceptual complexity" philosophically.** The
literature is blunt that nobody has: Cardon & Doğruöz (2025) find readability
formulas, human judgment, and automatic metrics correlate poorly *with each
other* — the field lacks an agreed construct. Chasing "true" conceptual
complexity is a tar pit. What the velocity model needs is weaker and
achievable: **cheap, deterministic covariates that correlate with observed
effort**, weighted empirically by the A8 statistics (regularized, capped
short-list, calibrated against our own actuals). If compressed-diff-size
carries signal, the regression will find it; if not, it gets dropped. The
construct question dissolves into a measurement-and-validation question.

**Code and prose unify at the byte level, and only there.** Every
format-*specific* approach (AST metrics for code, readability formulas /
discourse parsing for prose) fails the "any text" requirement by
construction. The information-theoretic family — compression, entropy,
LM-surprisal — is the *only* family in the literature that treats an Erlang
module, a Makefile, and an architecture ODD identically: as bytes with
measurable redundancy and surprise. The §5 language-agnosticism constraint
and the "any text" goal are the same requirement, and this family is its
unique solution.

One structural insight from mapping the covariate design onto the research:

- **Prose complexity is a *leading* covariate.** The slice-doc / cc-prompt /
  node body exist *before* work starts. Their measured density is available
  at forecast time.
- **Change complexity is a *lagging* covariate.** The diff exists only after.
  It trains the reference class and explains outcomes.

This slots cleanly into §5's existing leading/lagging split and tells us
where each measure earns its keep.

## 3. The theory map

Full treatments with formulas and citations live in the three sweep files;
this is the load-bearing summary.

### 3.1 The information-theoretic spine (format-agnostic — our family)

**Kolmogorov complexity** K(x) = length of the shortest program producing x.
The theoretically perfect "conceptual size" of a text — and provably
incomputable (Solomonoff 1960/1964; Kolmogorov 1965; Chaitin 1966). Every
practical measure below is a computable upper bound or proxy.

**Compression as the computable stand-in.** C(x) = compressed size under a
real compressor is an upper bound on K(x). Cilibrasi & Vitányi (IEEE Trans.
Inf. Theory 2005) make this rigorous with the **Normalized Compression
Distance**:

```
NCD(x, y) = [C(xy) − min(C(x), C(y))] / max(C(x), C(y))
```

— a quasi-universal similarity metric (their Theorem 6.3), proven to
approximate the ideal (incomputable) normalized information distance. NCD is
validated for clustering/similarity across arbitrary domains and used for
code-clone detection (Ishio et al., EMSE 2018). Known pitfalls: sensitive to
compressor choice and input length (compressor context windows distort
long-input results — prefer zstd/xz-class compressors with large windows over
gzip's 32KB window for document-scale inputs).

**Entropy of change scattering — the validated change-level metric.** Hassan,
"Predicting Faults Using the Complexity of Code Changes" (ICSE 2009; formulas
verified against the primary PDF): Shannon entropy over the distribution of
modified lines across files in a period,
`H(P) = −Σ pₖ·log pₖ` (normalized), aggregated per file into History
Complexity Metrics. Validated on six large OSS systems: 13–42% fault-
prediction error reduction vs churn baselines. Note carefully what it
measures: the *scattering* of change, not the content of the changed text.
Language- and format-agnostic (needs only VCS file-change data). This is the
formal ancestor of §5's "scatter" covariate.

**LM cross-entropy / surprisal — the semantic-ish end of the spine.** Two
converging literatures:

- *Code:* the "naturalness of software" line — Hindle et al. (ICSE 2012,
  10-year Most Influential Paper): code is statistically predictable;
  Ray et al. (ICSE 2016): **buggy lines are measurably less "natural"
  (higher entropy) than correct code**, and entropy drops when bugs are
  fixed. Extended neurally by CodeBERT-nt (2022) and 2024–25 AST-naturalness
  work.
- *Prose:* surprisal theory — Hale (2001), Levy (2008): human processing
  difficulty at a word ∝ −log P(word | context). Confirmed cross-lingually
  with neural LMs (TACL 2023). This is the *psycholinguistically validated*
  bridge between "information content" and "cognitive load" — the closest
  thing to a scientific warrant that LM-measured information density tracks
  how hard text is to process.

Caveats that must travel with any surprisal covariate: it conflates intrinsic
complexity with *corpus familiarity* (an unusual-but-simple idiom scores
high); it is tokenizer- and model-version-dependent (a reproducibility
hazard for a persisted index); low perplexity can reflect memorization, not
simplicity.

### 3.2 What the code-metrics literature actually supports (and doesn't)

- **Halstead (1977):** formulas survive; the theory's psychological basis
  (Stroud number fitted on 12 programs) was demolished by 1983 (Shen et al.,
  Coulter; NIST TN-1990 for a modern autopsy).
- **McCabe cyclomatic (1976):** empirically contested to this day. Shepperd
  (1988): "no more than a proxy for LOC." Jay et al. (2009, 1.2M files): LOC
  explains ~90% of CC's variance. But Landman et al. (2016, 17.6M methods):
  *no* strong correlation. Genuinely unresolved — and either way, no help
  for our constraint (per-language CFG parsing).
- **Cognitive Complexity (Campbell, SonarSource, first published
  2016/17 not 2018):** deliberately human-oriented rules; thin independent
  validation (essentially one ESEM 2020 study, positive on comprehension
  *time*, mixed on correctness). AST-dependent → fails our constraint.
- **fMRI/EEG comprehension studies (Peitek et al. ICSE 2021, n=19):**
  traditional metrics do not track measured comprehension difficulty —
  the modern neuroscience echo of Shepperd 1988.
- **What IS well-validated at scale:** *relative* (size-normalized) code
  churn — Nagappan & Ball (ICSE 2005, Windows Server: R²=0.811 for defect
  density, vs 0.052 for absolute churn); Hassan's change entropy (above);
  the JIT defect-prediction feature family (Kamei et al., TSE 2013 — 14
  change-level metrics incl. diffusion entropy, all VCS-derivable,
  language-agnostic).

The pattern is stark and convenient for us: **the metrics with the strongest
empirical validation are precisely the language-agnostic, VCS-derived ones**;
the language-coupled AST metrics are the contested ones. Our §5 rejection is
independently vindicated by the evidence base.

### 3.3 What the prose literature offers

- **Readability formulas** (Flesch/Kincaid, Fog, SMOG, …): surface arithmetic
  over sentence/syllable counts. Cheap, but they measure sentence-shape, not
  conceptual load — "Set the CAP theorem aside; use CRDTs" scores as easy.
  Not candidates for our purpose (and English-calibrated — another
  agnosticism failure).
- **Idea/propositional density** (Kintsch & Keenan 1973; CPIDR): propositions
  per word via POS heuristics — 50 years of cognitive-psych validation, but
  English-POS-dependent and validated on narrative/clinical text.
- **Integrative complexity** (Suedfeld & Tetlock 1977; AutoIC 2014/2020):
  scores differentiation-and-integration of perspectives — conceptually the
  closest published construct to "handles trade-offs well" in an ODD, but
  dictionary-based, domain-tuned to political discourse, and scoring
  reliability is contested even among trained humans.
- **Discourse structure (RST depth), topic entropy, embedding dispersion:**
  plausible, offline-feasible, but either parser-heavy, genre-unvalidated, or
  (flagged in the sweep) not established as citable named metrics at all.
- **Compression & surprisal** (§3.1) apply to prose unchanged — with the
  psycholinguistic warrant (surprisal↔reading time) that the surface
  formulas lack.

Verdict: for prose, the format-specific literature offers us *interpretive
vocabulary* but no adoptable metric that fits the constraint. The
information-theoretic spine is the same one code needs. Convergence, again.

### 3.4 The change-complexity gap (and the MUST→SHOULD problem)

No canonical "complexity of an edit" metric exists in either literature. The
best available framing (synthesized in the prose sweep, §6.4 — flagged as
synthesis, not citation) is **two orthogonal axes**:

1. **Magnitude** — how much changed: edit distance / churn / C(diff).
2. **Semantic displacement** — how much the *meaning* moved: embedding
   distance between versions (wDTW/wTED, Zhu et al. IJCNLP 2017).

Their divergence is the interesting signal: large-magnitude +
low-displacement = mechanical rewrite (rename, reformat); low-magnitude +
high-displacement = the "MUST→SHOULD" edit — one token, huge consequence.

Honest limit: **no cheap measure catches MUST→SHOULD.** Compression sees one
token. Surprisal may or may not spike (both words are locally plausible).
Embedding distance on the containing paragraph is the only automatic
candidate, and it is unvalidated for this. For the velocity model this is
acceptable: the *consequences* of meaning-critical edits surface in odm's own
lagging channels (iterations, rework-supersedes, evidence regressions), which
we already collect. We measure the blast, not the trigger.

## 4. The empirical scoreboard

| Claim | Status |
|---|---|
| Relative churn predicts defect density | **Validated** (R²=0.811, Nagappan & Ball 2005) |
| Change-scattering entropy predicts faults | **Validated** (Hassan 2009, 6 systems) |
| Code naturalness (LM entropy) flags buggy lines | **Validated** (Ray et al. 2016) |
| Word surprisal predicts human reading difficulty | **Validated** (Levy 2008; TACL 2023, 11 languages) |
| Cyclomatic complexity adds signal beyond LOC | **Contested** (Jay 2009 vs Landman 2016 — unresolved) |
| Cognitive Complexity tracks understandability | **Thin** (one ESEM 2020 study) |
| Readability formulas track conceptual difficulty | **Refuted-ish** (construct-validity failure, Cardon & Doğruöz 2025) |
| Single-doc compression ratio = complexity | **Plausible, unvalidated** (no peer-reviewed validation found; NCD corollary) |
| Static complexity metrics → story points / velocity | **No literature exists** (folk practice only) |
| Monte-Carlo/PERT forecasting + complexity covariates | **No literature exists** (genuine gap) |
| LLM effort estimation from ticket text | Emerging (2024–25: moderate correlation; few-shot project examples help) |

The two "no literature exists" rows cut both ways. Opportunity: odm's A8
design is doing something unpublished — the DAG + measured durations +
complexity covariates combination appears to be novel. Warning: nothing out
there will tell us the answer; the covariates proposed below are *hypotheses*
until the slice corpus scores them. This is the same discipline ODD-0018
already imposes on the two-clock split (baseline + held-out comparison).

## 5. Tooling verdict (answering "are there libraries?")

Full verified survey with versions, licenses, and staleness dates:
`docs/dev/research/0003-code-text-complexity-tooling-survey-july-2026.md`.
The decision-relevant summary:

**AST code metrics — exist, multi-language, wrong shape for us.**
rust-code-analysis (stalled, above); a genuine 2025–26 wave of new Rust
crates filled its vacuum (`debtmap`, `knots` 12 langs, `arborist-metrics`,
`cccc-core`, `hotspots-core` — all actively releasing as of Jul 2026). All
tree-sitter/syn-based → per-language grammars → none cover Erlang + Lisp +
Make + Markdown → all fail the constraint. Recorded here as
considered-and-rejected-with-evidence; worth a periodic re-look only if the
constraint ever changes. (`complexity` crate: Rust-only, stale since 2020.
`tokei` v14: active but counts lines, no complexity. Clippy's
`cognitive_complexity` lint: Rust-only, self-disclaimed approximation.)

**Prose metrics — thin everywhere, near-empty in Rust.** Python `textstat` is
mature; the Rust `textstat` crate is *a month old* (v0.1.1, 2026-06-21).
Coh-Metrix is gated/aging; CPIDR/AutoIC are niche academic tools. Nothing
adoptable — and per §3.3 we wouldn't want the formulas anyway.

**NCD — no maintained implementation exists in any language.** CompLearn is
abandoned (domain squatted; last commit 2015). Python's only packaged option
is a minor feature of `textdistance`. **This is not a blocker — it is an
invitation.** NCD over a chosen compressor is ~50 lines of Rust over crates
we can freely take: `zstd` (v0.13.3), `flate2` (v1.1.9), `brotli` (v8.0.3) —
all actively maintained, all already ubiquitous. The measure is trivially
implementable; the *research content* is in the definitions and validation,
which is this document's job.

**LM surprisal offline in Rust — feasible, heavier.** `candle` (HF,
v0.10.2, active, ships a perplexity example) or `llama-cpp-2` (active
binding; llama.cpp has first-party perplexity tooling). `rustformers/llm` is
archived — do not adopt. HF `tokenizers` is Rust-native and active.

**Git plumbing — already ours.** `gix` is a dep; per-commit file-change data
(Hassan entropy, churn, scatter) and diff bytes (compression measures) are
all derivable from what A7 already plans to collect.

**Conclusion: implement, don't adopt.** Nothing reusable computes what we
need on arbitrary text. Everything we need to *build* it is maintained,
permissively licensed, and mostly already in-tree. The core (Tier 1 below)
has no new dependencies at all beyond a compression crate.

## 6. Proposed measurement design (candidate covariates, tiered)

These are **candidates for the §5 pre-registered short-list**, subject to its
hard cap ("fewer-robust > many-fragile") — adopting any of them means
displacing or subsuming, not appending. All are deterministic, offline,
format-agnostic. Naming: working names, to be settled at adoption.

**Tier 1 — compression kit (no ML, ~zero new deps).** For a slice's change
set (concatenated per-file unified diffs, generated/vendored paths excluded
per §5), with a fixed, versioned compressor (proposal: zstd, level and
version recorded in the index snapshot — determinism is a provenance
requirement, ODD-0013):

- `change_info` = C(diff) — compressed size of the change, in bytes. The
  Kolmogorov-flavored upgrade of raw churn: a 500-line mechanical rename
  compresses to almost nothing; 40 lines of novel logic doesn't. *Lagging.*
- `change_density` = C(diff) / |diff| — compression ratio of the diff; the
  formal version of §5's "diff entropy (real vs repetitive change)". Splits
  magnitude from novelty. *Lagging.*
- `change_displacement` = NCD(before, after) per touched file, size-weighted
  mean — how much of the new version the old version doesn't explain.
  Distinguishes append/refactor from rewrite. *Lagging.*
- `change_scatter_H` = Hassan's normalized entropy of modified lines across
  files — formalizes §5's "scatter" with the one defect-validated formula in
  this space. *Lagging.*
- `spec_info` = C(slice-doc + cc-prompt bodies) — compressed size of the
  spec. The upgrade of §5's "cc-prompt/scope size" from raw length to
  information content. **Leading** — computable before work starts.

All five work identically on `.md`, `.erl`, `.lisp`, `Makefile`.

**Tier 2 — surprisal (gated, not now).** Mean per-token NLL of changed lines
/ spec text under a small pinned local model (candle or llama.cpp). The
psycholinguistic warrant is real (§3.1), but: model+tokenizer become index
provenance (version-pin or forfeit reproducibility); familiarity/memorization
confounds; a heavyweight dep against odm's minimal-infra ethos. **Gate:
adopt only if Tier 1 residuals on the slice corpus show unexplained variance
that a surprisal probe (offline experiment first, not shipped code) captures.**

**Tier 3 — semantic displacement (research horizon).** Embedding-based
distance to catch meaning-dense small edits (MUST→SHOULD). Unvalidated for
the purpose, heavy, and the consequences are already visible in
iterations/rework signals. Parked; revisit only with evidence that it's the
missing variable.

**Endogeneity cautions (from 0018, applied):** `spec_info` may proxy for
"how much we already knew to write down" — correlated with, not causal of,
effort; fine for forecasting, dangerous for explanation. Never let any of
these become targets (Goodhart — compressed-diff-size is *more* gameable than
LOC in adversarial hands, e.g. by injecting noise; acceptable only because
odm's covariates are metadata, never performance measures, per §1.6).

## 7. Answers of record (the six questions, compressed)

1. **Prior record?** Yes: §5 of `docs/dev/research/0004-odm-telemetry-forecasting-post-arc6-thread.md` — AST
   libraries considered and **rejected**; diff entropy retained. No library
   was ever "planned in."
2. **Which libraries / what data?** rust-code-analysis: Rust-built,
   7 input languages, code only, no prose, stalled. The 2025–26 crate wave:
   same shape, same disqualification.
3. **Prose complexity research?** Readability formulas (surface-only,
   construct-invalid for our purpose); idea density; integrative complexity;
   surprisal theory (the validated bridge); compression. Sweep:
   `docs/dev/research/0002-…-written-prose-a-literature-survey.md`.
4. **Code complexity research?** Halstead/McCabe/Cognitive (contested or
   thin); the validated family is VCS-derived: relative churn, change
   entropy, naturalness. Sweep:
   `docs/dev/research/0001-…-code-change-complexity-a-literature-review.md`.
5. **Implemented libraries?** For AST metrics and readability formulas, yes
   (wrong shape). For NCD/compression-complexity: effectively none anywhere —
   trivially buildable. Sweep:
   `docs/dev/research/0003-code-text-complexity-tooling-survey-july-2026.md`.
6. **In Rust?** Metrics crates: multi-language-code-only or stale. Building
   blocks: excellent (`zstd`/`flate2`/`brotli`, `gix`, `candle`,
   `tokenizers`). Verdict: implement Tier 1 in-tree.

## 8. Open research agenda (next few months)

1. **Adjudicate the short-list.** Which Tier-1 candidates enter the §5
   pre-registered cap, and what do they displace? (Decision with Duncan;
   updates `docs/dev/research/0004-odm-telemetry-forecasting-post-arc6-thread.md` §5 / the arc07 slice05
   slice-doc with a dated version-history entry.)
2. **Compressor bake-off** (small, empirical): zstd vs brotli vs xz on our
   actual node corpus + repo diffs; window-size effects (NCD pitfalls paper);
   stability across compressor versions. Output: pinned choice + level.
3. **Retro-scoring experiment.** A1–A6 slices are closed and have branches,
   diffs, ledgers, iteration counts. Score them with Tier 1; regress against
   iterations-used and active-work time. This is a free pilot of the entire
   A8 premise on data we already own — and the first evidence anywhere on the
   "complexity covariates → LLM-slice effort" link (§4's gap). Runs as
   A8-research-gate work; does not require A7 to have landed.
4. **Prose-leading hypothesis.** Test `spec_info` (leading) against observed
   slice outcomes in the same retro corpus.
5. **Tier-2 gate criteria.** Define, before looking at residuals, what
   "unexplained variance worth a surprisal probe" means (pre-registration
   discipline, PERT-21 style).
6. **Watch list.** `textstat` (Rust) maturation; the `debtmap`/`knots` wave
   (if the agnosticism constraint ever relaxes); LLM-effort-estimation
   literature (fast-moving, 2024–26); any first paper linking naturalness to
   diff-level risk (a flagged gap — someone will publish it).

## 9. Key citations (verification status inline; full lists in sweeps)

- Kolmogorov (1965); Solomonoff (1964); Chaitin (1966) — priority as usually
  stated; incomputability standard.
- Cilibrasi & Vitányi, "Clustering by Compression," *IEEE Trans. Inf. Theory*
  51(4), 2005. **Formula verified against author PDF.**
- Hassan, "Predicting Faults Using the Complexity of Code Changes," ICSE
  2009. **Formulas + results verified against primary PDF.**
- Nagappan & Ball, "Use of Relative Code Churn Measures…," ICSE 2005.
  **Verified (MSR PDF).**
- Hindle, Barr, Su, Gabel, Devanbu, "On the Naturalness of Software," ICSE
  2012; Ray et al., "On the 'Naturalness' of Buggy Code," ICSE 2016.
  **Verified.**
- Hale (2001); Levy, *Cognition* 106(3), 2008; "Testing the Predictions of
  Surprisal Theory in 11 Languages," TACL 2023. **Verified.**
- Kamei et al., "A Large-Scale Empirical Study of Just-in-Time Quality
  Assurance," *IEEE TSE* 39(6), 2013. **Verified.**
- Shepperd, *Softw. Eng. J.* 3(2), 1988; Jay et al. 2009 vs Landman et al.
  2016 — **the CC-vs-LOC contradiction is unresolved; cite both.**
- Campbell, "Cognitive Complexity," SonarSource whitepaper — **first
  published 2016/17, not the commonly cited 2018.**
- Peitek et al., "Program Comprehension and Code Complexity Metrics: An fMRI
  Study," ICSE 2021. **Verified.**
- Cardon & Doğruöz, "Readability Measures and ATS: In the Search of a
  Construct," 2025 preprint (arXiv:2511.09536). **Verified.**
- Jiang et al., "'Low-Resource' Text Classification…," ACL Findings 2023 —
  **cite only with the Kenschutte replication caveat** (top-2-accuracy bug;
  dataset contamination).
- di Biase, Rastogi, Bruntink, van Deursen, "The Delta Maintainability
  Model," TechDebt 2019. **Attribution corrected during this research** (a
  circulating mis-attribution was checked and found wrong).
- Zhu, Klabjan & Bless, "Semantic Document Distance Measures…," IJCNLP 2017.
  **Verified.**

Known synthesis (ours, not citable): the magnitude-vs-semantic-displacement
divergence framing for edit complexity (§3.4); single-document compression
ratio as a complexity measure (plausible NCD corollary, unvalidated).

---

*Companion files (the three raw sweeps, now dev research docs under
`docs/dev/research/`): `0001-measuring-software-code-and-code-change-complexity-a-literature-review.md`
· `0002-measuring-the-conceptualsemantic-complexity-of-written-prose-a-literature-survey.md`
· `0003-code-text-complexity-tooling-survey-july-2026.md`. Each carries its own
unverified-items ledger; nothing flagged there has been silently upgraded here.*

## Version History

### v1.1 — 2026-07-25
Repointed all companion-sweep references from `workbench/complexity-sweep-*.md`
to their promoted homes under `docs/dev/research/` (0001 code-metrics, 0002
prose, 0003 tooling — a new dev-research numbering series). Reference-only
change; no content moved or altered. Surfaced by: operator adding the sweeps to
the dev docs.

### v1.0 — 2026-07-25
Promoted unchanged in substance from `workbench/complexity-research.md`
(created earlier the same day) with frontmatter added, workbench-relative
paths repointed, and the scope decision of record captured in the header
(covariate feed via arc07 slice05 + the A8 research-gate; no arc09).
Surfaced by: the complexity research session (CDC + Duncan, 2026-07-25).
