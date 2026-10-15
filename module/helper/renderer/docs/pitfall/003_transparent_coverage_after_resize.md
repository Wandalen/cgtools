# Pitfall: Transparent Coverage Must Be Set Again After `resize`

`ToneMappingPass::transparent_coverage_texture_set` takes the texture handle
`Renderer::transparent_coverage_texture()` returns at that moment.
`Renderer::resize` replaces that texture, so a pass set up once at start-up
keeps a handle to a deleted texture after the first resize.

### Scope

- **Purpose**: Warn consumers that opting into the MSAA-edge tone mapping fix is not a one-time setup.
- **Responsibility**: Document the trap, the concrete failure, and the exact mitigation.
- **In Scope**: `ToneMappingPass::transparent_coverage_texture_set`, `Renderer::transparent_coverage_texture` and `Renderer::resize`.
- **Out of Scope**: The reconstruction itself (see `src/webgl/shaders/tonemapping/aces.frag`).

### Trap

`transparent_coverage_texture()` returns the WBOIT accumulation texture of the
renderer's current framebuffer context. `resize` drops that context, which
deletes its textures, and creates a new one. The tone mapping pass does not own
the texture and binds whatever handle it was last given; nothing tells it the
handle went stale, and nothing fails.

### Failure

After a resize the pass binds a deleted texture. WebGL2 rejects that bind with
`INVALID_OPERATION` and leaves the unit's previous binding in place, so the
pass reads transparent coverage from an unrelated texture. Depending on what
that texture holds, the MSAA-edge fix silently stops working (bright
silhouettes get their light fringe back) or misreads glass as uncovered and
draws a 1-px ring around transparent surfaces. A page that is never resized,
like most tests, never shows it.

### Mitigation

Call the setter again right after every `resize`, as `gltf_viewer` does:

```rust
renderer.resize( &gl, width, height, samples )?;
tonemapping.transparent_coverage_texture_set( renderer.transparent_coverage_texture() );
```

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | The tone mapping pass and the WBOIT transparency it reads coverage from |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/post_processing/tonemapping.rs` | `transparent_coverage_texture_set` stores the handle the pass binds |
| `src/webgl/renderer.rs` | `transparent_coverage_texture` returns the current handle; `resize` replaces the framebuffer context |
| `examples/minwebgl/gltf_viewer/src/main.rs` | Sets the texture after creation and after every resize |

### Tests

| File | Relationship |
|------|--------------|
| `tests/tonemapping_edge_readback_test.rs` | Covers the reconstruction with a valid texture; the stale handle itself is not tested |
