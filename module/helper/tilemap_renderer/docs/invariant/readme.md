# Invariant Doc Entity

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
