---
id: 01M0JZ8RVKFG3258M0QQ006QQG
number: 798832902
type: slice
schema: slice/v1.1
name: SS-6 proof slice -- node command body editing
created: 2026-08-21
updated: 2026-08-21
origin: authored
reserved: false
source:
  class: authored
edges:
  part_of: 01M0JZ83ARPH8QE169B7TD6F30
---
# SS-6 proof slice -- node command body editing

This body was replaced with odm node set-body after creation. The code tree no longer contains the planning docs corpus, and this authored slice remains valid with the store as the source of truth.

Evidence:

- created with odm node new
- parented under proof arc #798832901
- body replaced with odm node set-body
- read back with odm node show
- validated with odm check