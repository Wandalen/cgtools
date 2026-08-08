# Workspace Rulebook

Workspace-wide conventions for `cgtools`. This is the single source of truth
for repository-level lint and style rules; every crate in the workspace
follows the rules below unless an explicit, well-justified crate-local
override is recorded under the crate's own roof.

---

## Documentation layout

**Rule:** Each crate's design documentation lives in `docs/`, organized as
typed doc entity instances (`docs/feature/`, `docs/invariant/`, `docs/api/`,
and other types as warranted by content) per the standard doc entity
framework in `doc_des.rulebook.md`. Do not use a monolithic `spec.md` file
at the crate root.

Applies to all crates in the workspace that carry design documentation.

Companion files per crate:

- `docs/` — requirements, architecture, and design documentation as typed
  doc instances; present only when the crate has content that warrants it
  (see doc_des.rulebook.md's Documentation Necessity Test)
- `roadmap.md` — future work
- `readme.md` — user-facing entry point, may link into `docs/`
- `rulebook.md` — crate-local lint/style rules **only when overrides are
  needed**; absent by default, since this workspace rulebook is authoritative

**Rationale:** Typed doc instances keep documentation navigable by design
dimension and cross-referenceable at instance granularity — a requirement,
an invariant, and an API contract each evolve independently and are easier
to keep current as separate, focused files than as sections buried inside
one growing document.

---

## Test placement

**Rule:** Tests that exercise the **public API** live in `tests/` as
integration tests. Tests that exercise **private helpers** (e.g. internal
`fn` items, free functions inside `mod private`) live in a
`#[cfg(test)] mod tests { ... }` block inside the source file.

**Rationale:** Rust integration tests (`tests/`) are separate crates and
cannot access private items. Making an internal helper `pub` solely to move
its tests out of the source file is the wrong trade-off — it pollutes the
public API and removes the encapsulation the `pub`/`fn` distinction
provides. Unit tests inline in `src/` are the standard Rust idiom for this
case.

---

## Test file size

**Rule:** Test source files have **no fixed line-count limit** in this
workspace. Files SHOULD be split by domain (compile slice, feature area,
anchor kind, etc.) when one of the following triggers emerges:

- a contributor reports concrete navigation difficulty,
- incremental compile time on a single integration-test binary becomes a
  measurable bottleneck,
- a coherent sub-domain has grown large enough that a dedicated file would
  reduce cognitive load for future readers.

**Rationale:** Line count is a poor proxy for maintenance cost. Many crates
here share substantial fixture surface (spec / scene builders, mock data,
extractors) that is far cheaper to keep co-located in one file than to
mirror across many small files via `tests/common/mod.rs` plumbing.
Splitting on domain boundaries when a real pain point appears yields more
useful files than splitting on an arbitrary line threshold.

---

## `#![allow]` and `#[allow]` attributes in source files

**Rule:** File-level `#![allow(...)]` and item-level `#[allow(...)]`
attributes are **permitted** anywhere in this codebase.

**Rationale:** The workspace already sets `allow_attributes = "allow"` in
`[workspace.lints.clippy]`, acknowledging that targeted suppressions are a
legitimate tool. This repository uses proc-macros (`mod_interface!`, derive
macros, etc.) whose expansions can trigger lints at call sites where there
is no per-item scope available. Moving every suppression to the workspace
`Cargo.toml` would either (a) loosen lint policy globally across all crates,
or (b) require silencing lints in the workspace config that are
intentionally `warn` for most code.

Preferred suppression order (narrowest scope first):

1. Fix the code so the lint no longer fires.
2. `#[allow]` on the specific item if the warning is a false positive for
   that item.
3. `#![allow]` at file level if the warning is inherent to a macro
   expansion that affects the whole file (e.g. `mod_interface!` in
   `lib.rs`).
4. `[workspace.lints.clippy]` only when the suppression should apply
   crate-wide or workspace-wide by design.

Each `allow` attribute should have a short comment explaining why it is
needed.
