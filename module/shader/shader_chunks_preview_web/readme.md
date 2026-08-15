# shader_chunks_preview_web

**Keywords:** WebGPU, Browser Runner, wasm32, Live Preview, minwebgpu

WebGPU browser runner for shader chunk previews. Fetches the
`-preview.json`
[`shader_chunks_preview_core::PreviewBundle`](../shader_chunks_preview_core/readme.md)
that [`shader_chunks_preview`](../shader_chunks_preview/readme.md) wrote
next to this crate's `index.html`, compiles its already-composed,
already-naga-validated WGSL, creates one slider per bundle parameter
(`controls.js`), and renders full-screen — writing
`time`/params/`resolution` into one uniform buffer per the bundle's
layout convention
([`shader_chunks_preview_core::resolution_index`](../shader_chunks_preview_core/readme.md)).

**wasm32-only.** Every real dependency (`minwebgpu`, `serde_json`,
`serde-wasm-bindgen`, `web-sys`) is declared under
`[target.'cfg(target_arch = "wasm32")'.dependencies]`, and `main()` on
every other target is a stub printing a pointer to the real entry point —
so a native `cargo check`/`udeps` sweep sees zero dependencies for this
crate and there is nothing to unit-test on the host. It is served via
`action/browser_serve` (trunk), normally through
`shader_chunks preview <name>` rather than run directly.

**Shape:**

```text
main() [wasm32]
  -> bundle_fetch()                 // fetch + deserialize -preview.json; panics loudly if missing/stale
  -> minwebgpu: canvas, context, adapter, device, shader module, render pipeline
  -> one uniform buffer, sized resolution_index(params.len()) + 4 floats
  -> one slider per bundle.parameters (controls.js), wired to an on_change callback
  -> editor seeded with bundle.wgsl (controls.js), wired to a debounced on_edit callback
  -> exec_loop::run: each frame writes time/params/resolution, submits the draw
```

The render pipeline takes no explicit layout, so WebGPU derives an
"auto" bind-group layout from the shader's own uniform declaration
(`get_bind_group_layout(0)`) — the uniform buffer's field order must
therefore match the bundle's own convention exactly, which is why
[`shader_chunks_preview_core::resolution_index`] is shared rather than
recomputed here.

## Live editing

Editing the textarea recompiles the shader Shadertoy-style: ~500ms after
the last keystroke, `on_edit` fires with the current text and spawns an
async recompile task —
[`minwebgpu::shader::compilation_messages_get`](../../min/minwebgpu/readme.md)
awaits `GpuShaderModule.getCompilationInfo()`, and
[`has_blocking_error`](../../min/minwebgpu/readme.md) decides whether a
pipeline rebuild is even attempted. A blocking compile error, or a
`create_render_pipeline_async` rejection (e.g. an incompatible uniform
declaration), shows the diagnostic text in the panel under the editor and
returns without touching the render state; the last-good pipeline and bind
group — held in one combined `Rc<RefCell<PipelineState>>` cell so they
always swap together — keep rendering every frame in the meantime. Success
clears the diagnostics and swaps the new pipeline/bind-group pair in.

The uniform buffer's own layout (parameter count/order, the `resolution`
slot) is fixed from the bundle loaded at page load — an edit changes only
shader logic, never that layout. A shader edit that changes the uniform
struct shape incompatibly surfaces as a pipeline validation error in the
diagnostics panel rather than being auto-accommodated; re-composing which
chunks/params are in play is `shader_chunks preview`'s job, not this
runner's.

## Usage

Not run directly — invoke through the CLI, which builds the bundle,
writes `-preview.json` into this crate, and hands off to
`action/browser_serve`:

```sh
cargo run -p shader_chunks_preview -- preview fbm3
```

To check this crate compiles for its actual target without a browser:

```sh
cargo check -p shader_chunks_preview_web --target wasm32-unknown-unknown
```

## Consumer contract

This crate never builds or validates a bundle itself — that happens
entirely in [`shader_chunks_preview`](../shader_chunks_preview/readme.md)
before `-preview.json` is written. A missing or stale bundle file, or one
that fails to deserialize, panics loudly in the browser console with a
message naming the fix (`shader_chunks preview <name>`) rather than
rendering a blank or stale frame silently.

**Disclosed gap:** the live-edit recompile path (`recompile` in
`src/main.rs`) is not unit-tested — it can only be exercised against a
real `GpuShaderModule`/`GpuRenderPipeline` from a real `GpuDevice`, which
this workspace has no proven way to drive inside the headless native test
runner (see
[`tilemap_renderer/tests/webgpu_backend_test.rs`](../../helper/tilemap_renderer/tests/webgpu_backend_test.rs)'s
own identical disclosure). `minwebgpu::shader::has_blocking_error`, the
one piece of pure logic in that path, is covered by
[`shader_compilation_diagnostics_tests.rs`](../../min/minwebgpu/tests/shader_compilation_diagnostics_tests.rs);
the rest — seeding the editor, debouncing, spawning the recompile task,
swapping `PipelineState` on success, routing errors to the diagnostics
panel — is verified manually in-browser instead.
