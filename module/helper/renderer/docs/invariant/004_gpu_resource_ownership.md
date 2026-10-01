# Invariant: Every GPU Object Has Exactly One Deleter

Every WebGL object the crate creates is deleted by exactly one owner, and
never by a value that merely holds a handle to it. `web_sys` handles are
JS-object references, so cloning one aliases the GPU object instead of
duplicating it; ownership therefore has to be expressed in the types, not in
the handles.

### Scope

- **Purpose**: Pin who deletes which GPU object, so `Drop` impls release resources without double deletes or deleting objects still in use.
- **Responsibility**: State the ownership rules for the shared scene-graph types, name where they are enforced, and record what breaks when they are violated.
- **In Scope**: `Geometry`, `Texture` / `TextureOwner`, `TransformsData` / `DisplacementsData`, and the `Clone` semantics that keep them single-deleter.
- **Out of Scope**: Passes and render targets with their own `gl_resources_free` / `Drop` (`GBuffer`, `SwapFramebuffer`, bloom, outline, shadow, `IBL`), each documented at its type.

### Invariant Statement

- A **`Geometry`** owns exactly its VAO (created in `Geometry::new`) and deletes it on drop. Attribute and index buffers passed to `attribute_add` / `index_add` are borrowed: they may back several geometries (the glTF loader shares one buffer per bufferView), so no `Geometry` deletes them; the code that created a buffer releases it. `Geometry` is not `Clone`; it is shared through `Rc<RefCell<Geometry>>`, and `Primitive::clone` / `Mesh::clone` / `Node::tree_clone` share that `Rc`.
- A **`Texture`** is a non-owning view unless built with `Texture::owning` (or `Texture::load_from_path`). An owning texture holds an `Rc<TextureOwner>`; clones share it, and the GPU texture is deleted once, when the last clone drops. A view (Former builder, `Default`) never deletes anything.
- **`TransformsData` / `DisplacementsData`** own the textures their own `upload()` created. `Clone` aliases the handles but resets `gl` to `None`, so a clone deletes nothing until its first `upload()` has allocated textures of its own (BUG-437, BUG-533).

### Enforcement Mechanism

- `src/webgl/geometry.rs`: `impl Drop for Geometry` deletes only `vao`; `Geometry` derives only `Debug`.
- `src/webgl/texture.rs`: only `Texture::owning` can make a `TextureOwner` (its fields are private and it has no public constructor), and the `Former` builder has no `owner` setter; `TextureOwner` is the only type with a texture-deleting `Drop`.
- `src/webgl/skeleton.rs`: both `Clone` impls set `gl : None` (`Fix(BUG-533)`).
- Tests (`tests/webgl/`, wasm32, live context): `geometry_gl_lifecycle.rs` (shared buffers survive a dropped geometry), `texture_gl_lifecycle.rs` (owning texture outlives all but its last clone; views delete nothing), `skeleton_gl_lifecycle.rs` (a dropped clone keeps the original's textures).

### Violation Consequences

- Deleting a borrowed buffer: survivors keep drawing from the orphaned storage, but every later call naming the buffer (`Geometry::upload`, `attribute_add`, `bind_buffer` + data update) fails with `INVALID_OPERATION`, so updates silently go nowhere.
- Two owners of one handle: the second delete hits an object that may already be reused, or the first delete removes a texture the other owner still samples (black / incomplete texture).
- Views that delete: framebuffer attachments or shared glTF images disappear while still in use.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The scene graph whose geometries, textures and skins these rules govern |
