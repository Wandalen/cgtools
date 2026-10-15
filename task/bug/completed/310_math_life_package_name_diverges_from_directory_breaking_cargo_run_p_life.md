# BUG-310: `examples/math/life`'s `Cargo.toml` declared `name = "math_trivial"`, diverging from its own directory name and breaking `cargo run -p life`

- **Severity:** Low (discoverable, reproducible command-fails defect; no logic/behavior affected)
- **state:** Completed
- **Affects:** `examples/math/life/Cargo.toml`
- **Component:** examples/math/life
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`examples/math/life/Cargo.toml` declared `name = "math_trivial"`, while the crate's own directory
is `life`. This is the only such package-name/directory-name divergence anywhere in the
workspace. Any user or script naturally trying `cargo run -p life` (the obvious command matching
the visible directory name) fails with "package ID specification `life` did not match any
packages", while the actual working command (`cargo run -p math_trivial`) bears no visible
relation to the directory a reader would be looking at.

## Impact

**Who is affected:** anyone trying to run, test, or reference this crate by its directory name,
which is the natural first guess for any `cargo -p <name>` invocation.

**What breaks:** `cargo run -p life` / `cargo test -p life` / `cargo clippy -p life` all fail with
a "no such package" error, despite `life/` clearly existing as a real crate directory.

**Entity Scope:** `None` -- pure metadata mismatch, no behavioral/logic defect.

## How Discovered

Disclosed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer`
crates (task #183), which found this as a hygiene issue distinct from its 3 behavioral findings.
Independently verified by reading `Cargo.toml` directly.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "^name" examples/math/life/Cargo.toml
cargo run -p life 2>&1 | head -3
```
**Expected** (fixed): `name = "life"`, and `cargo run -p life` resolves to the crate. **Actual**
(pre-fix): `name = "math_trivial"`, and `cargo run -p life` failed with "did not match any
packages in the workspace" despite `examples/math/life/` genuinely existing.

## Root Cause

The crate's `Cargo.toml` was authored (or copy-pasted from an earlier, differently-named crate)
with `name = "math_trivial"` and never updated to match its own directory name when it was placed
at (or renamed to) `examples/math/life/`.

## Why Not Caught

No test or check in this workspace verifies that every crate's `Cargo.toml` `name` matches its
own directory -- this is the only crate in the entire workspace where the two diverge, so nothing
generalized ever surfaced it.

## Fix Applied (2026-08-18)

Changed `Cargo.toml`'s `name = "math_trivial"` to `name = "life"`. Pure package-metadata rename --
no source code, test, or behavioral logic changed. No new regression test was added: there is
nothing behavioral to guard against regressing, and a future re-divergence would be immediately
and loudly caught by `cargo run -p life`/`cargo check -p life` themselves failing, exactly as this
defect was discovered.

## Verification

- **Pre-fix:** `cargo run -p life` failed to resolve the package.
- **Post-fix, independently re-run by the orchestrating session:** `cargo check -p life`
  (`longrun`-detached sweep) → `Checking life v0.1.0
  (/home/user1/pro/lib/yrd_gamedev/cgtools/examples/math/life)`, `Finished` cleanly. `cargo
  clippy -p life --all-targets --all-features -- -D warnings` → clean.

## Generalized Version

A crate's `Cargo.toml` package `name` and its own directory name are two independently-editable
strings with no compiler-enforced link between them -- a workspace-wide sweep comparing the two
(one-time or as a lint) would catch a future recurrence; absent that, this is the only such
divergence found in this workspace during this bug-hunt.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer` crates (task #183, one of 3 parallel forks covering 27 `examples/` crates), alongside 3 sibling behavioral findings (BUG-307/308/309) in the same fork's scope. Independently verified by the orchestrating session (diff read, `cargo check -p life` independently re-run) before this report and its real ID were assigned. No pre-existing precedent this session for whether a pure zero-behavioral-impact rename warrants its own bug report (unlike BUG-301's dormant-but-real sibling "clean" crates, which get no report at all) -- filed here since `cargo run -p life` failing is a concrete, discoverable, reproducible command-fails defect, matching the shape of other command-fails bugs already filed this session (e.g. BUG-293, BUG-296), not merely a style nit. |
