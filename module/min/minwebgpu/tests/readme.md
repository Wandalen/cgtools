# tests

Browser tests for `minwebgpu`'s WebGPU bindings (`target_arch = "wasm32"`-gated), run via
`cargo test --target wasm32-unknown-unknown --all-features` (executed for real through
geckodriver — see the workspace `verb/test`'s Stage 2).

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| bind_group_layout_entry_tests.rs | Verifies `BindGroupLayoutEntry` conversion rejects a missing binding type |
| context_adapter_device_request_tests.rs | Verifies `adapter_request`/`device_request` return `Result`, never panic (BUG-162) |
| shader_compilation_diagnostics_tests.rs | Verifies shader compilation message severity classification |
| texture_descriptor_tests.rs | Verifies `TextureDescriptor`'s default format supports STORAGE_BINDING (BUG-300) |
| vertex_attribute_tests.rs | Verifies `format_to_size` covers all 41 `GpuVertexFormat` variants (BUG-163) |
| webgpu_unsupported_tests.rs | Verifies `preferred_format` returns `Result`, never panics, when WebGPU is unsupported (BUG-164) |
