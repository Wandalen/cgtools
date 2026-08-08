# Pattern: Script-as-Data

The declarative form of the L5 scene-script contract: the script is a pure
data document, and a deterministic compiler is the only thing that ever
executes it. The script cannot call the engine — the dependency arrow points
from compiler to document, never the reverse.

### Scope

- **Purpose**: Name and pin the script-form under which determinism is a construction guarantee rather than an authorial discipline.
- **Responsibility**: Define the pattern's guarantees, its costs, and its known use in the tile stack.
- **In Scope**: The pattern itself and the criteria for choosing it.
- **Out of Scope**: The imperative alternative (see [005_script_as_glue.md](005_script_as_glue.md)); the L5 layer contract both forms serve (see [../layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md)).

### Problem

L5's contract demands "same script → same frames" — that is what makes
scenes testable, diffable, and safely re-runnable. If the script is a
program with engine access, that guarantee becomes a discipline every
script author must individually maintain, and CI cannot prove it holds:
there is no general way to verify an arbitrary program is deterministic.

### Solution

Make the script data, not code:

- The canonical form is a serialized document (RON in the known use), with
  a schema the loader validates before anything executes.
- A compiler — ordinary, testable Rust — turns the document into engine
  commands. All behavior lives in the compiler.
- Every pseudo-random choice flows from a seed carried *in the document*,
  so randomness is part of the data, not of the run.

### Consequences

- **Determinism by construction**: the compiler is a pure function of
  document + seed — pinned for the known use by
  [`tilemap_scene` invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md).
- **GPU-free validation**: documents load and validate headless
  ([`tilemap_scene` invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md)),
  so CI proves the contract with snapshot tests, no browser required.
- **Diffable and toolable**: scenes are documents — version control, code
  review, and external editors all work on them directly.
- **Bounded expressiveness** (the cost): the schema is the ceiling. New
  behavior means extending the compiler, not just writing a cleverer script.

### When to Choose

Default choice for a new stack's L5 — reach for it first, and add
[glue](005_script_as_glue.md) only where data genuinely cannot express the
need. The determinism contract is much easier to keep when it is structural.

### Patterns

| File | Relationship |
|------|--------------|
| [005_script_as_glue.md](005_script_as_glue.md) | The contrasting form: trades this pattern's guarantees for expressiveness |

### Layers

| File | Relationship |
|------|--------------|
| [../layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md) | The layer contract this pattern is one realization of |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/tilemap_scene/src/scene.rs` | The document model (scene + seed) |
| `module/helper/tilemap_scene/src/compile/frame.rs` | The deterministic compiler |
