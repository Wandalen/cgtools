# ADR-004: Native Vulkan HAL Backend via a Dedicated `minvulkan` Driver

- **Date**: 2026-08-16
- **Status**: Accepted
- **Deciders**: wandalen

## Context

[ADR-003](003_d2_stack_hal_adoption.md) Decision #3 ruled that Vulkan is "a
backend-selection detail, not a new adapter": forcing `wgpu` onto its Vulkan
backend happens inside `adapter-native`'s construction, and
[`examples/orrery/readme.md`](../../examples/orrery/readme.md)'s "Planned
members" list echoed the same posture — Vulkan was to be "a run mode of" the
planned `wgpu/` orrery member, not a separate crate.

A new requirement supersedes that posture for one specific consumer.
`examples/orrery/flexible` needs four independently selectable rendering
backends — webgl, webgpu, wgpu, and vulkan — governed by an explicit product
principle: **only the `wgpu` backend option may link the `wgpu` crate; the
other three (webgl, webgpu, vulkan) must not pull it in, even
transitively.** Reusing `wgpu`-forced-to-Vulkan for the "vulkan" option would
violate this directly — `gpu_hal`'s existing `native` feature (the only path
to Vulkan today) depends on `minwgpu` and `wgpu` unconditionally
(`module/helper/gpu_hal/Cargo.toml`'s `native = ["dep:minwgpu", "dep:wgpu"]`),
so selecting "vulkan" would transitively select "wgpu" — collapsing two of
the four options the principle requires to stay distinct.

No existing L0 driver offers Vulkan without going through `wgpu`. Delivering
a genuinely `wgpu`-free Vulkan option requires a new driver.

## Decision

1. **A new L0 driver, `minvulkan`, wraps raw Vulkan via `ash`.** It follows
   [layer/001](../layer/001_l0_drivers.md)'s existing driver contract —
   backend-faithful (exposes real Vulkan objects and concepts, no
   cross-backend vocabulary), thin (setup/buffer/error ergonomics only, no
   policy), a terminal drill-down target. It does not depend on `wgpu`,
   `minwgpu`, or any other driver.
2. **`gpu_hal` (L1) gains a fourth backend variant, `vulkan`, backed by
   `minvulkan`.** It follows the same enum-per-backend dispatch pattern
   [ADR-002](002_gpu_hal_in_house.md) established for the existing three
   variants (`WebGpu`, `WebGl`, `Native`) — a plain enum with one
   `#[cfg(feature = "vulkan")]`-gated variant, public non-panicking `as_*()`
   and crate-private panicking `expect_*()` accessors, and its own explicit
   constructor (mirroring `Device::new_webgpu`/`new_webgl`/`new_native`). The
   existing `native` variant (`minwgpu` + `wgpu`, letting `wgpu` pick its own
   backend) is unchanged and remains the "wgpu" option; `vulkan` is a
   distinct, parallel variant, not a configuration of `native`.
3. **`examples/orrery/flexible` selects among the four via Cargo features,
   each mapping to exactly one `gpu_hal` backend feature**: `webgl` →
   `gpu_hal/webgl`, `webgpu` → `gpu_hal/webgpu`, `wgpu` → `gpu_hal/native`,
   `vulkan` → `gpu_hal/vulkan`. Selection is compile-time (mirroring
   `examples/gpu_hal/triangle_browser`'s existing `[features]` pattern) —
   `wgpu` and `vulkan` are additionally mutually exclusive with the two
   browser-only options at the `target_arch` level, same as every other
   native-vs-browser `gpu_hal` split today.
4. **Shader source stays canonical WGSL.** `minvulkan` compiles the same
   WGSL sources the other backends use, via `naga`'s WGSL→SPIR-V backend
   (already a workspace dependency, already used internally by `wgpu`
   itself) — no new shader-language fork, consistent with
   [layer/002](../layer/002_l1_gpu_hal.md)'s "shader access, not shader
   hiding" contract.

This decision is scoped to `gpu_hal`'s own backend set and its new
`examples/orrery/flexible` consumer. It does not reopen
[ADR-003](003_d2_stack_hal_adoption.md)'s own scope — `tilemap_renderer`'s
`adapter-native` keeps forcing `wgpu`'s Vulkan backend exactly as ADR-003
decided; nothing here requires `tilemap_renderer` (or any other existing
consumer) to adopt the new `vulkan` variant.

## Alternatives Considered

- **Force `wgpu` onto its Vulkan backend for the "vulkan" option (reuse
  `gpu_hal`'s existing `native` feature), same as ADR-003 Decision #3 and
  the original orrery `wgpu/` plan.** Rejected: this is the cheaper option,
  but it makes the "vulkan" and "wgpu" options structurally identical (both
  link `wgpu`), directly violating the stated one-quarter-only principle.
  ADR-003's reasoning ("Vulkan is a `wgpu` backend selection, not a distinct
  command-stream target") was sound for its own question — whether
  `tilemap_renderer`'s L3 `Backend` trait needed a dedicated Vulkan adapter —
  but does not answer this different question, whether an L1/L0 path to
  Vulkan can exist without `wgpu` at all.
- **Do nothing; drop "vulkan" from `examples/orrery/flexible`'s option set,
  ship 3 backends instead of 4.** Rejected: the four-backend set, with the
  one-quarter-wgpu constraint, is the explicit request this crate exists to
  satisfy — dropping the fourth option abandons the requirement rather than
  meeting it.
- **A standalone `minvulkan`-only example, bypassing `gpu_hal` entirely.**
  Rejected: would leave `vulkan` as a one-off, uncomposable with the other
  three options inside `examples/orrery/flexible`'s single configurable
  crate, and would bypass L1's whole purpose (one API written once per stack
  instead of once per backend — [layer/002](../layer/002_l1_gpu_hal.md)).

## Consequences

- **Positive**: `gpu_hal` gains a true fourth backend, extending its
  enum-per-backend pattern's proof surface; `examples/orrery/flexible` can
  honor the one-quarter-wgpu principle exactly, with each of its four Cargo
  features pulling in exactly the driver dependencies it needs and no
  others.
- **Negative**: A new driver crate (`minvulkan`) and a new `gpu_hal` backend
  variant are real, ongoing maintenance surface — Vulkan's C API is verbose
  and `ash` is unsafe-heavy compared to `wgpu`'s already-safe abstraction;
  this is the direct cost of the wgpu-free requirement.
- **Neutral**: `examples/orrery/readme.md`'s "Planned members" entry for
  `wgpu/` is narrowed — it no longer carries Vulkan as one of its own run
  modes; that capability now lives in `flexible/`'s dedicated `vulkan`
  feature instead.

## Related

- [002_gpu_hal_in_house.md](002_gpu_hal_in_house.md) — the enum-per-backend
  dispatch pattern this ADR extends with a fourth variant
- [003_d2_stack_hal_adoption.md](003_d2_stack_hal_adoption.md) — Decision
  #3's Vulkan-as-backend-selection ruling, narrowed by this ADR for the new
  one-quarter-wgpu requirement; `tilemap_renderer`'s own adapter surface is
  otherwise unaffected
- [layer/001_l0_drivers.md](../layer/001_l0_drivers.md) — L0's driver
  contract `minvulkan` follows
- [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) — L1's contract the
  new `vulkan` variant extends
- `examples/orrery/readme.md` — the new `flexible/` planned member and the
  narrowed `wgpu/` entry
