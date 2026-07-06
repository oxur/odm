---
number: 21
title: "A doc superseding a ghost"
author: "Anon"
component: engine
tags: [final]
created: 2026-03-01
updated: 2026-03-02
state: Final
supersedes: 999
superseded-by: null
version: 1.0
---

# A doc superseding a ghost

`supersedes: 999` has no target in the corpus → a reported warning; the node
still imports (its content is real), just without the dangling edge.
