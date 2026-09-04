# Feature: PBR Rendering Core

The frame pipeline of the d3 stack's engine: a glTF-shaped scene graph
rendered through physically-based materials into multisampled HDR targets,
resolved, post-processed, and reduced to display range.

### Scope

- **Purpose**: Navigational hub for the core render path — the one place the whole frame's shape is written down, since no single source file shows it end to end.
- **Responsibility**: Describe the pipeline stages and the scene/material structures they consume, linking each to its sources and governing invariants.
- **In Scope**: Scene graph, glTF import, per-frame pass sequence, render-target topology.
- **Out of Scope**: Environment lighting internals (see [002_image_based_lighting.md](002_image_based_lighting.md)); shadow generation (see [003_shadow_mapping.md](003_shadow_mapping.md)); the invariant statements themselves (see `../invariant/`).

### Design

**Scene side.** Content enters as glTF (`src/webgl/loaders/gltf.rs`) and
becomes a hierarchy of `Node`s carrying meshes, skinned skeletons, and
animation tracks (`src/webgl/{node,scene,mesh}.rs`, animation under
`src/webgl/animation/`). Every material maps onto the PBR metallic-roughness
baseline (`../invariant/002`), shaded by the main shader pair in
`src/webgl/material/pbr.rs`.

**Frame shape.** All targets are allocated by `src/webgl/renderer.rs` as
multisampled `RGBA16F` attachments (`../invariant/003`):

1. **Opaque pass** — depth-tested draw of opaque meshes into the main color
   (+ emission) targets.
2. **Transparent pass** — blending-alpha materials draw into the weighted
   accumulation + revealage targets (weighted-blended OIT,
   `../invariant/001`); no CPU-side sorting exists anywhere in the frame.
3. **Resolve + composite** — `resolve( gl, use_emission, has_transparent )`
   resolves multisampled targets to textures and composites the transparent
   result over the opaque image.
4. **Post-processing** — composable passes from
   `src/webgl/post_processing/` (bloom, outline, color grading, …) operate
   on the resolved HDR image.
5. **Display conversion** — tone mapping then sRGB encoding close the frame
   (`../invariant/003`).

**Note on portability**: everything above is written directly against
`minwebgl` under the `renderer::webgl::*` namespace — the per-backend-
duplication strategy that workspace ADR-001 (repository root
`docs/adr/001_multi_stack_rendering_architecture.md`) replaces with a shared
HAL once one exists.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_depth_buffer_visibility_with_oit.md](../invariant/001_depth_buffer_visibility_with_oit.md) | Visibility contract realized by passes 1–3 |
| [../invariant/002_pbr_metallic_roughness_baseline.md](../invariant/002_pbr_metallic_roughness_baseline.md) | Material contract realized by the scene side |
| [../invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md) | Range contract realized by the target topology and pass 5 |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md) | Environment requirement every consumer of this pipeline inherits |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/loaders/gltf.rs` | Content import onto the scene graph |
| `src/webgl/material/pbr.rs` | Main shader pair and material upload |
| `src/webgl/node.rs` | Scene-graph node and transform hierarchy |
| `src/webgl/post_processing/` | Composable HDR post passes |
| `src/webgl/renderer.rs` | Target allocation, pass sequence, resolve/composite |

### Tests

| File | Relationship |
|------|--------------|
| `tests/animation_tests.rs` | Animation tracks driving the scene graph |
| `tests/skeleton_tests.rs` | Skinning path of the scene side |
| `tests/tests.rs` | Test aggregator for the crate |
