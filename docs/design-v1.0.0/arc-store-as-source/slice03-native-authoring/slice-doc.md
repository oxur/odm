# Slice 03 -- native authoring commands (programmatic surface)

<!-- Name/title carries no document-role metadata, per ODD-0013 sec. 2.1. -->

**Arc:** Store-as-source-of-truth & native authoring -- **Kind:** code
**Opened:** 2026-08-03 -- **Design basis:** ODD-0026 sec. 2.5 (fork E) + ODD-0013 v2.6 (the
author-vs-odm field boundary) + slice 02 (authored nodes are already valid).

## Goal

Give odm a native path to **create and update a planning node's body and author-owned metadata
entirely through `./bin/odm`**, with the store node file as the source of truth and **odm owning the
`---` seam** -- the author never hand-edits fused YAML+markdown. After slice 02 an authored node is a
valid store citizen; this slice makes one *creatable and editable* from the CLI, closing the
`undeveloped-stub` gap (minting a child/slice becomes one command).

The single sentence this slice makes true: *`odm node new`/`node set`/`node set-body` create and
update an `origin: authored` node -- body as pure markdown, metadata as a structured partial -- with
odm fusing the two into the store file, validating the author's partial against the schema and the
author-vs-odm boundary before any write, and `check` staying green throughout.*

## The governing principle (ODD-0026 sec. 2.5)

**odm owns the seam. Every surface handed to an author is single-grammar** -- pure-markdown *body*
(no front-matter) or *structured metadata* -- and odm is the only thing that fuses them into the
on-disk node. This removes the exact surface where fused-YAML authoring corrupts. Two consequences
this slice implements: the **canonical metadata partial** (author-owned fields only, loadable as
JSON *or* TOML), and **validate-before-write** (the partial is checked against the schema and the
field boundary before the node is written; the body, being prose, is never parsed and cannot break
node parsing).

## Scope -- in (the programmatic surface: files + flags)

1. **The canonical metadata partial + dual JSON/TOML I/O.** One in-memory type carrying only
   author-owned fields (per ODD-0013 v2.6): `name`/title, `type`, `edges.part_of`, status intent,
   `tags`. Deserialize from `--metadata=<file>` dispatched on extension (`.json` -> `serde_json`,
   `.toml` -> `toml`; both deps already present). odm converts through the one canonical form -- an
   identical partial in either format yields an identical node.
2. **Validate-before-write (the invariant).** Before writing anything: validate the partial against
   the schema (valid `type`, well-formed `part_of`, etc.) **and** the author-vs-odm boundary --
   a partial that sets an **odm-owned** field (`id`, `number`, placement/`path`, or any of the
   `source`/provenance record) is **rejected with a clear error**, never silently accepted. Fail
   loudly; write nothing on failure.
3. **`odm node new <type> <name>` overhaul** (today it takes only type/name/parent and mints
   `Origin::Planned`):
   - `--metadata=<file.json|.toml>` supplies the author partial (merged over the odm-owned
     skeleton); `--content=<file.md>` (and/or `--body=<str>`, and/or `--from-file=<md>` as a content
     alias) supplies the body verbatim.
   - **bare `node new <type> <name>`** (no body) mints an **authored stub** and returns the ref --
     the skeleton-then-grow path; the body is filled later via `node set-body`.
   - Mints as **`origin: authored`** + `source: { class: authored }` (slice 02's model), with odm
     owning `id`/`number`/placement. `--dry-run` writes nothing; `--json` emits the created ref/shape.
4. **`odm node set <ref> <field> <value>`** -- field-addressed metadata edit of an author-owned
   field (`status` intent, `tags`, `edges.part_of`, `name`). Re-validates (schema + boundary);
   setting an odm-owned field is rejected. `--dry-run`/`--json`. The `---` block is never in the
   editable surface.
5. **`odm node set-body <ref> --from-file=<md>` / `--body=<str>`** -- replace the **whole body** as
   pure markdown; odm re-attaches the (unchanged) frontmatter on write. The author never sees or
   touches the seam. `--dry-run`/`--json`.
6. Everything created/edited this way **passes `check`** (authored nodes are valid per slice 02) and
   **reads back via `node show`** -- the round-trip that proves SS-4.

## Scope -- out (deferred to a follow-on: slice 07, interactive authoring ergonomics)

- **`node edit <ref> --body` / `--meta`** -- the interactive `$EDITOR` split surfaces (open only the
  stripped body, or only the metadata, in `$EDITOR`; re-attach on save). Needs an editor-spawn
  dependency + terminal handling -- a distinct concern from the file/flag surface here.
- **`node edit <ref> --section "## Heading" --from-file=<md>`** -- heading-anchored partial body
  edits. Needs markdown-section parsing; an ergonomic optimization for large docs, not on the
  SS-4/SS-6 critical path.

**Why this seam:** the file+flag surface here is the **automation/LLM path** -- it is exactly what
lets odm author its own planning docs (and what slice 04's cutover depends on), and it closes SS-4
(create **and** update, no hand-edit) and enables SS-6 (author an arc+slice via `./bin/odm`) on its
own. The `$EDITOR`/section surface is human ergonomics layered on top, plan-late (drawn when near),
and needs a new dependency this slice does not. Splitting here keeps slice 03 inside one context with
iteration headroom (sizing judgment, PROJECT-MANAGEMENT Part I).

- **No `./docs` deletion** (slice 04). **No read/query changes** (LLM arc). **No new node *type* or
  model change** -- the model is ODD-0026, already accepted; this is its CLI surface.

## Verification approach

Fixtures for each command (create dual-source; create skeleton; set metadata field; set-body;
boundary rejection; dual-format equivalence) plus an **end-to-end CLI round-trip**: `node new` an
authored node from a metadata+content pair, `node set`/`set-body` to update it, `node show` to read
it back, `check` exit 0 throughout -- the SS-4 shape. Cargo/exec rows attested -> CI (no macOS
toolchain in the CDC sandbox); structural rows reproduced by code read + fixture. The real leg (the
same round-trip on the live store) rides the operator's rebuilt binary, same pattern as prior slices.

## Exit criteria

SS-4 met (a node body created **and** updated via `./bin/odm`, no hand-edit, round-tripping through
`node show`); the metadata partial loads identically from JSON and TOML; validate-before-write
rejects odm-owned fields and schema-invalid partials before any write; created/edited nodes are
`origin: authored` and pass `check`; `--dry-run`/`--json` on each verb. Slice 04 (cutover) can then
rely on an odm-native authoring path, and the fresh-context DoD demo (SS-6) becomes runnable. The
interactive `$EDITOR`/section surface is disclosed as deferred (slice 07), not dropped.
