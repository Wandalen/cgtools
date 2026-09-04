# BUG-274: `minwebgl`'s `diagnostics` feature doesn't declare its real dependency on `future`/`file`, breaking `--features diagnostics` alone

- **Severity:** Medium (no runtime defect -- a compile-time feature-graph gap that breaks any
  consumer selecting `diagnostics` without also separately selecting both `future` and `file`)
- **state:** Completed
- **Affects:** `minwebgl`'s `diagnostics` Cargo feature (`Cargo.toml`); `src/diagnostics.rs`'s
  `own use crate::model::obj;` re-export
- **Component:** `module/min/minwebgl` (`Cargo.toml`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`minwebgl`'s `diagnostics` feature was declared as
`diagnostics = [ "mingl/diagnostics", "model_obj" ]` -- it does not require `future` or `file`.
But `src/diagnostics.rs`, the file this feature gates in (`lib.rs`:
`#[ cfg( feature = "diagnostics" ) ] layer diagnostics;`), contains a single line,
`own use crate::model::obj;`, with no `#[cfg(...)]` guard of its own. `lib.rs` gates the module
this line references (`layer model;`) behind `all( feature = "future", feature = "file" )` --
a *different* predicate than `model_obj`, the only prerequisite `diagnostics` actually forwarded.
Selecting `diagnostics` without also separately selecting both `future` and `file` fails to
compile with `E0432` ("unresolved import `crate::model`").

## Impact

**Who is affected:** any consumer selecting `minwebgl`'s `diagnostics` feature in isolation
without also happening to separately request `future` and `file`. The defect was invisible to
this crate's own test suite (`--all-features` enables everything, including `future`/`file`) and
to its own `default` feature bundle (which lists `diagnostics`, `future`, and `file` side by
side) -- no real invocation had ever selected `diagnostics` alone before this review.

**What breaks:**
`cargo build -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics`
(and any equivalent invocation that selects `diagnostics` without also separately selecting
`future` and `file`) fails outright with a compile error, not a runtime defect. (`web`, `log`,
`constants` are included in this baseline only to hold the crate's separate, unrelated
`enabled`-needs-`mingl/web` gap constant -- see Generalized Version below -- so the isolated
check exercises the `diagnostics`-specific gap alone.)

**Entity Scope:** `None` -- Cargo feature-graph defect, not entity directory instances.

## How Discovered

Assigned fork's file list for this session's bug-scouting review included
`module/min/minwebgl/src/diagnostics.rs`. Reading it in full showed its entire content is
`own use crate::model::obj;` -- surprising for a file named `diagnostics.rs` gated by a feature
called `diagnostics`. `git log`/`git show` across the file's full history (`dc8c8c1f` initial
commit through `67cea248`) confirmed this content (or its earlier `reuse crate::model;` variant)
has been present since the very first commit and was never touched by the one refactoring commit
(`dea7a008`) that fixed a sibling typo in `model.rs`'s own feature gate -- ruling out a recent
regression and pointing at a design-time gap instead. Cross-referencing `lib.rs`'s `layer model;`
gate (`all( feature = "future", feature = "file" )`) against `Cargo.toml`'s `diagnostics` feature
(only forwarding `model_obj`) identified the exact missing edge, then confirmed empirically via
`cargo check -p minwebgl --no-default-features --features enabled,diagnostics`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics
```
**Expected** (fixed): compiles, all tests pass (including the new regression test below).
**Actual** (pre-fix, confirmed via a temporary manual revert of only the `Cargo.toml` half of the
fix, real run):
```
error[E0432]: unresolved import `crate::model`
  --> module/min/minwebgl/src/diagnostics.rs:8:18
   |
 8 |   own use crate::model::obj;
   |                  ^^^^^ could not find `model` in the crate root
   |
note: found an item that was configured out
  --> module/min/minwebgl/src/lib.rs:91:9
   |
90 |   #[ cfg( all( feature = "future", feature = "file" ) ) ]
   |                ------------------ the item is gated behind the `future` feature
91 |   layer model;
   |         ^^^^^
error: could not compile `minwebgl` (lib) due to 2 previous errors
```

## Root Cause

`Cargo.toml` (pre-fix):
```toml
diagnostics = [
  "mingl/diagnostics",
  "model_obj"
]
```
`src/diagnostics.rs` (unchanged, both pre- and post-fix):
```rust
mod private
{

}

crate::mod_interface!
{
  own use crate::model::obj;
}
```
`lib.rs` (unchanged, both pre- and post-fix):
```rust
#[ cfg( feature = "diagnostics" ) ]
layer diagnostics;
// ...
#[ cfg( all( feature = "future", feature = "file" ) ) ]
layer model;
```
`diagnostics.rs`'s sole line unconditionally references `crate::model::obj`, but the module it
lives under (`crate::model`) is compiled only when *both* `future` and `file` are separately
enabled -- a predicate entirely disjoint from `model_obj`, the only prerequisite `diagnostics`
declared. `model_obj` does matter (it gates `model.rs`'s own `layer obj;` internally), but it is
necessary, not sufficient: without `future`+`file`, `crate::model` itself never exists for
`layer obj;` to even be evaluated.

## Why Not Caught

`diagnostics`, `future`, and `file` are all bundled together in this crate's own `default`
feature set (`default = [ "enabled", "constants", "diagnostics", "web", "future", "file", "log" ]`),
and every existing test invocation runs via `--all-features` or plain `cargo test` (default
features) -- both always carry `future`/`file` alongside `diagnostics`, so nothing had ever
selected `diagnostics` without them until this session's file-by-file review of
`diagnostics.rs`'s own (surprisingly small) content triggered a closer look at its feature gate.

## Fix Applied (2026-08-17)

**`Cargo.toml`:** changed `diagnostics = [ "mingl/diagnostics", "model_obj" ]` to
`diagnostics = [ "mingl/diagnostics", "model_obj", "future", "file" ]`, making the feature graph
match `diagnostics.rs`'s actual, unconditional dependency on `crate::model` (which itself
requires `future`+`file` per `lib.rs`'s own gate). No source file changed -- `diagnostics.rs`'s
`own use crate::model::obj;` already correctly referenced a real, working re-export; only the
feature declaration was incomplete.

**`tests/diagnostics_test.rs`** (new test): added to this crate's established
`tests/`-native-test convention (see `tests/readme.md`).
`diagnostics_obj_reexport_resolves_under_diagnostics_feature_alone`, gated
`#[cfg(feature = "diagnostics")]`, calls `minwebgl::diagnostics::obj::reports_make( &[], &[] )`
(a pure, native-callable function -- verified by reading `mingl::web::model::obj::reports_make`
and `mingl::web::model::ForBrowser`, neither of which touches `web_sys`/DOM) and asserts the
result is empty, exercising the exact isolated-feature combination
(`enabled,web,log,constants,diagnostics`, no separately-requested `future`/`file`) that the
pre-fix feature graph broke.

## Verification

Compiled and run via `longrun`-detached `cargo`, from repo root, using a private
`CARGO_TARGET_DIR` in this fork's scratchpad directory to avoid build-artifact contention from
the 13 other concurrently-running bug-scouting forks sharing this repo's default `target/`
(observed directly: two consecutive shared-`target/` runs failed with
`"couldn't create a temp dir"` / `"extern location ... does not exist"` before switching):

- `cargo check -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics`
  -- pre-fix (`Cargo.toml`'s `diagnostics` stanza manually reverted to the two-item form, new
  test file left in place): fails, `error[E0432]: unresolved import 'crate::model'`, exactly as
  diagnosed (both `lib` and `lib test` targets). Post-fix (manually restored): compiles clean,
  exit 0.
- `cargo test -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics`
  post-fix: `clean_test` 2/2, `data_type_test` 2/2, `diagnostics_test` 1/1 (the new regression
  test), `drawbuffers_test` 2/2, `geometry_test` 2/2, `sprite_upload_test` 5/5 -- all pass (14/14
  across this fork's file scope). `uniform_test` shows 2 failures
  (`f32_matrix_length_error_reports_matrix_not_vector`,
  `f32_matrix_length_error_display_mentions_matrix_and_valid_lengths`) -- confirmed unrelated:
  `src/uniform.rs`/`src/uniform/float32.rs` are outside this fork's assigned file list, not
  modified by this fix, and independently attributable to a different, concurrently-active
  fork's in-progress work (its own claimed `BUG-273`, visible mid-session in this same
  `Cargo.toml`'s sibling edits and in `tests/uniform_test.rs`).
- `cargo test -p minwebgl --all-features`: same result -- this fix's own 14 tests pass; the 2
  `uniform_test` failures persist identically (same panic messages), confirming they predate and
  are independent of this fix.
- `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings`: retried twice across
  this session, each time blocked by a *different* concurrently-active fork's in-progress file --
  neither ever in this fork's assigned list, neither touched by this fix. First retry: 2 errors
  in `src/uniform.rs:142` (`clippy::must_use_candidate` on `f32_matrix_length_error`). By the
  second retry that file had been fixed (confirming it was transient WIP), but a different fork's
  new test file, `tests/enabled_feature_web_gate_test.rs:73,78` (its own BUG-275 regression test),
  now fails `clippy::no_effect_underscore_binding`. Given 14 forks concurrently editing this same
  crate, a momentarily-red whole-crate clippy from someone else's in-flight file is expected
  noise, not a signal about this fix. This fork's own touched files carry no new clippy surface:
  `Cargo.toml` is TOML (not clippy-relevant), `src/diagnostics.rs` is byte-for-byte unchanged, and
  the new `tests/diagnostics_test.rs` was written to match this crate's established codestyle
  (2-space indent, space inside brackets/parens) used throughout the sibling test files it sits
  beside -- confirmed compiling and passing clean under the narrower, isolated-feature `cargo
  test` runs above, which is the actual reproduction surface for this bug.

## Generalized Version

**Broken assumption:** a Cargo feature graph tested only via `--all-features` (or via this
crate's own `default` bundle, which happens to always enable two features together) provides no
signal about whether either feature is safe to select *alone*. A source file unconditionally
referencing a *module* gated by a second, third feature combination -- with no `#[cfg(...)]`
split inside the file itself -- means the *first* feature's declared dependency list, not the
file's own contents, is the only thing standing between "compiles" and "silently E0432s the
moment someone requests just this one feature." This mirrors BUG-270
(`mdmath_core`'s `arithmetics` feature missing its real dependency on `approx`) exactly, one
level deeper: here the missing edge is not to a single sibling feature but to the *conjunction*
of two (`future` **and** `file`), gating a `layer`/module the referencing file sits one level
above rather than a directly-`use`d item.

**Adjacent, separate gap (not this fix's scope):** while isolating this bug, `enabled` alone
(without this crate's own `web` feature) was also observed to fail to compile --
`canvas.rs`/`dom.rs`/`exec_loop.rs`/`log.rs` unconditionally `reuse ::mingl::web::*`, which
requires `mingl`'s own `web` feature that `enabled` never forwarded. This is a distinct,
crate-wide gap (affecting every feature combination, not specifically `diagnostics`) requiring an
edit to `lib.rs`, which is outside this fork's assigned file list; a different, concurrently
active fork independently found and fixed it in this same session (visible mid-session in this
same `Cargo.toml` as its own claimed `BUG-275`), which this report defers to entirely rather than
duplicating.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found via this session's file-by-file bug-scouting review of `module/min/minwebgl/src/diagnostics.rs` (this fork's assigned file), whose entire content -- `own use crate::model::obj;` -- was suspicious for a file named/feature-gated as "diagnostics." Root cause: `diagnostics` feature omitted its real transitive dependency on `future`+`file`, which `lib.rs` requires for `crate::model` (referenced unconditionally by `diagnostics.rs`) to exist at all. Fixed by adding `future`/`file` to `diagnostics`'s feature-requirement list in `Cargo.toml`. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via a manual revert-and-restore of only the `Cargo.toml` half of the fix -- **not** `git stash`: an initial `git stash push -- Cargo.toml` / `git stash pop` pair was corrupted mid-session when a different, concurrently-running fork pushed its own unrelated stash entry onto this repo's single shared stash stack between this fork's push and pop, popping the wrong entry and silently dropping this fix; recovered by inspecting `git stash list`/`git stash show -p` to confirm which entries belonged to other forks, leaving both untouched, and manually re-applying this fix's known-exact diff via direct file edit instead) plus this fork's own full file-scope suite (14/14, `--features enabled,web,log,constants,diagnostics` and `--all-features` alike). Whole-crate `cargo clippy` was retried twice and stayed red both times, but from two *different* concurrently-in-progress forks' own files each time (`src/uniform.rs`, then `tests/enabled_feature_web_gate_test.rs` once the first was fixed) -- neither ever in this fork's assigned list; this fix's own touched files (`Cargo.toml`, new test file) carry no new clippy surface. Filed as BUG-274 after `task/bug/`'s on-disk highest ID (272) plus a same-session collision: `tests/readme.md` (a file shared and concurrently edited by every fork touching `module/min/minwebgl/tests/`) showed a different, concurrently-running fork had already claimed BUG-273 (for `tests/uniform_test.rs`) before this report's own ID was finalized, so this report took the next free number and records the collision here per this session's standing procedure. |
