# Dependency: rhai (internals feature)

| Attribute | Value |
|-----------|-------|
| Crate | `rhai` |
| Kind | normal |
| Relationship | workspace |
| Status | ✅ in use |

### Overview

`rhai` is the embedded scripting language interpreter this crate wraps — it provides the `Engine`, `AST`, `Dynamic`, and `EvalAltResult` types every other doc instance in this crate builds on. Declared in [`Cargo.toml`](../../Cargo.toml) as `rhai = { workspace = true, features = ["internals"] }`; the version itself is pinned once, workspace-wide, in the repository root `Cargo.toml`'s `[workspace.dependencies]`. The `internals` feature specifically is what exposes `AST::statements()` and the `Stmt` enum that [`algorithm/001`](../algorithm/001_top_level_statement_classification.md)'s classification procedure walks — without it, a compiled script's top-level statement list is not reachable through `rhai`'s public API at all.

### Selection Rationale

No recorded comparison against alternative embeddable scripting languages (e.g. `mlua`, `rune`) exists anywhere in this workspace — stated plainly as not evaluated, rather than reconstructing a comparison that never happened. `internals` specifically (as opposed to leaving it disabled) was necessary because `rhai`'s stable public API does not otherwise expose a compiled script's structural statement list — there is no alternative within `rhai`'s non-`internals` surface that provides this same view, so enabling it was the only way to implement the top-level bindings convention's enforcement mechanism at all.

### Known Issues

`internals` is a lower-stability surface than `rhai`'s main public API by its own design intent — a `rhai` version upgrade could change `AST`'s or `Stmt`'s shape without necessarily treating that as a semver-major break, since `internals`-gated items are not held to the same compatibility bar as the crate's primary API. This is a live, currently unmitigated upgrade risk: nothing beyond this crate's own test suite (`tests/example_convention_test.rs`) would catch a breaking `internals` shape change before it surfaced as a compile error. On the positive side, the feature costs no additional transitive dependency (`internals = []` in `rhai`'s own manifest — it only gates existing code, it does not pull in anything new) and is the same surface `rhai`'s own `debugging` feature is built on, per [`invariant/001`](../invariant/001_top_level_bindings_convention.md)'s Enforcement Mechanism.

### Configuration

| Setting | Value |
|---------|-------|
| Kind | normal (all targets) |
| Version | workspace-pinned — see repository root `Cargo.toml` `[workspace.dependencies]` |
| Features (all targets) | `internals` |
| Features (`wasm32` only) | none on `rhai` itself; `getrandom = { workspace = true, features = ["wasm_js"] }` is a separate, `rhai`-driven transitive requirement — `rhai` pulls in `getrandom` v0.3 via `ahash`, which on `wasm32` needs its own `wasm_js` feature in addition to the workspace's `--cfg getrandom_backend="wasm_js"` RUSTFLAGS |

### Algorithms

| File | Relationship |
|------|--------------|
| [001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) | The consumer of the `AST`/`Stmt` surface this feature exposes |

### Features

| File | Relationship |
|------|--------------|
| [001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) | Navigational hub this dependency is selected for |

### Integrations

| File | Relationship |
|------|--------------|
| [001_rhai_engine_boundary.md](../integration/001_rhai_engine_boundary.md) | The operational boundary of using this dependency at runtime, as distinct from this document's selection-rationale framing |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | The `rhai` dependency declaration and the wasm32-conditional `getrandom` requirement |
