# Pitfall: Backend Availability Is Compile-Time, Not Runtime

### Scope

- **Purpose**: Record that calling an unavailable backend's constructor in `gpu_hal` is a compile error, not a runtime error — the opposite failure shape from the sibling `min*` drivers' own target-mismatch traps.
- **Responsibility**: Document the trap, its observable failure, and the mitigation available to callers.
- **In Scope**: `target_arch`/feature-conditional compilation of backend variants and constructors across `lib.rs`, `device.rs`, `resource.rs`, `pass.rs`.
- **Out of Scope**: WebGPU/WebGL/native behavior once a backend is actually compiled in (see `feature/001`–`feature/006`).

### Trap

Assuming, by analogy with `minwebgpu`'s and `minwebgl`'s own "always compiles, fails at runtime on the wrong target" stub behavior (see `minwebgpu`'s `docs/pitfall/001_native_target_compiles_to_nonfunctional_stub.md`), that `gpu_hal` behaves the same way — that building with only the `native` feature enabled still lets code call `Device::new_webgpu`/`new_webgl` and get a runtime error back. It does not.

### Failure

Each backend's variants and constructors are `#[cfg(all(feature = "...", target_arch = "..."))]`-gated at the type level, not just inside the function body. When a backend's `cfg` doesn't hold, its `Device`/`Queue`/`Surface`/every resource enum's corresponding variant does not exist in the compiled crate at all — and neither does `Device::new_webgpu`/`new_webgl`. Code referencing `Device::new_webgpu` while compiled with only `--features native` fails with an unresolved-item compile error, not a runtime `Unsupported` or panic. This is the opposite shape from `minwebgpu`'s own pitfall: that crate always compiles cleanly off `wasm32` and fails only when its stub is actually called at runtime; `gpu_hal` instead fails to compile the call site itself.

A build with none of the three backend features enabled for the current target still compiles cleanly — down to just the error and descriptor types, per the crate's own `readme.md` — so cross-target feature unification (e.g. a workspace `full` feature enabled uniformly across `wasm32` and native builds) never breaks a consumer on its own. The trap is specifically expecting a feature you didn't enable *for the current target* to still exist as a runtime-checkable path.

### Mitigation

Gate calling code with the same `#[cfg(...)]` combination as the constructor being called, or feature-detect via `cfg!(...)` before referencing a backend-specific item — never assume a constructor exists and try to handle its `Result` as the availability check.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | The `#[cfg]`-gated variant mechanism this trap is a direct consequence of |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_backend_construction_and_device_acquisition.md](../feature/001_backend_construction_and_device_acquisition.md) | The constructors this pitfall is about |

### Cross-References

| File | Relationship |
|------|--------------|
| `module/min/minwebgpu/docs/pitfall/001_native_target_compiles_to_nonfunctional_stub.md` | The same family of trap (wrong target/feature), with the opposite compile-time/runtime shape |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | The 7 `#[cfg(...)]`-gated `mod_interface!` layers |
| `src/device.rs`, `src/resource.rs`, `src/pass.rs` | Every variant's own `#[cfg(...)]` gate |
| `Cargo.toml` | `webgpu`/`webgl`/`native` feature definitions |

### Tests

No test exercises this trap directly — it manifests as a compile error in consuming code, not a runtime assertion this crate's own test suite could catch.
