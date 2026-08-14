# Pattern: Cross-Stack Bridge via Foundation Resources

### Scope

- **Purpose**: Let content cross stack boundaries (3D content inside a 2D game, 2D overlays on a 3D scene) without entangling the stacks' invariant sets.
- **Responsibility**: Define what a bridge may exchange (foundation resources) and what it must never exchange (another stack's scene abstractions).
- **In Scope**: The bridging rule and its existing instances in this workspace.
- **Out of Scope**: What makes two stacks distinct in the first place (see [001_invariant_defined_stack.md](001_invariant_defined_stack.md)); intra-stack layering (see [002_strict_layering_one_step_drilldown.md](002_strict_layering_one_step_drilldown.md)).

### Problem

Real projects mix stacks: a tile game shows a rotating 3D trophy; a 3D scene
shows a 2D minimap. The tempting implementation imports the other stack's
scene types directly — the tile engine holds a `renderer` scene, or vice
versa. That single import couples both invariant tables: the 2D side now
transitively assumes a depth buffer and HDR targets, its vector
representability silently dies, and every future change in either stack can
break the other.

### Solution

Stacks compose **only through foundation resources** — the layer-L0/L1
currency both stacks already speak:

- **textures / framebuffers**: one stack renders into a texture; the other
  consumes that texture as an ordinary image resource;
- **command / data streams**: one system emits the other stack's *declared
  input format* (e.g. anything can emit `tilemap_renderer` `RenderCommand`s)
  without importing its internals.

A bridge crate may depend on the *lower* layers of both stacks, but its
public API exposes only resource hand-off — never a re-export of either
stack's scene model. Semantically: by the time content crosses the boundary,
it has been flattened to pixels or commands, and the receiving stack treats
it as it treats any other resource of that kind.

### Applicability

Apply whenever a feature needs two stacks' capabilities at once. If the
required exchange cannot be expressed as a foundation resource — e.g. the 2D
side must *pick* 3D objects, not just show them — that is not a bridge but a
new capability; classify it via [001](001_invariant_defined_stack.md)'s rules
instead of widening a bridge.

### Consequences

- Each stack's invariant table stays independently checkable; a bridge cannot
  smuggle a foreign assumption in, because pixels and command streams carry
  none.
- Bridges are explicit choke points — easy to find, test, and profile.
- Cost: crossing the boundary is lossy by design. The 2D side sees the 3D
  trophy as pixels; interaction semantics (picking, hit-testing) do not cross
  and must be handled on the owning side.

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/001_multi_stack_rendering_architecture.md](../adr/001_multi_stack_rendering_architecture.md) | Adopts this pattern as the only sanctioned cross-stack composition mechanism |

### Sources

| File | Relationship |
|------|--------------|
| `examples/minwebgl/lottie_surface_rendering/src/main.rs` | Bridge in use: `CanvasRenderer::new(…)` then `canvas_renderer.texture_get()` — content crosses as a texture handle |
| `module/helper/canvas_renderer/` | Existing bridge: "2D canvas renderer … with framebuffer rendering and 3D scene support" — 3D content crosses into 2D as a framebuffer, not as a scene type |
| `module/helper/tilemap_renderer/src/commands.rs` | The d2 command stream — the declared input format any external system may emit |

### Tests

No dedicated cross-stack test exists today — `canvas_renderer` currently
ships no `tests/` directory, so the framebuffer hand-off is exercised only by
examples, not pinned by a regression test.
