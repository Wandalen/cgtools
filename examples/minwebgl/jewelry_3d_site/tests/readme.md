# Tests Directory

This directory contains integration tests for the jewelry 3D configurator.

## Overview

All tests in this directory are WASM-based integration tests that run in a browser environment using `wasm-bindgen-test`. They require WebGL 2.0 support and test the full functionality of components with real GL contexts.

## Test Execution

Tests are configured to run in browser mode via `wasm_bindgen_test_configure!(run_in_browser)` in `mod.rs`:

```bash
# Chrome (recommended for CI)
wasm-pack test --chrome

# Firefox
wasm-pack test --firefox

# Headless mode (for CI/automation)
wasm-pack test --headless --chrome
```

**Note**: Some tests attempt to load real GLTF files from `assets/gltf/jewelry/` and gracefully fall back to mock data when files are unavailable.

## Test Files

| File | Lines | Tests | Type | Responsibility |
|------|-------|-------|------|----------------|
| `mod.rs` | 11 | - | Config | Test module configuration with `run_in_browser` |
| `configurator_test.rs` | 200 | 9 | Mixed | Configurator module state, utilities, and rendering setup |
| `cube_normal_map_generator_test.rs` | 154 | 3 | WASM | Cube normal map generation for gem reflections |
| `gem_material_test.rs` | 82 | 4 | WASM | GemMaterial shader properties and cloning |
| `surface_material_test.rs` | 57 | 3 | WASM | SurfaceMaterial PBR properties and cloning |
| `scene_utilities_test.rs` | 122 | 6 | WASM | Scene utilities, node filtering, texture creation |

**Total**: 25 tests across 5 test files

## Test Details

### configurator_test.rs (9 tests)

**Regular Rust tests** (no GL context required):
- `test_remove_numbers` - String number removal utility
- `test_ring_colors_default` - RingColors default values
- `test_ring_colors_clone` - RingColors cloning behavior

**WASM tests** (require browser/GL context):
- `test_animation_state_new` - AnimationState initialization
- `test_animation_state_update` - Animation frame updates
- `test_get_color_pbr_material` - Color extraction from PbrMaterial
- `test_get_color_gem_material` - Color extraction from GemMaterial
- `test_setup_camera` - Camera and controls configuration
- `test_create_shadow_texture` - Shadow texture creation (512x512, 5 cascade levels)
- `test_filter_nodes_with_gems` - Node filtering by gem names

**Helper functions**:
- `create_test_canvas()` - Creates HTML canvas element (800x600)
- `create_test_gl_context()` - Initializes WebGL context with no antialiasing
- `create_simple_test_scene()` - Creates test scene with named gem nodes

### cube_normal_map_generator_test.rs (3 tests)

**WASM tests**:
- `test_cube_normal_map_generator_creation` - Generator initialization (512x512 default)
- `test_cube_normal_map_generator_set_texture_size` - Texture size configuration (256x256, 1024x512)
- `test_cube_normal_map_generator_with_empty_node` - Handling of empty nodes without meshes

**Helper functions**:
- `create_test_canvas()` - Creates HTML canvas element
- `create_test_gl_context()` - Initializes WebGL context

**Configuration**: `wasm_bindgen_test_configure!(run_in_browser)` (line 136)

### gem_material_test.rs (4 tests)

**WASM tests**:
- `test_gem_material_clone` - Material cloning with property preservation
  - Tests: `ray_bounces`, `color`, `env_map_intensity`, `radius`, `n2`, `rainbow_delta`, `distance_attenuation_speed`
  - Verifies unique ID generation for clones
- `test_gem_material_type_name` - Type name identification ("GemMaterial")
- `test_gem_material_needs_update` - Update flag management
- `test_gem_material_dyn_clone` - Dynamic trait object cloning

**Helper functions**:
- `create_test_canvas()` - Creates HTML canvas element
- `create_test_gl_context()` - Initializes WebGL context

### surface_material_test.rs (3 tests)

**WASM tests**:
- `test_surface_material_clone` - Material cloning with color and update flag
- `test_surface_material_type_name` - Type name identification ("SurfaceMaterial")
- `test_surface_material_dyn_clone` - Dynamic trait object cloning

**Helper functions**:
- `create_test_canvas()` - Creates HTML canvas element
- `create_test_gl_context()` - Initializes WebGL context

### scene_utilities_test.rs (6 tests)

**WASM tests**:
- `test_create_empty_texture` - **Async** texture creation with 1x1 white pixel
- `test_filter_nodes_case_sensitive` - Case-sensitive node name filtering
- `test_filter_nodes_case_insensitive` - Case-insensitive node name filtering
- `test_filter_nodes_no_matches` - Handling of searches with no results
- `test_filter_nodes_partial_match` - Substring matching across multiple nodes
- `test_add_resize_callback` - Canvas resize callback registration

**Helper functions**:
- `create_test_canvas()` - Creates HTML canvas element
- `create_test_gl_context()` - Initializes WebGL context
- `create_test_scene()` - Creates test scene with 5 named nodes (gem_1, diamond_main, Crystal_Top, metal_ring, unnamed)

## Test Architecture

### Canvas Creation Pattern

All GL-dependent tests use a consistent canvas creation pattern (replaced previous `canvas::retrieve()` approach):

```rust
fn create_test_canvas() -> canvas::HtmlCanvasElement
{
  let window = gl::web_sys::window().expect("should have a window");
  let document = window.document().expect("should have a document");
  let canvas_element = document
    .create_element("canvas")
    .expect("should create canvas")
    .dyn_into::<gl::web_sys::HtmlCanvasElement>()
    .expect("should be canvas");

  canvas_element.set_width(800);
  canvas_element.set_height(600);

  canvas::HtmlCanvasElement::from(canvas_element)
}
```

This approach:
- ✅ Creates canvas programmatically (no DOM dependency)
- ✅ Avoids `CanvasRetrievingError`
- ✅ Works reliably in browser test environment


## Coverage

The test suite covers:

| Module | Coverage | Status |
|--------|----------|--------|
| Cube normal map generation | Initialization, sizing, empty node handling | ✅ Complete |
| Material systems | GemMaterial & SurfaceMaterial properties, cloning, type identification | ✅ Complete |
| Scene utilities | Node filtering (case-sensitive/insensitive), texture creation, callbacks | ✅ Complete |
| Configurator state | RingColors, AnimationState, color utilities, string processing | ✅ Complete |
| Rendering setup | Camera configuration, shadow texture creation, material color extraction | ✅ Complete |

**Not covered** (requires full integration environment):
- ⚠️ Full GLTF ring loading pipeline
- ⚠️ Shadow baking with real geometry
- ⚠️ Complete render loop execution
- ⚠️ UI state synchronization with JavaScript

## Dependencies

Tests depend on:
- `jewelry_3d_site_lib` - Main library crate
- `wasm-bindgen-test` - WASM test framework
- `minwebgl` / `gl` - WebGL bindings
- `renderer::webgl` - 3D rendering engine
- Browser with WebGL 2.0 support

## CI/CD Integration

### GitHub Actions Configuration

Tests run automatically on pull requests via `.github/workflows/rust-check.yml`:

```yaml
- name: Run WASM tests (Chrome)
  run: wasm-pack test --headless --chrome
```

**Requirements**:
- Chrome browser installed on runner
- `wasm-pack` installed via `cargo install wasm-pack`
- Runner must be amd64 architecture

### Memory Optimization

Clippy checks use memory optimization for WASM target compilation:

```yaml
- name: Run Clippy with memory optimization
  env:
    CARGO_PROFILE_DEV_CODEGEN_UNITS: 1
  run: cargo clippy -j 1 --target wasm32-unknown-unknown -- -D warnings
```

This prevents OOM errors on GitHub Actions runners (7GB limit).

## Known Limitations

- Tests require a browser environment (no pure Rust unit tests possible due to WebGL dependency)
- WebGL 2.0 support is mandatory
- Tests may be slow due to GL context initialization overhead
- Tests cannot run in parallel within the same browser instance
- GLTF file access depends on test environment (CI may not have asset files)
- Some async tests require proper browser event loop handling

## Best Practices

When adding new tests:

1. **Use helper functions**: Leverage existing `create_test_canvas()` and `create_test_gl_context()` patterns
2. **Add fallback logic**: For GLTF loading, always provide mock data fallback
3. **Configure browser mode**: Ensure `wasm_bindgen_test_configure!(run_in_browser)` is set
4. **Document async tests**: Mark async tests clearly in comments
5. **Test isolation**: Each test should be self-contained and not depend on other tests
6. **Cleanup resources**: WebGL resources are automatically cleaned up when GL context is dropped
7. **Update this readme**: Add new tests to the documentation tables

## Troubleshooting

### "Canvas was not found" error
✅ **Fixed**: All tests now create canvas programmatically instead of retrieving from DOM.

### "This test suite is only configured to run in node.js"
✅ **Fixed**: `mod.rs` configures `run_in_browser` globally.

### GLTF deserialization errors
✅ **Fixed**: Tests use graceful fallback when GLTF files unavailable.

### Memory exhaustion in CI
✅ **Fixed**: Clippy runs with `-j 1` and `CODEGEN_UNITS=1` to reduce memory usage.

### Tests hanging or timing out
- Ensure browser is properly installed on test runner
- Check that WebGL 2.0 is available in test environment
- Verify async tests properly await all promises
