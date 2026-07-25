# Code & Text Complexity Tooling Survey — July 2026

Methodology: multi-agent research pass using live fetches to crates.io (JSON API), docs.rs, GitHub, PyPI, and CRAN, cross-checked with web search. Every entry is labeled with a confidence tag:

- **[V]** — verified via a direct fetch in this session (API/HTML/README).
- **[S]** — search-corroborated only; a direct fetch was blocked by a persistent session-level `web_fetch` rate limit that did not clear despite repeated retries across ~10+ minutes and multiple agents. Treat version numbers/dates under **[S]** as reasonably confident but not independently re-verified.
- **[U]** — unresolved / could not confirm; flagged explicitly rather than guessed.

Verdict labels used throughout: **Rust code only** / **multi-language code** / **prose/text** / **any bytes** / **git history tooling**.

---

## 1. rust-code-analysis (Mozilla) and its offshoots

### rust-code-analysis / rust-code-analysis-cli **[V]**
- **Language:** Rust. **Verdict:** multi-language code (source code only, not prose).
- **Input languages (exact list from official docs):** C, C++, Java, JavaScript, TypeScript, Python, Rust — plus several Mozilla-internal tree-sitter grammar variants (Mozcpp, Ccomment, Preproc, Mozjs) that are dialects/preprocessing passes rather than distinct languages. So: **7 general-purpose languages**, 11 grammar entries total.
- **Metrics:** Cyclomatic Complexity (CC), Cognitive Complexity, Halstead (all sub-metrics), Maintainability Index (MI), ABC, WMC, SLOC/LLOC/PLOC/CLOC/BLANK, NOM, NARGS, NEXITS, NPA, NPM.
- **Markdown/prose support:** **Confirmed absent.** No Markdown or prose-complexity capability anywhere in the docs or language list.
- **License:** MPL-2.0 (grammars separately MIT).
- **Status:** **Not archived**, but stagnant at the release level. Last commit ~2026-01-20 (routine dependabot bump), repo pushed as recently as 2026-04-06, but the **last tagged release/crates.io publish is v0.0.25 from 2023-01-13** — over three years with no version bump despite continued light maintenance. No official successor announced by Mozilla; no evidence found of another party formally forking to take over stewardship as *the* successor (see below for de facto forks).
- **crates.io:** `rust-code-analysis` v0.0.25 (2023-01-13), `rust-code-analysis-cli` v0.0.25 (2023-01-13), both non-yanked.
- URLs: https://github.com/mozilla/rust-code-analysis · https://crates.io/crates/rust-code-analysis · https://mozilla.github.io/rust-code-analysis/

### rust-code-analysis-code-split — de facto fork **[V]**
- A community fork that exists specifically because upstream is stale: bumps the tree-sitter dependency to 0.26 and keeps the same metric set (cyclomatic, cognitive, Halstead, MI, LOC).
- Latest: 0.0.26-cs.0, published 2026-05-28.
- **Verdict:** multi-language code. This is the closest thing to an "active successor" that surfaced in this research — note it is a maintenance fork, not a reimagining.

---

## 2. Other Rust crates/tools for code metrics

### Legacy/well-known names
| Name | Verdict | Notes |
|---|---|---|
| **`complexity`** (rossmacarthur) **[S/U]** | Rust code only | Reportedly computes Cognitive Complexity for Rust via `syn` (not tree-sitter). GitHub topic-page fetch **[V]** shows last commit **2021-02-20** — stale ~5 years. Exact crates.io version/date could not be directly re-confirmed this session (rate-limited); do not cite a version number without re-checking. |
| **tokei** **[S]** | multi-language code | SLOC/comment/blank-line counter across many languages. Does **not** compute cyclomatic/cognitive complexity — long-standing GitHub feature request for McCabe complexity was never implemented, consistent with it staying a pure line-counter. Exact current version/last-publish date not independently re-verified this session (rate-limited) — treat as "still the standard tokei," but don't cite a specific version without re-checking. |
| **scc** (boyter/scc) **[S]** | multi-language code | Written in **Go**, not Rust. Computes a **heuristic, keyword/token-based approximation** of cyclomatic complexity per file during its line-counting pass (not a true AST-based CC), plus COCOMO cost estimates. Confirmed to exist and be a known, established tool; exact last-release date not independently re-fetched this session. |
| **cargo-geiger** **[V]** | Rust code only | Measures **unsafe-code usage**, not complexity — different tool category, included only to correct a possible conflation. Latest 0.13.0, published 2025-08-31. License Apache-2.0 OR MIT. Active. |
| **clippy::cognitive_complexity** **[S]** | Rust code only | Reportedly still present but lives in Clippy's opt-in `restriction` group (not in `clippy::all`/`pedantic`), with the lint's own docs reportedly disclaiming that true Cognitive Complexity "is not something we can calculate using modern technology" — i.e., Clippy's own maintainers flag it as an approximation, not authoritative. Default threshold reportedly 25. Not independently re-verified against a live clippy lint-index fetch this session. `clippy::too_many_arguments` (category `complexity`, threshold 7) and `clippy::too_many_lines` (threshold 100) are simpler arity/length checks, not structural complexity metrics. |

### New/actively maintained Rust tools (2025–2026) — all **[V]** via crates.io JSON API and/or GitHub
This is the most important update to the picture: **the rust-code-analysis vacuum has been filled by a wave of new, independently-built tree-sitter/syn-based crates in 2025–2026**, not by a single successor.

| Crate | Verdict | What it does | Latest version | Last publish |
|---|---|---|---|---|
| **debtmap** | multi-language code (Rust/Python/JS/TS) | Cyclomatic + cognitive complexity, coverage correlation, entropy, git-history risk; has a client-side dashboard | 0.21.0 (66 published versions) | 2026-07-02 |
| **knots** | multi-language code (C/C++/Rust/Python/JS/TS/Ada/Go/Java/C#/Kotlin/Swift/PHP — 12 languages) | Complexity + novel "AI authorship cost" metrics (AIRD, AICP); SARIF/JSON/CSV output; CI gates | 1.13.1 (11 releases since May 2026) | 2026-07-01 |
| **arborist-metrics** / **arborist-cli** | multi-language code (Rust, Python, JS, TS, Java, Go; optional C/C++/C#/PHP/Kotlin/Swift) | Cognitive/cyclomatic/SLOC via tree-sitter, feature-gated per language | 0.1.3 / 0.2.1 | 2026-05-19/20 |
| **kimun** | Rust code only | "Code health" score: complexity, duplication, hotspots, ownership | 0.24.0 (11 versions since Feb 2026) | 2026-06-04 |
| **cccc-core** (moznion/cccc) | multi-language code (via normalized IR) | Language-agnostic Cognitive (SonarSource-style) + Cyclomatic (McCabe) engine | 1.0.0 | 2026-07-01 |
| **crap-core / crap4rs / crap4ts** | Rust code only (crap4rs) / TS via oxc (crap4ts) | CRAP metric = complexity²×(1−coverage)³+complexity, i.e. complexity combined with test coverage | 0.9.0 / 0.6.2 / 2.0.0-rc.5 | 2026-06-24 |
| **cargo-crap** | Rust code only | Standalone CRAP calculator for Rust, positioned as a guardrail for AI-generated code | 0.3.0 | 2026-06-22 |
| **hotspots-core** | multi-language code (TS/JS/Go/Python/Rust/Java/C#/C) | "Local Risk Score" combining complexity + churn | 1.27.0 (32 versions) | 2026-07-02 |
| **git-cognitive** | git history tooling | "Cognitive debt"/AI-commit-risk auditing over git history — distinct from classic cognitive-complexity scoring | 0.3.10 | 2026-07-06 |
| **lynx_eye** | multi-language code (JS/TS/Rust) | NLOC, cyclomatic complexity (CCN), token count via tree-sitter | 0.0.3 | 2025-12-23 (no activity since — borderline-maintained) |
| **complexipy** | **implemented in Rust, but analyzes Python only** | Fast cognitive-complexity analyzer for Python code | — | last commit 2026-07-03; 721 GitHub stars — most popular newer entrant found |
| **paiml-mcp-agent-toolkit / pmat** | multi-language code (17–20+ languages claimed) | McCabe cyclomatic + cognitive complexity "with AST precision," proprietary technical-debt grade | version numbers inconsistent across sources **[U]** | crates.io explicitly marks the old package name deprecated, redirecting to `pmat` |

**Not complexity tools (commonly confused with them) — verified [V]:**
- **ast-grep** — tree-sitter structural search/lint/rewrite, many languages, no complexity metrics. v0.43.0, 2026-05-25.
- **difftastic** — tree-sitter structural diff, 30+ languages, no metrics. v0.69.0, 2026-04-30.
- **topiary** — tree-sitter-query-based code formatter, no metrics. v0.7.3, 2025-12-31.

No verifiable WASM-based/web-hosted multi-language complexity calculator specific to 2024–2026 was found; only generic, unattributed JS calculators surfaced, none confirmed as WASM-backed or independently notable.

---

## 3. Multi-language complexity tools (non-Rust)

| Tool | Language impl. | Input | Verdict | Metrics | License | Status (Jul 2026) |
|---|---|---|---|---|---|---|
| **lizard** **[V]** | Python | ~25 languages incl. **Rust**, Go, C/C++, Java, JS/TS, Python, Swift, Kotlin, Erlang, Fortran, Solidity, Zig, etc. | multi-language code | NLOC, cyclomatic complexity (CCN, incl. modified-CCN option), token count, param count; also copy-paste detection | MIT | Active — v1.20.0 (2026-01-13) / v1.23.0 (2026-06-02) |
| **radon** **[V]** | Python | Python source only | single-language code (not multi-language) | Cyclomatic complexity, raw metrics, full Halstead suite, Maintainability Index | MIT | **Stalled** — last release 6.0.1, **2023-03-26**, no release since; author now points users to companion tool `xenon` for CI gating |
| **PMD** **[V]** | Java (+Kotlin/Apex modules) | Rule engine: Java, Apex, Kotlin, Swift, Modelica, PL/SQL, Velocity, JSP, WSDL, POM, HTML/XML/XSL (16+ languages); bundled **CPD** copy-paste detector covers 30+ languages incl. Rust | multi-language code | `CyclomaticComplexityRule` (category/java/design.xml), configurable thresholds; no native cognitive-complexity rule found | BSD-style | Very active — monthly cadence, v7.26.0 (2026-06-29) |
| **SonarQube / SonarSource Cognitive Complexity** **[V]** | Java (platform) | 40+ languages claimed; **Rust confirmed supported** (1.0–1.92) | multi-language code | Both classic Cyclomatic Complexity and SonarSource's proprietary Cognitive Complexity (Campbell whitepaper, 2021) | **Dual-licensed since 29 Nov 2024**: core platform (SonarQube Community Build) is LGPLv3; bundled language analyzers moved to **Sonar Source-Available License v1.0 (SSALv1)** — not OSI-approved | Active; free tier ~20+ languages, paid tiers add C/C++/Swift/Objective-C/COBOL/RPG/ABAP/etc. |
| **PyDriller — Delta Maintainability Model (DMM)** **[V]** | Python | Git-mining tool; DMM inherits Lizard's ~15-language coverage since it uses Lizard internally | git history tooling (complexity-adjacent via DMM) | `dmm_unit_size`, `dmm_unit_complexity`, `dmm_unit_interfacing` per commit | Apache-2.0 | Active — v2.9 (2025-09-06); DMM paper authored by **di Biase, Rastogi, Bruntink, van Deursen** (TechDebt 2019) — not Vasilescu as commonly assumed |
| **CodeScene / Code Health** **[V, partial]** | Commercial (Empear AB / CodeScene AB — naming transition not fully resolved [U]) | "30+ languages" claimed | multi-language code | Proprietary 1–10 "Code Health" score from 25+ smell factors including cyclomatic complexity, nested complexity, "Bumpy Road," Brain Method/Class | Proprietary — **not open source** | Active commercial product, ongoing 2026 feature releases (CodeHealth MCP Server, AI-refactoring extensions) |
| **eslint-plugin-sonarjs** **[V]** | JavaScript | JS/TS | multi-language code (2 languages) | Implements SonarSource's Cognitive Complexity as a standalone ESLint rule | — | Active, Sonar-maintained, v0.23.0 |
| **cognitive_complexity** (Melevir, Python) **[V]** | Python | Python only | single-language code | Approximate reimplementation of Campbell's Cognitive Complexity via Python `ast` | MIT | **Stale** — last release v0.0.4, 2019-11-06, no activity since |

---

## 4. Text/prose complexity libraries and Rust NLP ecosystem

### Prose complexity — verified tools **[V]**
| Tool | Language | Verdict | Metrics | License | Status (Jul 2026) |
|---|---|---|---|---|---|
| **textstat** (Python) | Python | prose/text | Flesch Reading Ease/Kincaid Grade, Gunning Fog, SMOG, ARI, Coleman-Liau, Linsear Write, Dale-Chall, Spache, McAlpine EFLAW, plus non-English formulas (Spanish, Arabic, Italian, German) | MIT | Very active — v0.7.13 (2026-02-18); v1.0.0 rewrite in alpha; now org-owned at github.com/textstat/textstat |
| **`textstat` (Rust crate)** | Rust | prose/text | Flesch, Gunning Fog, SMOG, ARI, Coleman-Liau — explicitly modeled on Python's textstat, `#![no_std]`, zero deps | MIT OR Apache-2.0 | **Brand new** — v0.1.1 created 2026-06-21/22, only 31 downloads at check time; unproven but real and fills a genuine gap ("Rust's other readability crates are article extractors, not scorers" per its own README) |
| **`readability` (Rust crate)** | Rust | any bytes / HTML (NOT a scorer) | HTML content-extraction (Readability.js port), **not** a reading-level metric | MIT | v0.3.0, 2023-12-20 — maintained-ish, 2+ years stale |
| **`gunning-fog` (Rust crate)** | Rust | prose/text | Gunning Fog only | MIT | v0.1.0, 2024-08-07, single release — crates.io keywords ("gamedev, graphics") mismatch suggests low-effort/likely-abandoned |
| **`syllable` (Rust crate)** | Rust | prose/text (component) | Syllable counting for reading-level calcs | MIT OR Apache-2.0 | v0.1.0, 2021-03-02 — stale |
| **`hyphenation` (Rust crate)** | Rust | prose/text (component) | Knuth-Liang hyphenation, many languages (TeX patterns) | Apache-2.0/MIT | v0.8.4, 2021-08-19 — stale |
| **`punkt` (Rust crate)** | Rust | prose/text (component) | Statistical sentence-boundary detection | MIT/Apache-2.0 | v1.0.5, 2019-02-26 — effectively abandoned |
| **`flesch` (crates.io)** | — | — | **Does not exist.** Confirmed no such crate on crates.io — do not cite. | | |

**Bottom line for Rust prose-complexity tooling:** thin. The ecosystem lacked a real Flesch/Fog scorer until the brand-new (days-old at time of writing) `textstat` crate; everything else is either a component (syllables/hyphenation/tokenization) or several years stale.

### Other prose/readability tools
| Tool | Verdict | Status |
|---|---|---|
| **Coh-Metrix** | prose/text | **Not open source** — gated request form at ASU SoLET Lab; the lab now foregrounds successor/sibling tools (T.E.R.A., TAALES, TAACO, SEANCE) rather than Coh-Metrix itself, suggesting its role is partly superseded, not flatly defunct. |
| **quanteda** (R) | prose/text | Active; readability moved to spin-off `quanteda.textstats` package. Version currency **[S]** — CRAN direct-fetch returned stale cached pages (2017/2020 snapshots); search corroborates v4.4 (2026-05-09) / textstats v0.97.2 (2026-05-09), not independently re-confirmed via a fresh page load. |
| **koRpus** (R) | prose/text | Still maintained but lightly (single maintainer): v0.13-8/0.13-9 around Nov 2025–Feb 2026 **[S]**. Computes Flesch/SMOG/LIX plus lexical diversity (TTR, HD-D, MTLD). |
| **spacy-readability** (PyPI) | prose/text | **Abandoned** — last release v1.4.1, 2019-01-29, no updates in ~7 years. |
| **TextDescriptives** (spaCy v3, PyPI: `textdescriptives`) | prose/text | The modern replacement worth recommending: readability, descriptive stats, dependency distance, coherence, information-theory metrics. v2.8.4 **[S]**, has a published paper (arXiv:2301.02057). |
| **LanguageTool** | prose/text (grammar/style, not a readability-formula tool) | Java, LGPL-2.1, 25+ languages, very active (81,857+ commits, 2.1k open issues, 2026 copyright). Does **not** expose Flesch/Fog-style scores as a first-class feature — it's a rule-based grammar/style checker, a different category from textstat-style tools. |

### Rust NLP/ML ecosystem for offline perplexity (July 2026)
| Tool | Verdict | Status |
|---|---|---|
| **rust-bert** | any bytes (via tokenized text) | **[S]** Latest known version 0.23.0; no evidence of a newer release; not formally archived but release cadence has clearly slowed. Oriented toward ready-made pipelines (classification/NER/summarization), less clearly suited to raw perplexity computation than candle. |
| **candle** (Hugging Face) | any bytes (via tokenized text) | **[S]** Latest `candle-core` 0.11.0, 5.6M+ downloads, 37 versions; very active (PR activity through June 2026). Ships its own perplexity example — the best-evidenced pure-Rust option for offline perplexity. |
| **tokenizers** (Hugging Face) | any bytes | **[S]** Up to v0.21.3, crate page updated 2026-04-27; actively maintained, core HF infrastructure. |
| **llama-cpp-2** (utilityai/llama-cpp-rs) | any bytes | **[S]** Actively maintained, ~v0.1.150, 128 published versions, 693k+ downloads — the most active llama.cpp Rust binding. Does not follow semver meaningfully (tightly pinned to upstream llama.cpp). llama.cpp itself ships a first-party `llama-perplexity` tool that any binding exposing the raw API can drive. |
| **llama_cpp_rs** (underscore variant) | any bytes | **[S]** Stale — no update in 2+ years; distinct, older project from llama-cpp-2. |
| **rustformers/llm** | any bytes | **[V-corroborated by two independent sources]** **Archived** — GitHub page title itself reads "[Unmaintained, see README]," archived 2024-06-24; README points to **Ratchet** and **mistral.rs** as modern alternatives. |
| **`perplexity` crate (crates.io)** | — | **False positive** — this is an unofficial Rust SDK for the Perplexity.ai API product, unrelated to language-model perplexity metrics. Flagging to prevent miscitation. |

---

## 5. Compression / NCD (Normalized Compression Distance)

| Tool | Verdict | Status |
|---|---|---|
| **CompLearn** | any bytes | **Effectively abandoned/historical.** `complearn.org` now redirects to an unrelated, apparently domain-squatted site — do not cite it directly. Real project lives at sourceforge.net/projects/complearn/ (BSD) and github.com/rudi-cilibrasi/libcomplearn, last commit 2015-08-13. Useful as prior art only. |
| **Python NCD libraries** | any bytes | **No actively maintained, dedicated PyPI package exists.** `pyncd` on PyPI is an unrelated graph-analysis tool (name collision). `ncdlib` does not exist on PyPI. GitHub-only NCD implementations found (marcoalmeida/ncdlib, DavyLandman/ncd) are both stale since 2013–2015. The closest real, maintained, packaged fallback is **`textdistance`** (PyPI, MIT, v4.6.3, 2024-07-16), which includes NCD-family algorithms (ArithNCD, RLENCD, BWTRLENCD, SqrtNCD, EntropyNCD) as one small feature category among 30+ distance metrics — not a dedicated NCD tool. |
| **zstd** (Rust) **[V]** | any bytes | Legitimate, widely used. v0.13.3, published 2025-02-20, 321M+ total downloads. MIT. |
| **flate2** (Rust) **[V]** | any bytes | v1.1.9, published 2026-02-03 — actively maintained, 547M+ downloads. MIT OR Apache-2.0. |
| **brotli** (Rust) **[V]** | any bytes | v8.0.3, published 2026-05-28 — actively maintained. BSD-3-Clause AND MIT. |
| **xz2** (Rust) **[V]** | any bytes | v0.1.7, published 2022-06-06 — **the stale outlier**, no release in ~4 years, though still heavily downloaded (56M+). MIT/Apache-2.0. |

All four Rust compression crates are trivially usable to build a homegrown NCD/compression-complexity estimator (compress concatenated vs. separate byte streams); none of them compute NCD natively.

---

## 6. Git/diff-analysis tooling for complexity trends over history

| Tool | Verdict | Status |
|---|---|---|
| **PyDriller** **[V]** | git history tooling | Python framework for mining git history (commits, diffs, developers, process metrics, DMM — see §3). Apache-2.0. Very active — v2.10 published **2026-07-01**, days before this report. |
| **git2-rs** **[V]** | git history tooling (building block) | Rust libgit2 bindings. v0.21.0, published 2026-05-18. MIT OR Apache-2.0. 100M+ downloads. General-purpose git library, not complexity-specific. |
| **gix** (gitoxide) **[V]** | git history tooling (building block) | Pure-Rust git implementation. v0.84.0, published 2026-05-26. MIT OR Apache-2.0. 34.9M+ downloads. Very active, GitoxideLabs/gitoxide not archived. |
| **git-of-theseus** **[V]** | git history tooling | Confirmed exists, not archived, but **stalled** — last commit 2023-11-25 (Apache-2.0). Tracks code survival/age/LOC-over-time via blame across history (Erik Bernhardsson). Usable as reference, not under active development. |
| **gitvoyant** (Cre4T3Tiv3) **[V]** | git history tooling | Python (97.6%), uses tree-sitter for JS/Java/Go. Temporal/time-series complexity-trend regression analysis on commit history. Last commit 2026-03-30, 84 stars — a genuinely new (2025–2026) entrant in this specific niche. |
| **debtmap** / **hotspots-core** / **git-cognitive** (see §2) **[V]** | git history tooling (as one feature among several) | These Rust tools fold git-history risk/churn analysis into their complexity scoring rather than being dedicated history-mining tools. |

**Confirmed:** neither **rust-code-analysis** nor **lizard** has a native git-history-walking mode — both are single-snapshot analyzers and must be externally driven (e.g., looped by PyDriller or a shell script) to produce complexity-over-time data.

---

## Explicit unresolved items (do not treat as settled fact)

1. Exact current crates.io version/publish date for **`complexity`** (rossmacarthur), **tokei**, and **scc** — a targeted re-verification attempt at the end of this session was blocked by the same persistent rate limit; only GitHub-derived staleness signals (last commit dates) are confirmed for `complexity`.
2. Whether **Empear AB** formally renamed to **CodeScene AB** — conflicting entity-name signals found, not resolved.
3. Full text/methodology of SonarSource's Cognitive Complexity white paper — landing page and third-party citations corroborate authorship (G. Ann Campbell) and framing, but the PDF itself is gated behind a lead-capture form and was not read directly.
4. Exact current version numbers for **quanteda**, **quanteda.textstats**, **rust-bert**, **candle**, **tokenizers**, and **llama-cpp-2** are search-corroborated **[S]**, not freshly fetched in this session — reasonably reliable but flagged per the instruction to mark uncertainty explicitly.
5. No concrete, independently verifiable WASM-based multi-language complexity calculator (web-hosted, 2024–2026) was found; absence-of-evidence, not evidence-of-absence.
6. `pmat` (successor to the deprecated `paiml-mcp-agent-toolkit`) has wildly inconsistent version numbers across different search snippets (0.26.4 through 2.214.0) — treat all version claims about it as unverified until directly fetched.
