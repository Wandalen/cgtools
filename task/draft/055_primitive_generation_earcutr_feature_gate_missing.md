# Fix primitive_generation's missing font-processing feature gate on earcutr usage

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/primitive_generation
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`primitive_generation`'s `Cargo.toml` declares `earcutr` as an optional dependency, only pulled in by the
`font-processing` feature (`font-processing = ["text", "dep:earcutr"]`). But
`contours_to_fill_geometry` (`src/primitive.rs`) calls `earcutr::earcut(..)` unconditionally, with no
`#[cfg(feature = "font-processing")]` gate around either the call site or the function itself. Building
the crate with default features only — `cargo check -p primitive_generation`, no `--all-features` —
fails with `error[E0433]: cannot find module or crate 'earcutr' in this scope` at the unconditional call
site. Discovered as a byproduct of task 018's investigation (not part of either of that task's two named
issues — no doc contradiction, no NaN) and independently re-confirmed directly:
`cargo check -p primitive_generation` (default features) → E0433 at the exact cited call site.

**Not currently caught by any verification gate:** every command this workspace's test/CI machinery
runs (`will .test l::3`, `cargo nextest`, `cargo clippy`, etc.) passes `--all-features`, so this break is
invisible to the standard verification loop — it only manifests for a consumer building with default
features alone, e.g. `cargo build -p primitive_generation` or as a default-feature dependency from
another crate.

**Resolution is a design decision, not a mechanical fix — two candidate directions, pick one at
pickup:**
1. Gate `contours_to_fill_geometry` (and any other `earcutr`-using code path) behind
   `#[cfg(feature = "font-processing")]`, matching how `path_to_points` is already gated behind
   `#[cfg(feature = "text")]` in the same `mod_interface!` block — but this changes
   `contours_to_fill_geometry`'s public API surface (no longer callable without the feature), so check
   all current callers first.
2. Make `earcutr` a non-optional dependency (drop it from `font-processing`'s feature list, remove
   `optional = true`) if triangulation-via-`earcutr` is actually a core, always-needed capability rather
   than a text/font-specific one — re-examine why it was gated behind `font-processing` in the first
   place before choosing this path.

## Out of Scope

- The two TASK-018 issues themselves (doc-contradicting silent failure on triangulation `Err`;
  NaN-producing precondition gap in `curve_to_geometry`) — already fixed and closed separately.
- `text/ufo.rs` dead-code/doc-drift cleanup (task 021).
- The `csgrs`/`core2` yanked-dependency issue in this same crate (BUG-007/task 008).

## History

- **[2026-08-10]** `FILED` — Discovered as a byproduct of task 018's fix (silent failure + NaN gap in
  `primitive_generation`); independently re-confirmed via direct `cargo check -p primitive_generation`
  (default features) → E0433 unresolved-crate error at the unconditional `earcutr::earcut(..)` call
  site in `contours_to_fill_geometry`. Filed separately per this workspace's out-of-scope discipline —
  distinct from task 018's two named issues, task 021, and BUG-007/task 008, all sharing this crate.
