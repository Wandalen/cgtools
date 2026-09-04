# BUG-169: `mod private` missing `#[cfg(feature = "enabled")]` breaks `--no-default-features`

- **Severity:** Medium (a documented, dependency-gating feature flag makes the crate fail to
  compile at all in one of its 2 supported configurations -- not a runtime defect, but a hard
  build break for any caller opting out of the crate's functionality via `--no-default-features`)
- **state:** Completed
- **Affects:** The whole `browser_log` crate when built with `--no-default-features` (the
  `enabled` feature off) -- any downstream `Cargo.toml` that depends on `browser_log` but
  disables its default features (e.g. to strip it entirely from a build that doesn't need
  browser logging)
- **Component:** `module/helper/browser_log` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered in the same Explore review pass as BUG-167 and BUG-168 (task #96,
  `module/alias/browser_tools` -- resolved to the underlying `browser_log` crate it re-exports
  wholesale). Independent root cause from both: BUG-167 is a `file!()`/`line!()` lexical-
  resolution defect in `log/debug_log.rs`; BUG-168 is `panic.rs`'s `Config.with_location` flag
  being a no-op. This bug is a missing feature-gate in `lib.rs` itself, a build-configuration
  defect rather than a runtime one -- same class as BUG-053 (`RUSTFLAGS`/`web_sys_unstable_apis`
  silent override), whose Prevention convention (a documented dual-configuration Verify Command,
  no unit test) this report follows for the same structural reason: a single compiled test
  binary cannot exercise two different `cargo` feature-flag invocations of its own crate.

## Symptom

```rust
// pre-fix -- lib.rs
#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

mod private          // <- unconditional, but its body needs `crate::log`/`crate::panic`
{
  pub struct Config
  {
    pub log : crate::log::setup::Config,     // only declared by the gated mod_interface! below
    pub panic : crate::panic::Config,        // only declared by the gated mod_interface! below
  }
  pub fn setup( config : Config )
  {
    crate::panic::setup( config.panic );
    crate::log::setup::setup( config.log );
  }
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  own use { Config, setup };
  layer log;    // <- this is what actually declares `mod log;`
  layer panic;  // <- this is what actually declares `mod panic;`
}
```

## Impact

**Who is affected:** Any downstream crate depending on `browser_log` with
`default-features = false` and no explicit `enabled` feature re-added -- the documented,
supported way to fully strip this crate's functionality (and its `wasm-bindgen`/`web-sys`/`log`/
`mod_interface` dependencies) from a build that doesn't need browser logging.

**What breaks:** The crate fails to compile at all -- 4 `E0433` "cannot find `log`/`panic` in
`crate`" errors, all inside `mod private`, which references `crate::log::setup::Config`/
`crate::panic::Config` unconditionally even though those submodules are only ever declared by
the `mod_interface!` macro invocation, which is correctly `#[cfg(feature = "enabled")]`-gated
but does nothing to gate `mod private` itself.

**Magnitude:** Every `--no-default-features` build of this crate, 100% of the time -- there is
no configuration in which `enabled` is off that successfully compiles pre-fix.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via the same background Explore review of `module/alias/browser_tools` (task #96)
that surfaced BUG-167/168, resolving to `browser_log`'s own `lib.rs` as the bug surface: the
`enabled` feature's `Cargo.toml` definition gates every optional dependency
(`wasm-bindgen`/`web-sys`/`log`/`mod_interface`) and the `mod_interface!` invocation, but
`mod private` -- which needs types from the submodules that invocation declares -- was left
unconditional. Confirmed directly via `cargo check -p browser_log --no-default-features`,
producing the exact 4 `E0433` errors predicted by reading the source.

## Minimum Reproducible Example

```bash
cd module/helper/browser_log && cargo check -p browser_log --no-default-features
```

**Expected** (post-fix): clean compile, `Finished` with no errors.

**Actual** (pre-fix):
```
error[E0433]: cannot find `log` in `crate`
  --> module/helper/browser_log/src/lib.rs:17:22
   |
17 |     pub log : crate::log::setup::Config,
   |                      ^^^ could not find `log` in the crate root

error[E0433]: cannot find `panic` in `crate`
  --> module/helper/browser_log/src/lib.rs:19:24
   |
19 |     pub panic : crate::panic::Config,
   |                        ^^^^^ unresolved import

error[E0433]: cannot find `panic` in `crate`
  --> module/helper/browser_log/src/lib.rs:26:12
   |
26 |     crate::panic::setup( config.panic );
   |            ^^^^^ unresolved import

error[E0433]: cannot find `log` in `crate`
  --> module/helper/browser_log/src/lib.rs:27:12
   |
27 |     crate::log::setup::setup( config.log );
   |            ^^^ could not find `log` in the crate root

error: could not compile `browser_log` (lib) due to 4 previous errors
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_log && cargo check -p browser_log --no-default-features && cargo check -p browser_log
# both must exit 0 -- the first exercises `enabled` OFF, the second `enabled` ON (default)
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `mod private` references `crate::log`/`crate::panic`, but those submodules are declared only inside the `#[cfg(feature = "enabled")]`-gated `mod_interface!` invocation -- gating the macro call alone doesn't retroactively gate unconditional code elsewhere that depends on its expansion. | ✅ Root Cause | Confirmed by reading `lib.rs`: `mod private` (unconditional) references `crate::log::setup::Config`/`crate::panic::Config`; the only declarations of `log`/`panic` as real modules come from `layer log;`/`layer panic;` inside the gated `mod_interface!` block. Directly reproduced via `cargo check --no-default-features`, exact 4 errors predicted from the source read. | E1, E2 |
| H2 | `enabled` was never actually intended to support a fully-empty-shell build -- the feature exists only to make dependencies technically optional in `Cargo.toml`, not as a real, testable `--no-default-features` configuration. | ❌ Falsified | `Cargo.toml`'s `[package.metadata.docs.rs]` builds `all-features = true` (implying a non-`all-features` build is also expected to be meaningful), and every one of `enabled`'s 4 gated dependencies is `optional = true` specifically so a caller can omit them entirely -- the crate-level doc/feature design clearly intends `--no-default-features` to be a real, compiling configuration, just one nothing had verified yet. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_log/src/lib.rs` (pre-fix) | `mod private` has no `#[cfg(...)]` at all, while the `mod_interface!` invocation below it (which alone declares `mod log;`/`mod panic;` via `layer log;`/`layer panic;`) is `#[cfg(feature = "enabled")]`-gated. | H1 ✅ |
| E2 | `cargo check -p browser_log --no-default-features` (pre-fix, real compiler output) | Exactly 4 `E0433` errors, all inside `mod private`, all "cannot find `log`/`panic` in `crate`" -- matching the predicted failure mode precisely. | H1 ✅ |
| E3 | `module/helper/browser_log/Cargo.toml` | `enabled = ["dep:wasm-bindgen", "dep:web-sys", "dep:log", "dep:mod_interface"]`, `default = ["enabled"]` -- every dependency the crate has is optional and gated by this one feature, a design that only makes sense if `--no-default-features` is meant to compile to an empty shell. | H2 ❌ |

## Root Cause

```rust
// before -- mod private unconditional, but its body needs submodules only mod_interface! declares
mod private
{
  pub struct Config { pub log : crate::log::setup::Config, pub panic : crate::panic::Config }
  pub fn setup( config : Config ) { crate::panic::setup( config.panic ); crate::log::setup::setup( config.log ); }
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  layer log;    // declares `mod log;` -- but only when `enabled` is on
  layer panic;  // declares `mod panic;` -- but only when `enabled` is on
}
```

`enabled` gates every optional dependency (`log`, `mod_interface`, `wasm-bindgen`, `web-sys`)
*and* the `mod_interface!` invocation that declares the `log`/`panic` submodules -- but
`mod private`, which references both submodules' types, was never given the same gate. When
`enabled` is off, neither `log` (the external crate) nor `crate::log`/`crate::panic` (the
internal submodules) exist in any form, so `mod private`'s unconditional references fail to
resolve.

## Why Not Caught

No existing verification pass in this workspace exercises `browser_log` with
`--no-default-features` -- every documented Level 1-5 command (`will .test level::N`) and every
downstream consumer (`browser_tools`, the `minwebgl` examples) builds it with default features,
where `enabled` is always on and this code path is never reached.

## Fix Location

`module/helper/browser_log/src/lib.rs`.

```rust
// after -- mod private gated identically to the mod_interface! invocation it exists to serve
#[ cfg( feature = "enabled" ) ]
mod private
{
  // ...unchanged body...
}
```

`mod private` now carries the identical `#[cfg(feature = "enabled")]` already present on the
`mod_interface!` invocation and the `use ::mod_interface::mod_interface;` import 2 lines above
it -- when `enabled` is off, neither `mod private` nor the macro invocation that depends on it
exist, and the crate compiles to a fully empty shell, matching the design already implied by
every one of its dependencies being individually optional under this one feature.

## Prevention

Following the same-class precedent set by BUG-053 (`RUSTFLAGS`/`web_sys_unstable_apis`, also a
build-configuration defect rather than a runtime one): no unit test is added, because a single
compiled test binary runs under whatever feature flags `cargo test` itself was invoked with and
cannot exercise a second, different `cargo` invocation of the same crate from inside itself.
The durable regression check is the dual-configuration Verify Command documented above --
recorded here so a future verification pass can exercise both `enabled` directions, not just the
default-features direction every existing command already covers.

## Pitfall

When a feature gates a macro invocation that declares submodules (here, `mod_interface!`'s
`layer log;`/`layer panic;`), every *other* item referencing those submodules needs the
identical `#[cfg(...)]` -- gating the macro call alone doesn't retroactively gate unconditional
code elsewhere that merely depends on its expansion existing. This is easy to miss because the
gated macro invocation itself compiles fine in isolation; the break only surfaces in whichever
sibling code assumed the macro's expansion was always present.

## Generalized Version

**Broken assumption:** "gating the one macro invocation that declares a crate's submodules is
sufficient to make the whole feature togglable -- any code written before/after it that merely
references those submodules doesn't need its own gate, since the macro call is already gated."

**Confirmed general rule:** a `#[cfg(...)]` on a macro invocation only gates that invocation's
own expansion. Every unconditional item elsewhere in the same file (or crate) that references
symbols the expansion would have declared needs the identical `#[cfg(...)]`, checked
individually -- the gate does not propagate to dependents by association.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during the same background Explore review of `module/alias/browser_tools` (task #96) that surfaced BUG-167/168; confirmed via `cargo check -p browser_log --no-default-features` producing the predicted 4 `E0433` errors. |
| 2026-08-16 | fixed | Added `#[cfg(feature = "enabled")]` to `mod private`, matching the existing gate on the `mod_interface!` invocation it serves. |
| 2026-08-16 | verified | `cargo check -p browser_log --no-default-features` clean (native and `wasm32-unknown-unknown`); `cargo check`/`cargo clippy --no-default-features` clean; default and `full` feature configurations re-confirmed unaffected -- scoped native `cargo nextest`/`cargo clippy`/`cargo test --doc` clean across `browser_log` (9/9 tests, 10/10 doctests, all pre-existing BUG-167/168 coverage still green), plus `wasm32-unknown-unknown` compile-clean under both `--all-features` and `--no-default-features`. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass reasoned the failure from source before running anything; adversarial pass demanded the real compiler output rather than trusting the reasoning, running `cargo check --no-default-features` both pre-fix (4 `E0433`, matching prediction exactly) and post-fix (clean) before accepting the fix. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-167/168 (same review pass, same crate) and BUG-053 (same defect class, precedent for the no-unit-test Prevention approach) -- independent root causes, no coupling; recorded rather than left unstated. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading of both the referencing code (`mod private`) and the declaring code (`mod_interface!`'s `layer` lines), plus the real compiler's own error output confirming the exact mechanism. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Single-line `#[cfg(...)]` addition, the minimal fix that restores gate parity -- no broader refactor of the feature-gating structure attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `browser_log`'s `lib.rs` + this bug file touched; no unrelated crates modified. | — |
| D7 | Crate Locality | 🟢 | 🟢 | The single unconditional item causing the break (`mod private`) was found via full-file read and fixed at its own definition site; no other unconditional reference to `crate::log`/`crate::panic` exists elsewhere in the crate. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely a gate-parity correction; `mod private`'s own responsibility (holding `Config`/`setup`) is unchanged. | — |

**Reproduced:** YES -- pre-fix, `cargo check -p browser_log --no-default-features` failed with
exactly the 4 `E0433` errors predicted from reading the source. Post-fix, the identical command
exits 0, and `default`/`full` feature configurations (native and `wasm32-unknown-unknown`) are
confirmed unaffected. Scoped native `cargo nextest`/`cargo clippy`/`cargo test --doc` clean
across `browser_log`, 9/9 tests + 10/10 doctests passing, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_log/src/lib.rs` | `mod private` now carries `#[cfg(feature = "enabled")]` (full `Fix(BUG-169)` comment block), matching the pre-existing gate on the `mod_interface!` invocation and `use ::mod_interface::mod_interface;` import it serves. |

## Refs: tests/

None -- this is a build-configuration defect (see `## Prevention`); the regression check is the
documented dual-configuration Verify Command above, following the BUG-053 precedent for this
defect class.
