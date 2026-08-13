# BUG-110: `geometry_tests.rs`'s wasm32 suite is missing `wasm_bindgen_test_configure!( run_in_browser )`, so it silently ran under Node.js and failed with an unrelated-looking error

- **Severity:** Medium
- **state:** Completed
- **Affects:** The sole test in `module/helper/renderer/tests/geometry_tests.rs` (`tests::add_attribute_duplicate_name_returns_err_not_panic`) — its wasm32 test binary never ran in a real browser context
- **Component:** `module/helper/renderer` (`tests/geometry_tests.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-13
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-13
- **Fixed:** 2026-08-13

## Symptom

```
=== wasm32 test: module/helper/renderer ===
     Running tests/geometry_tests.rs (target/wasm32-unknown-unknown/debug/deps/geometry_tests-....wasm)
running 1 test
test tests::add_attribute_duplicate_name_returns_err_not_panic ... FAIL

---- tests::add_attribute_duplicate_name_returns_err_not_panic output ----
    error output:
        panicked at module/helper/renderer/tests/geometry_tests.rs:16:37:
        called `Result::unwrap()` on an `Err` value: CanvasRetrievingError("Failed to get window")

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 filtered out; finished in 0.03s
error: wasm32 test suite failed for module/helper/renderer
```

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/renderer
cargo test --target wasm32-unknown-unknown --all-features
```
**Expected:** `test tests::add_attribute_duplicate_name_returns_err_not_panic ... ok`.
**Actual (pre-fix):** panics inside `gl_init()`'s `gl::canvas::make().unwrap()` with `CanvasRetrievingError("Failed to get window")`.

## Impact

**Who is affected:** The workspace's `verb/test` command, which runs every `wasm_bindgen_test` suite discovered by content — this suite's single test always failed, blocking a clean full-workspace pass.

**What breaks:** `gl::canvas::make()` (→ `mingl::web::canvas::make`) calls `web_sys::window().ok_or(Error::CanvasRetrievingError("Failed to get window"))?` as its first line. `wasm-bindgen-test` defaults every test binary to running under Node.js unless `wasm_bindgen_test_configure!( run_in_browser )` is declared — under Node.js there is no global `window`, so this is unconditional, not flaky. Three sibling files in the same directory (`animation_tests.rs`, `pmrem_tests.rs`, `skeleton_tests.rs`) all declare the macro and all pass; `geometry_tests.rs` was created without it (`git log`: introduced in `9b71cf39`, 2026-08-10, never had the line).

The underlying test logic itself was never in question — `Geometry::add_attribute`'s duplicate-name-returns-`Err` behavior (see the pre-existing `## Root Cause`/`## Fix Applied` doc block on the same test function, a prior, unrelated fix) had literally never been exercised in a real browser context, only ever failing on harness setup before reaching its own assertions.

## How Discovered

`/tst_fix` full-workspace `verb/test` run: nextest, doctests, clippy, and all wasm32 example compile-checks passed clean; this was the sole remaining failure, isolated to one crate's wasm32 test stage.

## Fix

`tests/geometry_tests.rs`: added `wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );` immediately after the `wasm_bindgen_test` import, matching the exact placement already used by the three sibling suites in the same directory. 3-field comment (`Fix(BUG-110)` / `Root cause` / `Pitfall`) at the fix site, matching this codebase's established short-form convention for whole-file test-harness defects (see `BUG-046`'s identical treatment in the sibling file `skeleton_tests.rs`).

**Verification:** `cargo test --target wasm32-unknown-unknown --all-features` (package-scoped, via `longrun`) — exit 0; `test tests::add_attribute_duplicate_name_returns_err_not_panic ... ok`, all other suites in the crate (`animation_tests.rs`, `webgpu_backend_test.rs`-class suites, doctests) unaffected. Full-workspace `verb/test` re-run separately to confirm no regression elsewhere.
