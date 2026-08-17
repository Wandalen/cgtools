# BUG-170: `mod_interface` import/invocation ungated, breaks `--no-default-features`

- **Severity:** Medium (a documented, dependency-gating feature flag makes the crate fail to
  compile at all in one of its 2 supported configurations -- not a runtime defect, but a hard
  build break for any caller opting out of the crate's functionality via `--no-default-features`)
- **state:** Completed
- **Affects:** The whole `ndarray_tools` crate when built with `--no-default-features` (the
  `enabled` feature off) -- any downstream `Cargo.toml` that depends on `ndarray_tools` but
  disables its default features
- **Component:** `module/alias/ndarray_tools` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same defect class as BUG-169 (`browser_log`'s `mod private` missing the same
  gate) -- found immediately after fixing BUG-169, while starting task #97's review of
  `alias/ndarray_tools`. Independent occurrence, not a shared root cause: BUG-169 had the macro
  invocation correctly gated but a *sibling item* (`mod private`) missing the gate; this bug has
  *neither* the import nor the invocation gated at all -- a more complete omission of the same
  pattern in a different crate. Same Prevention convention as BUG-169/BUG-053 (documented
  dual-configuration Verify Command, no unit test) applies for the identical structural reason.

## Symptom

```rust
// pre-fix -- lib.rs, fully unconditional
use ::mod_interface::mod_interface;   // `mod_interface` is an optional dependency

mod private
{
  // use super::*;
}

crate::mod_interface!
{
  reuse ::ndarray_cg;                 // `ndarray_cg` is ALSO an optional dependency
}
```

## Impact

**Who is affected:** Any downstream crate depending on `ndarray_tools` with
`default-features = false` and no explicit `enabled` feature re-added -- the documented,
supported way to fully strip this alias crate down (it exists purely to `reuse ::ndarray_cg;`,
so disabling it is equivalent to opting out entirely).

**What breaks:** The crate fails to compile at the very first line that needs a dependency --
`E0432: unresolved import 'mod_interface'` -- since `mod_interface` (the crate) is
`optional = true` and only present when `enabled` is on, but the `use` statement pulling it in
has no matching `#[cfg(...)]`. Had that one line alone been fixed, the `mod_interface!`
invocation's `reuse ::ndarray_cg;` would fail next for the identical reason against `ndarray_cg`.

**Magnitude:** Every `--no-default-features` build of this crate, 100% of the time -- there is
no configuration in which `enabled` is off that successfully compiles pre-fix.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, at the start of task #97's review of `module/alias/ndarray_tools` -- immediately
after closing BUG-169 (the same defect class in the sibling alias crate `browser_log`), this
crate's `Cargo.toml` was checked for the same `enabled`-gates-every-dependency shape found there.
It matched (`enabled = ["dep:mod_interface", "dep:ndarray_cg"]`, `default = ["enabled"]`), so
`lib.rs` was read specifically looking for the BUG-169 pattern -- and found a more complete
version of it: neither the `mod_interface` import nor its invocation carried any gate at all.
Confirmed directly via `cargo check -p ndarray_tools --no-default-features`.

## Minimum Reproducible Example

```bash
cd module/alias/ndarray_tools && cargo check -p ndarray_tools --no-default-features
```

**Expected** (post-fix): clean compile, `Finished` with no errors.

**Actual** (pre-fix):
```
error[E0432]: unresolved import `mod_interface`
 --> module/alias/ndarray_tools/src/lib.rs:6:7
  |
6 | use ::mod_interface::mod_interface;
  |       ^^^^^^^^^^^^^ could not find `mod_interface` in the list of imported crates

error: could not compile `ndarray_tools` (lib) due to 1 previous error
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/alias/ndarray_tools && cargo check -p ndarray_tools --no-default-features && cargo check -p ndarray_tools
# both must exit 0 -- the first exercises `enabled` OFF, the second `enabled` ON (default)
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Neither `use ::mod_interface::mod_interface;` nor the `crate::mod_interface! { reuse ::ndarray_cg; }` invocation carries `#[cfg(feature = "enabled")]`, even though both `mod_interface` and `ndarray_cg` are optional dependencies only present when `enabled` is on -- a `--no-default-features` build has neither crate available at all. | ✅ Root Cause | Confirmed by reading `Cargo.toml` (`enabled = ["dep:mod_interface", "dep:ndarray_cg"]`) alongside `lib.rs` (no `#[cfg(...)]` anywhere on the import or invocation). Directly reproduced via `cargo check --no-default-features`, exact predicted `E0432` on the very first dependency reference. | E1, E2 |
| H2 | This is a distinct, unrelated defect from BUG-169, coincidentally found in another alias crate right after fixing it. | ❌ Falsified (partially) | Same *pattern* (an `enabled` feature gating 100% of a crate's dependencies, with at least one unconditional reference left behind), but a genuinely separate occurrence in a separate crate/file with no shared code or root cause -- correctly logged as "same class, independent occurrence" rather than either "the same bug" or "fully unrelated." | E1, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/alias/ndarray_tools/src/lib.rs` (pre-fix) | `use ::mod_interface::mod_interface;` and `crate::mod_interface! { reuse ::ndarray_cg; }` both fully unconditional -- no `#[cfg(...)]` anywhere in the file. | H1 ✅ |
| E2 | `cargo check -p ndarray_tools --no-default-features` (pre-fix, real compiler output) | `E0432: unresolved import 'mod_interface'` at `lib.rs:6:7`, exactly the first dependency-referencing line in the file -- matching the predicted failure mode. | H1 ✅ |
| E3 | `module/alias/ndarray_tools/Cargo.toml` vs. `module/helper/browser_log/Cargo.toml` (BUG-169) | Both crates define an `enabled` feature gating 100% of their dependencies with `default = ["enabled"]`, and both had at least one unconditional `lib.rs` item depending on a since-optional dependency -- same defect *shape*, but `browser_log`'s `mod_interface!` invocation was itself correctly gated (only a sibling item, `mod private`, was missed), while `ndarray_tools` had no gating anywhere at all -- confirming these are independent occurrences of the same class, not one shared bug. | H2 (partial) |

## Root Cause

```rust
// before -- nothing in lib.rs gated, despite Cargo.toml gating every dependency behind `enabled`
use ::mod_interface::mod_interface;

crate::mod_interface!
{
  reuse ::ndarray_cg;
}
```

`enabled` gates both of this crate's dependencies (`mod_interface`, `ndarray_cg`) in
`Cargo.toml`, but nothing in `lib.rs` carries the matching `#[cfg(feature = "enabled")]` --
Cargo's dependency-optionality and Rust's own `#[cfg(...)]` gating are two separate mechanisms
that must be kept in sync by hand; making a dependency optional in `Cargo.toml` provides no
compile-time guarantee that every reference to it elsewhere is correctly gated to match.

## Why Not Caught

No existing verification pass in this workspace exercises `ndarray_tools` with
`--no-default-features` -- every documented Level 1-5 command (`will .test level::N`) and every
downstream consumer builds it with default features, where `enabled` is always on and this code
path is never reached. Identical gap to BUG-169's, in a different crate.

## Fix Location

`module/alias/ndarray_tools/src/lib.rs`.

```rust
// after -- both the import and the invocation gated to match Cargo.toml's own dependency gating
#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

mod private
{
  // use super::*;
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  /// Reusing main crate.
  reuse ::ndarray_cg;
}
```

`mod private` itself needs no gate -- it is empty (a single commented-out line), with no
reference to any optional dependency, so it compiles fine unconditionally in either
configuration.

## Prevention

Following the same-class precedent set by BUG-169 and BUG-053: no unit test is added, because a
single compiled test binary runs under whatever feature flags `cargo test` itself was invoked
with and cannot exercise a second, different `cargo` invocation of the same crate from inside
itself. The durable regression check is the dual-configuration Verify Command documented above.

## Pitfall

A crate whose `Cargo.toml` gates 100% of its dependencies behind one feature (the "fully
optional, `--no-default-features` compiles to an empty shell" pattern already established by
BUG-169) needs that same feature applied to *every* unconditional item in `lib.rs` that
references those dependencies, checked individually -- Cargo.toml's own gating structure gives
no compile-time guarantee that `lib.rs` actually matches it. Two crates in this same workspace
(`browser_log`, `ndarray_tools`) independently adopted the gating convention in `Cargo.toml` and
independently forgot to fully apply it in `lib.rs` -- worth treating as a class of defect to
spot-check in any other alias/shim crate using this same `enabled`-gates-everything shape.

## Generalized Version

**Broken assumption:** "marking a dependency `optional = true` and gating it behind a Cargo
feature is, by itself, enough to make `--no-default-features` work -- the `Cargo.toml` structure
is the whole contract."

**Confirmed general rule:** `Cargo.toml`'s dependency optionality and `lib.rs`'s `#[cfg(...)]`
gating are two independently-maintained mechanisms. Every unconditional item in `lib.rs` (an
import, a macro invocation, a struct field, a function body) that references an optional
dependency needs its own matching `#[cfg(feature = "...")]` -- there is no propagation from one
mechanism to the other, and nothing catches the gap except an actual `--no-default-features`
build attempt.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered at the start of task #97's review of `module/alias/ndarray_tools`, immediately after fixing the same-class BUG-169 in the sibling alias crate `browser_log`; confirmed via `cargo check -p ndarray_tools --no-default-features` producing the predicted `E0432`. |
| 2026-08-16 | fixed | Added `#[cfg(feature = "enabled")]` to both the `mod_interface` import and the `mod_interface!` invocation. |
| 2026-08-16 | verified | `cargo check`/`cargo clippy --no-default-features` clean (native and `wasm32-unknown-unknown`); default and `full` feature configurations re-confirmed unaffected (native and `wasm32-unknown-unknown`); scoped native `cargo nextest`/`cargo clippy` clean across `ndarray_tools`, 272/272 tests passing (the full re-exported `ndarray_cg` suite run through this alias), 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass spotted the pattern by deliberately checking `Cargo.toml` against the just-fixed BUG-169 shape before reading `lib.rs`; adversarial pass demanded the real compiler output (not just source-reading inference) both pre-fix (`E0432`, matching prediction exactly) and post-fix (clean, across native + `wasm32`, all 3 feature configurations) before accepting. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Explicitly checked against BUG-169/BUG-053 (same defect class, precedent for the no-unit-test Prevention approach) and explicitly recorded as an independent occurrence, not a shared root cause, with the distinction (H2) reasoned through rather than assumed either way. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct comparison of `Cargo.toml`'s dependency-gating structure against `lib.rs`'s actual `#[cfg(...)]` coverage, plus the real compiler's own error output confirming the exact mechanism. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Two-line `#[cfg(...)]` addition, the minimal fix that restores gate parity between `Cargo.toml` and `lib.rs` -- no broader refactor attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `ndarray_tools`'s `lib.rs` + this bug file touched; no unrelated crates modified. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Both unconditional items causing the break were found via full-file read (the file is 17 lines) and fixed at their own definition sites; `mod private` correctly left ungated since it references nothing optional. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely a gate-parity correction; the crate's own responsibility (re-exporting `ndarray_cg`) is unchanged. | — |

**Reproduced:** YES -- pre-fix, `cargo check -p ndarray_tools --no-default-features` failed with
exactly the predicted `E0432` on the first dependency-referencing line. Post-fix, the identical
command exits 0, and `default`/`full` feature configurations (native and
`wasm32-unknown-unknown`) are confirmed unaffected. Scoped native `cargo nextest`/`cargo clippy`
clean across `ndarray_tools`, 272/272 tests passing, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/alias/ndarray_tools/src/lib.rs` | `use ::mod_interface::mod_interface;` and the `crate::mod_interface! { reuse ::ndarray_cg; }` invocation both now carry `#[cfg(feature = "enabled")]` (full `Fix(BUG-170)` comment block), matching `Cargo.toml`'s existing gating of both dependencies behind that same feature. |

## Refs: tests/

None -- this is a build-configuration defect (see `## Prevention`); the regression check is the
documented dual-configuration Verify Command above, following the BUG-169/BUG-053 precedent for
this defect class.
