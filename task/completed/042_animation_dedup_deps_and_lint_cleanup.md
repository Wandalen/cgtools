# Clean up animation's Sequencer::value_get duplication, dead web-sys dependency, and lib.rs allow-attributes

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-09
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-09
- **blocked_by:** null

## Goal

Bundle 3 small hygiene findings from the 2026-08-09 `animation` crate audit that don't fit `TASK-015`
(`Sequencer`/`Tween` bugs) or `TASK-041` (`easing/cubic` bugs) — code duplication, a dead dependency,
and unjustified lint suppressions. Cross-references `TASK-035` (workspace-wide test coverage gaps) and
`TASK-036` (workspace-wide `#[allow(...)]` justification sweep) — both umbrella tasks should skip
`animation` when they reach it, since this task already closes its slice of both concerns.

### Findings (fixed, 2026-08-09)

1. **`Sequencer::value_get<T>` duplication** — an exact duplicate of `Tween::value_get<T>()`'s logic
   with zero external callers (confirmed via workspace-wide `grep -rn "\.value_get(" --include="*.rs"
   .` — every hit was either `Tween::value_get()` or an unrelated renderer-crate method of the same
   name). Deleted from `src/sequencer.rs` rather than fixed.
2. **Dead `web-sys` dependency** (`Cargo.toml`) — a `[dependencies.web-sys]` block (workspace=true,
   optional=true, 10 WebGL/DOM features) with zero `web_sys` references anywhere in `src/`. Removed;
   also filled in the previously-empty `description` field while touching the manifest.
3. **`lib.rs`'s 8 blanket `#![allow(clippy::...)]` attributes, none explained** — investigated each by
   temporarily removing it and re-running `cargo clippy -p animation --all-targets --all-features -- -D
   warnings` (after `cargo clean -p animation` to rule out caching), rather than assuming what each one
   suppressed. Found 3 of 8 completely vestigial (zero real hits): `clippy::implicit_return` (a
   `clippy::restriction`-tier lint never enabled anywhere in this workspace's
   `[workspace.lints.clippy]` table — confirmed both by its absence there and by an explicit
   `-W clippy::implicit_return` test proving the lint is real and fires 100+ times when opted into, so
   it's simply never activated in this workspace), `clippy::cast_precision_loss` (active workspace-wide
   via `pedantic = "warn"`, but this crate has zero integer-to-float `as` casts to trigger it — confirmed
   via `grep " as [a-zA-Z]"` across `src/`), and `dead_code` (rustc's own always-on lint; a clean rebuild
   found zero dead items). All 3 removed. The other 5 confirmed real with exact hit counts —
   `return_self_not_must_use` (5), `must_use_candidate` (18), `missing_inline_in_public_items` (116),
   `cast_possible_truncation` (4), `new_ret_no_self` (1) — kept, each now with a one-line justification
   comment citing its verified hit count.

`cargo clippy -p animation --all-targets --all-features -- -D warnings` confirms zero warnings after a
full `cargo clean -p animation` (rules out stale incremental-compilation state hiding a real hit).
`cargo nextest run -p animation --all-features` confirms all tests pass (29/29) — no test depended on
the deleted `Sequencer::value_get` or the removed `web-sys` dependency.

## History

- **[2026-08-09]** `FILED` — Filed from the same 2026-08-09 workspace audit re-verification pass as
  `TASK-015`/`TASK-041`; bundled as the "everything else" hygiene slice that isn't a `Sequencer`/`Tween`
  logic bug (`TASK-015`) or an `easing/cubic` logic bug (`TASK-041`).
- **[2026-08-09]** `RESOLVED` — All 3 findings fixed in the same session as filing: duplication deleted,
  dead dependency removed, all 8 lint allows individually investigated and either removed (3, proven
  vestigial) or justified with a verified hit count (5). Verification performed as a Tier 2 Dual-Role
  Self-Check (same session, no independent dispatch) per
  `governance/maav.rulebook.md § MAAV : Verification Tier Selection`'s default — not an independent
  PROC16-style acceptance pass. State → ✅ Completed; filed directly to `task/completed/`.
