# Feature: Image-Based Lighting

Environment lighting via the split-sum approximation: three precomputed
textures stand in for the environment integral, and the crate can prefilter
them itself on the GPU.

### Scope

- **Purpose**: Navigational hub for the IBL subsystem — the texture triple, its GPU prefiltering, and where it plugs into shading.
- **Responsibility**: Describe the `IBL` structure and the PMREM generation path, linked to sources and tests.
- **In Scope**: The three IBL textures, their generation, and their consumption by the PBR shader.
- **Out of Scope**: The analytic-light part of shading (see [001_pbr_rendering_core.md](001_pbr_rendering_core.md)); the HDR range contract the textures live under (see `../invariant/003`).

### Design

**The texture triple.** `src/webgl/ibl.rs` defines `IBL` as three textures
plus a mip count, following the split-sum approach (the source cites Karis'
SIGGRAPH 2013 course notes and LearnOpenGL's IBL chapters):

- `diffuse_texture` — diffuse irradiance cubemap;
- `specular_1_texture` — prefiltered specular environment cubemap, roughness
  encoded across mip levels (`num_mips`, `max_lod = num_mips − 1`);
- `specular_2_texture` — the 2D BRDF integration lookup table.

**Prefiltering.** `src/webgl/loaders/pmrem.rs` generates the prefiltered
mip chain on the GPU by rendering into float targets — which is where the
`EXT_color_buffer_float` requirement bites hardest (see
`../pitfall/001_requires_ext_color_buffer_float.md`).

**Consumption.** The PBR fragment shader samples all three during shading,
adding environment radiance on top of analytic lights — same BRDF, same
HDR pipeline as everything else.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pbr_metallic_roughness_baseline.md](../invariant/002_pbr_metallic_roughness_baseline.md) | The material model whose BRDF the split-sum textures approximate |
| [../invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md) | Environment radiance stays linear HDR end to end |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_requires_ext_color_buffer_float.md](../pitfall/001_requires_ext_color_buffer_float.md) | PMREM renders into float targets — unavailable without the extension |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/ibl.rs` | The `IBL` texture triple and mip-count contract |
| `src/webgl/loaders/pmrem.rs` | GPU prefiltering of environment maps |
| `src/webgl/shaders/main.frag` | Sampling of the triple during shading |

### Tests

| File | Relationship |
|------|--------------|
| `tests/pmrem_tests.rs` | Pins the PMREM prefiltering path |
