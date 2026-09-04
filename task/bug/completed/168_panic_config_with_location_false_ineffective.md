# BUG-168: `panic::Config { with_location: false, .. }` never suppresses the panic location

- **Severity:** Medium (a documented opt-out flag silently does nothing -- not data loss or a
  crash, but a caller relying on it to keep source paths/line numbers out of a browser
  `console.error` payload gets them anyway)
- **state:** Completed
- **Affects:** `panic::hook`/`panic::setup` on the `wasm32-unknown-unknown` target -- any caller
  that sets `Config { with_location : false, .. }` expecting the panic message sent to
  `console.error` to omit the source location
- **Component:** `module/helper/browser_log` (`src/panic.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered in the same Explore review pass as BUG-167 and BUG-169 (task
  #96, `module/alias/browser_tools` -- resolved to the underlying `browser_log` crate it
  re-exports wholesale). Independent root cause from both: BUG-167 is a `file!()`/`line!()`
  lexical-resolution defect in `log/debug_log.rs`; BUG-169 is a missing feature-gate in
  `lib.rs`. This bug is `panic.rs`'s `Config.with_location` flag being a no-op.

## Symptom

```rust
// pre-fix -- wasm32-only hook_impl
let mut message = "=== Error\n\n".to_string();
message.push_str( &info.to_string() ); // <- already embeds "panicked at {file}:{line}:{col}:"
if config.with_location
{
  // this block only ever *adds* a second, redundant location -- it never had anything to do
  // with whether the first one (already written above, unconditionally) was present
  let _ = write!( message, "\n\n = Location:\n\n {}:{}", location.file(), location.line() );
}
```

## Impact

**Who is affected:** Any caller on `wasm32-unknown-unknown` that constructs
`panic::Config { with_location : false, with_stack_trace : .. }` -- e.g. to avoid leaking build
machine's absolute source paths into a production browser console -- and trusts the flag's own
doc comment ("Print location.") to do what it says.

**What breaks:** The panic message handed to `console.error` still contains
`"panicked at /full/path/to/src/whatever.rs:42:5:"` regardless of the flag, because
`PanicHookInfo`'s `Display` impl (which `info.to_string()` invokes) embeds the location
unconditionally -- there is no `Display` mode that omits it. `with_location` only ever gated a
second, fully redundant `"= Location:"` block appended after the first, already-unconditional
one.

**Magnitude:** Every wasm32 panic hook invocation with `with_location : false` set; zero effect
from the flag in every case.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via a background Explore review of `module/alias/browser_tools` (task #96), which
resolved to the underlying `browser_log` crate (a pure re-export shim) as the real bug surface.
Confirmed directly by reading `src/panic.rs`'s wasm32 `hook_impl` in full and reasoning through
`PanicHookInfo`'s `Display` semantics; further confirmed empirically via a native scratch
program (`rustc --edition 2021`, outside this crate) proving `PanicHookInfo::to_string()`
unconditionally contains the source location string for every panic shape tested (formatted
message, plain literal, and a non-string `panic_any` payload).

## Minimum Reproducible Example

```bash
cd module/helper/browser_log && cargo test -p browser_log --test panic_hook_test panic_message_with_location_false
```

**Expected** (post-fix): `panic_message( info, false )` returns just the panic message text,
with no `.rs:` location marker anywhere in the string.

**Actual** (pre-fix): no code path existed that could omit the location -- the message body was
always built from `info.to_string()`, which unconditionally embeds
`"panicked at {file}:{line}:{col}:"` ahead of the message, regardless of `with_location`.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_log && cargo test -p browser_log --test panic_hook_test
# all "ok" = fixed; a location string leaking through with with_location=false = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `hook_impl` builds its message from `PanicHookInfo::to_string()`, whose `Display` impl unconditionally embeds the location, so `with_location : false` has nothing left to suppress by the time it's checked. | ✅ Root Cause | Confirmed by reading `panic.rs`'s wasm32 `hook_impl`: `info.to_string()` is pushed to the message unconditionally, *before* `config.with_location` is ever checked; the flag only gates a second, additive `"= Location:"` block. | E1 |
| H2 | The redundant second block was the intended full feature, and a single location line in "= Location:" format (not the `Display`-embedded one) was the documented contract. | ❌ Falsified | The field doc says "Print location" with no qualifier about format or which of two locations -- a `false` value that still prints a location under a different label doesn't satisfy "print location : false" under any reading. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_log/src/panic.rs`, wasm32 `hook_impl` (pre-fix) | `message.push_str( &info.to_string() )` runs unconditionally before the `if config.with_location` block. | H1 ✅ |
| E2 | Native scratch program (`rustc --edition 2021`, outside this crate) against `PanicHookInfo::to_string()` | For 3 panic shapes (`panic!("msg {}", x)`, `panic!("literal")`, `panic::panic_any(123)`), `to_string()` always contains `.rs:` -- confirming `Display` never omits the location, so there is no way to reach a location-free string via `Display` alone. | H1 ✅, H2 ❌ |

## Root Cause

```rust
// before -- location embedded unconditionally via Display, flag only adds a second copy
let mut message = "=== Error\n\n".to_string();
message.push_str( &info.to_string() );
if config.with_location
{
  let _ = write!( message, "\n\n = Location:\n\n {}:{}", location.file(), location.line() );
}
```

`PanicHookInfo`'s `Display` impl unconditionally writes `"panicked at {file}:{line}:{col}:\n
{message}"` -- there is no `Display` mode that omits the location. Gating on `with_location`
therefore required bypassing `Display` entirely and reading the panic payload directly instead.

## Why Not Caught

`panic_hook_test.rs`'s own header comment already notes the wasm32-only behavior these flags
gate "cannot be observed natively without faking a console" -- so no existing test exercised
`hook_impl`'s message-building logic at all, on any target. The only pre-fix `Config`-related
tests (`config_default_enables_location_and_stack_trace`,
`config_fields_construct_independently`) pin the struct's field defaults and construction, never
the flags' actual effect on hook output.

## Fix Location

`module/helper/browser_log/src/panic.rs`.

```rust
// after -- pure function extracted so the flag's effect is unit-testable natively
pub fn panic_message( info : &panic::PanicHookInfo< '_ >, with_location : bool ) -> String
{
  if with_location
  {
    return info.to_string();
  }
  if let Some( payload ) = info.payload().downcast_ref::< &str >()
  {
    ( *payload ).to_string()
  }
  else if let Some( payload ) = info.payload().downcast_ref::< String >()
  {
    payload.clone()
  }
  else
  {
    "<non-string panic payload>".to_string()
  }
}
```

`hook_impl` now calls `super::panic_message( info, config.with_location )` instead of
`info.to_string()` directly. When `with_location` is `true`, behavior is byte-for-byte
unchanged (still `Display`-based, still followed by the pre-existing redundant `"= Location:"`
block). When `false`, the message is built straight from the panic payload
(`PanicHookInfo::payload()`, downcast to `&str`/`String` -- the same mechanism `panic!`'s own
payload machinery has used since before `Display`/`message()` existed), which never contains a
location at all. `panic_message` is split out of the wasm32-only `hook_impl` specifically so it
compiles and is unit-testable on every target, matching this crate's existing
pure-logic-extraction convention (`readback.rs`'s `#[doc(hidden)]` precedent in `minwgpu`,
`Fix(BUG-165)`/`Fix(BUG-166)`).

## Prevention

Added 4 tests to `tests/panic_hook_test.rs`:
`panic_message_with_location_false_omits_file_and_line` (`bug_reproducer(BUG-168)`),
`panic_message_with_location_true_includes_file_and_line` (control case, pins the unchanged
`true` behavior), `panic_message_with_location_false_handles_non_string_payload` (covers the
`panic_any` fallback branch). A 4th change, `with_panic_hook_locked`, replaced the pre-existing
ad hoc hook-swap code in `native_hook_runs_on_real_panic` and backs all 3 new tests: it
serializes every hook-swapping test in the file behind a shared `PANIC_HOOK_LOCK` mutex --
needed because `cargo test` runs tests within one file concurrently by default, and
`std::panic::set_hook`/`take_hook` is process-global mutable state, so 2+ hook-swapping tests in
the same file without this lock could steal each other's installed hook and corrupt both
results. This was a latent hazard already present in the file's single pre-existing
hook-swapping test; it only became a concrete race once a second such test was added here.

## Pitfall

A boolean config flag whose doc says "Print location" must be checked *before* the very first
point the location could enter the output, not only at the point a second, additive block is
appended -- a type's `Display` impl is not a neutral, location-free starting point to build on
top of just because the visible code only references it once. Separately: any test file with
more than one test that swaps process-global state (here, `std::panic`'s hook) needs its own
shared serialization primitive from the moment the second such test is added -- `cargo test`'s
default intra-binary parallelism makes this a real, not theoretical, race.

## Generalized Version

**Broken assumption:** "a type's `Display`/`to_string()` output is a safe, neutral starting
point to append config-gated sections onto, since the visible code only adds to it under a
flag."

**Confirmed general rule:** before gating any section of a message on a boolean flag, check
whether the *base* string the gated section is appended to already contains that section's
content by another path (here, `Display` embedding the location that `with_location` was
supposed to gate) -- a flag that only ever adds cannot ever effectively remove.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during a background Explore review of `module/alias/browser_tools` (task #96), resolved to `browser_log::panic`; confirmed via a native scratch program proving `PanicHookInfo::Display` always embeds the location. |
| 2026-08-16 | fixed | Extracted `panic_message` (pure, testable on every target); `hook_impl` now builds the message from it instead of `info.to_string()` directly. |
| 2026-08-16 | verified | Added 4 tests to `tests/panic_hook_test.rs` (including a shared-lock retrofit of the pre-existing hook-swap test). Scoped native `cargo nextest`/`cargo clippy`/`cargo test --doc` clean across `browser_log` (8/8 tests, 10/10 doctests) + `browser_tools` (2/2 tests), 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote unit tests against the fixed `panic_message`; adversarial pass independently re-verified `PanicHookInfo::to_string()`'s unconditional location embedding via a native scratch compile (not just inline reasoning) before committing to this fix's shape -- and separately caught a real intra-file test race (parallel `cargo test` execution against process-global `panic::set_hook`) before it shipped as a flaky test. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Explicitly checked against BUG-167/169 (same review pass, same crate) -- independent root causes, no coupling; recorded rather than left unstated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading plus an independent native scratch-program proof of `Display`'s unconditional location embedding across 3 panic shapes (formatted, literal, non-string payload). | — |
| D5 | Execution Scope | 🟢 | 🟢 | `panic_message` extracted only because it was needed both for the fix (bypass `Display`) and for unit-testability without a real wasm/console environment -- not a speculative refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `browser_log` src + 1 test file + this bug file touched; no unrelated crates modified. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `hook_impl`'s only call site for the message-building logic (wasm32 `mod imp`) was found via full-file read and updated; native `mod imp` deliberately left untouched (`_config` already ignored there, by design, per its own doc comment and `native_hook_runs_on_real_panic`'s existing contract). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `panic_message` has one job (build the message body honoring `with_location`); `#[doc(hidden)]` since it's exposed for test-reachability, not as a primary public API surface. | — |

**Reproduced:** YES -- pre-fix, a native scratch program confirmed `PanicHookInfo::to_string()`
always embeds the source location regardless of any flag, proving `with_location : false` had
no mechanism available to suppress it. Post-fix, `panic_message( info, false )` cleanly omits
any `.rs:` location marker while still returning the real panic message, confirmed via
`panic_message_with_location_false_omits_file_and_line` and 2 supporting tests. Scoped native
`cargo nextest`/`cargo clippy`/`cargo test --doc` clean across `browser_log` + `browser_tools`,
10/10 tests + 10/10 doctests passing, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_log/src/panic.rs` | New `panic_message` pure function (full `Fix(BUG-168)` comment), exposed via `mod_interface!`; wasm32 `hook_impl` now calls it instead of `info.to_string()` directly. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_log/tests/panic_hook_test.rs` | New shared `PANIC_HOOK_LOCK` + `with_panic_hook_locked` helper (also backs the retrofitted `native_hook_runs_on_real_panic`); 3 new tests: `panic_message_with_location_false_omits_file_and_line` (`bug_reproducer(BUG-168)`), `panic_message_with_location_true_includes_file_and_line`, `panic_message_with_location_false_handles_non_string_payload`. |
