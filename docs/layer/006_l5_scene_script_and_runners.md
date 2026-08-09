# Layer: L5 Scene Script and Runners

The top of the ecosystem: scenes as *scripts* — parsable, interpretable,
and deterministic (the same script must produce the same frames) — plus the
runners that execute them interactively or off-screen. This layer is where
"scene-as-script" from ADR-001 lives.

### Scope

- **Purpose**: Define the script layer's contract (determinism above all) and record its current occupants.
- **Responsibility**: Name the two contract halves (script + runner) and which stacks have each today.
- **In Scope**: `tilemap_scene`'s compile/runner path; `scene_script`'s Rhai glue; the reserved `d3_scene` slot.
- **Out of Scope**: The declarative data itself (see [005_l4_scene_model.md](005_l4_scene_model.md)); engine internals (see [004_l3_stack_engine.md](004_l3_stack_engine.md)).

### Contract

- **Deterministic**: same script + same seed → same frame sequence. This is
  what makes scripts testable, diffable, and safely re-runnable — pinned for
  the tile stack by
  [`tilemap_scene` invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md).
- **Two runner modes**: interactive (browser/window loop) and off-screen
  (headless compile → declarative output or snapshot) — the off-screen mode
  is what proves determinism in CI.

### Occupants Today

| Crate | Stack | Role |
|-------|-------|------|
| `tilemap_scene` | tile | Script-as-data: RON scenes compiled deterministically to `tilemap_renderer` commands; headless snapshot tests are the CI proof ([invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md), [invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md)) |
| `scene_script` | d2 | Script-as-glue: Rhai bindings (`vector_binding`, `tween_binding`, `build_engine()`) exposing math + tween vocabulary to embedded scripts; top-level script shape (bindings only, imperative code confined to `main()`) is checked by `top_level_lint` |
| `d3_scene` | d3 | Reserved (`module/blank/d3_scene/`) — no d3 script layer exists yet |

The two existing occupants embody the layer's two script forms, each a
workspace pattern in its own right:
[script-as-data](../pattern/004_script_as_data.md) (`tilemap_scene` —
declarative, compiled) and
[script-as-glue](../pattern/005_script_as_glue.md) (`scene_script` —
imperative snippets over a bound API). A future d3 script layer should
choose one deliberately — the patterns record the criteria and the default
recommendation.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/004_script_as_data.md](../pattern/004_script_as_data.md) | The declarative script form and when to choose it |
| [../pattern/005_script_as_glue.md](../pattern/005_script_as_glue.md) | The imperative script form and when to choose it |

### Layers

| File | Relationship |
|------|--------------|
| [005_l4_scene_model.md](005_l4_scene_model.md) | The declarative data scripts are built from |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/002_tile.md](../render_stack/002_tile.md) | The stack whose L5 is fully realized |
| [../render_stack/003_d3.md](../render_stack/003_d3.md) | The stack whose L5 slot is reserved |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/d3_scene/` | Reserved d3 script-layer slot |
| `module/helper/scene_script/src/engine.rs` | Rhai engine assembly (`build_engine()`) |
| `module/helper/scene_script/src/top_level_lint.rs` | Structural check that imperative code lives inside `main()`, not a proof of the temporal/order determinism the Contract section above requires |
| `module/helper/tilemap_scene/src/compile/frame.rs` | Deterministic scene→commands compilation |
