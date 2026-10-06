# Invariant: Owned GPU Objects Have Exactly One Deleter

Every GPU object a scene-graph type owns (a `Geometry`'s VAO, an owning
`Texture`'s texture, a skin's joint-matrix and morph-target textures) is
deleted by exactly one owner, and never by a value that merely holds a handle
to it. `web_sys` handles are JS-object references, so cloning one aliases the
GPU object instead of duplicating it; ownership therefore has to be expressed
in the types, not in the handles.

Not every object the crate creates has an owner yet: the glTF loader's
buffers and images have none (see Known Gaps), so the invariant is partly
implemented.

### Scope

- **Purpose**: Pin who deletes which GPU object, so `Drop` impls release resources without double deletes or deleting objects still in use.
- **Responsibility**: State the ownership rules for the shared scene-graph types, name where they are enforced, and record what breaks when they are violated.
- **In Scope**: `Geometry`, `Texture` / `TextureOwner`, `TransformsData` / `DisplacementsData`, the `Renderer`'s skybox, and the `Clone` semantics that keep them single-deleter.
- **Out of Scope**: Passes and render targets with their own `gl_resources_free` / `Drop` (`GBuffer`, `SwapFramebuffer`, bloom, outline, shadow, `IBL`), each documented at its type.

### Invariant Statement

- A **`Geometry`** owns exactly its VAO (created in `Geometry::new`) and deletes it on drop. Attribute and index buffers passed to `attribute_add` / `index_add` are borrowed: they may back several geometries (the glTF loader shares one buffer per bufferView), so no `Geometry` deletes them; the code that created a buffer releases it. `Geometry` is not `Clone`; it is shared through `Rc<RefCell<Geometry>>`, and `Primitive::clone` / `Mesh::clone` / `Node::tree_clone` share that `Rc`.
- A **`Texture`** is a non-owning view unless built with `Texture::owning` (or `Texture::load_from_path`). An owning texture holds an `Rc<TextureOwner>`; clones share it, and the GPU texture is deleted once, when the last clone drops. A view (Former builder, `Default`) never deletes anything.
- **`TransformsData` / `DisplacementsData`** own the textures their own `upload()` created. `Clone` resets the texture handles (and `gl`) to `None`, so a clone never holds the original's textures and allocates its own on its first `upload()` (BUG-437, BUG-533).
- The **`Renderer`** holds its skybox as a `Texture` and never deletes it itself: an owning skybox is released through its `TextureOwner` when the renderer's clone (and every other) drops; a view stays its creator's.

### Enforcement Mechanism

- `src/webgl/geometry.rs`: `impl Drop for Geometry` deletes only `vao`; `Geometry` derives only `Debug`.
- `src/webgl/texture.rs`: only `Texture::owning` can make a `TextureOwner` (its fields are private and it has no public constructor), and the `Former` builder has no `owner` setter; `TextureOwner` is the only type with a texture-deleting `Drop`.
- `src/webgl/skeleton.rs`: both `Clone` impls reset the texture handles and `gl` to `None` (`Fix(BUG-533)`).
- `src/webgl/renderer.rs`: `skybox_set` takes a `Texture`, kept outside `FramebufferContext`; no `delete_texture` touches it.
- Tests (`tests/webgl/`, wasm32, live context; run with `wasm-pack test --headless --chrome -- --features test_internals --test tests`, since `skeleton_gl_lifecycle.rs`, `renderer_gl_lifecycle.rs` and the `load_from_path` tests in `texture_gl_lifecycle.rs` compile only with `test_internals`): `geometry_gl_lifecycle.rs` (shared buffers survive a dropped geometry; a cloned primitive shares its geometry), `texture_gl_lifecycle.rs` (owning texture outlives all but its last clone; views delete nothing; a dropped `load_from_path` texture skips its pending upload), `skeleton_gl_lifecycle.rs` (a clone holds none of the original's textures and uploads its own, in either drop order), `renderer_gl_lifecycle.rs` (the renderer keeps an owning skybox across `resize()` and never deletes a view).

### Known Gaps

- The glTF loader (`src/webgl/loaders/gltf.rs`) creates one buffer per bufferView and one texture per image, and wraps the images as non-owning `Texture` views; nothing deletes either, so they live as long as the context. Until the loader gets a release path, these objects have no deleter rather than one.

### Violation Consequences

- Deleting a borrowed buffer: survivors keep drawing from the orphaned storage, but every later bind of the buffer (`Geometry::upload`, `attribute_add`, `bind_buffer` + data update) fails with `INVALID_OPERATION` and leaves the previous binding in place. minwebgl never unbinds `ARRAY_BUFFER`, so a following `bufferData` lands in whichever buffer is still bound (possibly another geometry's), and `vertexAttribPointer` can point a VAO at it.
- Two owners of one handle: the first delete removes a texture the other owner still samples. Its binds then fail with `INVALID_OPERATION`, so the unit keeps sampling whatever texture was bound there before, or an incomplete texture if none was. (WebGL never reuses a deleted object, so the second delete itself is a no-op.)
- Views that delete: framebuffer attachments or shared glTF images disappear while still in use.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The scene graph whose geometries, textures and skins these rules govern |
