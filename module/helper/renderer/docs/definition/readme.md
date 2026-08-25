# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Major subsystems (PBR core, IBL, shadow mapping) as navigational hubs over source, invariants, and pitfalls | [feature/readme.md](../feature/readme.md) | 3 |
| `invariant/` | The d3 stack's correctness guarantees: GPU-resolved visibility, PBR material baseline, HDR-internal pipeline | [invariant/readme.md](../invariant/readme.md) | 3 |
| `pitfall/` | Confirmed environment traps consumers hit (`EXT_color_buffer_float`) | [pitfall/readme.md](../pitfall/readme.md) | 1 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|-----------|-----|------|------|
| feature | 001 | PBR Rendering Core | [feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) |
| feature | 002 | Image-Based Lighting | [feature/002_image_based_lighting.md](../feature/002_image_based_lighting.md) |
| feature | 003 | Shadow Mapping | [feature/003_shadow_mapping.md](../feature/003_shadow_mapping.md) |
| invariant | 001 | Depth-Buffer Visibility with OIT | [invariant/001_depth_buffer_visibility_with_oit.md](../invariant/001_depth_buffer_visibility_with_oit.md) |
| invariant | 002 | PBR Metallic-Roughness Baseline | [invariant/002_pbr_metallic_roughness_baseline.md](../invariant/002_pbr_metallic_roughness_baseline.md) |
| invariant | 003 | HDR Internal, Tone-Mapped Output | [invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md) |
| pitfall | 001 | Requires EXT_color_buffer_float | [pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md) |
