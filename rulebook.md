# Workspace Rulebook

Workspace-wide conventions for `cgtools`. Crate-local `rulebook.md` files may
extend or refine these rules for their own scope; they do not override
workspace-wide decisions recorded here.

---

## Documentation layout

**Rule:** Each crate's specification, when it has one, lives in a single
`spec.md` file at the crate root. Requirements, architecture notes, and
conformance checklists are co-located in that one file — **not** split across
a per-entity `docs/feature/`, `docs/invariant/`, or `docs/api/` tree.

Applies to all crates in the workspace. Current users of this convention:
`tilemap_renderer`, `line_tools`, `tiles_tools`, `minwebgpu`, `minwgpu`.

Companion files per crate:

- `spec.md` — requirements, architecture, conformance checklist
- `roadmap.md` — future work
- `readme.md` — user-facing entry point, may link to `spec.md`
- `rulebook.md` — crate-local lint/style rules

**Rationale:** A single co-located spec is the right trade-off for crates of
this size — splitting across many per-requirement files adds navigation cost
without new signal. Uniformity across sibling crates keeps the repository
predictable.

**Scope:** the "doc entity / spec migration procedure" convention used in
some other organizations (with a `doc.rulebook.md` and a per-entity `docs/`
tree) is out of scope for this workspace. The absence of a `doc.rulebook.md`
is intentional and follows from the rule above.
