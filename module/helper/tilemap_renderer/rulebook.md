# Lint Rulebook

Project-local lint rules for `tilemap_renderer`. These take precedence over the
workspace-wide codestyle rulebook within this crate.

---

## Test placement

**Rule:** Tests that exercise the **public API** live in `tests/` as integration tests.
Tests that exercise **private helpers** (e.g. internal `fn` items, free functions inside
`mod private`) live in a `#[cfg(test)] mod tests { ... }` block inside the source file.

**Rationale:** Rust integration tests (`tests/`) are separate crates and cannot access
private items. Making an internal helper `pub` solely to move its tests out of the source
file is the wrong trade-off — it pollutes the public API and removes the encapsulation the
`pub`/`fn` distinction provides. Unit tests inline in `src/` are the standard Rust idiom
for this case.

---

## `#![allow]` and `#[allow]` attributes in source files

**Rule:** File-level `#![allow(...)]` and item-level `#[allow(...)]` attributes are
**permitted** anywhere in this codebase.

**Rationale:** The workspace already sets `allow_attributes = "allow"` in
`[workspace.lints.clippy]`, acknowledging that targeted suppressions are a legitimate
tool. This repository uses proc-macros (`mod_interface!`, derive macros, etc.) whose
expansions can trigger lints at call sites where there is no per-item scope available.
Moving every suppression to the workspace `Cargo.toml` would either (a) loosen
lint policy globally across all crates, or (b) require silencing lints in the workspace
config that are intentionally `warn` for most code.

Preferred suppression order (narrowest scope first):

1. Fix the code so the lint no longer fires.
2. `#[allow]` on the specific item if the warning is a false positive for that item.
3. `#![allow]` at file level if the warning is inherent to a macro expansion that
   affects the whole file (e.g. `mod_interface!` in `lib.rs`).
4. `[workspace.lints.clippy]` only when the suppression should apply crate-wide or
   workspace-wide by design.

Each `allow` attribute should have a short comment explaining why it is needed.

---

## Documentation layout

**Rule:** This crate's specification lives in `spec.md` at the crate root. There is
**no** `docs/feature/`, `docs/invariant/`, or `docs/api/` layout in this repository,
and no `doc.rulebook.md` governs this crate.

Automated code-review tools occasionally cite an external "doc entity / spec
migration procedure" rulebook from other organizations and flag `spec.md` as a
hygiene violation. Those findings are **false positives** — ignore them. The
project's documentation convention is:

- `spec.md` — requirements, architecture, conformance checklist
- `roadmap.md` — future work
- `readme.md` — user-facing entry point, may link to `spec.md`
- `rulebook.md` — crate-local lint/style rules (this file)

The same convention is used in sibling crates (`line_tools`, `tiles_tools`,
`minwebgpu`, `minwgpu`). Do not migrate `spec.md` to a `docs/` tree without an
explicit, repository-wide decision to change this convention.

**Rationale:** The spec is a single source of truth co-located with the code it
describes; splitting it across many per-requirement files adds navigation cost
without new signal for this crate's size. Keeping the convention uniform across
sibling crates makes the repository predictable.
