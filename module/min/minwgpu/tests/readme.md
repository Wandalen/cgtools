# tests

Native tests for `minwgpu`'s deterministic public surface (established by task 070),
runnable without a GPU via `cargo test -p minwgpu --all-features`: an instance created
with `wgpu::Backends::empty()` can never provide an adapter, so the adapter-request error
paths are exercisable on any host, and the builders' state accumulation is observable
through their `*_get` getters without ever calling `build`/`context_finish`.
`context_finish` and `texture::render_target_2d`'s actual `wgpu::Device::create_texture` call
now have native coverage too, in `live_context_test.rs`, against a real adapter on
`wgpu::Backends::PRIMARY` — skipped with a clear stderr reason (not a panic or hard failure)
on a host with no adapter reachable on that backend. `Buffer`/`VertexBuffer`'s own `build`
still has no native test story for the same reason ( it too needs a live `wgpu::Device` ) and
remains uncovered here.

The windowed presentation path added alongside `Windowed`/`from_window` is covered only at
its edges for the same reason — a real window handle is as unavailable here as a real
adapter. `context_test.rs` pins the two parts that are reachable without one: that
`instance_get` becomes available exactly at `instance_make` ( the step that makes
surface-before-adapter ordering possible at all ), and that `windowed_with` rejects a
zero-sized drawable *before* requesting an adapter. `pipeline_test.rs` covers the render
pipeline builder's state the same way `buffer_test.rs` covers the buffer builders. What
stays uncovered natively: `from_window`, `frame_acquire`'s outcome mapping, `Windowed`'s
own accessors, and `pass::Draw` — each needs either a live window or a live `wgpu::Device`
to construct its inputs.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| buffer_test.rs | Buffer and vertex-buffer builder state accumulation via getters |
| context_test.rs | Context builder state accumulation, adapter-request error paths, selector priority |
| helper_test.rs | helper attr field mapping and adapter_request shortcut error path |
| pipeline_test.rs | Render pipeline builder state accumulation via getters |
| live_context_test.rs | Real-adapter coverage: `context_finish` producing a usable `Device`/`Queue`, and `texture::render_target_2d`'s actual `create_texture` call against a live device |
| readback_test.rs | Pure row-padding/strip math and BGRA→RGBA swizzle behind readback::rgba8 (BUG-166) |
| surface_test.rs | Pure format-selection and zero-size validation behind surface::surface_configure (BUG-165) |
| texture_test.rs | Pure zero-size validation behind texture::render_target_2d (BUG-276) |
