# BUG-167: `DebugLog` trait methods report their own `file:line`, not the real caller's

- **Severity:** Medium (a diagnostic convenience API silently defeats its own purpose -- not
  data loss or a crash, but every `.debug_info()`/`.debug_trace()`/etc. call site becomes
  unlocatable from its own log output, which is the entire point of `file:line` in a log record)
- **state:** Completed
- **Affects:** `DebugLog::debug_log`/`debug_trace`/`debug_info`/`debug_warn`/`debug_error` on
  every target (no `wasm32` gating involved) -- any caller using these convenience methods to
  log a type's `Debug` representation
- **Component:** `module/helper/browser_log` (`src/log/debug_log.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered in the same Explore review pass as BUG-168 and BUG-169 (task #96,
  `module/alias/browser_tools` -- resolved to the underlying `browser_log` crate it re-exports
  wholesale). Independent root cause from both: BUG-168 is `panic.rs`'s `Config.with_location`
  flag being a no-op; BUG-169 is a missing feature-gate in `lib.rs`. This bug is `file!()`/
  `line!()`'s lexical (not dynamic) resolution inside `debug_log.rs`'s own trait default bodies.

## Symptom

```rust
// pre-fix -- every method calls the log! macros directly inside the trait's own default body
fn debug_trace( &self )
{
  log::trace!( "{:#?}", self ); // file!()/line!() resolve HERE, inside debug_log.rs
}
```

Any caller anywhere in the workspace doing `my_struct.debug_info();` gets a log record whose
`file`/`line` point at `debug_log.rs`'s own body -- never the caller's real source location.

## Impact

**Who is affected:** Any caller of `DebugLog::debug_trace`/`debug_info`/`debug_warn`/
`debug_error`/`debug_log` -- confirmed real callers exist in
`examples/minwebgl/attributes_matrix/src/main.rs` and
`examples/minwebgl/uniforms_ubo/src/main.rs`.

**What breaks:** The resulting log line's `file:line` context -- normally used to jump straight
to the call site from a log viewer or browser console -- always points at whichever line inside
`debug_log.rs` happens to hold that method's `log::trace!`/`log::info!`/etc. invocation, instead
of the real caller. `#[inline]` was already present on every method and changes nothing here:
it's a codegen hint, not a macro-hygiene mechanism, so it has no effect on where `file!()`/
`line!()` lexically resolve.

**Magnitude:** Every call to any of the 5 `DebugLog` convenience methods, on every target,
100% of the time -- there is no code path that reports the correct location pre-fix.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via the same background Explore review of `module/alias/browser_tools` (task #96)
that surfaced BUG-168/169, resolving to `browser_log::log::debug_log` as the real bug surface.
Confirmed by reading `src/log/debug_log.rs` in full and reasoning through how `file!()`/
`line!()` expand; the fix mechanism (`#[track_caller]` propagating correctly through a chain of
trait default methods calling each other, and the exact `log::Record::builder()`/
`log_enabled!()` API shape) was independently verified via 2 standalone scratch compiles against
the real toolchain and the workspace's exact pinned `log = "=0.4.33"` -- not assumed from
memory -- before either was written into the real fix.

## Minimum Reproducible Example

```bash
cd module/helper/browser_log && cargo test -p browser_log --test debug_log_test
```

**Expected** (post-fix): the captured `log::Record`'s `file()`/`line()` for each of the 5
methods match the real call site inside the test file itself.

**Actual** (pre-fix): no code path existed that could report anything other than
`debug_log.rs`'s own internal `log::trace!`/`log::info!`/etc. invocation line, regardless of
where the method was actually called from.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_log && cargo test -p browser_log --test debug_log_test
# "ok" with file/line asserted against the test's own call sites = fixed
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `log::trace!`/`log::info!`/etc. expand `file!()`/`line!()` lexically at their macro-invocation site (inside `debug_log.rs`'s own trait default bodies), not dynamically at the runtime call site, so every call site collapses to the same fixed internal location. | ✅ Root Cause | Confirmed by reading `debug_log.rs` pre-fix: all 5 methods invoke a `log!`-family macro directly inside their own body. Independently confirmed the only escape mechanism (`#[track_caller]` + `Location::caller()`) works correctly through a chain of trait default methods via a standalone scratch compile before relying on it in the real fix. | E1, E2 |
| H2 | The pre-existing `#[inline]` attribute on each method was expected to make the compiler treat the macro as if written at the call site, fixing the location. | ❌ Falsified | `#[inline]` was already present on all 5 methods pre-fix and fixed nothing -- it's a codegen/optimization hint, not a macro-hygiene mechanism; it has zero effect on where `file!()`/`line!()` lexically resolve. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_log/src/log/debug_log.rs` (pre-fix) | All 5 methods (`debug_log`, `debug_trace`, `debug_info`, `debug_warn`, `debug_error`) call a `log!`-family macro directly inside the trait's own default body, already `#[inline]`d, with no location-forwarding mechanism. | H1 ✅, H2 ❌ |
| E2 | Standalone scratch compile (`rustc --edition 2021`, outside this crate) chaining 2 `#[track_caller]` trait default methods, method A calling method B on `self` | `Location::caller()` inside B correctly resolves to A's real *external* caller, not A's internal call to B -- proving the exact mechanism this fix depends on (`debug_trace` → `debug_log`, both `#[track_caller]`) works for default-trait-method-vs-default-trait-method chains, not just plain functions. | H1 ✅ |

## Root Cause

```rust
// before -- file!()/line!() resolve at the macro's own lexical location, inside this trait body
fn debug_trace( &self )
{
  log::trace!( "{:#?}", self );
}
```

`file!()`/`line!()` (and therefore every `log!`-family convenience macro built on them) are
lexical: they always resolve to where the macro is *written*, never to where the enclosing
function was *called from*. Calling them from inside a trait's default method body means every
external caller's location is lost, collapsed to that one fixed internal line. The only
mechanism that captures the true runtime caller is `#[track_caller]` + `Location::caller()`,
which requires bypassing the `log!` convenience macros entirely (they provide no way to inject a
caller-supplied file/line) and constructing the `log::Record` manually instead.

## Why Not Caught

No test file for `DebugLog` existed at all prior to this fix -- `tests/debug_log_test.rs` is a
new file. The trait's only pre-existing exercise was indirect, through downstream example crates
that call `.debug_info()` for its logging side effect, never asserting on the resulting record's
`file`/`line`.

## Fix Location

`module/helper/browser_log/src/log/debug_log.rs`.

```rust
// after -- #[track_caller] + manual Record construction, bypassing the log! macros entirely
#[ track_caller ]
#[inline]
fn debug_log( &self, level : Level )
{
  // Mirrors the log! macro's own pre-filter so a disabled level still skips the
  // {self:#?} formatting entirely, matching the original macro-based laziness.
  if !log::log_enabled!( level )
  {
    return;
  }
  let location = std::panic::Location::caller();
  log::logger().log(
    &Record::builder()
      .level( level )
      .file( Some( location.file() ) )
      .line( Some( location.line() ) )
      .module_path( Some( module_path!() ) )
      .target( module_path!() )
      .args( format_args!( "{self:#?}" ) )
      .build()
  );
}

#[ track_caller ]
#[inline]
fn debug_trace( &self )
{
  self.debug_log( Level::Trace );
}
// debug_info/debug_warn/debug_error follow the same #[track_caller] + delegate-to-debug_log shape
```

All 5 methods gained `#[track_caller]`. `debug_log` (the one all 4 others delegate to) no longer
calls a `log!` macro at all -- it manually re-checks `log::log_enabled!(level)` (preserving the
macros' own zero-cost-when-disabled laziness for the `{self:#?}` formatting), reads
`std::panic::Location::caller()`, and builds/dispatches a `log::Record` directly via
`log::Record::builder()`/`log::logger().log()`. Because `debug_trace`/`debug_info`/`debug_warn`/
`debug_error` are themselves `#[track_caller]` and call `self.debug_log(...)` as their last
action, `Location::caller()` inside `debug_log` resolves through the chain to the *original*
external caller, not to any of the intermediate trait methods -- verified as a real propagation
guarantee (not an assumption) via the scratch compile in Evidence E2.

## Prevention

New file `tests/debug_log_test.rs`: one consolidated test,
`debug_log_methods_report_the_real_caller_location` (`bug_reproducer(BUG-167)`), covering all 5
methods against a custom `CapturingLogger` (`log::Log` impl storing captured records in a
`Mutex<Vec<CapturedRecord>>`). A single consolidated test was used deliberately rather than 5
separate tests: `log::set_logger` may only be called once per process, so splitting into
multiple tests would race for that one-time installation under `cargo test`'s default
intra-binary parallelism. Each of the 5 calls is made from a statically-known `line!() + 1`
call site in the test itself, and the captured record's `file()`/`line()` are asserted to match
that exact call site -- not `debug_log.rs`'s own internal location.

## Pitfall

`#[inline]` affects codegen, not macro hygiene -- it has no effect on where `file!()`/`line!()`
resolve, so it can never fix a lexical-location bug no matter how deep the inlining goes. More
generally: wrapping a `file!()`/`line!()`-based diagnostic macro inside any indirection layer
(a trait default method, a helper function) silently discards the real caller's location unless
that layer is explicitly `#[track_caller]` and reads `Location::caller()` itself -- the macro
has no way to "see through" the wrapper on its own.

## Generalized Version

**Broken assumption:** "calling a `file!()`/`line!()`-based logging macro from inside a
convenience wrapper method preserves the caller's perceived location automatically, especially
if the wrapper is `#[inline]`."

**Confirmed general rule:** `file!()`/`line!()`-based macros are always lexical, never dynamic.
Any indirection layer between the real call site and the macro invocation requires
`#[track_caller]` + `Location::caller()` explicitly, all the way through the chain -- `#[inline]`
never substitutes for this, and the mechanism composes correctly through multiple
`#[track_caller]` trait default methods calling each other (confirmed empirically, not assumed).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during the same background Explore review of `module/alias/browser_tools` (task #96) that surfaced BUG-168/169; resolved to `browser_log::log::debug_log`. |
| 2026-08-16 | fixed | Rewrote all 5 `DebugLog` methods around `#[track_caller]` + manual `Record::builder()`/`log::logger().log()`, replacing the direct `log!`-family macro calls. Mechanism (trait-method `#[track_caller]` chaining) and exact `log = 0.4.33` API shape both independently verified via standalone scratch compiles before touching real crate code -- caught and fixed 4 real compile errors along the way (an unstable-API assumption, an `E0716` temporary-lifetime issue, an `E0659` name-collision issue, and 2 clippy lints). |
| 2026-08-16 | verified | New `tests/debug_log_test.rs` (1 consolidated test, 5 call sites checked in one process to respect `log::set_logger`'s once-per-process constraint) + new `tests/readme.md` (Responsibility Table, `tests/` crossed the 3-file threshold). Scoped native `cargo nextest`/`cargo clippy`/`cargo test --doc` clean across `browser_log` (9/9 tests, 10/10 doctests) + `browser_tools` (2/2 tests, clippy clean); both real downstream consumers (`minwebgl_attributes_matrix`, `minwebgl_uniforms_ubo`) confirmed `wasm32-unknown-unknown` compile-clean against the changed trait signature. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote `debug_log_test.rs` against the fixed trait; adversarial pass independently re-verified BOTH the `#[track_caller]` trait-chain propagation mechanism AND the exact pinned `log = 0.4.33` `Record::builder()`/`log_enabled!` API shape via 2 separate standalone scratch compiles (not inline reasoning) before committing to this fix's shape -- and separately caught 4 real compile errors (unstable-API assumption, `E0716` temporary lifetime, `E0659` name collision, 2 clippy lints) before any of them shipped. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Explicitly checked against BUG-168/169 (same review pass, same crate) -- independent root causes, no coupling; recorded rather than left unstated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading plus 2 independent scratch-compile proofs of the exact mechanisms relied on (trait-chain `#[track_caller]` propagation; `log` crate's manual-dispatch API against the real pinned version). | — |
| D5 | Execution Scope | 🟢 | 🟢 | Manual `Record` construction was the minimal viable mechanism -- the `log!` macros give no way to inject a caller-supplied file/line, so bypassing them was required, not a speculative refactor; the `log_enabled!` pre-check was added specifically to preserve pre-existing laziness semantics, not as scope creep. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `browser_log` src + 2 new/changed test files + this bug file touched; no unrelated crates modified. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `debug_log.rs`'s only definition site for all 5 methods was found via full-file read and fixed; `mod_interface!`'s `prelude use DebugLog` block confirmed unchanged -- no new item needed separate exposure since the whole fix lives inside the trait itself. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `DebugLog` retains its single responsibility (convenience debug-logging shortcuts); the fix changes HOW the location is captured, not WHAT the trait does or its public shape. | — |

**Reproduced:** YES -- pre-fix reasoning plus a scratch-compile proof confirmed `file!()`/
`line!()`'s lexical resolution inside the trait's own body left no path to the real caller's
location. Post-fix, `debug_log_methods_report_the_real_caller_location` directly captures and
asserts `file()`/`line()` against 5 distinct real call sites in the same file, for all 5
methods, confirming the fix. Scoped native `cargo nextest`/`cargo clippy`/`cargo test --doc`
clean across `browser_log` + `browser_tools`, 11/11 tests + 10/10 doctests passing, 0 failures;
both real downstream consumers compile-clean for `wasm32-unknown-unknown`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_log/src/log/debug_log.rs` | `DebugLog`'s 5 methods rewritten: `#[track_caller]` added to all 5; `debug_log`'s body replaced with a manual `log_enabled!` pre-check + `Record::builder()` + `log::logger().log()` in place of the `log!` macro (full `Fix(BUG-167)` comment block); `debug_trace`/`debug_info`/`debug_warn`/`debug_error` unchanged in shape, just delegate through the now-`#[track_caller]` chain. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_log/tests/debug_log_test.rs` | New file. 1 consolidated test (`debug_log_methods_report_the_real_caller_location`, `bug_reproducer(BUG-167)`) with a custom `CapturingLogger` `log::Log` impl, asserting `file()`/`line()` for all 5 `DebugLog` methods match their real call sites in this file. |
| `module/helper/browser_log/tests/readme.md` | New file. Responsibility Table for `tests/`, which crossed the 3-file threshold (`basic_test.rs`, `panic_hook_test.rs`, `debug_log_test.rs`). |
