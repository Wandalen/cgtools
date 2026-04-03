# jewelry_configurator/src

WebGL jewelry product configurator with real-time gem ray-tracing, PBR metal rendering, and cinematic camera.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Provide WebGL jewelry renderer public API and WASM entry point |
| `main.rs` | Handle browser entry point, GUI setup, and render loop |
| `gem_material.rs` | Render gems with multi-bounce ray-tracing refraction |
| `surface_material.rs` | Render ground plane with shadow texture |
| `cube_normal_map_generator.rs` | Generate cube normal maps for gem refraction |
| `scene_setup.rs` | Configure scene: camera, materials, ground plane, shadows, gems |
| `lil_gui.rs` | Bind JavaScript lil-gui library via FFI |
