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

This crate follows the workspace-wide `spec.md` convention recorded in the
workspace root `rulebook.md` (section *Documentation layout*). No crate-local
override is in effect — this section exists only to point readers at the
authoritative workspace rule.
