# BUG-052: `minwebgl::geometry::Positions::new` panics on an unsupported `natoms` instead of returning `WebglError`

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `minwebgl::geometry::Positions::new` (public API) that passes a `natoms` value other than `2` — currently zero live call sites pass anything but `2` (see `## Impact`)
- **Component:** `module/min/minwebgl` — `geometry::private::Positions::new`
- **repo_identity:** self
- **Filed:** 2026-08-10
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-10
- **Fixed:** 2026-08-10
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# terminal output — wrong, current behavior (equivalent minimal reproduction; see ## Minimum Reproducible Example)
$ /tmp/mre052/repro
thread 'main' (2774447) panicked at /tmp/mre052/repro.rs:11:10:
Unsapported buffer descriptor
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo "exit: $?"
exit: 101

# terminal output — correct, expected behavior once fixed
$ /tmp/mre052/repro_fixed
handled recoverable error: NotSupportedForType("natoms other than 2 is not supported")
$ echo "exit: $?"
exit: 0
```

`Positions::new` (`module/min/minwebgl/src/geometry.rs`) already returns
`Result< Self, WebglError >` and propagates every other fallible step via `?`
(`buffer::create`, `vao::create`, `attribute_pointer`), but the final `match typ.natoms`
ended in `_ => panic!( "Unsapported buffer descriptor" )` — any `natoms` other than `2`
crashed the whole process instead of giving the caller a `WebglError` to handle.

## Impact

**Who is affected:** Any code calling `minwebgl::geometry::Positions::new(gl, positions,
natoms)` with `natoms != 2` — the function is `pub`, part of `minwebgl`'s public API, and
nothing in its signature (`Result< Self, WebglError >`) hints that some inputs bypass the
`Result` entirely and abort the process instead.

**What breaks:** Loud — the whole process panics immediately (`thread 'main' panicked
at ...: Unsapported buffer descriptor`), not a silent wrong value. Unlike a caller-side
logic error, there is no way for the caller to catch or recover from this: it is a hard
process abort, not a `Result` the caller declined to check.

**Magnitude — currently zero, confirmed by exhaustive search:** a workspace-wide grep for
every `Positions::new` call site found exactly 3 matches, all in
`examples/minwebgl/hexagonal_grid/src/main.rs` (lines 90-95, 96-101, 102-107), and all 3
pass a literal `2` for `natoms`. Every current caller in this workspace is therefore safe
from this defect today — the defect is real and confirmed, but dormant, which is why
Severity is High rather than Critical: it will immediately crash the process for the first
caller that passes any `natoms` other than `2`, and the function's own body carries a
`// qqq : xxx : move out switch and make it working for all types` comment signalling that
supporting more `natoms` values (e.g. `3` for 3D positions) is anticipated future work —
not a hypothetical edge case.

**Entity Scope:** `None` — the affected code is an ordinary source file
(`src/geometry.rs`), not an entity directory instance; `## Affected Entity Collections`
does not apply.

## How Discovered

```bash
$ grep -n "panic!\|\.unwrap()\|\.expect(" module/min/minwebgl/src/geometry.rs
64:        _ => { panic!( "Unsapported buffer descriptor" ) }
```

Found while re-investigating `task/draft/011_minwebgl_panic_on_recoverable_failure.md`'s
carried-forward claim that some `minwebgl` sites panic on recoverable/expected conditions
instead of surfacing them via `Result` — the filing explicitly required re-confirming any
such site fresh against current `module/min/minwebgl/src/` rather than trusting a prior,
unpreserved citation. A systematic sweep of every `.unwrap()`/`.expect(`/`panic!(`/
`unreachable!(` call site across `module/min/minwebgl/src/` (56 sites total) was triaged
against the crate's own established convention — functions such as `buffer::create` and
`vao::create` already convert a fallible WebGL call into `Result< _, WebglError >` via
`.ok_or( WebglError::FailedToAllocateResource( "..." ) )`. `Positions::new` was the
cleanest match: its own doc comment already promises `- Err(WebglError) if there is an
issue creating buffers, VAOs, or uploading the geometry data`, it already returns
`Result< Self, WebglError >`, and `WebglError::NotSupportedForType( &'static str )` is a
pre-existing variant whose doc string — "Error when operation is not supported for the
given type" — is an exact semantic match for "an unsupported `natoms` value was passed."

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates, no cgtools
paths. `minwebgl::WebglError` requires a live `WebGl2RenderingContext` transitively (via
`error::typed::Error` and the crate's `wasm-bindgen`/`web-sys` dependency chain) and isn't
reachable from a synthetic host-only script, so the script below reproduces the exact
defect *pattern* instead: a function that already returns `Result` but panics on an
unsupported input instead of returning `Err`, structurally identical to the real bug at
`module/min/minwebgl/src/geometry.rs` (pre-fix line 64).

```bash
mkdir -p /tmp/mre052
cat > /tmp/mre052/repro.rs <<'EOF'
#[ derive( Debug ) ]
enum GeomError { NotSupportedForType( &'static str ) }

// Mirrors module/min/minwebgl/src/geometry.rs's Positions::new: a function that
// already returns Result but panics instead of returning Err for an unsupported input.
fn make_geometry( natoms : i32 ) -> Result< String, GeomError >
{
  match natoms
  {
    2 => Ok( "ok: 2-component geometry created".to_string() ),
    _ => panic!( "Unsapported buffer descriptor" ),
  }
}

fn main()
{
  match make_geometry( 3 )
  {
    Ok( s ) => println!( "{s}" ),
    Err( e ) => println!( "handled recoverable error: {:?}", e ),
  }
}
EOF
rustc --edition 2021 /tmp/mre052/repro.rs -o /tmp/mre052/repro 2>&1
/tmp/mre052/repro
echo "exit: $?"
```

**Expected** (once fixed — i.e. `panic!` replaced with `Err( ... )`, captured from the
corrected variant of this same script, `/tmp/mre052/repro_fixed`):
```
handled recoverable error: NotSupportedForType("natoms other than 2 is not supported")
exit: 0
```

**Actual:**
```
thread 'main' (2774447) panicked at /tmp/mre052/repro.rs:11:10:
Unsapported buffer descriptor
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
exit: 101
```

**Verify Command:** `/tmp/mre052/repro; test $? -eq 101` — **What:** demonstrates that a
function whose signature already promises `Result` still aborts the whole process on an
unsupported input instead of returning `Err`, reproducing the exact invariant violated by
`Positions::new` at `module/min/minwebgl/src/geometry.rs` (pre-fix line 64).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The `_` arm of `Positions::new`'s `match typ.natoms` was written as `panic!(...)` instead of returning through the function's own `Result< Self, WebglError >` | ✅ Root Cause | `geometry.rs` (pre-fix line 64): `_ => { panic!( "Unsapported buffer descriptor" ) }` — the only non-`?`-propagated fallible path in the whole function | E1, E2, E3 |
| H2 | `natoms` other than `2` is a genuinely impossible/unreachable state by construction (e.g. validated upstream by every caller), so the `panic!` is a deliberate, justified invariant assertion, not a recoverable-failure bug | ❌ Disproved | `natoms : i32` is a caller-supplied function parameter with no validation anywhere before the match; nothing in `Positions::new`'s signature or doc comment restricts it to `2`, and the doc comment explicitly documents an `Err(WebglError)` path for exactly this kind of failure | E1, E4 |
| H3 | `WebglError` has no suitable variant for this failure, so `panic!` was the only option available at the time | ❌ Disproved | `WebglError::NotSupportedForType( &'static str )` already exists in `context.rs` with doc string "Error when operation is not supported for the given type" — an exact semantic match, unused by this function before the fix | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/minwebgl/src/geometry.rs` (pre-fix lines 45-65) | `Positions::new` already returns `Result< Self, WebglError >` and uses `?` on `buffer::create`, `vao::create`, and `attribute_pointer` — the `_ => panic!(...)` arm is the one fallible path not propagated through `Result` | H1 ✅, H2 ❌ |
| E2 | `module/min/minwebgl/src/geometry.rs` (pre-fix lines 28-31, doc comment) | `- Err(WebglError) if there is an issue creating buffers, VAOs, or uploading the geometry data.` — the function's own documentation already promises a `Result`-based failure path that the `panic!` arm bypassed | H1 ✅ |
| E3 | `module/min/minwebgl/src/buffer.rs`, `module/min/minwebgl/src/vao.rs` | The crate's established convention for a fallible WebGL operation: `gl.create_buffer().ok_or( WebglError::FailedToAllocateResource( "Buffer" ) )` / `gl.create_vertex_array().ok_or( WebglError::FailedToAllocateResource( "VAO" ) )` — every other resource-acquisition site in this crate already returns `Result`, never panics | H1 ✅ |
| E4 | `module/min/minwebgl/src/geometry.rs` (pre-fix line 45) | `pub fn new( gl : GL, positions : &[ f32 ], natoms : i32 ) -> Result< Self, WebglError >` — `natoms` is an ordinary `i32` parameter, no newtype or const-generic restricting it to `2`; three real call sites in `examples/minwebgl/hexagonal_grid/src/main.rs` (lines 90-95, 96-101, 102-107) all pass a literal `2`, confirming callers are expected to supply an arbitrary value, not a compile-time-guaranteed-valid one | H2 ❌ |
| E5 | `module/min/minwebgl/src/context.rs` | `#[ error( "Not supported for type {0}" ) ] NotSupportedForType( &'static str ),` — a pre-existing `WebglError` variant, unused by `geometry.rs` before this fix | H3 ❌ |

## Root Cause

```
Positions::new()  -> Result< Self, WebglError >   (already the function's own signature)
  ...
  match typ.natoms
  {
    2 => { ... attribute_pointer( &gl, 0, &position_buffer )?; },   (correct: propagates via ?)
    _ => panic!( "Unsapported buffer descriptor" ),                 (wrong: aborts instead of Err)
  }
```

`Positions::new` was written with `Result< Self, WebglError >` as its return type and
uses `?` for every other fallible operation in its body (`buffer::create`, `vao::create`,
`attribute_pointer`), but the final arm of its `natoms` match was authored as a bare
`panic!` instead of `Err( WebglError::NotSupportedForType( ... ) )`. The doc comment
(pre-fix lines 28-31) already documents an `Err(WebglError)` path for exactly this
situation, and a directly-applicable `WebglError` variant
(`NotSupportedForType( &'static str )`) already existed unused — confirming **H1 (✅ Root
Cause)** over the disproved alternatives H2 (deliberately-unreachable invariant — no
upstream validation exists) and H3 (no suitable error variant — one already existed).

## Why Not Caught

`minwebgl` had no `tests/` directory and no inline `#[cfg(test)]` module anywhere in
`src/` before this fix (confirmed via a full directory listing of
`module/min/minwebgl/src/`); the only executable check of `Positions::new` was its own
doc-comment example, which calls it with a literal `natoms = 2` and therefore never
reaches the `_` arm. All 3 real call sites in this workspace
(`examples/minwebgl/hexagonal_grid/src/main.rs`) also pass a literal `2`. Reaching the
buggy arm at all requires a live `WebGl2RenderingContext` (the function's first three
fallible steps — `buffer::create`, `vao::create`, and the upload — all need a real `gl`
handle), which does not exist outside a browser; this workspace's standard native
`cargo test`/`cargo nextest run` sweep therefore had no path to ever exercise this
function with an unsupported `natoms` value, even if such a test had been written against
the pre-fix code as originally structured.

## Fix Location

`module/min/minwebgl/src/geometry.rs`, `Positions::new` (pre-fix lines 45-65):

```rust
// Before:
pub fn new( gl : GL, positions : &[ f32 ], natoms : i32 ) -> Result< Self, WebglError >
{
  let position_buffer = buffer::create( &gl )?;
  let typ = VectorDataType::new( DataType::F32, natoms, 1 );
  buffer::upload( &gl, &position_buffer, positions, GL::STATIC_DRAW );
  let vao = vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  // qqq : xxx : move out switch and make it working for all types
  match typ.natoms
  {
    2 =>
    {
      BufferDescriptor::new::< [ f32; 2 ] >()
      .stride( 0 )
      .offset( 0 )
      .divisor( 0 )
      .attribute_pointer( &gl, 0, &position_buffer )?;
    },
    _ => { panic!( "Unsapported buffer descriptor" ) }
  }

  let nvertices = positions.len() as i32 / natoms;
  Ok( Positions { vao, typ, nvertices, gl } )
}

// After:
fn validate_natoms( natoms : i32 ) -> Result< (), WebglError >
{
  match natoms
  {
    2 => Ok( () ),
    _ => Err( WebglError::NotSupportedForType( "natoms other than 2 is not supported by Positions::new" ) ),
  }
}

pub fn new( gl : GL, positions : &[ f32 ], natoms : i32 ) -> Result< Self, WebglError >
{
  validate_natoms( natoms )?;
  let position_buffer = buffer::create( &gl )?;
  let typ = VectorDataType::new( DataType::F32, natoms, 1 );
  buffer::upload( &gl, &position_buffer, positions, GL::STATIC_DRAW );
  let vao = vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  // qqq : xxx : move out switch and make it working for all types
  match typ.natoms
  {
    2 =>
    {
      BufferDescriptor::new::< [ f32; 2 ] >()
      .stride( 0 )
      .offset( 0 )
      .divisor( 0 )
      .attribute_pointer( &gl, 0, &position_buffer )?;
    },
    _ => unreachable!( "natoms already validated by validate_natoms" ),
  }

  let nvertices = positions.len() as i32 / natoms;
  Ok( Positions { vao, typ, nvertices, gl } )
}
```

The `natoms` check is factored into a standalone `validate_natoms` and called as a guard
at the top of `Positions::new`, before any WebGL resource is allocated — this is what
makes the check reachable by a native (non-browser) test at all, and as a side benefit
avoids allocating a buffer/VAO that would otherwise be silently leaked on the old code's
error path. The pre-existing `match typ.natoms` is left in place (its `qqq : xxx` TODO
about supporting more types is unrelated to this fix); its `_` arm becomes
`unreachable!(...)` because `validate_natoms` has already returned early via `?` for any
value other than `2` by the time this match runs.

## Fix Applied

Applied exactly as documented above (`geometry.rs`), with the fix-time comment in the
standard 3-field form (`Fix(BUG-052)` / `Root cause` / `Pitfall`, `geometry.rs:26-35`).
TDD cycle run for real, natively, package-scoped to `minwebgl` (`cargo test -p minwebgl
--all-features validate_natoms`):

- **Red (before fix):** `validate_natoms`'s `_` arm still `panic!`'d —
  `geometry::private::tests::validate_natoms_rejects_unsupported_value` FAILED (the test
  process itself panicked: `Unsapported buffer descriptor`) while
  `validate_natoms_accepts_supported_value` passed — `1 passed; 1 failed`, exit 101.
- **Green (after fix):** both tests pass — `2 passed; 0 failed`, exit 0.
- **Full crate regression (`cargo test -p minwebgl --all-features`):** 4 unit tests passed
  (0 failed) — the 2 new tests plus 2 pre-existing, unrelated tests in `clean.rs`; 1
  doc-test passed, 7 ignored (all pre-existing, require a live GL context, unrelated to
  this change).
- **`cargo check -p minwebgl --target wasm32-unknown-unknown --all-features`** (the
  crate's real deployment target): exit 0, unaffected by the fix.
- **`cargo clippy -p minwebgl --all-features --all-targets -- -D warnings`** (host) and
  **`cargo clippy -p minwebgl --target wasm32-unknown-unknown --all-features -- -D
  warnings`** (wasm32 lib): both exit 0, no warnings.

`cargo test -p minwebgl --all-features --tests` (test-binary compilation for the
wasm32 target specifically) remains blocked in this environment by a pre-existing,
out-of-scope dependency gap — the same `getrandom v0.2.17` class of issue already
documented by `task/bug/completed/046_skeleton_test_compile_errors.md`'s `## Why Not
Caught`. Confirmed unrelated to this fix: `cargo check -p minwebgl --target
wasm32-unknown-unknown --all-features --tests` fails identically (`error: could not
compile 'getrandom' (lib) due to 1 previous error`) regardless of whether this fix is
present, because host-target (non-wasm32) test compilation for this same crate succeeds
cleanly (`cargo check -p minwebgl --all-features --tests` → exit 0) — the new test module
itself was written to need nothing beyond plain Rust, so it runs on the host target
without requiring wasm32 at all.

## Prevention

Whenever a function's own signature already returns `Result< _, E >`, grep its body for
`panic!`/`unwrap()`/`expect(` before considering it done — any such macro inside a
`Result`-returning function is either a genuine invariant (and should say so, e.g. via
`unreachable!` with a comment explaining why) or a bug exactly like this one. Detection:

```bash
grep -n "panic!\|\.unwrap()\|\.expect(" module/min/minwebgl/src/geometry.rs
```

should show no bare `panic!` inside any function whose signature returns `Result`.

**Pitfall:** a function that already returns `Result` is exactly where a stray
`panic!`/`unwrap`/`expect` is easiest to miss in review, because the signature makes it
*look* fully fallible-safe at a glance — always check every branch actually returns
through the `Result`, not just that the function signature promises one.

## Generalized Version

**Broken assumption:** "A function returning `Result< _, E >` propagates every failure
through that `Result`." False whenever one branch of the function's control flow was
authored with `panic!`/`unwrap()`/`expect(...)` instead of `return Err(...)`/`?`, despite
the surrounding function already being `Result`-typed.

**Detection invariant:**
```
for every function f with return type Result< T, E >,
every reachable panicking macro inside f's body ( panic!, unwrap, expect ) is either
  (a) replaced by a `return Err( ... )` / `?` through E, or
  (b) an `unreachable!` whose unreachability is guaranteed by a prior guard clause
      already covered by (a), with a comment stating which guard makes it safe.
```

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-10 | filed  | Found while re-confirming `task/draft/011_minwebgl_panic_on_recoverable_failure.md`'s carried-forward claim against current `module/min/minwebgl/src/`; root cause confirmed via source read + crate-convention cross-check + synthetic MRE before filing |
| 2026-08-10 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS; MRE re-executed and reproduces (exit 101) |
| 2026-08-10 | completed | `validate_natoms` extracted and wired into `Positions::new` as an early guard; native reproducer added (`geometry::private::tests::validate_natoms_rejects_unsupported_value`), confirmed failing pre-fix (`1 passed; 1 failed`, exit 101) and passing post-fix (`2 passed; 0 failed`, exit 0) via `cargo test -p minwebgl`. Full crate regression, wasm32 `cargo check`, and host+wasm32 `cargo clippy` all clean. Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per `governance/maav.rulebook.md`'s default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | Adversarial pass: `## Fix Location`'s "After" block was drafted before the file's on-disk state was reconfirmed stable (an unrelated concurrent filesystem write to `geometry.rs` was observed and resolved mid-session — see `## Refs: src/`); line citations needed re-verification against the final, stable file | Re-read `geometry.rs` after confirming two consecutive stable reads and a passing test run; verified every citation in this report against that confirmed-stable state |
| D4 | Root Cause Quality | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 fixed | 1/1 |

**Reproduced:** YES — exit 101, 2026-08-10 (`/tmp/mre052/repro`, verbatim output captured
into `## Symptom` and `## Minimum Reproducible Example`).

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round
(`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no
subagent dispatch (Verification Delegation would be forbidden per `file.rulebook.md §
Report New Bug : Step 9 - VERIFY Gate`).

## Refs: src/

- `module/min/minwebgl/src/geometry.rs` — extracted `validate_natoms` guard
  (lines 20-43) called from `Positions::new` (line 72); `_` arm of the pre-existing
  `match typ.natoms` (line 92) changed from `panic!` to `unreachable!` with a comment
  explaining why the guard makes it safe. **Note:** during this fix, an unrelated
  process was observed independently writing to this same file concurrently (a stray,
  unauthored doc comment briefly appeared on the new test function, referencing a
  "temporary probe" this session never created — see `## Verification Record`'s D3 row).
  The final on-disk state was reconfirmed stable across two consecutive reads plus a
  passing `cargo test -p minwebgl --all-features --lib` run before this report was
  finalized; every citation above was checked against that confirmed-stable state.
