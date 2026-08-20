# Render Stack: tile

Extension of [d2](001_d2.md): lattice-addressed game space compiled
deterministically into the d2 command stream. Every d2 invariant is
inherited unchanged; the tile invariants below are pure additions — which is
exactly why this is an extension stack, not a sibling
([../pattern/001](../pattern/001_invariant_defined_stack.md), rule 2).

### Scope

- **Purpose**: The tile stack's identity card — its additional invariants over d2, and which crates live in it.
- **Responsibility**: Aggregate the extension invariants with links to the crate-level instances pinning each; record membership and layer occupancy.
- **In Scope**: What tile adds on top of d2.
- **Out of Scope**: The inherited d2 invariants (see [001_d2.md](001_d2.md)); each invariant's detail (the linked crate instances).

### Invariant Table (additions over d2)

| ID | Invariant | Pinned at |
|----|-----------|-----------|
| T-1 | Lattice address primacy: game state lives at typed lattice coordinates; pixels are derived, never stored | [`tiles_tools` invariant/002](../../module/helper/tiles_tools/docs/invariant/002_lattice_address_primacy.md) |
| T-2 | Compiles to renderer commands only: no GPU, backend, or platform vocabulary above the d2 command seam | [`tilemap_scene` invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md) |
| T-3 | Deterministic compilation: same `(spec, scene, time, seed)` → identical command stream | [`tilemap_scene` invariant/004](../../module/helper/tilemap_scene/docs/invariant/004_deterministic_compilation.md) |

Membership test for a tile-stack crate: it may assume the d2 table *plus*
this table — nothing more.

### Member Crates

| Crate | Layer | Role |
|-------|-------|------|
| `tilemap_scene` | L4 + L5 | Declarative scene model (RON) and the compile/runner path producing d2 commands |
| `tiles_tools` | logic beside L3 | Lattice coordinates, pathfinding, field of view, ECS — the game-space vocabulary T-1 governs |

The stack's L0–L3 are the d2 stack's own layers, reused as-is —
`tilemap_scene` depends on `tilemap_renderer`, never on anything below it.

### Layers

| File | Relationship |
|------|--------------|
| [../layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md) | The script layer `tilemap_scene` implements for this stack — off-screen compile path only; no interactive runner or example consumer exists yet |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/001_invariant_defined_stack.md](../pattern/001_invariant_defined_stack.md) | The extension-stack rule this stack instantiates |
| [../pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md) | The conforming dependency chain (`tilemap_scene` → `tilemap_renderer` only) |

### Render Stacks

| File | Relationship |
|------|--------------|
| [001_d2.md](001_d2.md) | The base stack every tile invariant extends |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/tilemap_scene/Cargo.toml` | The dependency surface enforcing T-2 and the layering claim above |
