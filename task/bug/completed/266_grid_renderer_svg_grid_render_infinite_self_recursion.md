# BUG-266: `GridRenderer::svg_grid_render`'s fallback arm calls itself instead of a square-grid helper, causing unconditional infinite recursion and a stack overflow

- **Severity:** High (guaranteed process abort -- `SIGABRT` -- for any caller exporting SVG grid art for `GridStyle::Triangular` or `GridStyle::Isometric`)
- **state:** Completed
- **Affects:** `tiles_tools::debug::GridRenderer::svg_grid_render` (`src/debug.rs`)
- **Component:** `module/helper/tiles_tools` (`src/debug.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`GridRenderer::svg_grid_render`'s `match self.style { .. }` has a dedicated arm for
`Square4`/`Square8` and `Hexagonal`, and a wildcard `_` arm intended as the fallback for every
other `GridStyle` variant (`Triangular`, `Isometric`). That fallback arm called
`self.svg_grid_render(writer, cell_size)` -- the enclosing function itself. Since `self.style`
never changes between calls, the recursive call re-enters the exact same `_` arm every time, with
no base case, terminating condition, or state change of any kind.

## Impact

**Who is affected:** any caller invoking `GridRenderer::svg_export` (the only public entry point
reaching `svg_grid_render`) on a `GridRenderer` configured with `GridStyle::Triangular` or
`GridStyle::Isometric`.

**What breaks:** the call never returns -- the process aborts with a stack overflow (`SIGABRT`)
before any SVG output is written, for 100% of invocations with either affected style. This is not
a slow-path or edge-case defect; it is unconditional and deterministic on every call.

**Entity Scope:** `None` -- source-level control-flow defect, not entity directory instances.

## How Discovered

During this session's Group J review of
`tiles_tools/src/{collection,debug,ecs/*,events,field_of_view,flowfield,game_systems,pathfind,serialization,spatial,lib}.rs`,
full read of `debug.rs`'s `svg_grid_render` showed its wildcard match arm's body was a call to
`self.svg_grid_render(writer, cell_size)` -- identical in name and receiver to the function
currently executing, with no change to `self` or the matched value between the outer call and the
inner one.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tiles_tools --all-features --test debug_test test_svg_export_triangular_and_isometric_styles_do_not_recurse_infinitely
```
**Expected** (fixed): 1 passed.
**Actual** (pre-fix, confirmed via temporary direct-source-edit revert of the fix, real isolated
run): process aborts -- `thread '...' has overflowed its stack`, `fatal runtime error: stack
overflow, aborting`, `signal: 6 (SIGABRT)`, exit 101.

## Root Cause

`GridRenderer::svg_grid_render` (pre-fix), abbreviated:
```rust
fn svg_grid_render(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
  let offset = 50;
  match self.style {
    GridStyle::Square4 | GridStyle::Square8 => {
      // (line-drawing body, inline)
    },
    GridStyle::Hexagonal => {
      // (hexagon-drawing body, inline)
    },
    _ => {
      self.svg_grid_render(writer, cell_size)?;   // <- calls itself
    }
  }
  Ok(())
}
```
The `Square4`/`Square8` arm's line-drawing logic was written inline rather than factored into a
callable helper, so when the `_` fallback arm needed "the same square-grid behavior other callers
get," there was no separately-named function to delegate to -- the call was left pointing at the
enclosing method itself. `self.style` is immutable for the lifetime of the call chain, so the
recursive call matches the identical `_` arm again on every level, an unconditional infinite
recursion with no way to bottom out before the stack is exhausted.

## Why Not Caught

No existing test exercised `svg_export()` (the only path reaching `svg_grid_render`) with
`GridStyle::Triangular` or `GridStyle::Isometric` -- prior SVG-path coverage only reached
`Square4` (directly) and `Hexagonal` (indirectly, via `PathfindingDebugger`), so the `_` fallback
arm was never reached end-to-end by any test. The bug produces no compiler warning: recursive
calls are ordinary, valid Rust, and nothing about the call site's syntax distinguishes "delegating
to a different case" from "infinite self-recursion."

## Fix Applied (2026-08-17)

**`src/debug.rs`:** extracted the `Square4`/`Square8` arm's line-drawing body into a new private
helper, `square_svg_grid_render(&self, writer, cell_size, offset)`, and changed the `_` fallback
arm to call that helper instead of `self.svg_grid_render(..)`. The `Square4`/`Square8` arm itself
now also calls the extracted helper, so both arms share one implementation with no behavioral
change for the styles that were already working.

**`tests/debug_test.rs`** (new test):
`test_svg_export_triangular_and_isometric_styles_do_not_recurse_infinitely` loops over
`[GridStyle::Triangular, GridStyle::Isometric]`, calls `svg_export()` to a temp-dir path for each,
and asserts the call returns `Ok` and the written file contains `"<line"` (confirming the
square-grid fallback actually rendered content, not just "didn't crash").

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tiles_tools --all-features --test debug_test
  test_svg_export_triangular_and_isometric_styles_do_not_recurse_infinitely` -- pre-fix (temporary
  direct-source-edit revert of the fix, real isolated run): stack overflow, `SIGABRT`, exit 101.
  Post-fix (restored): 1 passed, exit 0.
- `cargo test -p tiles_tools --all-features` (full scoped suite, `--no-fail-fast`, all 4 of this
  session's bugs simultaneously reverted to confirm independent pre-fix failures): `debug_test`
  target unaffected by the other 3 reverts, 8/8 passed including the new test. Post-fix (all 4
  restored): full suite green across all 10 test binaries plus 40 doctests, 0 failed.
- `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a `match` arm intended as "fall back to the same behavior another arm
already has" can safely call a method by name that happens to share the enclosing function's own
name, without checking whether that name actually resolves to a different implementation.
When a case's inline body is not yet factored into its own callable unit, "delegate to it" and
"call yourself" are syntactically indistinguishable at the call site -- extracting the shared
logic into its own named function first, then calling that function from every arm that needs it
(including the arm defining the original behavior), removes the ambiguity structurally rather than
relying on a reviewer to notice `self.<method>` matches the method they're currently reading.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group J review of `tiles_tools/src/debug.rs`. Root cause: `svg_grid_render`'s wildcard fallback arm called `self.svg_grid_render(..)` -- itself -- instead of a square-grid helper, since the square-grid body had never been factored out of the `Square4`/`Square8` arm into a callable unit. Fixed by extracting that body into `square_svg_grid_render` and calling it from both arms. Verified via 1 new native unit test (confirmed fail pre-fix via isolated single-test run -- real stack overflow / SIGABRT / exit 101 -- and pass post-fix), the full scoped suite (all 10 test binaries + 40 doctests green), and clean clippy. Filed as BUG-266 after a fresh on-disk scan immediately before filing found 265 as the highest existing ID (concurrent session actors had already claimed the originally-provisional 264/265 for unrelated `tilemap_scene` bugs). |
