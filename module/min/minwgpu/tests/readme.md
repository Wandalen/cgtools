# tests

Native tests for `minwgpu`'s deterministic public surface (established by task 070),
runnable without a GPU via `cargo test -p minwgpu --all-features`: an instance created
with `wgpu::Backends::empty()` can never provide an adapter, so the adapter-request error
paths are exercisable on any host. Builder state-accumulation coverage lives in the two
documented-exception inline test modules in `src/` (they read internal fields no public
getter exposes); GPU-dependent behavior (`build`, `finish_context`, `texture`) has no
native test story — it needs a real adapter/device.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| context_test.rs | Context builder adapter-request error paths and selector priority |
| helper_test.rs | helper attr field mapping and request_adapter shortcut error path |
