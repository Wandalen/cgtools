# Invariant: Coverage Alpha Reaches the Canvas Unchanged

With `ToSrgbPass::alpha_forward_set( true )`, the canvas keeps the scene's
coverage alpha: background pixels stay transparent and silhouette edges
partly so, so the canvas can be composited over a photo or video (AR).
That only works if every pass between the renderer and `ToSrgbPass` passes
the alpha channel through untouched.

### Scope

- **Purpose**: Pin the contract a pass must keep to sit in an AR post-processing chain, so a pass added later cannot quietly make the canvas opaque again.
- **Responsibility**: State which channel carries coverage, which passes keep it, how each does, and what breaks when one does not.
- **In Scope**: The alpha channel from the renderer's resolved main target through tone mapping, colour grading and `ToSrgbPass`; the premultiplied form `ToSrgbPass` writes.
- **Out of Scope**: Depth-only occluders, which decide *what* has coverage (see [001_depth_buffer_visibility_with_oit.md](001_depth_buffer_visibility_with_oit.md)); the default opaque output, which needs none of this.

### Invariant Statement

The alpha of the renderer's main texture is the scene's coverage: 1 on
geometry, the clear alpha (0) on background, and fractional on MSAA edge
pixels. Every pass that runs between the renderer and an alpha-forwarding
`ToSrgbPass` writes, as its output alpha, exactly its input alpha. Only the
colour channels may change. `ToSrgbPass` then encodes the straight colour and
multiplies the alpha back in, so its output is premultiplied (`rgb <= alpha`)
for the WebGL default `premultipliedAlpha: true` canvas. The colour is
premultiplied on input only when the scene is cleared to colour 0.

### Enforcement Mechanism

- **`ToneMappingPass< ToneMappingAces >`** (`shaders/tonemapping/aces.frag`) writes `src.a` on every path: background, fully covered and edge pixels. On edge pixels it tone-maps the geometry's share of the colour and keeps the coverage as alpha.
- **`ColorGradingPass`** (`shaders/post_processing/color_grading.frag`) writes `vec4( graded, src.a )`.
- **`ToSrgbPass`** (`shaders/post_processing/to_srgb.frag`) reads the alpha only under `FORWARD_ALPHA`, the variant `alpha_forward_set( true )` compiles. Off, it writes alpha 1, which is the default opaque canvas.
- A new pass meant for an AR chain must do the same as these: write its input alpha. Writing a constant 1 is only correct after `ToSrgbPass`, or in a chain that never forwards alpha.

### Violation Consequences

- A pass that writes alpha 1 makes the whole canvas opaque: the photo behind it disappears, and the clear colour fills the background.
- A pass that scales alpha (or writes 0) makes geometry partly or wholly see-through over the photo.
- A pass that changes the colour without regard to coverage breaks `rgb <= alpha` on edge pixels. The premultiplied canvas then shows a bright halo around every silhouette.
- A non-zero clear colour leaves that colour blended into the edges, even when every pass keeps the alpha.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The frame pipeline and post chain this contract runs through |

### Invariants

| File | Relationship |
|------|--------------|
| [001_depth_buffer_visibility_with_oit.md](001_depth_buffer_visibility_with_oit.md) | Occluders drawn first decide where coverage is 0 in front of opaque geometry |
| [003_hdr_internal_tone_mapped_output.md](003_hdr_internal_tone_mapped_output.md) | The same passes, seen from the colour range side |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/post_processing/to_srgb.rs` | `ToSrgbPass::alpha_forward_set`, `fragment_source` (the `FORWARD_ALPHA` variant) |
| `src/webgl/shaders/post_processing/to_srgb.frag` | Premultiplied output under `FORWARD_ALPHA`, alpha 1 otherwise |
| `src/webgl/shaders/tonemapping/aces.frag` | Writes the input alpha on every path |
| `src/webgl/shaders/post_processing/color_grading.frag` | Writes the input alpha |

### Tests

| File | Relationship |
|------|--------------|
| `tests/ar_alpha_readback_test.rs` | `alpha_forward_output_is_premultiplied`, `default_output_is_opaque`, `color_grading_forwards_alpha` |
| `tests/tonemapping_edge_readback_test.rs` | The ACES pass over real frames: MSAA edges and transparent coverage |
| `tests/legacy_glsl_shader_compile_test.rs` | Compiles the `FORWARD_ALPHA` variant from `ToSrgbPass::fragment_source` |
