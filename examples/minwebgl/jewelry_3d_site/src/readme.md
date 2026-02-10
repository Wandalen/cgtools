# Source Directory

This directory contains the core Rust/WASM implementation for the 3D jewelry configurator.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Exposes public API for library crate, enables integration tests, and configures global lint settings |
| `main.rs` | Initializes WASM application, coordinates render loop, and handles UI state synchronization |
| `configurator/` | Modular jewelry configurator with focused submodules for state, animation, rendering, and ring loading (see `configurator/readme.md`) |
| `gem.rs` | Renders gemstone materials with ray-traced refraction, dispersion, and environment mapping |
| `surface_material.rs` | Configures metallic surface material properties, shaders, and texture binding |
| `cube_normal_map_generator.rs` | Generates cube normal maps for environment reflections on gem surfaces |
| `scene_utilities.rs` | Provides utility functions for scene node filtering, texture templates, and canvas event handling |
| `ui.rs` | Provides JavaScript interop layer for bidirectional UI state synchronization |
| `gem_frag.rs` | Defines minified fragment shader source for gemstone ray-tracing rendering |
| `gem_vert.rs` | Defines minified vertex shader source for gemstone geometry transformation |

## Architecture Overview

The codebase follows a component-based architecture:

- **Rendering Pipeline**: `main.rs` → `configurator/mod.rs` → material systems (`gem.rs`, `surface_material.rs`)
- **State Management**: `ui.rs` ↔ JavaScript ↔ `configurator/mod.rs`
- **Asset Loading**: `configurator/rings.rs` (lazy GLTF loading with race condition prevention via `gpu_sync.rs`)
- **Shader System**: `gem_vert.rs` + `gem_frag.rs` → compiled into materials

## Key Features

- Lazy loading of ring models to optimize initial load time
- Runtime shadow baking using `bake_plane_shadow`
- Per-ring color state persistence across ring switches
- WebGL error resilience with graceful fallbacks
