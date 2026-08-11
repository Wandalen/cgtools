# Invariant: Compiles to Renderer Commands Only

The compile layer's entire output is a stream of
`tilemap_renderer::commands::RenderCommand` — this crate touches no GPU, no
window, no platform API. Scene portability is therefore exactly the set of
`tilemap_renderer` backends, inherited for free.

### Scope

- **Purpose**: Pin that scene compilation targets the d2 command vocabulary and nothing lower — the tile stack's "compiles to d2" invariant.
- **Responsibility**: State the property, name the dependency-surface enforcement, and record what a violation would cost.
- **In Scope**: What the `compile` layer may produce and which crates `tilemap_scene` may depend on.
- **Out of Scope**: The command vocabulary's own guarantees (see `tilemap_renderer/docs/invariant/004_vector_representability_of_commands.md`); how a runner feeds commands to a `Backend` (see [../api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md)).

### Invariant Statement

Compiling a scene produces only `tilemap_renderer::commands::RenderCommand`
values (plus asset declarations from the same crate's vocabulary). No code
path in `tilemap_scene` names a GPU concept, a rendering backend, or a
platform binding; every visual effect a scene can express must be expressible
in the command vocabulary.

### Enforcement Mechanism

- **Dependency surface**: `Cargo.toml` depends on `error_tools`,
  `mod_interface`, `serde`, `ron`, `slotmap`, `tiles_tools`, and
  `tilemap_renderer` — no `min*` driver, no `web-sys`, no windowing or GPU
  crate. A GPU code path cannot exist without a dependency that would be
  visible in review.
- **Declared design**: `src/lib.rs`'s crate doc states the contract — "a
  compile layer that turns them into a stream of
  `tilemap_renderer::commands::RenderCommand`s consumable by existing
  backends".
- **Headless test suite**: the integration tests compile scenes and inspect
  command streams with no GPU or browser present — the suite itself would
  fail to build headlessly if the invariant broke.

### Violation Consequences

- A direct GPU/backend dependency would bind every scene file to one
  backend, forfeiting SVG export, terminal preview, and headless snapshot
  testing in one step — the properties the tile stack exists to keep.
- Effects added outside the command vocabulary (bypassing
  `tilemap_renderer`) would render in one runner and silently not exist in
  others, breaking the "parsable and interpretable everywhere" scene
  contract.

### Invariants

| File | Relationship |
|------|--------------|
| [004_deterministic_compilation.md](004_deterministic_compilation.md) | Sibling property: *what* is emitted is commands-only; *how* it is emitted is deterministic |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | The enforcing dependency surface — no GPU/platform crates |
| `src/compile/frame.rs` | Frame compilation emitting command-stream draws |
| `src/lib.rs` | Crate doc declaring the compile-to-commands contract |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | Compiles scenes and asserts on the produced `RenderCommand` stream headlessly — no GPU in the test environment |
| `tests/sorted_batching_test.rs` | Inspects batching structure of the compiled command stream, likewise headless |
