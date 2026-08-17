# BUG-270: `mdmath_core`'s `arithmetics` feature doesn't declare its real dependency on `approx`, breaking `--features arithmetics` alone and the `full` feature bundle

- **Severity:** Medium (no runtime defect -- a compile-time feature-graph gap that breaks any
  consumer selecting `arithmetics` without also separately selecting `approx`)
- **state:** Completed
- **Affects:** `mdmath_core`'s `arithmetics` Cargo feature and its own `full` bundle feature
  (`Cargo.toml`); `src/vector/arithmetics.rs`'s `is_orthogonal` function
- **Component:** `module/math/mdmath_core` (`Cargo.toml`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`mdmath_core`'s `arithmetics` feature is declared as `arithmetics = [ "float" ]` -- it does not
require `approx`. But `src/vector/arithmetics.rs`, the file the feature gates in, contains
`is_orthogonal`, which unconditionally uses `crate::approx::ulps_eq` and bounds its generic
parameter `E : approx::UlpsEq`, with no `#[cfg(feature = "approx")]` guard. Selecting
`arithmetics` without also separately selecting `approx` fails to compile with E0432/E0433
("unresolved module or unlinked crate `approx`"). The crate's own `full` bundle feature
(`default + index + nd + arithmetics + general`) omits `approx` and is affected identically.

## Impact

**Who is affected:** any consumer selecting `mdmath_core`'s `arithmetics` feature in isolation
(or via `full`) without happening to also request `approx` separately. The defect was invisible
to this crate's own test suite (`--all-features` enables everything, including `approx`) and to
its sibling crate `ndarray_cg` (which explicitly requests both `arithmetics` and `approx`
together in its own `Cargo.toml`) -- no real workspace consumer had ever hit the gap before this
review.

**What breaks:** `cargo build -p mdmath_core --no-default-features --features enabled,arithmetics`
(and any equivalent invocation, including `--features full` alone) fails outright with a
compile error, not a runtime defect.

**Entity Scope:** `None` -- Cargo feature-graph defect, not entity directory instances.

## How Discovered

During this session's Group P review of `mdmath_core` (12 files unrelated to `arithmetics.rs`
itself), the fork additionally spot-checked isolated feature combinations as an adversarial
check beyond its assigned file list and found `--features enabled,arithmetics` alone fails to
build. Independently re-verified via direct `grep` of `arithmetics.rs`'s `approx` usage and a
fresh `cargo build` run before accepting the finding, per this session's standing practice of
never trusting a fork's claim without independent confirmation.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p mdmath_core --no-default-features --features enabled,arithmetics
```
**Expected** (fixed): compiles, all tests pass (including the new regression test below).
**Actual** (pre-fix, confirmed via temporary `git stash` revert of only the `Cargo.toml` half of
the fix, real run):
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `approx`
error[E0432]: unresolved import `approx`
error: could not compile `mdmath_core` (lib) due to 2 previous errors
```

## Root Cause

`Cargo.toml` (pre-fix):
```toml
approx = [ "float", "dep:approx" ]
arithmetics = [ "float" ]
```
`src/vector/arithmetics.rs` (unchanged, both pre- and post-fix):
```rust
use crate::approx::ulps_eq;
// ...
pub fn is_orthogonal< E, const N : usize >( ... ) -> bool
where
  E : NdFloat + approx::UlpsEq,
  // ...
{
  // ... calls ulps_eq( ... )
}
```
`is_orthogonal` is the sole `approx`-dependent function among roughly 30 functions in
`arithmetics.rs`, and its dependency was never reflected in the `arithmetics` feature's own
requirement list -- the feature declared only `float` as a prerequisite, omitting `approx`
despite the gated file needing it unconditionally (no `#[cfg(...)]` split within the file).

## Why Not Caught

Every existing test invocation exercises `mdmath_core` via `--all-features` (which enables
`approx` regardless of whether `arithmetics` itself declares the dependency) or via
`ndarray_cg`, this crate's only real internal consumer, which explicitly requests
`arithmetics` and `approx` together in its own `Cargo.toml` -- no code path had ever selected
`arithmetics` without also independently selecting `approx`, so the missing feature-graph edge
had no way to surface as a build failure until a fork deliberately tried the isolated
combination.

## Fix Applied (2026-08-17)

**`Cargo.toml`:** changed `arithmetics = [ "float" ]` to `arithmetics = [ "float", "approx" ]`,
making the feature graph match `arithmetics.rs`'s actual, unconditional dependency on `approx`.
No source file changed -- `is_orthogonal` already correctly used `approx`'s items; only the
feature declaration was wrong.

**`tests/inc/arithmetics.rs`** (new test):
`test_is_orthogonal_builds_under_arithmetics_feature_alone` calls `is_orthogonal` under a
`#[cfg(feature = "arithmetics")]`-gated test, exercising the exact isolated-feature combination
(`enabled,arithmetics`, no separately-requested `approx`) that the pre-fix feature graph broke.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p mdmath_core --no-default-features --features enabled,arithmetics` -- pre-fix
  (temporary `git stash push -- module/math/mdmath_core/Cargo.toml`, reverting only the
  manifest fix while leaving the new test live): fails to compile,
  `error[E0432]`/`error[E0433]` referencing unresolved `approx`, exactly as diagnosed.
  Post-fix (`git stash pop`, restoring the manifest fix): compiles clean, 3 passed (0 failed, 4
  ignored doctests), including `test_is_orthogonal_builds_under_arithmetics_feature_alone`.
- `cargo test -p mdmath_core --all-features`: 95 passed / 0 failed (unit + integration), 3
  passed doctests -- full regression-free confirmation across every existing feature
  combination.
- `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings`: clean, exit 0.

## Generalized Version

**Broken assumption:** a Cargo feature graph tested only via `--all-features` (or via a
downstream consumer that happens to always request two features together) provides no signal
about whether either feature is safe to select *alone*. A source file unconditionally using a
second feature's items, with no `#[cfg(...)]` split inside the file itself, means the *first*
feature's declared dependency list -- not the file's own contents -- is the only thing standing
between "compiles" and "silently E0432/E0433s the moment someone requests just this one
feature." The gap is invisible to any test suite that never exercises features in true
isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found via Group P's adversarial isolated-feature-combination spot check during task #176's `mdmath_core`/`ndarray_cg` bug-scouting review (file itself, `vector/arithmetics.rs`, owned by Group Q, which only tested via `--all-features` and so never hit the gap; `Cargo.toml` owned by no fork). Root cause: `arithmetics` feature omitted its real dependency on `approx`, which `is_orthogonal` uses unconditionally with no `#[cfg(...)]` guard. Fixed by adding `approx` to `arithmetics`'s feature-requirement list. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via temporary `git stash` revert-and-rerun of only the manifest half of the fix) plus the full `--all-features` suite (95/95 + 3 doctests) and clean clippy. Filed as BUG-270 after a fresh on-disk scan (both `task/` and `task/bug/` namespaces) found 269 as the highest existing bug ID and 254 as the highest existing task ID. |
