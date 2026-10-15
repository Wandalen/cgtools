# BUG-324: `filters`'s "Stack Blur" shader is an unweighted uniform average identical in shape to its own Box Blur, not a triangular-weighted stack blur

- **Severity:** Medium (a named, user-selectable algorithm behaves identically to a different one)
- **state:** Completed
- **Affects:** `examples/minwebgl/filters/src/filters/blur.rs`
- **Component:** examples/minwebgl/filters
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The "Stack Blur" shader's sampling loop computed an unweighted uniform average over its kernel
radius -- the exact same shape as the crate's own `Blur<Box>` implementation (same kernel body,
only the uniform name differed). A triangular weight (heavier toward the center, tapering at the
kernel's edges) is what actually distinguishes a stack blur from a box blur.

## Impact

**Who is affected:** any user selecting "Stack Blur" specifically, expecting a distinguishably
different (smoother, more center-weighted) blur profile than "Box Blur".

**What breaks:** the two named filter variants are visually and mathematically identical --
choosing "Stack Blur" over "Box Blur" changes nothing about the output.

**Entity Scope:** `None` -- confined to this crate's own blur-kernel shader source.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by comparing the three near-identical `Blur<T>` shader-string implementations in the same
file against each other rather than reviewing each in isolation. Independently verified by the
orchestrating session: the pre-fix "Stack Blur" loop body was a byte-for-byte structural match to
`Blur<Box>`'s own loop, differing only in the uniform's name.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p filters --test blur_kernel_test
```
**Expected** (fixed): the Stack Blur kernel's sampling weight varies with distance from center
(triangular), distinct from Box Blur's uniform weight. **Actual** (pre-fix): both kernels weighted
every sample identically.

## Root Cause

Copy-pasted box-average loop body across the three near-identical `Blur<T>` shader-string
implementations in the same file -- the "Stack Blur" variant never received its own distinguishing
triangular-weight computation.

## Why Not Caught

No test exercised the actual per-tap weight shape of either kernel -- both variants still visibly
blur the image, so an unintentionally-identical kernel has no symptom distinguishable from the
intended (different) one without reading the shader source directly.

## Fix Applied (2026-08-18)

Added a triangular weight (`float(u_radius + 1 - abs(i))`) to the Stack Blur kernel's sampling
loop, replacing its uniform-average body -- heaviest at the center tap, tapering linearly to the
kernel's edges, matching the standard stack-blur weighting profile and distinguishing it from
`Blur<Box>`'s own uniform-average kernel. Added `tests/blur_kernel_test.rs`: `include_str!`-based
structural assertion that the Stack Blur shader source computes a non-uniform per-tap weight
(distinct from a bare running sum), distinguishing it from the Box Blur shader string.

## Verification

- **Pre-fix (RED):** reverted the Stack Blur kernel to its unweighted-average form; new test
  failed (kernel shape indistinguishable from Box Blur's).
- **Post-fix (GREEN):** `cargo test -p filters` -- new test passes (alongside sibling BUG-325's own
  test in the same crate); `cargo check --target wasm32-unknown-unknown -p filters` and
  `cargo clippy --all-targets --all-features -p filters -- -D warnings` both clean.

## Generalized Version

Three or more sibling shader-string implementations living in one file, each meant to demonstrate
a visually and mathematically distinct algorithm, is exactly where a copy-pasted kernel body
hides -- the result still visibly performs the general category of operation (here, blurring),
giving no symptom that two supposedly-distinct named variants are actually identical; diff sibling
implementations against each other directly, not just review each independently.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed and tested under a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-324 after a fresh on-disk collision scan. Related: BUG-325, a second, unrelated-root-cause bug in the same crate's HSL adjustment filter. |
