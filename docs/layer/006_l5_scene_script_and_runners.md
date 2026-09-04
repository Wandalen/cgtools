# Layer: L5 Scene Script and Runners

The top of the ecosystem: scenes as *scripts* — parsable, interpretable,
and deterministic (the same script must produce the same frames) — plus the
runners that execute them interactively or off-screen. This layer is where
"scene-as-script" from ADR-001 lives.

### Scope

- **Purpose**: Define the script layer's contract (determinism above all) and record its current occupants.
- **Responsibility**: Name the two contract halves (script + runner) and which stacks have each today.
- **In Scope**: `tilemap_scene`'s compile/runner path; `scene_script`'s Rhai glue and its off-screen runner-mode realization; the reserved `d3_scene` slot.
- **Out of Scope**: The declarative data itself (see [005_l4_scene_model.md](005_l4_scene_model.md)); engine internals (see [004_l3_stack_engine.md](004_l3_stack_engine.md)).

### Contract

- **Deterministic**: same script + same seed → same frame sequence. This is
  what makes scripts testable, diffable, and safely re-runnable — pinned for
  the tile stack by
  [`tilemap_scene` invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md),
  and for `scene_script` by its own off-screen determinism tests (see
  Occupants Today below).
- **Two runner modes**: interactive (browser/window loop) and off-screen
  (headless compile → declarative output or snapshot) — the off-screen mode
  is what proves determinism in CI.

### Occupants Today

| Crate | Stack | Role |
|-------|-------|------|
| `tilemap_scene` | tile | Script-as-data: RON scenes compiled deterministically to `tilemap_renderer` commands via `compile/frame.rs`, then executed by its own `Renderer` (`src/renderer.rs`, ~24 dedicated tests across `renderer_test.rs`/`renderer_cache_test.rs`) — the runner half of this layer's compile/runner path, documented in [algorithm/002](../../module/helper/tilemap_scene/docs/algorithm/002_scene_rendering_pass.md) and [api/001](../../module/helper/tilemap_scene/docs/api/001_renderer_integration_api.md); headless snapshot tests are the CI proof ([invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md), [invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md)) |
| `scene_script` | d2 | Hosts both script forms, per-script rather than per-crate: `pingpong_animation.rhai` is script-as-glue (imperative `main()` driving a registered vector binding — this particular script calls no tween binding, though the engine exposes one); the orrery example's `scene.rhai` (`examples/orrery/webgpu`, evaluated through this crate's engine) is script-as-data (a pure literal document — zero engine calls). Rhai bindings (`vector_binding`, `tween_binding`, `engine_build()`) expose math + tween vocabulary to scripts that choose to call them; `top_level_lint` checks top-level *shape* only (imperative code confined to `main()`), never whether a script calls the engine — see [pattern/005](../pattern/005_script_as_glue.md)'s boundary-case note; a third tracked example, `f32x2_vector_arithmetic.rhai` (`examples/scene_script/f32x2_vector_arithmetic`), is that boundary case in concrete form — a `let`/`let`/trailing-expression sequence that reads as shape-declarative yet calls the registered `f32x2(...)` constructor and operator overloads, so it is script-as-glue in substance despite the declarative shape (matching [pattern/005](../pattern/005_script_as_glue.md)'s own analysis); like the other two, it carries its own dedicated determinism test — [`determinism_test.rs`](../../examples/scene_script/f32x2_vector_arithmetic/tests/determinism_test.rs); all three tracked examples carry off-screen, CI-run determinism tests proving the Contract above — the one just named, [`simulation_test.rs`](../../examples/scene_script/pingpong_animation/tests/simulation_test.rs) (glue), and the orrery example's [`scene_test.rs`](../../examples/orrery/webgpu/tests/scene_test.rs) (data) |
| `d3_scene` | d3 | Reserved (`module/blank/d3_scene/`) — no d3 script layer exists yet |

The two existing occupants embody the layer's two script forms, though not
on a strict one-crate-one-pattern basis: `tilemap_scene` is
[script-as-data](../pattern/004_script_as_data.md) throughout, while
`scene_script` (d2) hosts both — most of its tracked examples are
[script-as-glue](../pattern/005_script_as_glue.md), but the orrery example's
`scene.rhai` is script-as-data, following that pattern's own recommendation
to default to data and add glue only where expressiveness is actually
needed. A future d3 script layer should make the same per-script choice
deliberately — the patterns record the criteria and the default
recommendation.

### Off-Screen Runner Realization

`scene_script`'s off-screen runner mode (the Contract's second runner mode,
above) is realized per-example rather than through a shared crate:
`pingpong_animation`'s
[`render.rs`](../../examples/scene_script/pingpong_animation/src/render.rs)
`frame_to_commands()` function compiles the script's simulated per-frame
output into `tilemap_renderer` `RenderCommand`s, which route to a headless
`SvgBackend` — proven by 6 tests in
[`render_test.rs`](../../examples/scene_script/pingpong_animation/tests/render_test.rs)
(T01, T02, T03, T05, T06, AF2). This wiring is named "L5→L3 wiring" and
formalized as Decision #4 in
[ADR-003](../adr/003_d2_stack_hal_adoption.md) — example-local glue, not a
new shared crate, until a second consumer triggers extraction.

### Invariants

| File | Relationship |
|------|--------------|
| [../../module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md](../../module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md) | The convention `scene_script` enforces to realize this layer as script-as-glue |
| [../../module/helper/scene_script/docs/invariant/004_script_as_data_purity.md](../../module/helper/scene_script/docs/invariant/004_script_as_data_purity.md) | The purity invariant `scene_script` enforces to realize this layer as script-as-data |

### Layers

| File | Relationship |
|------|--------------|
| [005_l4_scene_model.md](005_l4_scene_model.md) | The declarative data scripts are built from |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/004_script_as_data.md](../pattern/004_script_as_data.md) | The declarative script form and when to choose it |
| [../pattern/005_script_as_glue.md](../pattern/005_script_as_glue.md) | The imperative script form and when to choose it |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/002_tile.md](../render_stack/002_tile.md) | The stack whose L5 off-screen compile path is implemented — no interactive runner yet |
| [../render_stack/003_d3.md](../render_stack/003_d3.md) | The stack whose L5 slot is reserved |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/d3_scene/` | Reserved d3 script-layer slot |
| `module/helper/scene_script/src/engine.rs` | Rhai engine assembly (`engine_build()`) |
| `module/helper/scene_script/src/top_level_lint.rs` | Structural check that imperative code lives inside `main()`, not a proof of the temporal/order determinism the Contract section above requires |
| `module/helper/scene_script/src/purity_lint.rs` | Companion structural check for the script-as-data form: `check_whole_ast_is_pure` rejects any call expression, enforcing the no-engine-calls half of the [script-as-data](../pattern/004_script_as_data.md)/script-as-glue split this layer names above — proven against the real orrery `scene.rhai` end-to-end (`purity_lint_test.rs`); wired into the production load path via `script_load.rs` below, so a violation is rejected at script-load time, not merely caught on the next test run someone remembers to write ([invariant/004](../../module/helper/scene_script/docs/invariant/004_script_as_data_purity.md)) |
| `module/helper/scene_script/src/script_load.rs` | Production compile-and-lint entry points — `script_as_glue_load`/`script_as_data_load` compile `source` against an `Engine`, then run the form-appropriate lint (`top_level_lint`/`purity_lint` above) before returning the linted `AST`; all 3 real consumers (`pingpong_animation`, `f32x2_vector_arithmetic`, the orrery `scene.rhai`) load through these, not raw `engine_build()` + direct compile/eval — closes the gap the two rows above used to describe (`script_load_test.rs`) |
| `module/helper/tilemap_scene/src/compile/frame.rs` | Deterministic scene→commands compilation |
| `module/helper/tilemap_scene/src/renderer.rs` | The runner half: executes compiled commands, ~24 dedicated tests |
