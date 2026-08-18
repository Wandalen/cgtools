# tests

Native tests for `minwgpu`'s deterministic public surface (established by task 070),
runnable without a GPU via `cargo test -p minwgpu --all-features`: an instance created
with `wgpu::Backends::empty()` can never provide an adapter, so the adapter-request error
paths are exercisable on any host, and the builders' state accumulation is observable
through their `*_get` getters without ever calling `build`/`context_finish`.
GPU-dependent behavior (`build`, `context_finish`, `texture::render_target_2d`'s actual
`wgpu::Device::create_texture` call) has no native test story — it needs a real adapter/device.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| buffer_test.rs | Buffer and vertex-buffer builder state accumulation via getters |
| context_test.rs | Context builder state accumulation, adapter-request error paths, selector priority |
| helper_test.rs | helper attr field mapping and adapter_request shortcut error path |
| readback_test.rs | Pure row-padding/strip math and BGRA→RGBA swizzle behind readback::rgba8 (BUG-166) |
| surface_test.rs | Pure format-selection and zero-size validation behind surface::surface_configure (BUG-165) |
| texture_test.rs | Pure zero-size validation behind texture::render_target_2d (BUG-276) |
