# Pitfall Doc Entity

### Scope

- **Purpose**: `tilemap_renderer`'s GPU buffer handling and per-adapter `Path`-source asset handling contain confirmed edge cases that are not obvious from reading the adapters alone.
- **Responsibility**: Document each confirmed trap, its observable failure, and its mitigation (or lack thereof).
- **In Scope**: Confirmed implementation pitfalls in the SVG and WebGL2 backend adapters.
- **Out of Scope**: Hypothetical or not-yet-observed failure modes; general WebGL2/SVG pitfalls not specific to this crate's design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [ArrayBuffer Swap-Remove Buffer-Binding Violation](001_arraybuffer_swap_remove_buffer_binding_violation.md) | WebGL2 spec forbids self-to-self GPU buffer copy; mitigated via a scratch buffer | ✅ |
| 002 | [GPU Instance Struct Field-Reorder Desync](002_gpu_instance_struct_field_reorder_desync.md) | Compile-time size/align assertions don't catch a same-size field reorder | ⚠️ |
| 003 | [SVG Geometry Path Source Silently Skipped](003_svg_geometry_path_source_silently_skipped.md) | Unlike image `Path` sources, geometry `Path` sources are dropped with no diagnostic | ⚠️ |
