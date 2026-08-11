# Pitfall: Requires EXT_color_buffer_float

The crate renders into `RGBA16F` color attachments, but never enables the
WebGL2 extension that makes float formats color-renderable. Enabling it is
the caller's job, and forgetting it fails at framebuffer setup — on some
devices, only in the field.

### Scope

- **Purpose**: Warn every consumer that `Renderer` (and the PMREM/IBL path) has a hard, caller-side environment requirement.
- **Responsibility**: Document the trap, the concrete failure, and the exact mitigation.
- **In Scope**: The `EXT_color_buffer_float` requirement created by the crate's float render targets.
- **Out of Scope**: Why the pipeline uses float targets at all (see [../invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md)).

### Trap

Core WebGL2 can *sample* float textures but cannot *render into* them:
`RGBA16F` becomes color-renderable only when the `EXT_color_buffer_float`
extension is enabled on the context. This crate allocates `RGBA16F`
color attachments throughout (`src/webgl/renderer.rs`) and prefilters
environment maps into float targets (`src/webgl/loaders/pmrem.rs`), yet it
never calls `get_extension` itself — activation is left entirely to the
caller, and nothing in the API signature reveals that.

### Failure

Without the extension, framebuffer completeness fails the moment a float
attachment is bound for rendering — `src/webgl/loaders/pmrem.rs` documents
the exact condition: when "`EXT_color_buffer_float` is unavailable", the
"RGBA16F attachment is not color-renderable". Symptoms are a
`FRAMEBUFFER_INCOMPLETE`-class error (or a construction-time panic where a
result is unwrapped) rather than a message naming the missing extension —
and since desktop dev machines virtually always support the extension, the
failure typically first appears on end-user hardware.

### Mitigation

Enable the extension on the context before constructing any renderer or
loading any environment, exactly as the crate's own quick-start does
(`readme.md`, Basic Rendering Setup):

```rust
gl.get_extension( "EXT_color_buffer_float" )
```

Treat a `None`/`Err` return as "this device cannot run the HDR pipeline" and
fail with a clear message — do not proceed to renderer construction.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/003_hdr_internal_tone_mapped_output.md](../invariant/003_hdr_internal_tone_mapped_output.md) | The HDR contract whose `RGBA16F` targets create this requirement |

### Sources

| File | Relationship |
|------|--------------|
| `readme.md` | Quick-start shows the required caller-side `get_extension` call |
| `src/webgl/loaders/pmrem.rs` | Documents the not-color-renderable failure when the extension is absent |
| `src/webgl/renderer.rs` | The `RGBA16F` attachments that impose the requirement |

### Tests

| File | Relationship |
|------|--------------|
| — | Not testable headlessly in this suite; the condition depends on the browser context's extension support |
