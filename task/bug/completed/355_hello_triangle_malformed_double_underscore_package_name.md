# BUG-355: `hello_triangle`'s package name was the malformed `"minwebgpu__"` instead of following the `minwebgpu_<crate>` sibling convention

- **Severity:** Low (naming-only defect -- compiles, links, and runs identically either way)
- **state:** Completed
- **Affects:** `examples/minwebgpu/hello_triangle/Cargo.toml`
- **Component:** examples/minwebgpu/hello_triangle
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Cargo.toml`'s `[package] name` was `"minwebgpu__"` -- a bare double-underscore with no crate
suffix -- instead of following the `minwebgpu_<crate>` convention every sibling crate in this
same directory correctly follows: `minwebgpu_hello_triangle_quickstart`,
`minwebgpu_deffered_rendering`, `minwebgpu_renderer_pbr_scene`.

## Impact

**Who is affected:** anyone running `cargo build/test/check -p <name>` against this crate by
name, or reading `Cargo.lock`/build output to identify which crate a compiler message belongs to.

**What breaks:** nothing functionally -- the crate compiles, links, and runs identically under
any valid package name. The defect is entirely in the name's own legibility: `minwebgpu__` gives
no hint this is the `hello_triangle` example, and doesn't match every sibling's self-describing
naming pattern.

**Entity Scope:** `None` -- confined to this crate's own `Cargo.toml`/`Cargo.lock` entry; nothing
else in the workspace references it by package name (`Cargo.toml`'s workspace members are
glob/path-based, not name-based).

## How Discovered

Found while systematically auditing `examples/minwebgpu`'s 5 crates for task #185, cross-checking
each crate's declared package name against its sibling crates' naming convention within the same
parent directory rather than assuming a name compiles correctly. Independently confirmed via
`grep -rn "minwebgpu__"` across the workspace: only `Cargo.toml`/`Cargo.lock`'s own entry and
`locales.md`'s asset-listing row (a generated file, tracks the name descriptively -- not itself a
defect) referenced it; three historical `task/` reports (058, 099, 306) also mention the old name
accurately describing what was true when they were filed, and are correctly left untouched.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep "^name" examples/minwebgpu/hello_triangle/Cargo.toml
cargo test -p minwebgpu_hello_triangle --test package_name_test
```
**Expected** (fixed): `Cargo.toml` reads `name = "minwebgpu_hello_triangle"`, and the new
regression test passes. **Actual** (pre-fix): `Cargo.toml` read `name = "minwebgpu__"`, and no
test existed to catch a malformed package name.

## Root Cause

Likely a truncated in-progress rename: the trailing double underscore reads as though a suffix
(`hello_triangle`) was meant to follow but was never typed in, leaving the placeholder-looking
`minwebgpu__` committed as-is.

## Why Not Caught

A malformed-but-syntactically-valid package name produces no compiler error, warning, or lint --
Cargo accepts any valid crate-name string, so the wrong name has zero build-time symptom. Nothing
in the codebase cross-checked a crate's declared name against its own directory name or its
siblings' naming convention.

## Fix Applied (2026-08-18)

Renamed the package to `minwebgpu_hello_triangle` in `Cargo.toml`, with a `Fix(BUG-355)` comment
documenting the root cause. Regenerated `Cargo.lock`'s corresponding entry via `cargo check`
(no manual lock-file editing). Added `tests/package_name_test.rs`
(`cargo_toml_package_name_matches_sibling_convention`): `include_str!`-based assertion that
`Cargo.toml`'s declared name matches `minwebgpu_hello_triangle` and never reverts to the
malformed `minwebgpu__` form, following the same plain-text-check pattern already established by
this crate's own `tests/doc_comment_test.rs` (BUG-306-A).

Left untouched, by design: `locales.md` (headed "Generated. Do not edit manually." -- its own
regeneration process will pick up the new name); the three historical task/bug reports (058, 099,
306) that reference `minwebgpu__` accurately describing the crate's name at the time they were
filed.

## Verification

- **Pre-fix (RED):** `cargo test -p minwebgpu__ --test package_name_test` (run against the
  pristine `Cargo.toml`, before the rename) -- `cargo_toml_package_name_matches_sibling_convention`
  failed on both assertions (name still contained the malformed string, didn't yet contain the
  corrected one).
- **Post-fix (GREEN):** `longrun`-detached combined sweep --
  `cargo check --target wasm32-unknown-unknown -p minwebgpu_hello_triangle` clean;
  `cargo test -p minwebgpu_hello_triangle --test package_name_test` -- new test passes;
  `cargo test -p minwebgpu_hello_triangle --test doc_comment_test` -- pre-existing BUG-306-A test
  still passes (1/1, unaffected by the rename); `cargo clippy -p minwebgpu_hello_triangle
  --all-targets --all-features -- -D warnings` clean. `Cargo.lock` independently re-checked:
  exactly one `minwebgpu_hello_triangle` entry, zero remaining `minwebgpu__` entries.

## Generalized Version

A crate's declared package name is a factual claim about its own identity, exactly like a doc
comment or a readme's prose -- but unlike prose, a malformed name is *syntactically valid* to the
compiler and produces no error, warning, or lint of any kind. When auditing a directory of sibling
example crates sharing a naming convention (`minwebgpu_<crate>`, `minwebgl_<crate>`, etc.), check
each crate's actual `Cargo.toml` name against that convention explicitly -- a build passing proves
nothing about whether the name itself is right.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found while auditing `examples/minwebgpu`'s 5 crates for task #185. Fresh on-disk collision scan confirmed 355 as the next free ID (`task/readme.md`'s `highest_id: 354` already matched the on-disk max at scan time, following a concurrent actor's earlier 341-354 filing in an unrelated scope). |
