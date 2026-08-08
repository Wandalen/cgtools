# Feature Doc Definition

### Scope

- **Purpose**: Navigational hubs for `renderer`'s major subsystems, tying each to its sources, invariants, and pitfalls.
- **Responsibility**: Document what each subsystem does today, at the level not reconstructible from any single source file.
- **In Scope**: The PBR rendering core, image-based lighting, and shadow mapping.
- **Out of Scope**: Guarantees the subsystems uphold (see `invariant/`); environment traps (see `pitfall/`); forward-looking work (crate has no committed roadmap file — future scope is workspace-level, see the repository root `docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [PBR Rendering Core](001_pbr_rendering_core.md) | The frame pipeline: scene graph → opaque + transparent passes into MSAA HDR targets → resolve → post-processing → display | ✅ |
| 002 | [Image-Based Lighting](002_image_based_lighting.md) | Environment lighting from three precomputed textures; PMREM prefiltering on the GPU | ✅ |
| 003 | [Shadow Mapping](003_shadow_mapping.md) | Depth-from-light shadow maps and their composition into the lit image | ✅ |
