# BUG-450: `Mat3::from_axis_angle`'s `angle` parameter was hardcoded `f32`, silently truncating precision for `E = f64` callers

- **Severity:** Low (no crash, no wrong result for the common `E = f32` case; a precision loss only
  observable for `E = f64` callers, and only once `sin`/`cos` are taken of the truncated angle)
- **state:** Completed
- **Affects:** Any caller of `d2::mat3x3::transformation::from_axis_angle::<f64, _>` -- i.e. any `Mat3`
  rotation-matrix construction from an axis and angle where the crate is instantiated with `E = f64`.
  `E = f32` callers (the overwhelmingly common case throughout this workspace) were unaffected: the
  hardcoded parameter type happened to already match.
- **Component:** `module/math/ndarray_cg` (`src/d2/mat3x3/transformation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- an isolated signature-consistency defect, unrelated to the floating-point
  domain-clamping defects (BUG-445/446/447) found in the same sweep.

## Symptom

```rust
// pre-fix signature
pub fn from_axis_angle< E, Vec3 >( axis : Vec3, angle : f32 ) -> Mat3< E, ... >
where
  E : ...,
{
  // ...
  let s = E::from( angle.sin() ).unwrap(); // angle already truncated to f32 before .sin() ran
  let c = E::from( angle.cos() ).unwrap();
  // ...
}
```

Every sibling constructor in the same file (`from_angle_x`/`from_angle_y`/`from_angle_z`) correctly
takes `angle : E`, but `from_axis_angle` took `angle : f32` and routed it through `E::from(
angle.sin() ).unwrap()`/`E::from( angle.cos() ).unwrap()`. For an `E = f64` caller, the angle argument
was silently truncated to `f32` precision *before* `sin`/`cos` ever ran, discarding roughly half the
caller's precision with no warning -- not even a clippy lint, since the truncation happens at the call
boundary via a normal, valid `f32` argument, not a lossy cast the compiler can statically flag.

## Impact

**Who is affected:** `E = f64` callers of `from_axis_angle` specifically -- every other constructor in
the file was already fully generic. `E = f32` callers (the default and overwhelmingly common case
throughout this workspace) saw no behavioral change, since `f32`-typed input was never truncated
further.

**What breaks:** Silent precision loss, not a crash or gross wrong-value -- the resulting rotation
matrix is correct to `f32` precision even when the caller requested `f64`, which may or may not be
acceptable depending on the caller's own precision requirements, but was never surfaced as a choice.

**Consumer audit:** No `E = f64` call site of `from_axis_angle` exists anywhere in the workspace today
(grep confirms every call site in examples/modules instantiates `Mat3`/rotation constructors at `f32`)
-- so this fix has zero behavioral effect on any current caller; it closes a latent API trap for any
future `f64` caller.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide discovery sweep as BUG-445/446/447/448/449, specifically cross-checking
every generic function's parameter list against its own declared generic type parameters (the pattern:
"a generic numeric function with one hardcoded concrete-typed parameter"). `from_axis_angle` was the
only constructor in `d2/mat3x3/transformation.rs` with a mismatched parameter type; its three siblings
(`from_angle_x/y/z`) already established the fully-generic `angle : E` pattern to compare against.

## Minimum Reproducible Example

```rust
// conceptual -- demonstrates the pre-fix precision loss (not a panic/wrong-shape defect)
let precise_angle : f64 = 0.123_456_789_012_345; // more precision than f32 can hold
let m = Mat3::< f64, _ >::from_axis_angle( axis, precise_angle as f32 as f64 );
// pre-fix signature forced this f32-roundtrip implicitly at the call boundary;
// post-fix, `angle : E` accepts the full f64 value with no intermediate truncation
```

**Verify Command** (<=3 lines, standalone) -- compiles cleanly for both `E=f32` and `E=f64` instantiation
now that `angle : E` is fully generic (previously `E=f64` calls with a bare `f64` angle would not
compile at all without an explicit truncating cast, which is itself part of what made the defect easy to
miss -- callers were forced to truncate explicitly rather than the function silently doing it for them
via a mismatched signature):
```bash
cd module/math/ndarray_cg && cargo nextest run -p ndarray_cg -E 'binary(mat3x3_test)'
```

## Root Cause

Every sibling constructor in `d2/mat3x3/transformation.rs` (`from_angle_x`/`from_angle_y`/
`from_angle_z`) correctly takes `angle : E`, but `from_axis_angle` took `angle : f32` and converted the
already-`f32`-truncated `sin`/`cos` results into `E` via `E::from(...).unwrap()`. A generic numeric
function with one hardcoded concrete-typed parameter compiles fine and looks correct for the common `E
= f32` case -- the mismatch is only visible by explicitly checking every parameter against the
function's own generic type parameter, not just the return type and the other arguments.

## Why Not Caught

No `E = f64` call site existed anywhere in the workspace to exercise the truncation, and the function
compiled and produced correct results for every `E = f32` caller (the parameter's hardcoded type
happened to already match). The truncation is not a lossy *cast* the compiler or clippy can flag --
it happens because the parameter's declared type was narrower than the function's own generic parameter,
a signature-level inconsistency, not an expression-level one.

## Fix Location

`module/math/ndarray_cg/src/d2/mat3x3/transformation.rs::from_axis_angle`: `angle : f32` changed to
`angle : E`, matching the sibling `from_angle_x`/`from_angle_y`/`from_angle_z` pattern. `angle.sin()`/
`angle.cos()` now operate on the full-precision `E` value directly (no `E::from(...).unwrap()` needed
for the trig results themselves, since they are already type `E`).

## Prevention

No new runtime test added: no `E = f64` call site exists in the workspace to exercise the fixed
precision path with an observable before/after difference, and the fix is a pure signature-generalization
verified by type analysis (the function's `where` clause already required `E : ... + Float`-style bounds
sufficient for `.sin()`/`.cos()` on `E` directly, matching the sibling constructors exactly) plus this
crate's own compilation succeeding for its existing `E = f32` call sites, confirmed by the full nextest
run passing. Any future `f64` caller now gets full precision automatically with no code change on their
part.

## Pitfall

A generic numeric function with one hardcoded concrete-typed parameter compiles fine and looks correct
for the common case -- always check every parameter against the function's own generic type parameter,
not just the return type and the other arguments, especially when sibling functions in the same file
already establish the fully-generic pattern to follow. This class of defect produces no compiler
warning and no clippy lint, since the truncation happens at the call boundary via a normal, valid
argument of the (wrong, narrower) declared type.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep, cross-checking generic function signatures against sibling constructors in the same file. |
| 2026-08-20 | fixed | `angle : f32` -> `angle : E`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness | — | 🟢 | Confirming pass: signature now matches `from_angle_x`/`from_angle_y`/`from_angle_z`'s established `angle : E` pattern exactly; `where` clause already carried sufficient trait bounds for `.sin()`/`.cos()` on `E` (no new bound needed). Adversarial pass: searched the full workspace for any `E = f64` call site that might depend on the old `f32` parameter type or an explicit truncating cast at the call site that would now double-truncate or fail to compile -- none found, so the fix is a pure widening with zero call-site impact. `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass, and `cargo clippy -p mdmath_core -p ndarray_cg --all-targets --all-features -- -D warnings` clean, confirming the crate (including this function) compiles cleanly post-fix. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-450)`/`Root cause`/`Pitfall` 3-field format applied at the call site. | — |

**Reproduced:** Precision-loss defect confirmed by type/signature analysis (the pre-fix parameter type
`f32` is strictly narrower than the function's own generic `E`, which callers could instantiate as
`f64`) rather than a runtime assertion, since no `E = f64` call site exists in the workspace to produce
an observable before/after numeric difference to assert against. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/d2/mat3x3/transformation.rs` | `from_axis_angle`: `angle : f32` -> `angle : E`; `Fix(BUG-450)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

No test file changes -- see Prevention above for why no new runtime test was added (no `E=f64` call site
exists to exercise an observable difference); existing `mat3x3_test` suite continues to cover the
`E=f32` path unchanged.
