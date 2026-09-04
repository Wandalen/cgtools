# BUG-330: `morph_targets`'s `gui_weights` buffer used `0.0` both as a real slider value and as the "untouched" sentinel, making a slider permanently unable to reset back to 0 once raised

- **Severity:** Medium (visible interaction defect -- a slider becomes permanently stuck above 0)
- **state:** Completed
- **Affects:** `examples/minwebgl/morph_targets/src/main.rs`
- **Component:** examples/minwebgl/morph_targets
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`gui_weights` was initialized filled with `0.0`, matching a slider the user has actively dragged
down to its minimum. Downstream logic treated `gui_weights[i] > 0.0` as "this slider has a GUI
override" -- so "untouched" and "explicitly zeroed" were indistinguishable, and once a slider was
raised above 0 it could never be reset back to 0 via that same slider (the write would still land,
but the read-side `> 0.0` check would then treat it as "no override" and fall back to the
animation-driven weight instead of the user's explicit 0).

## Impact

**Who is affected:** any user of this demo's morph-target GUI sliders who raises a slider above 0
and then attempts to lower it back to exactly 0.

**What breaks:** the slider visually shows 0 but the morph target doesn't actually zero out --
the demo silently reverts to the animation-driven weight instead of honoring the user's explicit
override, because the sentinel used to distinguish "no override" from "override" collides with a
legitimate override value.

**Entity Scope:** `None` -- confined to this crate's own GUI-weight-override state.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by tracing what value `gui_weights` is initialized with versus what the "has an override"
check actually tests for, rather than assuming a min-of-range initial value is a safe "untouched"
marker. Independently verified by the orchestrating session: the read-side check is a strict
`> 0.0` comparison, which by construction cannot distinguish an explicit `0.0` override from the
initial "untouched" `0.0` fill.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p morph_targets --test gui_weight_override_test
```
**Expected** (fixed): a slider explicitly set to `0.0` is distinguishable from an untouched slider,
and its override is honored (the morph target actually zeroes out). **Actual** (pre-fix): both
states were represented identically as `0.0`, and the explicit-zero case was silently treated as
"no override".

## Root Cause

The initial fill value's own sign/magnitude was used as a proxy for "has this slider been
touched", conflating a real, meaningful value (`0.0`, a legitimate slider position) with the
sentinel meant to represent "no GUI override yet" -- a min-of-range value makes a poor "untouched"
sentinel whenever that same value is also a legitimate, settable state.

## Fix Applied (2026-08-18)

Changed `gui_weights`'s initial fill from `0.0` to `f32::NAN`, a value no legitimate slider
position can ever equal, and updated the override check accordingly (`!gui_weights[i].is_nan()`
in place of `gui_weights[i] > 0.0`) so an explicit `0.0` override is now correctly distinguishable
from an untouched slider. Added `tests/gui_weight_override_test.rs`: asserts an untouched weight
is NaN (no override), and that explicitly setting a weight to `0.0` is recognized as a real
override rather than reverting to "untouched" behavior.

## Verification

- **Pre-fix (RED):** reverted the initial fill to `0.0` and the check to `> 0.0`; new test failed
  (explicit-zero override indistinguishable from untouched).
- **Post-fix (GREEN):** `cargo test -p morph_targets` -- new test passes (alongside sibling
  BUG-339's own lil_gui test in the same crate); `cargo check --target wasm32-unknown-unknown -p morph_targets`
  and `cargo clippy --all-targets --all-features -p morph_targets -- -D warnings` both clean.

## Generalized Version

A minimum-of-range value (here, a slider's own `0.0` minimum) makes a poor "untouched" sentinel
whenever that same value is also a legitimate, settable state -- the two are then permanently
indistinguishable to any downstream check testing for the sentinel. Use a value genuinely outside
the legitimate range (`NAN`, `Option::None`, a dedicated flag) as the "untouched" marker instead of
overloading a real domain value for double duty.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-330 after a fresh on-disk collision scan. Distinct from BUG-339's own `lil_gui.rs` fix in this same crate -- unrelated root cause, different files. |
