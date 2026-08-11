# Clean up root todo.md: delete false claim, relocate legitimate TODO

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-09
- **blocked_by:** null

## Goal

Root `todo.md` contains a claim that workspace math is limited to `i32`/`u32` integer types — confirmed
false this session by reading `module/math/ndarray_cg/src/vector/arithmetics.rs` (generic
`impl<E: MatNum, const LEN: usize> Vector<E, LEN>` with explicit overflow-semantics doc notes for integer
scalars) and `tests/inc/integer_test/arithmetic_test.rs` (parameterized integer tests covering `i32`,
`i64`, `u32`, `u64` via macros) — arithmetic is generic over any `MatNum`-bounded type, not restricted.
P5 (doc drift, Fix-in-place): delete the false claim. Separately, `todo.md` also has a legitimate,
unverified GLTF-loader bounding-box TODO — relocate that into the task system as its own properly-scoped
task (re-derive its exact content and target crate at pickup) rather than leaving it in a root todo.md
that this same triage plan is retiring as a knowledge site.

## Outcome

A follow-up rigorous investigation (2026-08-09, prompted by the user re-pasting both `todo.md` claims
and asking "are those legit tasks/bugs?") found both of this task's original guesses were half right,
in a symmetric way — neither claim was simply "false" or simply "legitimate as stated"; both were
**broadly false but with one narrow, real gap buried inside them**:

- **i32/u32 Vectors claim:** not simply false as this task originally concluded. `dot`/`mag2`/
  `distance_squared` are indeed already generic (confirming the original finding) — but
  `mdmath_core::vector::{min, max}` genuinely are bound to `NdFloat` (float-only) despite being pure
  ordering comparisons with no float-specific requirement, and `ndarray_cg`'s commutative
  scalar×vector `Mul` is only implemented for `f32`/`f64`, not `i32`/`i64`/`u32`/`u64`. Filed as
  [044](../completed/044_mdmath_core_min_max_integer_bound.md) (`mdmath_core`) and
  [048](../completed/048_ndarray_cg_integer_min_max_and_scalar_mul.md) (`ndarray_cg`, `blocked_by: 044`),
  both 🎯 Verified. Also surfaced, as an unrelated byproduct of the same read-through:
  [BUG-043](../bug/completed/043_vector_w_wrong_index.md) (`Vector<E,4>::w()` returns `z`'s value).
- **GLTF loader claim:** not simply legitimate as this task originally assumed. Per-node world
  matrices and per-node own-mesh bounding boxes are already eagerly computed and already
  automatically invoked at the end of `load()` (`gltf.rs:1102`'s `scene.update_world_matrix()` call
  cascades through the whole node tree) — but the *scene-level hierarchical* bounding box is never
  cached or invoked at load time (`Scene.bounding_box` is dead state), and zero tests anywhere in the
  crate assert on any bounding-box value. Filed as
  [047](../completed/047_renderer_scene_bounding_box_cache_and_tests.md) (`renderer`), 🎯 Verified. Also
  surfaced, as an unrelated byproduct of the same read-through:
  [BUG-046](../bug/completed/046_skeleton_test_compile_errors.md) (`skeleton_tests.rs` fails to compile).

`todo.md` has been deleted — both claims are now fully and more precisely accounted for by the four
task/bug files above, which supersede its content entirely.

## Verification

### Checklist

- [x] C1 — Is `todo.md` absent from the repository root (deleted, not merely emptied)? `find /home/user1/pro/lib/yrd_gamedev/cgtools -maxdepth 1 -name todo.md` → no match.
- [x] C2 — Does the relocated i32/u32 `mdmath_core` task file exist at its claimed path? `task/completed/044_mdmath_core_min_max_integer_bound.md` present.
- [x] C3 — Does the relocated i32/u32 `ndarray_cg` task file exist at its claimed path? `task/completed/048_ndarray_cg_integer_min_max_and_scalar_mul.md` present.
- [x] C4 — Does the relocated GLTF scene-bounding-box `renderer` task file exist at its claimed path? `task/completed/047_renderer_scene_bounding_box_cache_and_tests.md` present.
- [x] C5 — Does the byproduct `BUG-043` (Vector w() wrong index) file exist at its claimed path? `task/bug/completed/043_vector_w_wrong_index.md` present.
- [x] C6 — Does the byproduct `BUG-046` (skeleton_tests compile errors) file exist at its claimed path? `task/bug/completed/046_skeleton_test_compile_errors.md` present.

### Measurements

- [x] M1 — count of links in this file's own Outcome prose pointing at a superseded state directory (task- or bug-scoped) instead of the files' current location: `grep -c -E '\.\./(bug/)?verified/' task/completed/033_root_todo_cleanup.md` → `0` (was: `5` — links for 044/047/048 plus BUG-043/BUG-046 all pointed at the directory those files occupied before their later relocation).

### Invariants

- [x] I1 — Doc-only task (no source/test changes of its own): repository-wide test suite invariant is inherited from the 4 task/bug files this task spawned (044, 047, 048, BUG-043, BUG-046), each independently verified at their own completion; not re-run here per the non-code-task Test Matrix exemption.
- [x] I2 — All 4 outbound links in this file's Outcome section resolve to existing files: `044`, `047`, `048`, `BUG-043`, `BUG-046` — confirmed present at their corrected `completed/`/`bug/completed/` paths (see C2–C6).

### Anti-faking checks

- [x] AF1 — Guards against the stale-link shortcut recurring: re-running M1's grep after any future task/bug relocation must still return `0`; a nonzero result means an Outcome link was left pointing at a superseded directory instead of being updated alongside the move.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket (root file, not crate-scoped).
- **[2026-08-09]** `COMPLETED` — Both claims re-investigated to file-able precision; relocated into
  044/047/048 (tasks) and BUG-043/BUG-046 (bugs), all 🎯 Verified; `todo.md` deleted.
