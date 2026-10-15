# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Major subsystems (PBR core, IBL, shadow mapping, text engraving) as navigational hubs over source, invariants, and pitfalls | [feature/readme.md](../feature/readme.md) | 4 |
| `format/` | Asset and data-file contracts beyond plain glTF (engravable models, `engraving_config.json`) | [format/readme.md](../format/readme.md) | 1 |
| `invariant/` | The d3 stack's correctness guarantees: GPU-resolved visibility, PBR material baseline, HDR-internal pipeline, GPU resource ownership | [invariant/readme.md](../invariant/readme.md) | 4 |
| `pitfall/` | Confirmed environment traps consumers hit (`EXT_color_buffer_float`) | [pitfall/readme.md](../pitfall/readme.md) | 1 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|-----------|-----|------|------|
| feature | 001 | PBR Rendering Core | [feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) |
| feature | 002 | Image-Based Lighting | [feature/002_image_based_lighting.md](../feature/002_image_based_lighting.md) |
| feature | 003 | Shadow Mapping | [feature/003_shadow_mapping.md](../feature/003_shadow_mapping.md) |
| feature | 004 | Text Engraving | [feature/004_text_engraving.md](../feature/004_text_engraving.md) |
| format | 001 | Engraving Asset Pipeline | [format/001_engraving_asset_pipeline.md](../format/001_engraving_asset_pipeline.md) |
| invariant | 001 | Depth-Buffer Visibility with OIT | [invariant/001_depth_buffer_visibility_with_oit.md](../invariant/001_depth_buffer_visibility_with_oit.md) |
| invariant | 002 | PBR Metallic-Roughness Baseline | [invariant/002_pbr_metallic_roughness_baseline.md](../invariant/002_pbr_metallic_roughness_baseline.md) |
| invariant | 003 | HDR Internal, Tone-Mapped Output | [invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md) |
| invariant | 004 | Owned GPU Objects Have Exactly One Deleter | [invariant/004_gpu_resource_ownership.md](../invariant/004_gpu_resource_ownership.md) |
| pitfall | 001 | Requires EXT_color_buffer_float | [pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md) |
