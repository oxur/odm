---
id: 01KYP5H8P95DWTF1Y0242BTNA3
number: 727942200
type: note
schema: note/v1.1
name: 'Measuring Software Code and Code-Change Complexity: A Literature Review'
created: 2026-07-25
updated: 2026-07-25
tags:
- research
origin: planned
reserved: false
source:
  paths:
  - docs/dev/research/0001-measuring-software-code-and-code-change-complexity-a-literature-review.md
  class: dev-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
---
# Measuring Software Code and Code-Change Complexity: A Literature Review

**Scope:** Classical structural metrics, information-theoretic/compression-based measures, language-model "naturalness" approaches, diff/change-level complexity, empirical validity evidence, and estimation/LLM applications. Compiled July 2026 via multi-agent web research with primary-source verification (arXiv, ACM DL, IEEE Xplore, author pages) where fetches succeeded. Every item not independently confirmed against a primary source is explicitly flagged **[unverified]** — treat those as leads to re-check, not settled facts.

---

## 1. Classical Structural Metrics

### 1.1 Halstead's Software Science (1977)

**Citation:** Maurice H. Halstead, *Elements of Software Science*, Elsevier/North-Holland, 1977, 128 pp., ISBN 0-444-00205-7.

**Inputs:** Lexical token counts only — no AST or control-flow graph required in principle, though tools typically use a parser for reliable operator/operand classification.

**Formulas:**
- n1 = distinct operators, n2 = distinct operands; N1, N2 = total operator/operand occurrences
- Vocabulary: **n = n1 + n2**
- Length: **N = N1 + N2**
- Estimated length: **N̂ = n1·log2(n1) + n2·log2(n2)**
- Volume: **V = N·log2(n)**
- Difficulty: **D = (n1/2)·(N2/n2)**
- Effort: **E = D·V**
- Time: **T = E/18** (the divisor 18 is the "Stroud number," a psychological constant for mental discriminations/second, fitted from a handful of programs)
- Delivered bugs: **B = V/3000** (simplified) or **B = E^(2/3)/3000** (the form more often treated as canonical)

**Language dependence:** Nominally language-agnostic, but operationally language-dependent — no standard exists for what counts as an "operator" vs. "operand" across languages, which undermines cross-language/cross-tool reproducibility.

**Strengths:** Purely lexical (cheap to compute, no parsing required for a rough version); one of the first attempts to quantify a "size" that isn't just LOC.

**Weaknesses / critiques:**
- Shen, Conte, Dunsmore, "Software Science Revisited: A Critical Analysis of the Theory and Its Empirical Support," *IEEE TSE* SE-9(2):155–165, 1983, DOI 10.1109/TSE.1983.236460 — the central critical paper questioning the theory's empirical support.
- Hamer & Frewin, "M.H. Halstead's Software Science — A Critical Examination," ICSE '82, pp. 197–206 — found Halstead's underlying experiments poorly designed.
- Coulter, "Software Science and Cognitive Psychology," *IEEE TSE* SE-9(2):166–171, 1983 — argues the model is unsupported by cognitive psychology (targets the Stroud-number-18 assumption directly).
- Fitzsimmons & Love, "A Review and Evaluation of Software Science," *ACM Computing Surveys* 10(1), 1978, DOI 10.1145/356715.356717.
- Al-Qutaish & Abran, "An Analysis of the Design and Definitions of Halstead's Metrics," IWSM 2005, pp. 337–352 — units/dimensions of V, E, T are unclear.
- David Flater (NIST), "'Software Science' Revisited: Rationalizing Halstead's System Using Dimensionless Units," NIST Technical Note 1990, 2018, DOI 10.6028/NIST.TN.1990 — modern rigorous re-derivation; notes the Stroud number was fitted from only 12 machine-language programs.
- **[unverified]** Pre-1977 Halstead papers (1972/1975) sometimes cited as earlier origins — not independently confirmed. **[unverified]** A dedicated Fenton-authored critique specifically of Halstead (Fenton & Pfleeger's textbook reproduces a Halstead equation but is not itself a critique paper).

### 1.2 McCabe Cyclomatic Complexity (1976)

**Citation:** Thomas J. McCabe, "A Complexity Measure," *IEEE Transactions on Software Engineering*, Vol. SE-2, No. 4, December 1976, pp. 308–320, DOI 10.1109/TSE.1976.233837. (Full text fetched and formula verified directly.)

**Inputs:** Control-flow graph derived from parsed source — requires more than lexical tokens (McCabe's own tool, FLOW, parsed Fortran into basic blocks).

**Formula:** **v(G) = e − n + p** (edges, nodes, connected components); for a single connected procedure, **v(G) = e − n + 2**, which reduces to **v = π + 1** (number of decision points + 1) for structured programs. McCabe's convention: a compound predicate ("if c1 AND c2") counts as two decisions; an N-way switch/case contributes N−1. Recommended threshold: v(G) ≤ 10.

**Language dependence:** Conceptually language-agnostic (any imperative control-flow graph works), but practically ambiguous for constructs like short-circuit boolean operators or exception handling, where different languages/tools count decision points differently.

**Weaknesses / critiques:**
- Martin Shepperd, "A critique of cyclomatic complexity as a software metric," *Software Engineering Journal* 3(2):30–36, 1988, DOI 10.1049/sej.1988.0003 (full text fetched). Arguments: (1) weak theoretical foundation — ignores nesting depth and data complexity, weights all decisions equally; (2) v(G) behaves "bizarrely" under modularization, sometimes *increasing* when code is split into more/simpler modules; (3) survey of ~17 empirical studies found "erratic" results, several showing LOC outperforming v(G); (4) conclusion — for large classes of software, cyclomatic complexity "is no more than a proxy for lines of code."
- Jay, Hale, Smith, Hale, Kraft, Ward, "Cyclomatic Complexity and Lines of Code: Empirical Evidence of a Stable Linear Relationship," *Journal of Software Engineering and Applications* 2(3):137–143, 2009, DOI 10.4236/jsea.2009.23020 — analyzed 1.2M SourceForge files, found LOC predicts ~90% of CC's variance, concluding CC has little explanatory power beyond LOC. **Note:** this directly conflicts with Landman et al. 2016 below — see §4.1 for the unresolved contradiction.
- Basili & Perricone, "Software Errors and Complexity: An Empirical Investigation," *Communications of the ACM* 27(1):42–52, 1984, DOI 10.1145/69605.2085 — found error *density* decreasing with increasing complexity in their dataset.
- Fenton & Neil, "Software Metrics: Roadmap," ICSE 2000 Future of SE Track, pp. 357–370, DOI 10.1145/336512.336588 — cyclomatic complexity is "not significantly better [than LOC] as predictors" and "very strongly correlated to size metrics."
- Landman, Serebrenik, Bouwers & Vinju, "Empirical analysis of the relationship between CC and SLOC in a large corpus of Java methods and C functions," *Journal of Software: Evolution and Process* 28(7):589–618, 2016, DOI 10.1002/smr.1760 — 17.6M Java methods + 6.3M C functions; found **no strong linear correlation** between CC and SLOC — contradicts the Shepperd/Jay "CC≈LOC" line. The empirical picture is genuinely contested, not settled.
- Peitek, Apel, Parnin, Brechmann & Siegmund, "Program Comprehension and Code Complexity Metrics: An fMRI Study," ICSE 2021, IEEE Xplore doc 9402616 — modern neuroscience-based echo of Shepperd: traditional metrics like cyclomatic complexity do not properly track perceived difficulty or comprehension correctness (see §4.3).

### 1.3 SonarSource Cognitive Complexity

**Citation:** G. Ann Campbell, "Cognitive Complexity: a new way of measuring understandability," SonarSource whitepaper, https://www.sonarsource.com/docs/CognitiveComplexity.pdf. Current version 1.7 (Aug 2023); earliest dated revision in the changelog is v1.1 (Feb 2017); a companion blog post is dated Dec 2016. **The commonly cited "2018" date appears to correspond to a later revision (v1.3, March 2018) rather than first publication — flag this discrepancy.**

**Inputs:** AST/parse tree (needs to identify nesting structure and control-flow-breaking constructs, plus distinguish "shorthand" idioms).

**Rules:**
1. Ignore "shorthand" structures that don't add reading burden.
2. +1 for each break in linear flow: if/else if/else, ternary, switch, for/foreach, while/do-while, catch, goto/break/continue to a label, sequences of binary logical operators, each method in a recursion cycle.
3. +1 additional per nesting level for *nested* flow-breaking structures.

Notable design choices: a switch with all its cases counts only **one** structural increment (unlike cyclomatic complexity's per-case increment); a catch adds one point regardless of exception-type count; straight-line method-call chains are free; early returns/guard clauses don't increment.

**Rationale:** SonarSource argues McCabe's metric was designed to measure *testability* (minimum independent test paths), not human *understandability*, and weights all branch types equally regardless of nesting. Cognitive Complexity abandons the graph-theoretic model for rules calibrated to programmer intuition — penalizing nesting explicitly, not penalizing shorthand.

**Independent validation:**
- Muñoz Barón, Wyrich, Wagner, "An Empirical Validation of Cognitive Complexity as a Measure of Source Code Understandability," ESEM 2020, DOI 10.1145/3382494.3410636 (arXiv:2007.12520) — positive correlation with comprehension time/subjective ratings; mixed results on correctness.
- **[unverified in depth]** Lavazza, Abualkishik, Liu, Morasca, *Journal of Systems and Software* 197:111561, 2023 — title/venue confirmed, but full conclusions not independently verified.

**Weaknesses:** Rule set is heuristic/hand-tuned rather than derived from a formal theory; language-dependent in how "shorthand" constructs are enumerated per language; independent validation evidence is still thin (essentially one ESEM 2020 study plus one 2023 J.Syst.Softw. paper not fully verified here).

### 1.4 Maintainability Index

**Origin:**
1. Oman & Hagemeister, "Metrics for Assessing a Software System's Maintainability," ICSM 1992, pp. 337–344, DOI 10.1109/ICSM.1992.242525 (introduces the hierarchical model).
2. Oman & Hagemeister, "Construction and Testing of Polynomials Predicting Software Maintainability," *Journal of Systems and Software* 24(3):251–266, 1994, DOI 10.1016/0164-1212(94)90067-1 (derives the polynomial).
3. Coleman, Ash, Lowther, Oman, "Using Metrics to Evaluate Software System Maintainability," *IEEE Computer* 27(8):44–49, 1994, DOI 10.1109/2.303623 (the practitioner-facing paper that popularized "the" index).

**Inputs:** Composite of Halstead Volume (lexical), Cyclomatic Complexity (AST/CFG), LOC (raw), optionally comment percentage.

**Formula (original, per Coleman et al.):**
```
MI = 171 − 5.2·ln(aveV) − 0.23·aveV(g′) − 16.2·ln(aveLOC) + 50·sin(√(2.4·perCM))
```
The comment term was added later; the earliest version used Halstead Effort in place of Volume but was replaced because Effort proved non-monotonic. Microsoft's Visual Studio variant drops the comment term and rescales to 0–100:
```
MI = MAX(0, (171 − 5.2·ln(HalsteadVolume) − 0.23·CyclomaticComplexity − 16.2·ln(LOC)) × 100/171)
```

**Weaknesses / critiques:**
- Arie van Deursen (TU Delft), "Think Twice Before Using the Maintainability Index," 2014 blog post: original validation used only 11 industrial HP/DoD-contractor systems (C/Pascal, 1,000–10,000 LOC each) with no reported significance testing; coefficients were never recalibrated for OO languages despite tool vendors reusing them unchanged; multicollinearity among Volume, Cyclomatic Complexity, and LOC (all confounded with size) undermines the composite's added value over size alone.
- El Emam, Benlarbi, Goel & Rai, "The confounding effect of class size on the validity of object-oriented metrics," *IEEE TSE*, 2001, DOI 10.1109/32.935855 — many OO-metric-to-fault correlations shrink/vanish once size is controlled for. (Note: a published rebuttal, IEEE Xplore doc 1214331, disputes whether size is a true *confound* in the causal sense — this methodological debate is not fully resolved.)
- Sjøberg, Anda & Mockus, "Questioning Software Maintenance Metrics," ESEM 2012, DOI 10.1145/2372251.2372269 — four-company replication finding "sophisticated" composite metrics overrated relative to simple size measures.
- Andreas Zeller, commenting on van Deursen's post, called the sine-of-comments term "a most elaborate example of sophisticated curve fitting."

---

## 2. Information-Theoretic Measures of Code

### 2.1 Kolmogorov Complexity

**Definition:** K(x) = length of the shortest program (for a fixed universal machine) that outputs string x with no input. "The Kolmogorov complexity of a file is the length of the ultimate compressed version of the file" (Cilibrasi & Vitányi, 2005, Sec. IV).

**Origins:** Genuinely three-way priority — Ray Solomonoff (1960, 1964, *Information and Control*), Andrey Kolmogorov (1965, *Problems of Information Transmission*), Gregory Chaitin (*JACM*, submitted 1966/rev. 1968). Kolmogorov acknowledged Solomonoff's priority once aware of it.

**Incomputability:** K is not computable by any algorithm — proved via a Berry-paradox-style self-reference argument, and K is Turing-equivalent to the halting problem. Chaitin's incompleteness theorem shows no formal system can prove K(s) ≥ L beyond a system-dependent threshold. Consequence: in practice only *upper bounds* are obtainable (compress the string; total length of compressed data + decompressor is an upper bound on K) — this is the direct motivation for compression-based proxies (§2.2).

**Standard reference:** Ming Li and Paul Vitányi, *An Introduction to Kolmogorov Complexity and Its Applications*, Springer, 1st ed. 1993 (ISBN 0387940537), 2nd ed. 1997, 3rd ed. 2008.

**Inputs / language dependence:** Purely string-level (raw text); language-agnostic by construction, though what counts as "the string" (raw source vs. tokenized/normalized source) affects results.

**Software-specific application:** No canonical paper was found applying *raw* (uncomputed) Kolmogorov complexity directly as a software metric — expected, since it's incomputable. The literature moves straight to compression-based approximations.

### 2.2 Compression-Based Approximation: Normalized Compression Distance (NCD)

**Citation (verified against primary PDF):** Rudi Cilibrasi and Paul M.B. Vitányi, "Clustering by Compression," *IEEE Transactions on Information Theory* 51(4):1523–1545, April 2005, DOI 10.1109/TIT.2005.844059. Preprint: arXiv:cs/0312044. Author PDF: https://homepages.cwi.nl/~paulv/papers/cluster.pdf.

**Formula (verified verbatim from the paper):**
- Compression distance: **E_C(x,y) = C(xy) − min{C(x), C(y)}**
- **Normalized Compression Distance: NCD(x,y) = [C(xy) − min{C(x),C(y)}] / max{C(x),C(y)}**

where C(·) is compressed size under a fixed real-world "normal" compressor (idempotent, monotonic, symmetric, distributive). NCD approximates the theoretical Normalized Information Distance NID(x,y) = max{K(x|y),K(y|x)} / max{K(x),K(y)}, which relies on the noncomputable K. The paper proves NCD is a "quasi-universal similarity metric" (Theorem 6.3) satisfying metric inequalities when the compressor is normal.

**Inputs:** Raw byte/text streams — needs only a general-purpose compressor (gzip, bzip2, etc.), no parsing.

**Language dependence:** Language-agnostic by construction (operates on any byte string); this is its main appeal for cross-language comparison.

**Applications to source code found:**
- Ishio et al., "A Comparison of Code Similarity Analysers," *Empirical Software Engineering*, 2018, DOI 10.1007/s10664-017-9564-7, and "Cloned Buggy Code Detection in Practice Using Normalized Compression Distance" (IEEE) — NCD used directly for source-code clone/similarity detection.
- NCDSearch tool (Ishio et al.) — grep-like similar-code search using NCD and the faster Lempel–Ziv Jaccard Distance approximation: https://github.com/takashi-ishio/NCDSearch.

**[unverified]** No single seminal "first NCD-for-source-code" paper was found predating the Ishio line of work — the technique appears to have migrated into SE gradually through the 2010s.

**Strengths:** Language-agnostic, needs no parser, theoretically grounded (approximates a provable metric). **Weaknesses:** Only an upper-bound approximation of K; sensitive to choice of compressor and to input size (compressors have limited context windows, distorting results on large files); doesn't decompose into interpretable sub-factors the way Halstead/McCabe do.

### 2.3 Entropy-Based Code Metrics

**Token/symbol entropy of source code:**
- N. Chapin, "An Entropy Metric for Software Maintainability," *IEEE TSE* 18(11):1025–1029, Nov. 1992 — cited within Hassan (2009) as prior entropy-of-source-code work.
- W. Harrison, "An Entropy-Based Measure of Software Complexity," *IEEE TSE*, cross-referenced in Hassan's bibliography. **[unverified]** — the page range listed for this in secondary sources (1025–1029) is identical to Chapin's, which looks like a possible transcription artifact; re-verify directly against IEEE Xplore before citing.
- Marcin Cholewa, "Shannon Information Entropy as Complexity Metric of Source Code," IEEE, 2018 (IEEE Xplore doc 8005255) — computes Shannon entropy over syntactic token counts (keywords, literals, operators) across ten languages implementing quicksort; found lower entropy correlated with more compact/optimal code in that study. **[unverified]** exact venue/page numbers (fetch blocked).

**Inputs:** Token stream (needs tokenizer, not necessarily full AST). **Language dependence:** Token categories (keywords, operators) must be defined per language — moderately language-dependent.

### 2.4 Hassan's Entropy of Change (ICSE 2009) — the key change-level information-theoretic metric

**Citation (verified against full primary PDF):** Ahmed E. Hassan, "Predicting Faults Using the Complexity of Code Changes," *Proceedings of the 31st International Conference on Software Engineering (ICSE 2009)*, Vancouver, May 2009, pp. 78–88 (page range per secondary sources, not independently confirmed from a primary bibliographic record), DOI 10.1109/ICSE.2009.5070510. Full text: https://sailresearch.github.io/sail-website/data/pdfs/ICSE2009_PredictingFaultsUsingTheComplexityOfCodeChanges.pdf.

**Inputs:** Git/VCS history — per-commit file-change data over a defined time window. This is fundamentally a *change-level*, not static, metric.

**Formulas (verified from the paper):**
- Shannon entropy of change distribution: **H_n(P) = −Σ_{k=1}^{n} (p_k · log₂ p_k)**, where p_k is the fraction of modified lines attributed to file k in a period.
- Normalized Static Entropy (accounts for varying file counts n): **H(P) = −Σ_{k=1}^{n} (p_k · log_n p_k)** = H_n(P) / log₂n, bounded to [0,1].
- Adaptive Sizing Entropy (H′): same formula, normalized by count of *recently modified* files (a 6-month trailing window) rather than total system files.
- "Periods" defined three ways: time-based (e.g., 3-month quarters), modification-count-based (600 mods/period), or **burst-based** (a period ends after ~1 hour of commit inactivity) — the ICSE 2009 case study itself uses the burst method.
- History Complexity Metric per file j: **HCM_{a..b}(j) = Σ_i HCP_Fi(j)**, aggregating entropy contribution across periods, with three weighting variants (HCM1s/2s/3s) and a decay variant (HCM1d, decay factor φ=10, empirically tuned).

**Validation:** Six large OSS systems (NetBSD, FreeBSD, OpenBSD, Postgres, KDE, KOffice); linear regression predicting fault counts in years 4–5 from metrics in years 2–3. Entropy-based HCM1d reduced prediction error by roughly 13–42% (avg ~32%) versus a prior-modifications baseline, and outperformed even a prior-faults baseline in most systems.

**Interpretation:** Concentrated changes (low entropy, few files touched repeatedly) are easier to reason about and less fault-prone than changes scattered thinly across many files (high entropy) — entropy of change *scattering*, not entropy of code content, is the risk signal.

**Strengths:** Directly validated against real fault data across six large systems; captures a dimension (change scattering) that raw churn/LOC cannot. **Weaknesses:** Requires substantial commit history to be meaningful; period-definition choice (time/count/burst) affects results and isn't standardized; doesn't measure the complexity of the changed code itself, only its distribution.

### 2.5 "Naturalness of Software" and Language-Model Cross-Entropy

**Foundational paper:** A. Hindle, E. T. Barr, Z. Su, M. Gabel, P. Devanbu, "On the Naturalness of Software," *ICSE 2012*, Zurich, pp. 837–847, IEEE (acceptance rate 87/408 = 21.32%). Author PDF: http://softwareprocess.ca/pubs/hindle2012ICSE.pdf.

**Core claim:** Code, despite being formal, is statistically repetitive and predictable much like natural-language corpora, and this can be captured by the same n-gram statistical language models used in speech recognition/NLP.

**Metric:** Cross-entropy (equivalently, perplexity) of an n-gram LM trained on a large code corpus, evaluated over token sequences. Lower cross-entropy = more "natural"/predictable; higher = more surprising.

**Inputs:** Tokenized source code (token stream); needs a training corpus in the target language. **Language dependence:** Strongly language- and even corpus/style-dependent — a model trained on one codebase's idioms will score unfamiliar-but-valid idioms as "unnatural."

**Application:** Built a code-completion engine using the n-gram model, outperforming Eclipse's built-in completion in next-token prediction accuracy. Received the ICSE 2022 Most Influential Paper (10-year retrospective) award.

**Follow-on — naturalness as a defect signal:**
B. Ray, V. Hellendoorn, Z. Tu, C. Nguyen, S. Godhane, A. Bacchelli, P. Devanbu, "On the 'Naturalness' of Buggy Code," *ICSE 2016*, pp. 428–439, DOI 10.1145/2884781.2884848, arXiv:1506.01159 (verified from primary PDF/abstract). Studying ~8,296 bug-fix commits across 10 Java projects: **buggy lines are more entropic (less "natural") than correct code**, and entropy decreases once bugs are fixed. Ranking static-analyzer warnings (PMD, FindBugs) by entropy improved their cost-effectiveness for bug-finding. Framed as a cheap, largely language-independent complement to static analysis, not a replacement.

**Other confirmed/partially-confirmed follow-on work:**
- Allamanis, Barr, Devanbu, Sutton, "A Survey of Machine Learning for Big Code and Naturalness," *ACM Computing Surveys*, 2018, DOI 10.1145/3212695, arXiv:1709.06182 — major survey anchoring the entire naturalness-hypothesis research program.
- **[moderately verified]** Tu, Su, Devanbu (2014) — cache-based LM exploiting "localness" (code is even more repetitive within a file/module than globally); commonly cited as "On the Localness of Software," exact venue not independently confirmed here.

**Neural/LLM-era work (2020–2026):**
- Khanfir, Jimenez, Papadakis, Le Traon, "CodeBERT-nt: Code Naturalness via CodeBERT," arXiv:2208.06042, 2022 (verified from PDF) — replaces n-gram models with CodeBERT (masked-LM), quantifying naturalness via token-masking/prediction (exact match, embedding similarity, model confidence); evaluated on 2,510 buggy vs. non-buggy lines, directly extending Ray et al. 2016 into the neural era.
- **[found, not deeply verified]** "Dependency-Aware Code Naturalness," arXiv:2409.00747 (2024) — extends naturalness to code dependencies/structure rather than flat token sequences.
- **[found, not deeply verified]** "Bringing Structure to Naturalness: On the Naturalness of ASTs," arXiv:2504.08234 (2025) — applies naturalness/entropy to ASTs.
- **[title/abstract-level only]** "Rethinking Code Complexity Through the Lens of Large Language Models," arXiv:2602.07882 — proposes "LM-CC," using model uncertainty over code semantics as a complexity proxy intended to improve on cyclomatic complexity.
- **[found, not deeply verified]** "Enhancing LLM-Based Code Generation with Complexity Metrics: A Feedback-Driven Approach," arXiv:2505.23953 (2025) — uses complexity metrics as feedback in LLM code-gen loops.
- Related human-comprehensibility LLM-judgment work: "Human-Aligned Code Readability Assessment with Large Language Models," arXiv:2510.16579; "Relative Code Comprehensibility Prediction," arXiv:2510.03474 (2025) — these use LLMs as *raters/classifiers* of comprehensibility rather than repurposing raw perplexity, and report that automated readability outputs correlate poorly with actual human comprehensibility judgments (a direct empirical challenge to any LM-score-as-comprehension-proxy).
- Survey: "Natural Language Generation and Understanding of Big Code for AI-Assisted Programming: A Review," arXiv:2307.02503 (2023) — discusses naturalness/entropy measures within the broader "Big Code" LLM landscape.
- **Explicit gap:** no paper was found directly using LM perplexity/naturalness specifically for git-diff/commit-level risk scoring — searches for "LLM commit risk prediction" / "GPT code review risk estimation" via naturalness metrics did not surface a matching title. Flag as likely gap rather than confirmed non-existence.

**Critiques of perplexity/cross-entropy as a complexity proxy:**
- **Training-corpus/familiarity confound:** CodeBERT-nt's own paper acknowledges naturalness models are "sensitive to code patterns (and practices) encountered during training" — the metric conflates intrinsic complexity with how well-represented an idiom/library/style is in the training corpus.
- **Vocabulary/tokenization sensitivity:** perplexity is highly sensitive to vocabulary size and tokenization scheme (general NLP-metric critique, directly applicable to code's unusual mixed identifier/literal/keyword vocabulary).
- **Memorization/data contamination:** broader LLM literature (e.g., arXiv:2507.05578 "The Landscape of Memorization in LLMs"; arXiv:2606.12764 "Detecting Functional Memorization in Code Language Models") shows low perplexity often reflects memorization of near-duplicate training sequences rather than genuine simplicity — a serious confound when pretraining and evaluation corpora overlap (common in public code).
- **Weak correlation with human comprehension:** arXiv:2510.16579-related work reports automated readability/comprehensibility scores correlate poorly with actual developer comprehensibility judgments, especially for AI-generated code.

No single paper was found explicitly titled as "a critique of Hindle/Ray-style naturalness metrics" — these critiques are reconstructed from limitation sections and adjacent empirical work, i.e., convergent evidence rather than one settled rebuttal.

---

## 3. Change/Diff-Level Complexity

### 3.1 Code Churn

**Origin:** Munson & Elbaum, "Code Churn: A Measure for Estimating the Impact of Code Change," ICSM 1998, pp. 24–31, DOI 10.1109/ICSM.1998.738486 — churn as rate of change in a module's relative complexity across builds, on a ~300 KLOC/3,700-module embedded C system.

**Key validating paper:** Nagappan & Ball, "Use of Relative Code Churn Measures to Predict System Defect Density," ICSE 2005, pp. 284–292, DOI 10.1145/1062455.1062514. Full text (MSR PDF): https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/icse05churn.pdf.

**Inputs:** Git/VCS diff stats — lines added/deleted/churned per file/period, plus total file size for normalization.

**Finding (verified):** Using Windows Server 2003 vs. SP1 (44.97M LOC, 2,465 binaries, 96,189 files) — **absolute** churn measures fit defect density poorly (R²=0.052); **relative** (size-normalized) churn measures fit dramatically better (R²=0.811; test Pearson r=0.889, Spearman ρ=0.929); a discriminant model using relative measures classified fault-prone binaries at ~89% accuracy. **Takeaway: normalized churn, not raw churn, is what predicts defects.**

**Language dependence:** Language-agnostic (pure line-count diff statistics). **Strengths:** Cheap, needs only VCS history, well-validated at scale. **Weaknesses:** Purely size-based; doesn't capture *what kind* of change occurred (structural vs. cosmetic) or where in the code it landed.

### 3.2 Delta Maintainability Model (DMM)

**Citation:** di Biase, Rastogi, Bruntink, van Deursen, "The Delta Maintainability Model: Measuring Maintainability of Fine-Grained Code Changes," IEEE/ACM TechDebt 2019 (co-located with ICSE 2019), pp. 113–122, DOI 10.1109/TechDebt.2019.00030. (Note: authorship confirmed as di Biase et al. — an earlier guess attributing this to "Lin/Palomba/Bavota" was checked and found **incorrect**; do not cite that attribution.)

**Basis:** Extends the SIG Maintainability Model (Heitlager, Kuipers & Visser, "A Practical Model for Measuring Maintainability," QUATIC 2007) to commit granularity, using benchmark-derived thresholds per Alves, Ypma & Visser, "Deriving Metric Thresholds from Benchmark Data," ICSM 2010.

**Algorithm sketch:** Each changed method in a commit is classified low-risk vs. not, per three sub-dimensions:
- **Unit size** — method LOC ≤ 15 → low-risk
- **Unit complexity** — cyclomatic complexity ≤ 5 → low-risk
- **Unit interfacing** — parameter count ≤ 2 → low-risk

Then **dmm = good_change / (good_change + bad_change)**, where growth in low-risk code or shrinkage of high-risk code counts as "good." (The original paper also proposed coupling and clone dimensions; PyDriller's implementation omits these.)

**Inputs:** AST-level per-method metrics (size, CC, parameter count) computed before/after each commit — needs a parser (Lizard, in PyDriller's case), not just diff text.

**PyDriller implementation:** exposes `Commit.dmm_unit_size`, `.dmm_unit_complexity`, `.dmm_unit_interfacing`, computed via Lizard across ~15 languages, each returning 0.0–1.0 or `None`. Docs: https://pydriller.readthedocs.io/en/latest/deltamaintainability.html.

**Language dependence:** Depends on Lizard (or equivalent) having a parser for the target language; thresholds (15 LOC, CC≤5, ≤2 params) were derived from Java/general-purpose benchmarks and may not transfer cleanly to all languages/paradigms.

**Strengths:** Operates directly at change granularity (not just file-level aggregates); grounded in a validated static model (SIG) rather than ad hoc thresholds. **Weaknesses:** Binary low/high-risk classification per method loses gradation; thresholds are calibrated on specific benchmarks and may need re-derivation per ecosystem.

### 3.3 Just-in-Time (JIT) Defect Prediction

**Citation:** Kamei, Shihab, Adams, Hassan, Mockus, Sinha, Ubayashi, "A Large-Scale Empirical Study of Just-in-Time Quality Assurance," *IEEE TSE* 39(6):757–773, June 2013, DOI 10.1109/TSE.2012.70.

**Inputs:** 14 change-level metrics across five dimensions — **Diffusion** (NS = files changed, ND = directories changed, NF = subsystems changed, Entropy — reuses Hassan-style entropy of change distribution), **Size** (LA = lines added, LD = lines deleted, LT = lines in file before change), **Purpose** (FIX = is this a bug-fix commit), **History** (NDEV = number of developers who touched the file, AGE, NUC = number of unique changes to the file), **Experience** (EXP, REXP, SEXP — developer experience measures).

**Findings:** Across 11 open-source projects, 68% accuracy / 34% precision / 64% recall predicting defect-inducing commits; effort-aware analysis: reviewing only the top 20% of predicted-risky *effort* catches 35% of all defect-inducing changes — establishing a cost-effectiveness framing for change-risk triage.

**Related:** Mockus & Weiss, "Predicting Risk of Software Changes," *Bell Labs Technical Journal* 5(2):169–180, 2000 **[DOI unverified — cite cautiously]**; Kim, Whitehead, Zhang, "Classifying Software Changes: Clean or Buggy?", *IEEE TSE* 34(2):181–196, 2008 — 78% accuracy but on an artificially balanced sample, a limitation Kamei et al. explicitly critique.

**Review-effort prediction from diffs** (thinner literature): Baysal, Kononenko, Holmes, Godfrey, "The Influence of Non-Technical Factors on Code Review," WCRE 2013, pp. 122–131 — patch size, priority, and component affect review latency on WebKit. **[unverified]** A weak Spearman ρ≈0.24–0.25 correlation between diff size and merge time reported in an arXiv multi-language study, and a "predicting review completion time" arXiv paper (2109.15141) — exact authorship not independently confirmed. No mature, widely-cited "diff complexity" metric analogous to McCabe complexity exists specifically for changes; AST-edit-distance/CFG-diff measures appear mainly in automated-program-repair literature, not established JIT defect-prediction literature.

### 3.4 Tools Operationalizing Change-Level Metrics

- **PyDriller** (Spadini, Aniche, Bacchelli, ESEC/FSE 2018, DOI 10.1145/3236024.3264598) — exposes churn, commit/contributor counts, and DMM. https://pydriller.readthedocs.io
- **CommitGuru** — Rosen, Grawi, Shihab, "Commit Guru: Analytics and Risk Prediction of Software Commits," ESEC/FSE 2015, pp. 966–969, DOI 10.1145/2786805.2803183 — directly operationalizes Kamei et al.'s 13-metric JIT model as a per-commit risk score. (Live status of commit.guru unverified.)
- **CodeScene** (Adam Tornhill, *Your Code as a Crime Scene*) — change/temporal coupling and "hotspots" (complexity × change frequency); no cited academic link to Hassan's entropy work in vendor docs. https://codescene.io/docs
- **SonarQube** "New Code" period metrics scope bugs/smells/coverage/debt to diff lines only, with PR Quality Gates — vendor-documented, not academically validated in a peer-reviewed paper as far as located. https://docs.sonarsource.com

---

## 4. Empirical Evidence on Validity

### 4.1 Does Cyclomatic Complexity Add Anything Beyond LOC?

**Contested, not settled.** Two credible large-scale empirical studies reach opposite conclusions:
- Jay, Hale et al. (2009), *JSEA* — 1.2M SourceForge files: LOC explains ~90% of CC's variance; CC has "absolutely no explanatory power of its own" beyond LOC.
- Landman, Serebrenik, Bouwers & Vinju (2016), *J. Software: Evolution and Process* 28(7):589–618 — 17.6M Java methods + 6.3M C functions: **no strong linear correlation** found between CC and SLOC at method/function granularity.

Both are large-N, credible studies; the contradiction was not resolved in this research pass and should be treated as an open empirical question, not a settled "CC is redundant" verdict — the original Shepperd (1988) critique remains the most-cited foundational skepticism, but later large-scale replications diverge.

### 4.2 Confounding Effects (OO Metrics / Size)

El Emam, Benlarbi, Goel & Rai, "The confounding effect of class size on the validity of object-oriented metrics," *IEEE TSE*, 2001, DOI 10.1109/32.935855 — many OO-metric-to-fault-proneness correlations from prior validation studies shrink or vanish once class size is statistically controlled for, implying size is a confound driving apparent validity. A published rebuttal (IEEE Xplore doc 1214331) disputes whether size qualifies as a genuine *confound* in the strict causal sense (since size doesn't necessarily temporally precede the OO metrics) — this methodological debate itself is unresolved and worth flagging in any downstream argument.

### 4.3 Comprehension Studies: Eye-Tracking, EEG, fMRI

This is a real but small, active niche (roughly a dozen identifiable studies, concentrated around the Siegmund/Apel/Brechmann research group and a few independent groups):

- **Peitek, Apel, Parnin, Brechmann, Siegmund, "Program Comprehension and Code Complexity Metrics: An fMRI Study," ICSE 2021**, IEEE Xplore doc 9402616, PDF: https://www.tu-chemnitz.de/informatik/ST/publications/papers/ICSE21.pdf. n=19; examined program comprehension of short snippets at varying complexity via fMRI. Found activation in five working-memory/attention/language brain regions plus reduced default-mode-network activity — a genuine neural cognitive-effort signature exists — but the paper's own conclusion (echoed in a CACM commentary) is that **traditional metrics like cyclomatic complexity do not properly track perceived difficulty or comprehension correctness**, a modern replication of Shepperd's 1988 skepticism.
- "On the accuracy of code complexity metrics: A neuroscience-based guideline for improvement," *Frontiers in Neuroscience*, 2022 — proposes adjusting complexity metrics using neuroscience-derived comprehension signals.
- "Correlates of programmer efficacy and their link to experience: a combined EEG and eye-tracking study," ACM ESEC/FSE 2022, DOI 10.1145/3540250.3549084.
- "EEG as a potential ground truth for the assessment of cognitive state in software development activities," PMC10919648.
- "An eye tracking study assessing source code readability rules for program comprehension," *Empirical Software Engineering*, 2024.
- **[unverified/gap]** Direct large-N eye-tracking studies regressing cyclomatic-complexity/Halstead scores against fixation time or pupillometry at scale were **not found** — most eye-tracking work targets readability *patterns* or specific refactorings rather than classical metric scores directly. Flag this as a genuine literature gap.

### 4.4 Systematic Reviews

- Isong & Ekabua, "State-of-the-Art in Empirical Validation of Software Metrics for Fault Proneness Prediction: Systematic Review," *IJCSES* 6(6), 2015, DOI 10.5121/ijcses.2015.6601 — reviewed 29 empirical studies on CK/size metrics vs. class fault-proneness; found inconsistent metric-to-fault relationships across studies.
- Nguyen-Duc et al., "The impact of software complexity on cost and quality — A comparative analysis between Open source and proprietary software," arXiv:1712.00675 — meta-analysis aggregating Spearman correlations from 59 datasets/57 primary studies; CK metrics most-used but "not all of them are good quality attribute indicators"; no significant OSS-vs-proprietary difference found.
- **[unverified]** A broader SLR covering 106 papers (1991–2011) on complexity metrics vs. software quality was referenced in secondary search results but its exact author/venue could not be confirmed.

---

## 5. Estimation, Velocity, and LLMs

### 5.1 Complexity Metrics → Story Points / Velocity

**Literature is sparse to essentially absent at the peer-reviewed level.** Story points are explicitly defined in agile practice as a subjective composite of effort+complexity+uncertainty, not derived from static code metrics. No rigorous empirical paper was found directly regressing McCabe/Halstead scores against historical story-point velocity — what surfaced is practitioner content (Atlassian, ClickUp, etc.), not validated research. **Flag: could not verify any direct empirical link; treat the code-metric-to-story-point connection as folk practice, not established science.**

### 5.2 LLM-Based Software Effort Estimation (2023–2026) — active, fast-growing area

- Shetty, Balakrishnan, Xu, Xi, Yu, "Agile Story-Point Estimation: Is RAG a Better Way to Go?", arXiv:2603.06276 — evaluates retrieval-augmented LLM approaches for story-point prediction.
- "Llama3SP: A resource-efficient large language model for agile story point estimation," ScienceDirect, 2025 — fine-tunes Llama 3.2 via QLoRA on story-point datasets.
- "GPT2SP" — pretrained-LM-based story-point estimator, fine-tuned per project; referenced as a baseline/SOTA comparator across multiple 2024–2025 papers.
- "Large Language Models for Early-Stage Software Project Estimation: A Systematic Mapping Study," *Applied Sciences* (MDPI) 15(24):13099, 2025 — surveys primary studies 2022–2025 on LLMs for effort/size/user-story-quality/productivity estimation; peak publication years 2024 (11 studies) and 2025 (5 studies); models surveyed span GPT-2/3.5/4, BERT (most frequent, 9 studies), and newer entries (DeepSeek-V3.2, Kimi K2, Gemini Flash Lite, GPT-5 Nano).
- "Toward LLM-aware software effort estimation: a conceptual framework," *Frontiers in AI*, 2026, PMC13050940 — conceptual/framework paper, not empirical.
- **Reported pattern across these:** modern LLMs show "moderate correlation" with actual human effort out-of-the-box; few-shot project-specific examples (as few as 5) substantially improve prediction/ranking performance.

### 5.3 Monte Carlo / PERT Forecasting + Code Complexity

**Literature appears sparse to nonexistent at the peer-reviewed level.** Everything found on Monte Carlo simulation for agile forecasting is practitioner-oriented content (blog posts, vendor material) applying simulation to historical velocity/throughput/cycle-time distributions — none of it ties simulation inputs to static code-complexity metrics. PERT's classic three-point estimate ((O+4M+P)/6) is well documented as a generic technique but not integrated with complexity metrics in any academic source found. **Flag explicitly: no academic literature found connecting Monte Carlo/PERT forecasting to code complexity metrics — this looks like either an open research gap or a purely practitioner-tooling concept (e.g., Jira/Azure DevOps forecasting features) lacking empirical validation.**

---

## Summary Table

| Approach | Inputs | Language-dependent? | Key citation | Validated against defects/effort? |
|---|---|---|---|---|
| Halstead Software Science | Token counts (operators/operands) | Weakly (classification ambiguous) | Halstead 1977 | Contested (Shen/Conte/Dunsmore 1983 critical) |
| McCabe Cyclomatic Complexity | Control-flow graph | Weakly | McCabe 1976 | Contested — Shepperd 1988 vs. Landman 2016 disagree |
| SonarSource Cognitive Complexity | AST | Yes (per-language rule tuning) | Campbell 2016/2017/2023 | Partial (Muñoz Barón et al. 2020, ESEM) |
| Maintainability Index | Halstead V + CC + LOC (+comments) | Weakly | Coleman et al. 1994 | Weak (van Deursen 2014 critique; multicollinearity) |
| Kolmogorov Complexity | Raw string | No (but incomputable) | Kolmogorov 1965 / Li & Vitányi | N/A (theoretical) |
| NCD (compression) | Raw bytes | No | Cilibrasi & Vitányi 2005 | Used for clone detection, not defect prediction directly |
| Token entropy | Token stream | Moderate | Chapin 1992, Cholewa 2018 | Thin evidence |
| Hassan entropy-of-change | Git history (per-commit file changes) | No | Hassan ICSE 2009 | Yes — 13–42% error reduction vs. baselines |
| N-gram naturalness (cross-entropy) | Token stream + corpus | Strong (corpus/style-dependent) | Hindle et al. ICSE 2012 | Yes for code completion; Ray et al. 2016 for defects |
| Neural naturalness (CodeBERT-nt etc.) | Token stream + pretrained model | Strong (training-corpus dependent) | Khanfir et al. 2022 | Preliminary (2,510-line study) |
| Code churn (raw) | Git diff line counts | No | Munson & Elbaum 1998 | Weak (R²=0.052, Nagappan & Ball 2005) |
| Relative code churn | Git diff line counts / size | No | Nagappan & Ball 2005 | Strong (R²=0.811) |
| DMM | Per-method AST metrics, pre/post commit | Yes (parser-dependent) | di Biase et al. 2019 | Derived from validated SIG model; direct defect validation not separately confirmed here |
| JIT defect prediction (14 metrics) | Git history + diff + developer data | No | Kamei et al. 2013 | Yes — 68%/34%/64% acc/prec/recall |
| LLM story-point estimation | Historical tickets + text | Language-of-description dependent | Shetty et al. 2026, Llama3SP 2025 | Moderate correlation reported |
| Monte Carlo/PERT + code complexity | — | — | **No literature found** | Not established |

---

## Master List of Explicitly Flagged Unverified Items

- Pre-1977 Halstead papers (1972/1975) as earlier origins.
- A dedicated Fenton-authored critique specifically of Halstead metrics.
- Exact first-publication date of the SonarSource Cognitive Complexity whitepaper (evidence points to late 2016/early 2017, not 2018).
- Full conclusions of Lavazza et al. 2023 (*J. Systems and Software* 197:111561) — venue/authorship confirmed, content not independently verified.
- W. Harrison's exact IEEE TSE page range (identical to Chapin's in secondary sources — possible transcription artifact).
- Exact venue/page numbers for Cholewa 2018 (Shannon entropy of source code).
- Exact page range for Hassan ICSE 2009 (78–88, sourced from secondary listings, not independently confirmed from a primary bibliographic record).
- Tu, Su & Devanbu's "localness" paper — exact title/venue not independently confirmed (commonly cited as ICSE/FSE 2014).
- Authorship/venue details for arXiv:2409.00747, arXiv:2504.08234, arXiv:2602.07882, arXiv:2505.23953 — found via search, not deeply verified.
- Existence of any paper using LM perplexity/naturalness specifically for git-diff/commit-level risk scoring — apparent gap, not confirmed non-existence.
- Mockus & Weiss (2000) DOI.
- Exact authorship of the arXiv review-completion-time and diff-size/merge-time correlation studies (including arXiv:2109.15141).
- CodeScene/SonarQube claims of academic grounding for their change-level features (vendor-documented, not peer-reviewed as far as located).
- The "106 papers, 1991–2011" systematic literature review — author/venue unconfirmed.
- Any direct empirical study regressing static complexity metrics against historical story-point velocity — appears not to exist in peer-reviewed literature.
- Any peer-reviewed work connecting Monte Carlo/PERT forecasting to code complexity metrics — appears to be a genuine gap; only practitioner content found.
- The Jay et al. (2009) vs. Landman et al. (2016) contradiction on CC-vs-LOC correlation was not resolved to a single consensus; both are reported as competing, credible, large-N findings.
