# Browser Input Dependency Cleanup

## Issue
browser_input currently depends on minwebgl just for two math types:
- `I32x2` (2D integer coordinates)
- `F64x3` (3D float vectors for wheel events)

This creates unnecessary coupling between input handling and WebGL functionality.

## Current Dependencies
```toml
minwebgl = { workspace = true, features = ["math"] }
```

## Proposed Solution
Replace minwebgl dependency with ndarray_cg, which provides the same math types:

```toml
ndarray_cg = { workspace = true }
```

## Benefits
- Removes WebGL coupling from input handling
- Same vector types available (`I32x2`, `F64x3`)
- More logical dependency graph
- Allows browser_input in non-WebGL contexts
- Lighter dependency footprint

## Changes Needed
1. Update Cargo.toml dependency
2. Change imports in src/input.rs from `minwebgl` to `ndarray_cg`  
3. Change imports in src/util.rs from `minwebgl` to `ndarray_cg`
4. Test functionality remains intact

## Files to Modify
- `Cargo.toml`
- `src/input.rs` (line 5-6)
- `src/util.rs` (line 4-5)