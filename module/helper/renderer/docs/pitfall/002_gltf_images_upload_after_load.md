# Pitfall: glTF Images Upload After `load` Returns

`webgl::loaders::gltf::load` returns before any image is on the GPU: every
texture starts as a 1x1 white placeholder and receives its pixels later, from
the image element's `onload`. A caller that draws only when something changes
keeps showing the placeholder frame.

### Scope

- **Purpose**: Warn consumers that a resolved `load` future does not mean the model's textures are ready.
- **Responsibility**: Document the trap, the concrete failure, and the exact mitigation.
- **In Scope**: The asynchronous image uploads of the glTF loader and redraw-on-change render loops.
- **Out of Scope**: Image decode failures, which keep the placeholder and are logged (see `src/webgl/loaders/gltf.rs`).

### Trap

`load` creates each texture with a 1x1 white placeholder, starts an `<img>`
decode and returns. The decoded pixels replace the placeholder in the
element's `onload` handler, at some later task — after `load` has resolved
and usually after the first frame has been drawn. Nothing in the returned
`GLTF` signals that, and the upload changes no scene state a render loop
would compare.

### Failure

A loop that skips frames when the camera and scene are unchanged draws the
model once, with the white placeholders, and then stops drawing. The
textures arrive on the GPU but never reach the screen: a textured model
renders blank white until an unrelated change, such as a camera move,
triggers the next frame. Loops that draw every frame never notice, which is
why the examples don't show it.

### Mitigation

Treat a change of `webgl::loaders::gltf::texture_upload_count_get()` as a
reason to redraw. The count grows by one each time a decoded image replaces
its placeholder, for every `load` since the page started, and reading it is
one thread-local load:

```rust
let uploads = renderer::webgl::loaders::gltf::texture_upload_count_get();
if uploads != last_uploads || scene_changed
{
  last_uploads = uploads;
  // draw the frame
}
```

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The material textures the placeholders stand in for |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/loaders/gltf.rs` | `texture_upload` creates the placeholder and uploads the image on `onload`; `texture_upload_count_get` counts the uploads |

### Tests

| File | Relationship |
|------|--------------|
| — | Not covered: the upload depends on the browser's image decode timing |
