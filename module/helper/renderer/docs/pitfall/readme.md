# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `renderer`, each instance is a confirmed trap grounded in the current source, spelling out its concrete failure mode and how to mitigate it. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for confirmed traps consumers of `renderer` hit in practice.
- **Responsibility**: Document each trap, its concrete failure mode, and its mitigation.
- **In Scope**: Traps confirmed against the current source.
- **Out of Scope**: Guaranteed behavior (see `invariant/`); subsystem design (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Requires EXT_color_buffer_float](001_requires_ext_color_buffer_float.md) | The crate renders into `RGBA16F` targets but never enables the WebGL2 extension that makes them color-renderable — callers must | ✅ |
| 002 | [glTF images upload after load](002_gltf_images_upload_after_load.md) | `gltf::load` returns with placeholder textures; a redraw-on-change loop must also redraw when `texture_upload_count_get()` moves | ✅ |
| 003 | [Transparent coverage after resize](003_transparent_coverage_after_resize.md) | `ToneMappingPass::transparent_coverage_texture_set` must be called again after every `Renderer::resize`, which deletes the texture it was given | ✅ |
| 004 | [The meshopt decoder snippet must be deployed](004_meshopt_decoder_snippet_deployment.md) | The glTF loader imports meshoptimizer's decoder statically, so every page needs `pkg/snippets/` and `wasm-unsafe-eval`, and `--target no-modules` cannot build | ✅ |
