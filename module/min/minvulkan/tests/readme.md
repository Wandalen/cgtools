# tests

Integration tests for `minvulkan`, runnable via `cargo test -p minvulkan --all-features`
against a live local Vulkan ICD (see the crate readme's Status section).

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| context_test.rs | `Context::builder()` construction against a live Vulkan ICD (BUG-290) |
| surface_test.rs | Pure edges of the windowed presentation path reachable without a real window |
| swapchain_test.rs | `Swapchain::rebuild` error-path handle/view cleanup (BUG-424) |
