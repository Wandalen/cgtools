# BUG-335: `simple_pbr`'s light-0 rotation branch uses `*=` where its two sibling branches use `=`, multiplying the rotated value into itself instead of replacing it -- not a rotation, and not even magnitude-preserving

- **Severity:** Medium (visible incorrect light animation, not a crash)
- **state:** Completed
- **Affects:** `examples/minwebgl/simple_pbr/shaders/shader.frag`
- **Component:** examples/minwebgl/simple_pbr
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The light-animation block's `i == 0` branch read
`lightDir[i].xy *= rot(time) * lightDir[i].xy;` -- light 0's xy was overwritten with itself
component-wise multiplied by its own rotated value, instead of being replaced by the rotated value
outright. The `i == 1` and `i == 2` sibling branches correctly use plain assignment
(`lightDir[i].xz = rot(time) * lightDir[i].xz;` / `.yz = rot(time) * .yz;`).

## Impact

**Who is affected:** every user of this demo -- light 0's animated position is visibly wrong from
the very first frame.

**What breaks:** light 0 does not rotate at all in the intended sense -- its xy component is
squared/scaled by its own rotated value every frame rather than being replaced by a rotation,
producing neither a correct rotation nor even a magnitude-preserving transform (the vector's
length changes every frame instead of staying constant as a true rotation would).

**Entity Scope:** `None` -- confined to this crate's own fragment shader.

## How Discovered

Found by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task
#184), by diffing the three structurally-parallel `i == 0`/`i == 1`/`i == 2` branches against each
other character by character rather than reviewing each independently. Independently verified by
the orchestrating session: the `i == 1`/`i == 2` branches both use plain `=`, confirming `i == 0`'s
`*=` was the sole divergence.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "lightDir\[ i \].xy" examples/minwebgl/simple_pbr/shaders/shader.frag
```
**Expected** (fixed): the `i == 0` branch reads `lightDir[i].xy = rot(time) * lightDir[i].xy;`
(plain assignment, matching its siblings). **Actual** (pre-fix): the branch used `*=`, compounding
the rotated value into the existing one instead of replacing it.

## Root Cause

Copy-paste across the three near-identical branches left one compound-assignment operator (`*=`)
unconverted to the plain assignment (`=`) the other two branches correctly use.

## Why Not Caught

No test exercised each light's per-frame animation update individually -- the shader still
compiles and renders *some* moving light, so a wrong operator on one of three structurally
identical branches has no symptom beyond visually noticing light 0's motion looks different from
lights 1/2.

## Fix Applied (2026-08-18)

Changed the `i == 0` branch's operator from `*=` to `=`, matching its two sibling branches:
`lightDir[i].xy = rot(time) * lightDir[i].xy;`. No other line in the block was touched -- lights 1
and 2 were already correct.

## Verification

- **Pre-fix (RED):** reverted the `i == 0` branch to `*=`; direct source inspection confirms the
  divergence from its two correctly-assigned siblings.
- **Post-fix (GREEN):** `cargo check --target wasm32-unknown-unknown -p minwebgl_simple_pbr` and
  `cargo clippy --all-targets --all-features -p minwebgl_simple_pbr -- -D warnings` (native + wasm32) both
  clean; this crate has no pre-existing `tests/` directory and the defect is confined to a GLSL
  shader string with no isolable pure-Rust logic to unit test, so verification here is
  build/lint-level plus direct source diff against the corrected sibling branches, matching this
  crate's existing test-coverage baseline.

## Generalized Version

Three or more structurally-parallel branches (`if`/`else if` sharing the same right-hand-side
expression shape) are exactly where a single stray compound-assignment operator (`*=`/`+=` typo'd
in place of `=`) hides during copy-paste -- diff sibling branches character by character rather
than reviewing each in isolation, since the shader still compiles and "does something" regardless
of which operator is used.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by one of 4 parallel forks bug-hunting `examples/minwebgl`'s 44 remaining crates (task #184). Fixed under a `BUG-XXX-E` placeholder marker (disambiguated from sibling findings in the same fork's other crates) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session before this report and its real ID were assigned; placeholder replaced with BUG-335 after a fresh on-disk collision scan. |
