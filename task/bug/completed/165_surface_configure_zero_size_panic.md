# BUG-165: `surface_configure` panics on a transient zero-size resize input

- **Severity:** Medium (an undocumented panic on an ordinary, common windowing event -- not
  data corruption, but reachable by any windowed `wgpu` app that lets its window minimize)
- **state:** Completed
- **Affects:** `surface::surface_configure` -- any caller that reconfigures a presentation
  surface on window resize, including a transient `(0, height)`/`(width, 0)` size
- **Component:** `module/min/minwgpu` (`src/surface.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- discovered independently during a from-scratch review of
  `module/min/minwgpu` (task #95), not derived from or coupled to BUG-162/163/164
  (`module/min/minwebgpu`, a different crate, WebGPU/wasm32 rather than native `wgpu`).

## Symptom

```rust
// pre-fix
pub fn surface_configure( .., size : ( u32, u32 ) ) -> wgpu::SurfaceConfiguration
{
  let ( width, height ) = size;
  let mut config = surface.get_default_config( adapter, width, height ).expect( ".." );
  config.format = preferred_format( &surface.get_capabilities( adapter ).formats );
  surface.configure( device, &config ); // panics here when width or height is 0
  config
}
```

## Impact

**Who is affected:** Any caller that calls `surface_configure` again on window resize (which
its own doc explicitly invites: "Call again on every resize... this function is deliberately
idempotent-safe for that purpose") and lets a transient `0×0` size reach it -- e.g. a window
minimize event on Windows/`winit`, which commonly reports a `0×0` `inner_size()`.

**What breaks:** `wgpu::Surface::configure` panics (`wgpu-core`'s
`ConfigureSurfaceError::ZeroArea`, surfaced through `wgpu`'s default uncaptured-error handler
since `minwgpu` never installs a custom one) -- an unrecoverable process crash, not a
`Result::Err` a caller could handle.

**Magnitude:** One call, no existing guard inside the library; only mitigated where a caller
independently discovers and hand-writes the guard themselves.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, not speculative: a from-scratch Explore review of `module/min/minwgpu` (task #95)
found this crate's own `examples/minwgpu/flecs_bouncing_circles/src/main.rs` already has a
hand-written `if width == 0 || height == 0 { return; }` guard immediately before its own
`surface_configure` call in `graphics_resize`, with a doc comment explicitly naming the reason:
"skipped for a transient `0×0` size ( reported while the window is minimized ), which `wgpu`
would otherwise reject." Confirmed directly against `wgpu-30.0.0`/`wgpu-core-30.0.0` source in
the local cargo registry cache that `Surface::configure` does panic on a zero-area
configuration via its default uncaptured-error handler.

## Minimum Reproducible Example

```bash
cd module/min/minwgpu && cargo test -p minwgpu --test surface_test validate_size_rejects_zero
```

**Expected** (post-fix): `validate_size( ( 0, 512 ) )` returns
`Err( Error::ZeroSizeSurface( 0, 512 ) )`.

**Actual** (pre-fix): no precondition check existed at all; `surface_configure( .., ( 0, 512 ) )`
would forward straight into `surface.configure`, which panics.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwgpu && cargo test -p minwgpu --test surface_test
# all "ok" = fixed; a wgpu ConfigureSurfaceError::ZeroArea panic = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `surface_configure` has no precondition check on `size`, so a zero-area resize forwards straight into `wgpu::Surface::configure`, which panics. | ✅ Root Cause | Confirmed against real `wgpu-core-30.0.0` source: `configure` returns via its default uncaptured-error handler (`panic!` unless a custom `on_uncaptured_error` handler is installed) on `ConfigureSurfaceError::ZeroArea`; `minwgpu::Context` never installs a custom handler. | E1 |
| H2 | This is a purely theoretical edge case never actually hit by a real caller. | ❌ Falsified | The crate's own `flecs_bouncing_circles` example already needed to hand-write a guard against exactly this input, with a doc comment naming the exact real-world trigger (window minimize). | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `wgpu-core-30.0.0` source (local cargo registry cache), `Surface::configure`'s uncaptured-error path | `ConfigureSurfaceError::ZeroArea` is raised for a zero-width/height configuration and surfaces as a panic absent a custom error handler. | H1 ✅ |
| E2 | `examples/minwgpu/flecs_bouncing_circles/src/main.rs`, `graphics_resize` (pre-fix) | Hand-written `if width == 0 \|\| height == 0 { return; }` guard immediately before the crate's own `surface_configure` call, with a doc comment naming the window-minimize trigger. | H2 ❌ |

## Root Cause

```rust
// before -- size forwarded unconditionally, no precondition check
pub fn surface_configure( .., size : ( u32, u32 ) ) -> wgpu::SurfaceConfiguration
{
  let ( width, height ) = size;
  let mut config = surface.get_default_config( adapter, width, height ).expect( ".." );
  config.format = preferred_format( &surface.get_capabilities( adapter ).formats );
  surface.configure( device, &config );
  config
}
```

`surface_configure`'s own doc explicitly invites being called again on every resize, but never
validated that the new `size` was actually usable before handing it to `wgpu`, which panics
rather than erroring on a zero-area configuration.

## Why Not Caught

No test called `surface_configure` (or any precondition check backing it) with a zero size; the
crate's test-house-style itself notes GPU-touching behavior is exercised only via the real
example binary, and that example's own author had to discover and route around the gap by hand
rather than the library ever asserting it.

## Fix Location

`module/min/minwgpu/src/surface.rs`, `src/error.rs`.

```rust
// after -- explicit precondition check before touching wgpu at all
pub fn validate_size( size : ( u32, u32 ) ) -> Result< (), crate::Error >
{
  let ( width, height ) = size;
  if width == 0 || height == 0
  {
    return Err( crate::Error::ZeroSizeSurface( width, height ) );
  }
  Ok( () )
}

pub fn surface_configure( .., size : ( u32, u32 ) ) -> Result< wgpu::SurfaceConfiguration, crate::Error >
{
  validate_size( size )?;
  ..
  Ok( config )
}
```

`validate_size` is split out as its own function specifically so the precondition is
unit-testable without a real GPU adapter/device/surface, matching this crate's existing
pure-logic-only testing scope (`tests/surface_test.rs`'s own stated convention). New
`Error::ZeroSizeSurface( u32, u32 )` variant added. Both call sites in
`examples/minwgpu/flecs_bouncing_circles/src/main.rs` updated to `.expect(..)` the `Result`,
each with a message naming the specific reason that call site can never actually see the `Err`
branch (a `.max(1)`-clamped initial size; the pre-existing resize-handler guard).

## Prevention

Added 2 tests to `tests/surface_test.rs` (`bug_reproducer(BUG-165)`):
`validate_size_rejects_zero_width_or_height` (all 3 zero-containing combinations) and
`validate_size_accepts_nonzero_width_and_height` (control case).

## Pitfall

An "idempotent-safe, call again on every resize" contract invites exactly the kind of resize
input (a transient zero size, e.g. a minimized window) that the underlying GPU API panics on --
a resize-shaped function must validate the resize size itself, not assume every caller will
independently discover and guard the same edge case the way this crate's own example had to.

## Generalized Version

**Broken assumption:** "a helper documented as safe to call repeatedly on resize doesn't need
its own input validation, because `wgpu`'s lower-level API will handle whatever it's given."

**Confirmed general rule:** any function whose contract is "call again on resize" must validate
the resize dimensions itself before forwarding them to a lower-level API known to panic (rather
than error) on a degenerate size -- a documented invitation to call repeatedly is also an
invitation to eventually receive a degenerate input.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during a from-scratch Explore review of `module/min/minwgpu` (task #95); confirmed reachable via the crate's own example's pre-existing hand-written workaround. |
| 2026-08-16 | fixed | Added `validate_size` precondition check and `Error::ZeroSizeSurface`; `surface_configure` converted to `Result`-returning; both example call sites updated. |
| 2026-08-16 | verified | Added 2 tests to `tests/surface_test.rs`. Scoped native `cargo nextest`/`cargo clippy` clean across `minwgpu` + 4 downstream crates (`flecs_bouncing_circles`, `grid_render`, `hello_triangle`, `shader_chunks_render_core`), 45/45 tests passing, 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote a unit test against the fixed `validate_size`; adversarial pass re-read `wgpu-core-30.0.0` source directly (not just the Explore agent's claim) to confirm `ConfigureSurfaceError::ZeroArea`'s panic path before committing to this fix's shape. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | No coupling to BUG-162/163/164 (different crate, different failure family) -- explicitly checked and recorded as unrelated rather than left unstated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct `wgpu-core` source evidence plus a deliberate rejection of the "purely theoretical" alternative (H2), using the crate's own example as proof of real reachability. | — |
| D5 | Execution Scope | 🟢 | 🟢 | `surface_configure` widened to `Result` as the minimum correct fix; `validate_size` extracted only because it was needed for unit-testability, not as a speculative refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `minwgpu` src + 1 test file + 1 example + this bug file touched; no unrelated crates modified. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Both real call sites of `surface_configure` in this workspace (`flecs_bouncing_circles`, lines 203 and 256) were identified via `grep` and updated; no call site missed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `validate_size` is a single-purpose precondition check, exposed via `mod_interface!` only because it needed to be unit-testable from `tests/`. | — |

**Reproduced:** YES -- pre-fix, `surface_configure` had no defense against a zero-size input and
would panic via `wgpu::Surface::configure`, as proven by the crate's own example needing a
hand-written guard to avoid it. Post-fix, `validate_size` cleanly returns
`Err( Error::ZeroSizeSurface )` for every zero-containing size, confirmed via
`validate_size_rejects_zero_width_or_height`. Scoped native `cargo nextest`/`cargo clippy` clean
across `minwgpu` + 4 downstream crates, 45/45 tests passing, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwgpu/src/error.rs` | `Error` gained `ZeroSizeSurface( u32, u32 )`. |
| `module/min/minwgpu/src/surface.rs` | New `validate_size` precondition check (full `Fix(BUG-165)` comment); `surface_configure` converted to `Result`-returning, calling `validate_size` before touching `wgpu`. |
| `examples/minwgpu/flecs_bouncing_circles/src/main.rs` | Both `surface_configure` call sites updated to `.expect(..)` the new `Result`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwgpu/tests/surface_test.rs` | 2 new tests: `validate_size_rejects_zero_width_or_height` (`bug_reproducer(BUG-165)`), `validate_size_accepts_nonzero_width_and_height`. |
