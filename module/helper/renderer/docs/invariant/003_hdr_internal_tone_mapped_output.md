# Invariant: HDR Internal, Tone-Mapped Output

Lighting is computed in linear high dynamic range and stays there for the
whole frame; conversion to display range (tone mapping, then sRGB encoding)
happens exactly once, at the end. Nothing mid-pipeline clamps to `[0, 1]`.

### Scope

- **Purpose**: Pin the d3 stack's light-transport range contract — values above 1.0 are meaningful throughout the pipeline.
- **Responsibility**: State the property, name the float-target and pass-ordering enforcement, and record the failure modes of breaking range discipline.
- **In Scope**: The numeric format of intermediate render targets and the position of display conversion in the frame.
- **Out of Scope**: Which tone-mapping curve is used (an aesthetic choice among the provided passes); the environment-availability trap the float targets create (see [../pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md)).

### Invariant Statement

All lighting computation — analytic lights, IBL, emission, post-processing —
reads and writes linear `RGBA16F` targets. Radiance above 1.0 survives every
intermediate pass (enabling bloom thresholds, exposure control, and physical
light intensities). Only the final passes reduce range: a tone-mapping pass
compresses HDR to displayable range, and an sRGB pass encodes for the
display. No earlier pass may clamp, saturate, or gamma-encode.

### Enforcement Mechanism

- **Float targets by construction**: `src/webgl/renderer.rs` allocates the
  main color, emission, and transparent accumulation attachments as
  `RGBA16F` — HDR is the storage format, not an option flag.
- **Fixed conversion position**: display conversion exists only as the
  dedicated post-processing passes `tonemapping.rs` (including an ACES
  variant) and `to_srgb.rs` in `src/webgl/post_processing/` — run at the end
  of the post chain, as the crate's usage examples show.
- **Linear-space assets**: the IBL prefiltering path likewise renders into
  float targets (see the pitfall instance for the extension this requires),
  keeping environment radiance linear end to end.

### Violation Consequences

- Inserting a pass that renders to an 8-bit target mid-chain silently clips
  all radiance above 1.0 — highlights flatten, bloom loses its source
  signal, and exposure adjustments stop being lossless.
- Tone mapping twice (or encoding sRGB before a linear-space pass) skews all
  subsequent math — colors wash out or oversaturate.
- Skipping the final passes displays raw linear HDR: dark midtones and
  clipped highlights.
- The `RGBA16F` choice is what makes the whole pipeline depend on
  `EXT_color_buffer_float` — the trap documented in
  [../pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md).

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The frame pipeline whose targets and pass order realize this contract |
| [../feature/002_image_based_lighting.md](../feature/002_image_based_lighting.md) | Supplies HDR environment radiance that only makes sense under this invariant |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md) | The environment requirement this invariant's float targets impose on every consumer |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/post_processing/to_srgb.rs` | Final sRGB encoding pass |
| `src/webgl/post_processing/tonemapping.rs` | HDR→display compression passes (incl. ACES) |
| `src/webgl/renderer.rs` | `RGBA16F` allocation of the main, emission, and transparent targets |

### Tests

| File | Relationship |
|------|--------------|
| `tests/color_grading_tests.rs` | Exercises post-chain color transforms downstream of the HDR targets |
