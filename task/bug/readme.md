<!-- bug_system_metadata
type: integrated
version: 1.0
-->

# bug

Bug reports for the cgtools workspace. IDs share the tsk Unified ID namespace tracked in
`../readme.md` (`highest_id`).

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| readme.md | Bug index and open bugs tracking |
| draft/ | Newly filed bugs, structurally incomplete |
| unverified/ | Structurally complete bugs awaiting the VERIFY gate |
| verifying/ | Bugs actively undergoing the VERIFY gate |
| verified/ | Bugs confirmed and claimable for fix work |
| executing/ | Bugs with an in-progress fix |
| executed/ | Bugs whose fix landed, awaiting acceptance review |
| accepting/ | Bugs under acceptance review |
| completed/ | Bugs whose fix is verified and closed |
| cancelled/ | Bugs closed as won't-fix or duplicate |
| mixed/ | Bugs with cross-boundary or entirely-foreign fix scope |
| orphan/ | Mixed bugs confirmed for full external relocation |

## Open Bugs

_None currently open._

## Closed Bugs

| ID | Title | Severity | Component | Filed | Closed | Root Cause | Round | Accepted By |
|----|-------|----------|-----------|-------|--------|------------|-------|-------------|
| BUG-007 | [csgrs's mandatory core2 dependency is permanently yanked](./completed/007_csgrs_core2_yanked_dependency.md) | Critical | workspace root Cargo.toml | 2026-08-08 | 2026-08-08 | core2 ^0.4 (csgrs's mandatory dep) is entirely yanked | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ |
| BUG-043 | [`Vector<E,4>::w()` returns the `z` component instead of the `w` component](./completed/043_vector_w_wrong_index.md) | Medium | module/math/ndarray_cg | 2026-08-09 | 2026-08-09 | `w()`'s body copy-pasted from `z()`; index never bumped from `2` to `3` | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-046 | [`skeleton_tests.rs`'s shared setup helper fails to compile — missing `Node` import and nonexistent `gltf.scene` field](./completed/046_skeleton_test_compile_errors.md) | High | module/helper/renderer | 2026-08-09 | 2026-08-09 | Missing `Node` import + `gltf.scene` (singular) should be `gltf.scenes` (plural) | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-050 | [`Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut`'s shared `index` cursor aliases `&mut` references under mixed-direction iteration](./completed/050_mdmath_core_itermut_shared_cursor_aliasing.md) | High | module/math/mdmath_core | 2026-08-10 | 2026-08-10 | `next()` and `next_back()` each map the shared `index` cursor to a tuple field independently, never cross-checked, so interleaving both on one iterator re-yields a field already handed out as a live `&mut` | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-051 | [`BindGroupLayoutEntry`'s conversion to `web_sys` panics on `BindingType::Other`, its own documented default](./completed/051_bind_group_layout_entry_panics_on_documented_placeholder.md) | High | module/min/minwebgpu | 2026-08-10 | 2026-08-10 | Infallible `From` conversion panicked on `Other`, the type's own documented default/placeholder variant | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-052 | [`minwebgl::geometry::Positions::new` panics on an unsupported `natoms` instead of returning `WebglError`](./completed/052_geometry_natoms_unsupported_panic.md) | High | module/min/minwebgl | 2026-08-10 | 2026-08-10 | `match typ.natoms`'s `_` arm used `panic!` instead of returning through the function's own `Result< Self, WebglError >` | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-053 | [An explicit `RUSTFLAGS` override silently disables `web_sys_unstable_apis`, flipping `get_image_data`/`MouseEvent` accessors between two incompatible web-sys signatures](./completed/053_web_sys_unstable_apis_rustflags_override.md) | High | module/min/minwebgl + module/helper/browser_input + 3 examples | 2026-08-10 | 2026-08-10 | An explicit `RUSTFLAGS` env var completely replaces (never merges with) `.cargo/config.toml`'s `[build] rustflags`, silently flipping `web_sys_unstable_apis` and the web-sys method signatures it gates | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
| BUG-054 | [`[E]`'s `ArrayMut::vector_mut` casts via `as_ptr()` instead of `as_mut_ptr()`, producing a `&mut` reference with `SharedReadOnly` provenance](./completed/054_slice_vector_mut_shared_provenance_ub.md) | High | module/math/mdmath_core | 2026-08-10 | 2026-08-10 | Copy-pasted from the immutable `array_ref()` sibling without switching `as_ptr()` to `as_mut_ptr()`, leaving the returned `&mut` retagged `Unique` from a `SharedReadOnly`-tagged pointer | 0 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self) |
