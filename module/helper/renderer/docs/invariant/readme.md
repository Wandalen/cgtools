# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `renderer`, these are the d3 stack's defining correctness guarantees, each one recorded with its statement, how it is enforced, and what happens if it is violated. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `renderer`'s correctness guarantees — the d3 stack's defining invariants, pinned where they are enforced.
- **Responsibility**: Document each invariant's statement, enforcement mechanism, and violation consequences.
- **In Scope**: Properties every scene rendered through this crate can rely on: visibility resolution, the material baseline, light-transport range.
- **Out of Scope**: Pipeline structure and subsystem design (see `feature/`); environment-dependent traps (see `pitfall/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Depth-Buffer Visibility with OIT](001_depth_buffer_visibility_with_oit.md) | Visibility is GPU-resolved: depth-tested opaques plus weighted-blended order-independent transparency — callers never sort | ✅ |
| 002 | [PBR Metallic-Roughness Baseline](002_pbr_metallic_roughness_baseline.md) | Every mesh renders through the glTF-style PBR material model; extensions refine it, never replace it | ✅ |
| 003 | [HDR Internal, Tone-Mapped Output](003_hdr_internal_tone_mapped_output.md) | Lighting is computed in linear HDR (`RGBA16F`) and reduced to display range only at the end of the frame | ✅ |
