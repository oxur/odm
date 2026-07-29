---
id: 01KYP5H8V4MNRF2TY2NFBMJ61P
number: 730075300
type: note
schema: note/v1.1
name: 'Measuring the Conceptual/Semantic Complexity of Written Prose: A Literature Survey'
created: 2026-07-25
updated: 2026-07-25
tags:
- research
origin: planned
reserved: false
source:
  paths:
  - docs/dev/research/0002-measuring-the-conceptualsemantic-complexity-of-written-prose-a-literature-survey.md
  class: dev-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
---
# Measuring the Conceptual/Semantic Complexity of Written Prose: A Literature Survey

> Raw research sweep (one of three) feeding **ODD-0021**
> (`docs/design/01-draft/0021-research-information-theoretic-complexity-of-text-code-and-prose.md`;
> formerly `workbench/complexity-research.md`). Compiled
> 2026-07-25 via parallel web research with primary-source fetches. Citations
> verified against primary sources where fetched directly; items marked
> "unverified" rely on secondary-source snippets. See the companion sweeps:
> `complexity-sweep-code-metrics.md`, `complexity-sweep-tooling.md`.

---

## 1. Surface Readability Formulas

These are the textbook baseline every complexity discussion has to position itself against. All operate on **surface counts** — sentence length, syllable count, character count, word-frequency-list lookups — with no model of meaning, argument structure, or novelty.

| Formula | Author/Year | Formula | Inputs |
|---|---|---|---|
| Flesch Reading Ease | Rudolf Flesch, 1948 | `RE = 206.835 − 1.015·(words/sentences) − 84.6·(syllables/words)` | sentence count, word count, syllable count |
| Flesch-Kincaid Grade Level | Kincaid et al., 1975 (US Navy) | `FKGL = 0.39·(words/sentences) + 11.8·(syllables/words) − 15.59` | same |
| Gunning Fog Index | Robert Gunning, 1952 | `0.4·[(words/sentences) + 100·(complex words/words)]` | complex words = ≥3 syllables |
| SMOG Index | G. Harry McLaughlin, 1969 | grade ≈ `3 + √(polysyllable count per 30 sentences)` | polysyllabic word count |
| Coleman-Liau Index | Coleman & Liau, 1975 | `CLI = 0.0588·L − 0.296·S − 15.8` (L = avg letters/100 words, S = avg sentences/100 words) | character counts only — no syllabification |
| Dale-Chall | Dale & Chall, 1948 (revised 1995) | weighted function of % words *not* on a curated 3000-word "familiar word" list, plus avg sentence length | requires a word list, not just counts |
| Automated Readability Index (ARI) | Smith & Senter, 1967 | `ARI = 4.71·(characters/words) + 0.5·(words/sentences) − 21.43` | characters, words, sentences |

**Algorithm sketch (generic):** tokenize into sentences → tokenize into words → count syllables (or characters) → plug into a linear regression fit originally calibrated against grade-level reading tests (e.g., McCall-Crabbs passages for Flesch/Kincaid).

**Known limitations (well documented in the literature):**
- They rely *exclusively* on surface features (sentence length, syllable/character counts) as proxies for difficulty, ignoring semantics, discourse coherence, vocabulary sophistication, and reader background knowledge — Klare's classic critique, extended further in later commentary. ([ResearchGate: "Readability formulas have even more limitations than Klare discusses"](https://www.researchgate.net/publication/220517614_Readability_formulas_have_even_more_limitations_than_Klare_discusses); [Effortmark, "seven reasons to avoid them"](https://www.effortmark.co.uk/readability-formulas-seven-reasons-to-avoid-them-and-what-to-do-instead/))
- They assume a uniform reader and ignore prior knowledge, motivation, and L1/L2 status.
- Different formulas disagree substantially on the same text; no single formula is authoritative, and cross-formula variance is itself informative (a "consensus grade level" — averaging several formulas — is a common practical mitigation). ([wordcounttool.com summary](https://www.wordcounttool.com/blog/writing/readability-grade-levels-explained))
- A short sentence with dense, novel technical concepts (e.g., "Set the CAP theorem aside; use CRDTs.") scores as "easy" despite being conceptually loaded — this is the core gap this whole research topic exists to fill.
- Recent NLP work (2023–2025) explicitly revisits whether readability formulas even correlate with human-judged difficulty or with automatic-text-simplification (ATS) quality metrics, and finds **low correlation** among all three angles (readability formulas, human judgment, ATS automatic metrics) — Cardon & Doğruöz, "Readability Measures and Automatic Text Simplification: In the Search of a Construct" (2025 preprint). ([arXiv:2511.09536](https://arxiv.org/pdf/2511.09536))

**Computability:** Trivially offline/CLI-computable — pure arithmetic over tokenization output. Off-the-shelf libraries: `textstat` (Python, [PyPI](https://pypi.org/project/textstat/), [GitHub](https://github.com/textstat/textstat)), `py-readability-metrics` ([GitHub](https://github.com/cdimascio/py-readability-metrics)), and R's `quanteda`/`koRpus`. Zero ML dependency, sub-millisecond per document.

**Strengths:** cheap, deterministic, interpretable, decades of calibration data, trivially embeddable in a CLI/lint tool.
**Weaknesses:** measures *reading effort of the sentence surface*, not conceptual load; easily gamed by chopping sentences; doesn't detect jargon density, novel abstractions, or argument complexity — exactly the gap the rest of this report addresses.

---

## 2. Information-Theoretic Approaches

### 2.1 Shannon entropy of English

Claude Shannon's **"Prediction and Entropy of Printed English,"** *Bell System Technical Journal* 30 (1951): 50–64, estimated English entropy via a guessing-game experiment (human subjects predicting the next letter given prior context), bounding entropy at roughly **0.6–1.3 bits/character** (commonly cited as ≈1.3 bits/character upper bound, 5.9 bits/word). ([Wiley DOI](https://onlinelibrary.wiley.com/doi/abs/10.1002/j.1538-7305.1951.tb01366.x); [Internet Archive scan](https://archive.org/details/bstj30-1-50))

- **Caveat (flag):** the original experiment used **a single subject**, which later researchers (e.g., entropy-rate re-estimation work using Mechanical Turk and modern LM-based extrapolation) have flagged as a weak statistical basis — later studies (Takahira, Tanaka-Ishii & Dębowski, *Entropy* 2016; PMC 2020 Mechanical-Turk-based re-estimation) revisit and refine the estimate using large corpora and modern compressors. ([MDPI *Entropy* 18(10):364](https://mdpi.com/1099-4300/18/10/364/html); [PMC7514546](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7514546/))

**Algorithm sketch:** estimate a probability model P over word/character sequences (n-gram, neural LM, or human-guessing game); entropy rate H = −(1/N)·log₂ P(sequence). Lower entropy = more predictable/repetitive text; higher entropy = more "surprising"/information-dense text — but note entropy conflates *unpredictability* with *conceptual difficulty*; a random-letter string has maximal entropy and zero meaning.

### 2.2 Cross-entropy / perplexity as a complexity proxy

Perplexity `PP = 2^H(model)` is the exponentiated cross-entropy of a language model against held-out text. ([topbots.com explainer](https://www.topbots.com/perplexity-and-entropy-in-nlp/); [thegradient.pub](https://thegradient.pub/understanding-evaluation-metrics-for-language-models/))

**Algorithm sketch:** run a pretrained LM (n-gram, or modern transformer) over the target document; average negative log-likelihood per token; exponentiate. High perplexity relative to a *general-domain* LM often flags unusual vocabulary/register (which correlates with, but isn't identical to, conceptual difficulty — an idiosyncratic style can be "surprising" to a general LM while being conceptually simple, and vice versa).

**Computability:** Offline-feasible with a local n-gram model (KenLM etc. — see [github.com/jhnwnstd/shannon](https://github.com/jhnwnstd/shannon) for an entropy-estimation tool built on KenLM) or a small local transformer (e.g., GPT-2-scale) — no cloud API needed, though quality scales with model size.

### 2.3 Compression-based complexity: NCD and "Clustering by Compression"

**Cilibrasi, R. & Vitányi, P. M. B., "Clustering by Compression,"** *IEEE Transactions on Information Theory* 51(4), 2005 (arXiv preprint 2003, [cs/0312044](https://arxiv.org/abs/cs/0312044); PDF at [math.ucdavis.edu](https://www.math.ucdavis.edu/~saito/data/acha.read.w17/cilibrasi-vitanyi_clustering-by-compression.pdf)).

Defines the **Normalized Compression Distance (NCD)**:

```
NCD(x, y) = [C(xy) − min(C(x), C(y))] / max(C(x), C(y))
```

where `C(·)` is the compressed size under a real-world compressor (gzip, bzip2) and `xy` is concatenation. NCD approximates the (non-computable) **normalized information distance**, which is itself grounded in Kolmogorov complexity — the compressor is a computable stand-in for the ideal (uncomputable) universal compressor. ([Wikipedia: NCD](https://en.wikipedia.org/wiki/Normalized_compression_distance))

**Applications:** language/phylogenetic tree reconstruction, general clustering across arbitrary domains, anomaly detection. For prose complexity specifically, a related idea (not from Cilibrasi & Vitányi directly, but a natural corollary) is: **a text's own compression ratio** (compressed size / original size under gzip) is a crude proxy for its redundancy — highly repetitive, boilerplate prose compresses well (low complexity); dense, information-rich, low-redundancy prose compresses poorly (high complexity). This is a real technique used informally in complexity-estimation tooling but no canonical peer-reviewed citation was found establishing "gzip ratio of a single document" as a *validated* complexity metric — flag as a plausible-but-unverified extrapolation of NCD/Kolmogorov-complexity ideas rather than a directly cited result.

**Known pitfalls of NCD in practice:** documented in "Common Pitfalls Using the Normalized Compression Distance: What to Watch Out for in a Compressor" ([ResearchGate](https://www.researchgate.net/publication/38357258_Common_Pitfalls_Using_the_Normalized_Compression_Distance_What_to_Watch_Out_for_in_a_Compressor)) — results are highly sensitive to compressor choice, window size, and input length; real compressors are crude approximations of the theoretical ideal and can give misleading distances for very short or very long inputs.

**Computability:** Extremely CLI-friendly — literally `gzip -9` and measure output size, no ML model needed. This is arguably the *most offline-friendly* complexity signal in this entire report.

### 2.4 "gzip beats BERT" (Jiang et al., ACL 2023 Findings) and its replication problems

**Jiang, Z., Yang, M., Tsirlin, M., Tang, R., Dai, Y., & Lin, J., "'Low-Resource' Text Classification: A Parameter-Free Classification Method with Compressors,"** Findings of ACL 2023, pp. 6810–6828, Toronto. DOI: [10.18653/v1/2023.findings-acl.426](https://doi.org/10.18653/v1/2023.findings-acl.426). ([ACL Anthology](https://aclanthology.org/2023.findings-acl.426/))

**Method:** combine gzip compression with a k-NN classifier (k=2) using NCD-like compressed-concatenation distance as the similarity metric. Claimed: competitive with non-pretrained DNNs on 6 in-distribution datasets; **outperforms BERT on all 5 out-of-distribution/low-resource-language datasets.**

**Documented replication problems (verified via direct fetch of the critique):**
- **Kenschutte (2023), "Bad numbers in the 'gzip beats BERT' paper?"** ([kenschutte.com/gzip-knn-paper](https://kenschutte.com/gzip-knn-paper/)) found a bug/non-standard choice in the paper's official code (`calc_acc` in `experiments.py`): for k=2, when the two nearest neighbors have different labels (a tie), the code marks the prediction correct if *either* label matches the true label — effectively reporting **top-2 accuracy**, not standard kNN(k=2) accuracy. Kenschutte's corrected re-implementation shows accuracy drops materially on several datasets — e.g., on KirundiNews, the gzip method goes from **best-performing to worst-performing** among the compared methods once ties are broken properly (paper: 0.905 → corrected: 0.858, vs. reported 0.891→0.835 on KinyarwandaNews).
- A **follow-up post** ("gzip beats BERT? Part 2: dataset issues, improved speed...") identifies additional dataset contamination issues (train/test overlap) in some of the benchmark datasets used. ([kenschutte.com/gzip-knn-paper2](https://kenschutte.com/gzip-knn-paper2/))
- A separate arXiv paper, **"Gzip versus bag-of-words for text classification with KNN"** ([arXiv:2307.15002](https://arxiv.org/pdf/2307.15002)), shows a much simpler bag-of-words + kNN baseline matches gzip's performance, undercutting the "compression captures something special" framing.
- GitHub issue thread on the original repo documents community discussion of the bug: [github.com/bazingagin/npc_gzip/issues/3](https://github.com/bazingagin/npc_gzip/issues/3).

**Relevance to prose complexity:** the underlying idea (compressed-concatenation distance as a similarity/complexity signal) is sound and grounded in real information theory (§2.3); the *specific* empirical claim that it beats BERT is contested and should be cited with the caveat attached.

**Computability:** the compression+kNN method itself is fully offline/CLI-friendly (no GPU, no pretrained weights) — this remains true regardless of the accuracy-reporting controversy.

---

## 3. Psycholinguistic / Cognitive Measures

### 3.1 Surprisal theory

- **Hale, J. (2001), "A Probabilistic Earley Parser as a Psycholinguistic Model,"** NAACL 2001 — introduced surprisal as a per-word processing-difficulty measure tied to parser state changes.
- **Levy, R. (2008), "Expectation-Based Syntactic Comprehension,"** *Cognition* 106(3): 1126–1177 — formalized surprisal theory: processing difficulty at word *wᵢ* is proportional to `−log P(wᵢ | w₁...wᵢ₋₁)`, i.e., its negative log-probability given context. Reading time is predicted to scale with this quantity.

This is a *token-level* difficulty measure, not directly a document-level "conceptual complexity" score, but it's the theoretical ancestor of using LM perplexity/negative-log-likelihood as a complexity proxy (see §2.2 and §5). It has been extensively retested in modern NLP: e.g., **"Testing the Predictions of Surprisal Theory in 11 Languages,"** *TACL* 2023, confirms the effect cross-linguistically using neural LMs as the probability source. ([arXiv:2307.03667](https://arxiv.org/abs/2307.03667); [MIT Press TACL](https://direct.mit.edu/tacl/article/doi/10.1162/tacl_a_00612/118718/Testing-the-Predictions-of-Surprisal-Theory-in-11))

**Uniform Information Density (UID) hypothesis:** speakers/writers prefer utterances where information (surprisal) is distributed *evenly* across the signal, avoiding both under- and over-loaded spans, to minimize risk of miscommunication under noise. Re-examined critically in **Meister, Pimentel, et al., "Revisiting the Uniform Information Density Hypothesis,"** EMNLP 2021. ([ACL Anthology](https://aclanthology.org/2021.emnlp-main.74/); [arXiv:2109.11635](https://arxiv.org/pdf/2109.11635)) — this paper finds mixed/qualified support for UID depending on operationalization, an important nuance for anyone wanting to build a "spikiness of surprisal across a document" complexity signal.

**Relevance for design docs:** a document with wildly uneven information density (a wall of boilerplate followed by one dense, jargon-packed paragraph) could in principle be flagged by computing per-sentence surprisal (via a local LM) and measuring its variance — this is a plausible novel application, not something found directly published for design-doc analysis specifically.

### 3.2 Idea density / propositional density

- **Origin: Kintsch, W. & Keenan, J. (1973), "Reading rate and retention as a function of the number of propositions in the base structure of sentences,"** *Cognitive Psychology* 5(3): 257–274 — defined idea density as (number of propositions)/(number of words); showed reading time and recall depend on proposition count, not just word count, holding word count constant.
- **CPIDR (Computerized Propositional Idea Density Rater):** automates counting via POS-tagging heuristics (roughly: verbs, adjectives, adverbs, prepositions, and conjunctions each usually signal one proposition). Key paper: **Covington, M. A. et al., "Automatic Measurement of Propositional Idea Density from Part-of-Speech Tagging,"** *Behavior Research Methods* (see [PMC2423207](https://pmc.ncbi.nlm.nih.gov/articles/PMC2423207/)). Widely used in clinical/cognitive-aging and aphasia research (e.g., idea density as an early marker of dementia risk — the famous "Nun Study" linking low idea density in young-adult writing to later Alzheimer's). ([PMC5345557](https://pmc.ncbi.nlm.nih.gov/articles/PMC5345557/); [Frontiers in Psychology 2024](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2024.1434506/full))

**Algorithm sketch:** POS-tag the text → apply rule set counting proposition-bearing categories → divide by total word count.

**Computability:** Fully offline-feasible — needs only a POS tagger (e.g., spaCy, NLTK), no LLM required. CPIDR is a standalone downloadable tool. This is one of the more directly *portable-to-a-CLI-tool* candidates for measuring conceptual density of a design doc, since "propositions per sentence" is a reasonable operationalization of "ideas packed into this paragraph."

**Strengths:** grounded in 50 years of cognitive-psychology validation; directly measures "how many distinct claims are packed in per word," which is closer to what a design-doc reviewer means by "dense."
**Weaknesses:** rule-based POS heuristics are a coarse proxy for actual propositional content; doesn't distinguish trivial propositions from conceptually hard ones; developed/validated mainly on spoken/narrative discourse and clinical transcripts, not technical/architectural prose — transfer to Markdown design docs is unvalidated.

### 3.3 Coh-Metrix

**Graesser, A. C., McNamara, D. S., Louwerse, M. M., & Cai, Z. (2004), "Coh-Metrix: Analysis of text on cohesion and language,"** *Behavior Research Methods* 36(2): 193–202 ([Springer](https://link.springer.com/article/10.3758/BF03195564)); see also Graesser, McNamara & Kulikowich (2011), *Educational Researcher* 40(5) ([SAGE](https://journals.sagepub.com/doi/abs/10.3102/0013189x11413260)).

Computes **over 200 measures** spanning: referential cohesion, causal/temporal/logical connectivity, syntactic complexity, word concreteness/frequency/familiarity, and latent semantic analysis (LSA)-based coherence between sentences/paragraphs. Unlike single-number readability formulas, Coh-Metrix explicitly targets multi-dimensional text difficulty including **cohesion gaps** (places where a reader must infer a bridging inference because the text doesn't state the connection) — this is much closer to "conceptual difficulty" than syllable counts. ([Wikipedia overview](https://en.wikipedia.org/wiki/Coh-Metrix))

**Computability:** Originally a web tool/licensed software (not fully open-source), limiting offline CLI use; some measures (POS-based, connective counts) are replicable with open NLP pipelines, but the full LSA-cohesion component requires a semantic-space model — feasible offline with local embeddings (e.g., a local sentence-transformer) as a substitute, though not identical to the original LSA implementation.

**Strengths:** the most empirically validated multi-dimensional model of text difficulty in the psycholinguistics literature; explicitly separates cohesion from other difficulty factors.
**Weaknesses:** heavy, many-featured, partially proprietary tooling; validated on educational/narrative text, not technical/software documentation.

### 3.4 Lexical diversity: MTLD and vocd-D

**McCarthy, P. M. & Jarvis, S. (2010), "MTLD, vocd-D, and HD-D: A validation study of sophisticated approaches to lexical diversity assessment,"** *Behavior Research Methods* 42(2): 381–392. ([Springer](https://link.springer.com/article/10.3758/BRM.42.2.381))

- **MTLD (Measure of Textual Lexical Diversity):** mean length of sequential word strings that maintain a Type-Token Ratio (TTR) ≥ a threshold (typically 0.72) before it "resets"; averaged over forward and backward passes (MTLDbi) to reduce order sensitivity. Designed specifically to be **less sensitive to text length** than raw TTR, a known flaw of naive diversity metrics.
- **vocd-D:** models the theoretical TTR-vs-text-length curve (approximating a hypergeometric distribution) and fits a D parameter; higher D = more diverse vocabulary independent of sample size.
- McCarthy & Jarvis recommend reporting **multiple** indices (MTLD, vocd-D/HD-D, and Maas) rather than relying on any single measure.

**Computability:** Fully offline-computable from tokenized text; no external model needed beyond a tokenizer. Implemented in R (`koRpus`), Python (`lexicalrichness`), and Coh-Metrix.

**Relevance:** lexical diversity is an imperfect complexity proxy — a design doc could have low lexical diversity (highly repetitive jargon, e.g., "node," "edge," "ULID" reused constantly) yet be conceptually dense; diversity and conceptual load are related but distinct axes.

---

## 4. Conceptual / Semantic Complexity Proper

### 4.1 Integrative complexity (Suedfeld & Tetlock) and AutoIC

**Origin:** Suedfeld, P. & Tetlock, P. E. (1977), "Integrative Complexity of Communications in International Crises," *Journal of Conflict Resolution* 21(1): 169–184 ([SAGE](https://journals.sagepub.com/doi/10.1177/002200277702100108)) — building on earlier Harvey/Hunt/Schroder conceptual-complexity theory. The construct: text is scored 1–7 on the degree to which it shows **differentiation** (recognizing multiple distinct dimensions/perspectives on an issue) and **integration** (making explicit connections among those differentiated dimensions). Score 1 = no differentiation or integration (flat, single-perspective); score 7 = fully integrated multi-perspective synthesis. Manual scoring requires trained human coders working from a published scoring manual (see the [conceptual/integrative complexity scoring manual chapter](https://www.cambridge.org/core/books/abs/motivation-and-personality/conceptualintegrative-complexity-scoring-manual/AFB11B389544D191A034C5E52CBF2224), and a 2014 retrospective/debate: Suedfeld, "Integrative Complexity at Forty," *Political Psychology* 35(5) ([Wiley](https://onlinelibrary.wiley.com/doi/abs/10.1111/pops.12206)), with a critical response by Tetlock, "Integrative Complexity Coding Raises Integratively Complex Issues," same issue ([Wiley](https://onlinelibrary.wiley.com/doi/abs/10.1111/pops.12207)) — flagging known scoring-reliability difficulties even among trained human coders, a caveat directly relevant to anyone hoping to automate this.

**AutoIC (automated version):** **Conway, L. G. III, Conway, K. R., Gornick, L. J., & Houck, S. C. (2014), "Automated Integrative Complexity,"** *Political Psychology* 35(5): 603–624 (per secondary sourcing — **secondary-sourced, not primary-verified**), and a later validation paper: **Conway, L. G. III, Conway, K. R., & Houck, S. C. (2020), "Validating Automated Integrative Complexity: Natural Language Processing and the Donald Trump Test,"** *Journal of Social and Political Psychology* ([open access](https://jspp.psychopen.eu/index.php/jspp/article/view/5261); project site [autoic.org](https://www.autoic.org/)).

**Algorithm sketch:** dictionary-based approach — a curated lexicon of >3,500 words/phrases empirically associated with differentiation (e.g., "on the other hand," "however," "depends on") and integration (e.g., "as a result of both," "in conjunction with"), each weighted by an empirically-derived probability of signaling complexity; score is computed from weighted counts of dictionary hits, normalized per unit of text.

**Computability:** Fully offline-CLI-feasible — it's a weighted lexicon lookup, similar in spirit to a sentiment-dictionary tool (LIWC-style). No LLM required, though modern reimplementations could use embeddings to fuzzy-match dictionary concepts.

**Strengths:** directly targets "does this text show multi-perspective reasoning and synthesis" — arguably the single closest published construct to what a design-doc reviewer means by "conceptually sophisticated" or "handles trade-offs well." Validated against human coders with reported above-chance correlation.
**Weaknesses:** dictionary approach is shallow (surface phrase-spotting can be fooled by rote use of hedge words without actual differentiation); domain-tuned for political/psychological discourse, not software-architecture prose — the dictionary would likely need re-tuning (e.g., "trade-off," "however," "on the other hand," "given the constraints of X, we chose Y" are exactly the phrases you'd want it to catch in an ODD/ADR).

### 4.2 Semantic network / concept-map density

No single canonical, widely-cited paper was found establishing "concept-map edge density" as a validated document-complexity metric in the way idea density or integrative complexity are validated. The general idea — extract a concept graph (entities + relations) from text (via NER + relation extraction, or via co-occurrence graphs) and measure graph metrics (node count, edge density, average path length, clustering coefficient) as a complexity proxy — is a natural technique used in various knowledge-graph/text-mining papers, but flagged as **methodologically plausible but not backed by a specific, citable, widely-recognized benchmark paper**. This is a gap worth being honest about rather than inventing a citation.

### 4.3 Embedding-based measures: semantic dispersion, topic entropy

- **Topic modeling foundation:** Blei, D. M., Ng, A. Y., & Jordan, M. I. (2003), "Latent Dirichlet Allocation," *JMLR* 3: 993–1022 — the canonical LDA paper (standard/uncontroversial attribution). A document's **topic entropy** — the Shannon entropy of its LDA topic-probability distribution — is a natural derived metric: low entropy = document is "about one thing" (topically simple/focused); high entropy = topically diffuse (could mean rich synthesis across topics, or could mean unfocused/rambling — entropy alone can't distinguish these).
- **Embedding-based topic/semantic-dispersion approaches** are an active, less standardized area: represent each sentence/paragraph as an embedding vector (e.g., sentence-transformers), then measure the **dispersion** (average pairwise cosine distance, or variance around the centroid) of those vectors across the document as a proxy for semantic breadth/complexity. General discussion exists of "embedding-based models overcoming limitations of probabilistic topic models" ([Wiley chapter, "Embedding Semantics in LDA Topic Models"](https://onlinelibrary.wiley.com/doi/abs/10.1002/9780470689646.ch10)) and of "document-topic entropy" as a described concept, but **no single seminal, widely-cited paper specifically proposing "embedding semantic dispersion" as a document-complexity metric was found** — flag as **an assembled/derived technique from well-established building blocks (LDA, embeddings, entropy) rather than a single citable "semantic dispersion" paper.**

**Computability:** Fully offline-feasible. LDA: `gensim` (Python), pure CPU. Embeddings: any local sentence-transformer model (e.g., MiniLM, ~80MB, CPU-inferable) — no API call needed. This is realistically the most implementable "conceptual complexity" signal for a CLI tool among the semantic-proper approaches, because the components (LDA, cosine distance, entropy) are all standard, small, and local.

**Strengths:** captures topical breadth/focus, which correlates with something reviewers mean by "this doc covers too much ground" or "this doc is unfocused."
**Weaknesses:** entropy/dispersion is symmetric — it can't distinguish "richly synthesizing many ideas" (good complexity) from "randomly jumping topics" (bad complexity, i.e., poor structure) — same ambiguity as topic entropy generally. Requires a trained topic model or embedding model as a dependency (larger footprint than pure-arithmetic readability formulas, but still local/offline).

### 4.4 Discourse-structure complexity: RST depth

**Mann, W. C. & Thompson, S. A. (1988), "Rhetorical Structure Theory: A Theory of Text Organization,"** *Text* 8(3) (the foundational RST paper; commonly cited via this title, secondary-sourced here but a well-established, uncontroversial attribution in the discourse-parsing literature). RST represents a document as a tree over **Elementary Discourse Units (EDUs)**, connected by ~23 possible rhetorical relations (e.g., ELABORATION, CONTRAST, CONDITION, ATTRIBUTION), each EDU pair marked as NUCLEUS (core) or SATELLITE (supporting).

**Complexity signal:** **RST tree depth** and **branching factor** are natural structural-complexity proxies — a flat list of unconnected EDUs (shallow tree) suggests low argumentative structure; a deep tree with many nested ELABORATION/CONTRAST/CONDITION relations suggests an argument that builds through multiple layers of qualification — closer to what "conceptually complex reasoning" means structurally. Referenced use: RST tree patterns have been shown to improve automated speech/writing **proficiency scoring** ([discussed generally in discourse-parsing surveys](https://arxiv.org/pdf/2309.04141); see also Ferracane et al./discourse-and-dependency-distance work on "Discourse Tree Structure and Dependency Distance in EFL Writing," TLT workshop 2021, [ACL Anthology](https://aclanthology.org/2021.tlt-1.10.pdf)).

**Computability:** RST *parsing itself* requires a trained discourse parser — this is nontrivial. Modern parsers are neural (e.g., DMRST — "Document-Level Multilingual RST Discourse Segmentation and Parsing," [arXiv:2110.04518](https://arxiv.org/pdf/2110.04518)) and can run locally/offline once the model is downloaded, but accuracy on non-narrative, technical (design-doc/Markdown) text is unvalidated — RST parsers are trained mostly on newswire (RST Discourse Treebank) and may not transfer cleanly to bullet-heavy, code-block-interspersed engineering prose.

**Strengths:** the only approach in this report that directly models *argumentative/explanatory structure depth*, which is arguably the closest formal analog to "how hard is this design doc's reasoning to follow."
**Weaknesses:** requires a discourse parser (heavier dependency than readability formulas or lexicon lookups); parser accuracy itself is imperfect (RST parsing remains a hard NLP task); untested on software-engineering-document genre.

---

## 5. LLM-Era Approaches (2022–2026)

### 5.1 LLM perplexity as a complexity/difficulty signal

Using a modern LLM's per-token negative-log-likelihood (perplexity) on a passage is a direct descendant of surprisal theory (§3.1) and cross-entropy (§2.2), just with a much stronger probability model. Recent work:

- **"Rethinking the Role of Text Complexity in Language Model Pretraining"** (2025, [arXiv:2509.16551](https://arxiv.org/html/2509.16551v1)) — studies how text complexity (operationalized via multiple measures including LM-based ones) affects pretraining dynamics; relevant background on how "complexity" gets operationalized in the LLM literature itself.
- Text-simplification evaluation work increasingly uses **the difference in normalized perplexity between an in-domain and out-of-domain LM** as a readability/difficulty signal, rather than (or alongside) classic formulas — noted in recent ATS survey/evaluation work.
- **Cardon & Doğruöz (2025), "Readability Measures and Automatic Text Simplification: In the Search of a Construct"** ([arXiv:2511.09536](https://arxiv.org/pdf/2511.09536)) is directly relevant and worth flagging prominently: they find that **readability formulas, human judgment of simplification quality, and automatic simplification-evaluation metrics correlate poorly with one another**, arguing the field lacks a clear, agreed *construct* of what "readability/simplicity" even means. This is a strong caution against treating any single automated score (formula-based or LLM-based) as ground truth for "complexity."

### 5.2 LLM-as-judge for text difficulty/complexity

- **"Toward Trustworthy Difficulty Assessments: Large Language Models as Judges in Programming and Synthetic Tasks"** (2025, [arXiv:2511.18597](https://arxiv.org/pdf/2511.18597)) — directly evaluates LLM-as-judge reliability for *difficulty* assessment specifically (adjacent domain: programming/synthetic tasks, not prose, but methodologically relevant).
- **General LLM-as-judge reliability findings** (from a 2024 survey, ["A Survey on LLM-as-a-Judge," arXiv:2411.15594](https://arxiv.org/html/2411.15594v1), and **"Judge Reliability Harness: Stress Testing the Reliability of LLM Judges"** [arXiv:2603.05399](https://arxiv.org/pdf/2603.05399)) document systematic biases relevant to any LLM-as-judge complexity-scoring pipeline: **position bias**, **verbosity/length bias** (LLM judges tend to rate longer responses as better/more sophisticated, a serious confound for "complexity" scoring since verbose ≠ conceptually complex), and **self-enhancement bias** (LLM judges favor text similar in style to their own outputs). Only the largest judge models show reasonable alignment with human graders, and reliability is sensitive to prompt design.
- **"How Reliable is Multilingual LLM-as-a-Judge?"** EMNLP 2025 Findings ([ACL Anthology](https://aclanthology.org/2025.findings-emnlp.587.pdf)) extends the reliability question cross-lingually — relevant if design docs are ever multilingual.

**Algorithm sketch (LLM-as-judge for complexity):** prompt an LLM with the document (or a rubric) asking it to rate conceptual complexity on a scale, optionally with chain-of-thought or pairwise comparison (A vs. B, which is more complex) rather than absolute scoring (pairwise comparison is generally found more reliable than absolute Likert-style scoring in the LLM-judge literature).

**Computability:** Perplexity-based scoring is offline-feasible with a local open-weight LLM (e.g., a small local model via `llama.cpp`). True LLM-as-judge with strong reliability generally requires a large model — practically this means either an API call (breaks "offline CLI" requirement) or a locally-hosted large open model (heavy but possible, e.g., a quantized 30B+ model on-device).

**Strengths:** LLMs bring real world-knowledge and can plausibly assess *conceptual* difficulty (jargon density, novel abstraction, argument sophistication) far better than surface formulas or dictionary lookups.
**Weaknesses:** documented biases (length, position, self-preference), non-determinism, cost, and lack of a standard validated benchmark for "conceptual complexity of technical/design prose" specifically — the reliability literature above evaluates LLM judges on general QA/task-difficulty, not on this specific construct, so treat any complexity number from an LLM judge as a **rough heuristic signal, not a validated measurement**, absent further domain-specific validation.

---

## 6. Measuring the Complexity of a Text CHANGE (Diff) Rather Than a Single Document

This is the least mature area of the six, with fewer directly-on-point canonical papers; most relevant work comes from **document revision detection** and **semantic-diff tooling** rather than a unified "diff complexity" literature.

### 6.1 Semantic document distance / revision detection

**Zhu, X., Klabjan, D., & Bless, P. N. (2017), "Semantic Document Distance Measures and Unsupervised Document Revision Detection,"** IJCNLP 2017 ([arXiv:1709.01256](https://arxiv.org/pdf/1709.01256); [ACL Anthology I17-1095](https://aclanthology.org/I17-1095/)). Proposes two measures:
- **wDTW (word-vector Dynamic Time Warping):** aligns two documents' word-embedding sequences using DTW, producing a distance that respects word order and semantic similarity (not just exact-match tokens).
- **wTED (word-vector Tree Edit Distance):** represents documents structurally and computes tree edit distance using embeddings for node substitution cost.

Framed as a minimum-cost-branching problem for detecting revision chains in a corpus (e.g., Wikipedia revision history) — directly relevant to "how different, semantically, is version N of a design doc from version N+1."

### 6.2 Semantically-informed edit distance

**"Semantically-informed distance and similarity measures for paraphrase/plagiarism identification"** ([arXiv:1805.11611](https://arxiv.org/pdf/1805.11611)) extends classic **Levenshtein edit distance** by weighting substitution cost by semantic similarity of the substituted words/phrases (rather than binary match/no-match), and by weighting insertions/deletions by their semantic "impact" rather than counting them uniformly.

**Time-sensitive semantic edit distance (t-SED):** "Analysing user identity via time-sensitive semantic edit distance," [arXiv:1901.05228](https://arxiv.org/pdf/1901.05228) — a related but domain-specific (social-media authorship) application of the same idea: edit distance weighted by embedding similarity rather than raw token identity.

### 6.3 Line-level vs. token-level diff granularity

General diff-algorithm literature (Myers diff algorithm, the standard behind `diff`/`git diff`) computes **minimum edit distance** at whatever granularity it's given (character, token, or line). A key practical finding: **token-level edit distance is often too fine-grained and can underestimate semantic importance of a change** — a single-token change ("MUST" → "SHOULD" in a spec) can be semantically enormous while registering as edit-distance 1; conversely a large-looking diff (reformatting, reordering) can be semantically null. Line-based diffing partially mitigates this by treating a changed line as one unit, but still conflates "cosmetic rewording" with "meaning-changing edit." ([Overview: "How Diff Algorithms Work"](https://upliftorch.com/tools/text-diff/en/blog/diff-algorithm-explained.html))

### 6.4 Semantic diff tooling (practitioner/engineering side, not academic)

**Graphtage** (Trail of Bits, 2020) — a general-purpose **semantic diff** tool for structured data (JSON, XML, etc.) that diffs based on tree edit distance over parsed structure rather than raw text, explicitly designed to avoid the "diff finds a spurious huge change because of reordering" problem. ([Trail of Bits blog](https://blog.trailofbits.com/2020/08/28/graphtage/)) Relevant as an *engineering pattern* for a design-doc diff tool: parse the Markdown/YAML-frontmatter structure first, diff the structured representation, and only fall back to raw text diff for prose spans.

**Synthesis for a "diff complexity" metric applicable to design docs:** No single canonical paper gives a ready-made "conceptual complexity of an edit" score. The literature suggests a defensible composite approach:
1. Compute standard line/token edit distance (cheap, exact) as a *magnitude* signal.
2. Compute embedding-based semantic distance between old and new versions (whole-doc or per-paragraph cosine distance on sentence embeddings) as a *meaning-change* signal, independent of magnitude.
3. Flag divergence between the two: large edit-distance + small semantic-distance = "cosmetic rewrite" (low true complexity of change); small edit-distance + large semantic-distance = "small but meaning-critical edit" (high true complexity of change, the "MUST→SHOULD" case) — this divergence-based framing is a natural synthesis of Zhu et al.'s semantic-distance work and standard diff tooling, **flagged as a derived/synthesized recommendation, not a directly-cited existing metric.**

**Computability:** All components are offline-feasible: Myers diff is what `git diff` already runs; sentence embeddings via a small local model (as in §4.3); DTW/tree-edit-distance computations are standard algorithms with no GPU requirement (wDTW/wTED in the Zhu et al. paper were run without exotic infrastructure).

---

## Summary Table: Computability for an Offline CLI Tool

| Approach | Offline CLI feasible? | Dependency weight |
|---|---|---|
| Readability formulas (§1) | Yes, trivially | None (arithmetic) |
| Compression ratio / NCD (§2.3) | Yes, trivially | `gzip`/`bzip2` only |
| N-gram/local-LM perplexity (§2.2, §2.4) | Yes | Small (KenLM or small local LM) |
| Idea density / CPIDR (§3.2) | Yes | POS tagger (spaCy/NLTK) |
| Lexical diversity: MTLD/vocd-D (§3.4) | Yes, trivially | Tokenizer only |
| Coh-Metrix-style cohesion (§3.3) | Partial | Needs LSA/embedding model; original tool not fully open |
| AutoIC-style dictionary scoring (§4.1) | Yes | Curated lexicon (would need domain re-tuning) |
| LDA topic entropy / embedding dispersion (§4.3) | Yes | Local topic model or small sentence-embedding model |
| RST discourse-depth parsing (§4.4) | Yes, but heavier | Neural discourse parser (e.g., DMRST); untested on technical prose |
| Large LLM-as-judge (§5.2) | Only with a large local open-weight model, or breaks "offline" via API | Large (7B–70B+ local model) |
| Semantic diff (wDTW/wTED, embedding-distance diffing) (§6) | Yes | Small embedding model + standard diff algorithm |

## Flagged Uncertainties (do not treat as settled fact without further checking)

1. **Conway et al. (2014) "Automated Integrative Complexity"** exact journal/page numbers — sourced only from secondary search snippets, not fetched from a primary bibliographic record. The 2020 validation paper (Conway, Conway & Houck, *JSPP*) was found and is open-access-verifiable.
2. **Mann & Thompson (1988) RST** — canonical and uncontroversial in the field, but not independently re-fetched from the original *Text* journal in this session; attribution is standard/safe but technically secondary-sourced here.
3. **"Semantic dispersion" and "concept-map density"** as named, citable metrics — no single seminal paper found; these appear to be composite/derived techniques built from well-established primitives (embeddings, LDA, entropy, graph metrics) rather than established named constructs with a canonical citation. Present them as such, not as if a specific paper coined them.
4. **Gzip compression ratio of a single document as a validated complexity metric** — a plausible corollary of Cilibrasi & Vitányi's NCD/Kolmogorov-complexity framework, but no peer-reviewed paper was found directly validating "compressed-size ratio of one document" (as opposed to *pairwise* NCD between two documents) as a complexity measure.
5. The "divergence between edit-distance and semantic-distance" framing for diff-complexity (§6.4) is a synthesis of Zhu et al. (2017) plus standard diff-tooling literature — not a directly cited existing metric. Flagged explicitly above.
