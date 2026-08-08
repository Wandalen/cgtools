# Invariant Doc Definition

### Scope

- **Purpose**: `tilemap_renderer` makes cross-backend correctness guarantees that are not obvious from any single adapter's source alone.
- **Responsibility**: Document each confirmed invariant, how it's enforced, and what happens if it's violated.
- **In Scope**: Correctness properties that hold across the crate's public `Backend` implementations.
- **Out of Scope**: Adapter-specific, non-cross-cutting behavior (see the relevant `feature/` instance's Design section).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Y-Up Coordinate System](001_y_up_coordinate_system.md) | Every backend presents Y-up regardless of native convention | ✅ |
| 002 | [SVG Injection-Safe Output](002_svg_injection_safe_output.md) | Caller-controlled text/attribute strings cannot inject markup into SVG output | ✅ |
| 003 | [Z-Layer Draw Ordering](003_z_layer_draw_ordering.md) | Submission order is the portable ordering; `Transform::depth` reorders only where a depth buffer exists, opaque-only | ✅ |
| 004 | [Vector Representability of Commands](004_vector_representability_of_commands.md) | Every command stays expressible as declarative, GPU-free output — the SVG backend is the proof | ✅ |
