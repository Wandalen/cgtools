# Pitfall: GPU Instance Struct Field-Reorder Desync

### Scope

- **Purpose**: Record the exact boundary of what the compile-time layout assertions on `SpriteInstanceData`/`MeshInstanceData` do and do not catch.
- **Responsibility**: Document the trap, the concrete failure mode, and the partial nature of the existing mitigation.
- **In Scope**: `SpriteInstanceData` and `MeshInstanceData` (`src/adapters/webgl/webgl_helpers.rs`) and the hardcoded vertex-attribute offsets that assume their field order.
- **Out of Scope**: The `swap_remove` buffer-binding constraint on the `ArrayBuffer<T>` that stores these structs (see [pitfall/001](001_arraybuffer_swap_remove_buffer_binding_violation.md)).

### Trap

Assuming the compile-time assertions `assert!( core::mem::size_of::<SpriteInstanceData>() == 72 )` / `== 56` and the matching `align_of` assertions (`src/adapters/webgl/webgl_helpers.rs`, immediately after the two struct definitions) are sufficient to keep the GPU vertex-attribute setup in sync with the struct definitions after any future edit to `SpriteInstanceData` or `MeshInstanceData`.

### Failure

Both structs are `#[repr(C)]`, so their field order is fixed to declaration order — but the VAO attribute setup that reads them (`vertex_attrib_pointer_with_i32` calls in `webgl_helpers.rs`) uses **hardcoded byte offsets** that must independently match that field order: for `SpriteInstanceData`, offsets `0`/`12`/`24` (the three `transform` rows), `36` (`region`), `52` (`tint`), `68` (`depth`); for `MeshInstanceData`, offsets `0`/`12`/`24` (`transform`), `36` (`depth`), `40` (`tint`). The `size_of`/`align_of` assertions only check the struct's *total* footprint and alignment. Reordering two same-size fields — for example swapping `region` and `tint` in `SpriteInstanceData`, both `[f32; 4]` — leaves `size_of` and `align_of` unchanged, so the assertions still pass and the crate still compiles cleanly, but every hardcoded offset after the reordered fields now points at the wrong field. The GPU would read `tint` bytes where the shader expects `region` bytes (or vice versa) with no compiler error and no runtime error — only visually wrong sprite/mesh rendering (wrong tint or wrong sprite-sheet sub-rect) would reveal it.

### Mitigation

Partial. The two `size_of` assertions and two `align_of` assertions (`webgl_helpers.rs`, directly below the struct definitions, comment: "GPU attrib setup depends on these exact sizes") catch any edit that changes a struct's total size or alignment — e.g. adding, removing, or widening a field. They do **not** catch a same-footprint field reorder, because no per-field offset assertion (e.g. an `offset_of!`-based check) exists in the source as of this migration. Keeping the hardcoded offsets correct after a field reorder currently depends entirely on the person making the edit updating them by hand.

### Features

| File | Relationship |
|------|--------------|
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | `SpriteInstanceData`/`MeshInstanceData` are the per-instance GPU layout for sprite and mesh batches |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/webgl/webgl_helpers.rs` | Struct definitions, the four compile-time assertions, and the hardcoded `vertex_attrib_pointer_with_i32` offsets for both sprite and mesh batch VAOs |

### Tests

| File | Relationship |
|------|--------------|
| — | No automated test currently exercises the WebGL2 adapter (it requires a `wasm32` target and a live WebGL2 context); see [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) for the crate's WebGL2 test-coverage status |
