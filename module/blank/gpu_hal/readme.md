# gpu_hal

Reserved crate for the L1 GPU hardware abstraction layer of the cgtools
rendering architecture — the one crate through which stack engines will reach
`minwebgl`, `minwebgpu`, and `minwgpu` without knowing which backend they run
on. No implementation yet.

The contract this slot must satisfy (WebGPU-shaped API, canonical WGSL with
per-backend override, one-step drill-down to the raw driver, no stack
vocabulary) is documented at the workspace level:

- `docs/layer/002_l1_gpu_hal.md` — the layer's contract
- `docs/explorations/001_gpu_hal_buy_vs_build.md` — the open build-vs-buy question gating implementation
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves
