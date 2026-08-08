# Clean up root todo.md: delete false claim, relocate legitimate TODO

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
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

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket (root file, not crate-scoped).
