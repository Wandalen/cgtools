# Consolidate minwebgl's exec_loop.rs to reuse mingl via mod_interface

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/min/minwebgl/src/exec_loop.rs` (63 lines) duplicates the render-loop logic already present in
`module/min/mingl/src/web/exec_loop.rs` (75 lines), instead of reusing it the way
`module/min/minwebgpu/src/exec_loop.rs` correctly does today — that file is just 7 lines:
`mod private { }` plus `crate::mod_interface! { reuse ::mingl::web::exec_loop; }` (confirmed by direct
read this session). Rewrite minwebgl's `exec_loop.rs` to match minwebgpu's `reuse` pattern, deleting the
duplicated logic, unless a genuine minwebgl-specific behavioral difference is found on closer inspection
(re-confirm before assuming pure duplication — diff the two files' actual bodies, not just line counts,
before deleting anything). P3 — dead-code/hygiene bucket, Fix-in-place.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code /
  identity cleanup) tier, Fix-in-place bucket.
