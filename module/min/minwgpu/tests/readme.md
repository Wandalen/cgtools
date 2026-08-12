# tests

Native tests for `minwgpu`'s deterministic public surface (established by task 070),
runnable without a GPU via `cargo test -p minwgpu --all-features`: an instance created
with `wgpu::Backends::empty()` can never provide an adapter, so the adapter-request error
paths are exercisable on any host, and the builders' state accumulation is observable
through their `get_*` getters without ever calling `build`/`finish_context`.
GPU-dependent behavior (`build`, `finish_context`, `texture`) has no native test story —
it needs a real adapter/device.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| buffer_test.rs | Buffer and vertex-buffer builder state accumulation via getters |
| context_test.rs | Context builder state accumulation, adapter-request error paths, selector priority |
| helper_test.rs | helper attr field mapping and request_adapter shortcut error path |
