# BUG-318: `attributes_vao` merges two independent vertex datasets into a single VAO instead of demonstrating two switchable VAOs, leaving one dataset's position/point-size fields and a whole draw call unused

- **Severity:** Medium (the demo's own stated purpose -- switching between VAOs -- is not what it does)
- **state:** Completed
- **Affects:** `examples/minwebgl/attributes_vao/src/main.rs`
- **Component:** examples/minwebgl/attributes_vao
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

A single `vao` was built by binding position/point-size attribute pointers from `vert_buffer2`
alongside color attribute pointers from `vert_buffer` -- one VAO composed from two different
buffers, rather than each buffer getting its own complete, self-contained VAO.

## Impact

**Who is affected:** any reader of this crate's own `readme.md`, which states the demo shows
"switch[ing] between different vertex configurations with a single binding call."

**What breaks:** that claim requires two independent VAOs to switch between; the actual code built
one VAO mixing fields from both buffers, so `vert_buffer`'s own position/point-size fields (and an
entire second draw call demonstrating the switch) were unused dead data -- the demo cannot show
what its own readme says it shows.

**Entity Scope:** `None` -- confined to this crate's own VAO setup.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184). Independently verified by the orchestrating session: `vert_buffer` and `vert_buffer2` are
each already complete position+size+color records (confirmed via their own upload calls), which
only makes sense as two independently switchable datasets, not one to be spliced together.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "let vao2\b\|gl.bind_vertex_array( Some( &vao2 ) )" examples/minwebgl/attributes_vao/src/main.rs
```
**Expected** (fixed): a second, independent `vao2` is created and bound with `vert_buffer`'s own
attribute pointers. **Actual** (pre-fix): no `vao2` existed at all -- only one `vao` mixing both
buffers' fields.

## Root Cause

`vert_buffer`'s attribute-pointer calls were bound against the same `vao` as `vert_buffer2`'s,
instead of each buffer receiving its own VAO -- collapsing what should have been two independent
vertex configurations into one spliced-together configuration.

## Why Not Caught

No test exercised the VAO setup's structure (attribute-pointer call targets vs. buffer sources) --
the demo still visibly renders points on screen either way, so a merged VAO has no visible symptom
distinguishing it from the intended two-VAO design.

## Fix Applied (2026-08-18)

Split the single `vao` into two independent VAOs (`vao` bound to `vert_buffer`'s own
position/point-size/color attribute pointers, `vao2` bound to `vert_buffer2`'s), matching the
readme's "switch between different vertex configurations" description. Added `tests/two_vao_switching_test.rs`
asserting (via `include_str!` structural checks on `main.rs`) that two distinct VAO handles are
created and that each one's attribute-pointer calls reference only its own buffer, not the other's.

## Verification

- **Pre-fix (RED):** reverted to the single merged `vao`; new test failed (no second VAO,
  cross-buffer attribute-pointer calls detected).
- **Post-fix (GREEN):** `cargo test -p minwebgl_attributes_vao` -- new test passes;
  `cargo check --target wasm32-unknown-unknown -p minwebgl_attributes_vao` and
  `cargo clippy --all-targets --all-features -p minwebgl_attributes_vao -- -D warnings` both clean.

## Generalized Version

When a demo's own readme describes switching between N independent configurations, verify the
source actually constructs N independent GPU objects (VAOs, buffers, programs) rather than one
object built by splicing fields from multiple data sources -- a merged setup can still render
something plausible-looking while silently failing to demonstrate the technique the demo exists
to show.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-YYY` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-318 after a fresh on-disk collision scan. |
