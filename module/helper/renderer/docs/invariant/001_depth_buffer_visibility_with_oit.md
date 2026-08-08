# Invariant: Depth-Buffer Visibility with OIT

Visibility is resolved on the GPU, not by the caller: opaque geometry through
the depth buffer, transparent geometry through weighted-blended
order-independent transparency. Draw submission order never determines what
occludes what — the d3 stack's visibility invariant, and the direct
counterpart to the d2 stack's z-layer ordering.

### Scope

- **Purpose**: State the visibility contract scenes rendered through `Renderer` can rely on.
- **Responsibility**: Pin that neither opaque nor transparent correctness depends on CPU-side draw ordering, and name the framebuffer machinery enforcing it.
- **In Scope**: Occlusion of opaque geometry; compositing of transparent geometry.
- **Out of Scope**: The material model deciding *which* path a mesh takes (see [002_pbr_metallic_roughness_baseline.md](002_pbr_metallic_roughness_baseline.md)); the numeric range of the targets involved (see [003_hdr_internal_tone_mapped_output.md](003_hdr_internal_tone_mapped_output.md)); shadow visibility from the light's viewpoint (see [../feature/003_shadow_mapping.md](../feature/003_shadow_mapping.md)).

### Invariant Statement

For any submission order of the same scene, the rendered image is the same:
opaque fragments are occluded per-pixel by depth testing, and transparent
fragments are composited by weighted blending — an order-independent
approximation that needs no back-to-front sorting. No caller-side sorting,
bucketing, or traversal-order discipline is required for correctness.

### Enforcement Mechanism

- **Opaque path**: standard depth-tested rendering into the multisampled
  HDR color target.
- **Transparent path**: materials with a blending alpha mode render into two
  dedicated targets — a weighted color *accumulation* buffer and a
  *revealage* buffer (`src/webgl/renderer.rs`: the
  `multisample_transparent_accumulate_renderbuffer` /
  `multisample_transparent_revealage_renderbuffer` pair and their resolved
  texture counterparts, attached as extra color attachments of the main
  framebuffer). Because the technique is commutative in the blended terms,
  fragment arrival order cannot change the composite.
- **Composite**: `resolve( gl, use_emission, has_transparent )` merges the
  resolved transparent accumulation into the opaque image after multisample
  resolve — one fixed pipeline step, not a per-scene decision.

### Violation Consequences

- Weighted-blended OIT is an *approximation*: it is order-independent but
  weight-dependent — stacked high-contrast transparents can show weighting
  artifacts that true sorted blending would not. This is the accepted cost of
  the invariant; callers must not "fix" it by sorting (sorting has no effect
  on the weighted composite).
- A custom material that writes depth for transparent fragments would let
  transparents occlude like opaques — reintroducing order-visible artifacts
  the invariant exists to exclude.
- Code ported from the d2 stack that relies on submission order for layering
  renders differently here by design; layering intent must be expressed in
  world-space z, not submission order.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The frame pipeline in which both visibility paths and the composite step live |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/renderer.rs` | Depth-tested opaque target; accumulate/revealage transparent targets; `resolve` composite |
| `src/webgl/shaders/` | Shader side of the opaque and transparent passes |

### Tests

| File | Relationship |
|------|--------------|
| — | No dedicated order-permutation test pins this yet; the property currently rests on the framebuffer structure above |
