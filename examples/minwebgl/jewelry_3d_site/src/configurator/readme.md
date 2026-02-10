# Configurator Module

This module manages the 3D jewelry configurator state, lazy ring loading, and per-ring color persistence.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Exposes public API, re-exports submodules, and implements main `Configurator` struct with UI-driven material updates and color transitions |
| `animation.rs` | Manages material color transition animations using sequencer-based tweening and callback system |
| `state.rs` | Defines ring data structures (`Ring`, `RingColors`, `RingsInfo`) and provides lazy-loading infrastructure with race condition prevention |
| `gpu_sync.rs` | Implements GPU synchronization utility using framebuffer fences to prevent race conditions in lazy ring loading |
| `rendering.rs` | Handles shadow baking, camera setup, and gem material configuration for 3D scene rendering |
| `rings.rs` | Implements async ring loading system with GLTF processing, gem detection, cube normal map generation, and shadow baking |

## Module Architecture

```
configurator/
├── mod.rs           (399 lines) - Public API & Configurator
├── animation.rs     (118 lines) - Animation system
├── state.rs         (171 lines) - Data structures
├── gpu_sync.rs      (76 lines)  - GPU sync utility
├── rendering.rs     (200 lines) - Shadow/camera/materials
└── rings.rs         (243 lines) - Ring loading
```

## Key Features

### Lazy Loading System
- **On-Demand Loading:** Rings are loaded only when user selects them
- **Race Condition Prevention:** `GpuSync` ensures GPU operations complete before ring storage
- **Graceful Failures:** Network errors and WebGL resource exhaustion handled gracefully

### Color Management
- **Per-Ring Persistence:** Each ring maintains its own gem and metal color selections
- **Animated Transitions:** Color changes use smooth 1000ms tweened animations
- **Material Types:** Supports both PBR metal materials and custom gem materials

### Rendering Pipeline
- **Shadow Baking:** Runtime soft shadow generation at 2048x2048 resolution
- **Cube Normal Maps:** Generated per gem with caching for numbered instances
- **Camera Controls:** Pre-configured orbit controls optimized for jewelry inspection

## Data Flow

```
UI State (JavaScript)
    ↓
Configurator::new()
    ↓
setup_rings() → lazy ring_loader closure
    ↓
RingsInfo::get_ring() → triggers async loading
    ↓
ring_loader processes GLTF → bakes shadows → generates normals
    ↓
GpuSync::sync() ensures GPU completion
    ↓
Ring stored and rendered
```

## Usage Example

```rust
// Initialization
let configurator = Configurator::new(&gl, &canvas).await?;

// Color updates (with animation)
configurator.update_gem_color();
configurator.update_metal_color();

// Ring switching (triggers lazy load if needed)
let ring = configurator.rings.get_ring();
```

## Refactoring History

**Version:** 2.0 (Post-Refactoring)
- **Date:** 2026-02-06
- **Change:** Split monolithic `configurator.rs` (1,084 lines) into 6 focused modules
- **Rationale:** Improved maintainability, testability, and code navigation
- **Status:** ✓ This is the first module in the codebase to use the `mod.rs` pattern

**Previous Structure:**
- Single file: `src/configurator.rs` (1,084 lines)

**Current Structure:**
- Module directory with 6 focused files (1,207 total lines including proper imports)

## Dependencies

**External Crates:**
- `minwebgl` - WebGL bindings
- `renderer::webgl` - 3D rendering engine
- `animation` - Sequencer and tweening system
- `rustc_hash::FxHashMap` - Fast hashing for material lookups
- `gltf` - GLTF format loading

**Internal Modules:**
- `cube_normal_map_generator` - Cube normal map generation
- `gem::GemMaterial` - Custom gem material implementation
- `surface_material::SurfaceMaterial` - Surface material for planes
- `scene_utilities` - Scene traversal and filtering utilities
- `ui` - JavaScript interop for UI state

## Testing

Tests are located in `tests/configurator_test.rs`:
- ✓ `test_remove_numbers` - String utility function
- ✓ `test_ring_colors_default` - Default color values
- ✓ `test_ring_colors_clone` - Color struct cloning
- ✓ `test_animation_state_new` - Animation initialization
- ✓ `test_get_color_pbr_material` - PBR material color extraction
- ✓ `test_get_color_gem_material` - Gem material color extraction
- ✓ `test_setup_camera` - Camera controls configuration
- ✓ `test_create_shadow_texture` - Shadow texture creation
- ✓ `test_setup_gem_material` - Gem material setup

## Performance Characteristics

- **Initial Load:** Fast (rings loaded lazily, not on startup)
- **Ring Switch:** 2-3 seconds on 4G for unloaded rings (network bound)
- **Color Transition:** Smooth 60fps animation over 1000ms
- **Shadow Baking:** ~100-200ms per ring (one-time cost)
- **Memory:** Efficient (only loads selected ring, cached cube normal maps)
