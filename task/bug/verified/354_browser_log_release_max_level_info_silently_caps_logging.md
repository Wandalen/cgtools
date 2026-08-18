# BUG-354: `browser_log`'s `release_max_level_info` Cargo feature silently caps `log::STATIC_MAX_LEVEL` at `Info` in every release build, discarding `debug!`/`trace!` calls workspace-wide

- **Severity:** High
- **state:** Verified
- **Affects:** every release-profile build (`cfg(not(debug_assertions))`, e.g. `cargo build
  --release`/`cargo test --release`) of `browser_log` itself, and of any application that
  depends on it — the cap applies to the ENTIRE dependency graph via Cargo feature
  unification, so a consuming binary's own unrelated `log::debug!`/`log::trace!` calls are
  silently discarded too, not just browser_log's internal logging
- **Component:** `module/helper/browser_log` (`Cargo.toml`'s `log` dependency feature list)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ (self)
- **verification_date:** 2026-08-18

## Symptom

`browser_log`'s own pre-existing, already-committed test (`debug_log_test.rs`, pinning
BUG-167/BUG-229) silently loses 2 of its 5 expected log records the moment the crate is built
in release profile — with zero compile warning and zero change to the test's own code:

```bash
# cargo test -p browser_log --release  (wrong — pre-fix Cargo.toml: log = { ..., features = [ "std", "release_max_level_info" ] })
test debug_log_methods_report_the_real_caller_location_and_module ... FAILED
assertion `left == right` failed: all 5 calls must have reached the logger: [(Info, ...), (Warn, ...), (Error, ...)]
  left: 3
 right: 5
# ^ the Trace-level and Debug-level calls (sample.debug_trace / sample.debug_log) vanished;
#   only Info/Warn/Error reached the logger

# cargo test -p browser_log --release  (correct — post-fix: "release_max_level_info" removed)
test debug_log_methods_report_the_real_caller_location_and_module ... ok
```

The direct cause, confirmed with a standalone probe printing `log::STATIC_MAX_LEVEL`:

```bash
# `log::STATIC_MAX_LEVEL` with browser_log's exact pre-fix feature list, `cargo run --release`  (wrong)
STATIC_MAX_LEVEL=Info

# same crate, dev profile (`cargo run`, no --release)  (correct, but only by accident of profile)
STATIC_MAX_LEVEL=Trace

# plain `log = "0.4.33"` with no feature flags, `cargo run --release`  (control — proves the cap
# is caused by the feature choice, not an inherent property of release builds in general)
STATIC_MAX_LEVEL=Trace
```

## Impact

**Who is affected:** every release-profile build (`cfg(not(debug_assertions))`) of
`browser_log` itself, and — because the `log` crate's `STATIC_MAX_LEVEL` constant is resolved
once per compiled dependency graph via Cargo feature unification, not per-crate — every
application that depends on `browser_log` and also calls `log::debug!()`/`log::trace!()`
anywhere in its OWN code, even code that has nothing to do with `browser_log`.

**What breaks:** silent, not loud. No compile warning, no runtime error, no panic — `log::debug!`
and `log::trace!` calls simply compile to a permanently-false branch
(`lvl <= STATIC_MAX_LEVEL` folds to `false` at the call site) and never reach any installed
`Log` implementation, regardless of what `browser_log::log::setup::setup`'s runtime `Config`/
`Level` requested. This directly contradicts the crate's own documented contract:
`Config::default()`/`Config::new()`'s doc comment ("Specify the maximum level you want to log",
`src/log/setup.rs:35`) and `readme.md`'s own claims — line 25 ("**Production Ready** -
Configurable log levels for deployment"), line 77 (`Config::new(Level::Debug)` as the
documented way to opt into debug-level output), line 104 (`log::debug!(...)` shown as normal,
expected usage). All of these promise a runtime-configurable level; the compile-time cap
silently overrides that promise for `Debug`/`Trace` in every release build.

**Magnitude:** every release build, unconditionally — this is not input-dependent or
intermittent. It is invisible unless a consumer specifically compares dev vs. release output
(as this report's Symptom section does), which nothing in this crate's own documented test
invocation (`tests/readme.md`: `cargo test -p browser_log --all-features`) or any other
workspace tooling seen so far actually does.

**Entity Scope:** `None` — source-level (Cargo manifest) generator defect, not entity
directory instances.

## How Discovered

Found by a prior investigation-stage agent during a workspace bug-hunt pass, reading
`browser_log/Cargo.toml`'s `log` dependency feature list and recognizing
`release_max_level_info` as a `log`-crate-native compile-time level cap, then confirming the
consequence via three independent methods: (1) building the real crate in debug vs. release and
observing `log::STATIC_MAX_LEVEL` change; (2) a control build with plain `log = "0.4.33"` (no
feature flags) in release, confirming `STATIC_MAX_LEVEL` stays `Trace` there — isolating the
cause to the feature choice, not release builds in general; (3) a generic capturing-logger
mirror confirming a `Level::Debug`-configured logger never receives a DEBUG-level message in
release profile.

Independently re-confirmed while filing this report, against the actual current source:

```bash
$ grep -n 'release_max_level_info' module/helper/browser_log/Cargo.toml
42:log = { workspace = true, optional = true, features = [ "std", "release_max_level_info" ] }

$ grep -n 'STATIC_MAX_LEVEL' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/log-0.4.33/src/lib.rs
1637:pub const STATIC_MAX_LEVEL: LevelFilter = match cfg!(debug_assertions) {
1641:    false if cfg!(feature = "release_max_level_info") => LevelFilter::Info,

$ cd module/helper/browser_log && cargo test -p browser_log --release --no-fail-fast
error: 2 targets failed:
    `-p browser_log --test debug_log_test`
    `-p browser_log --test static_max_level_test`
```
(`static_max_level_test.rs` is this report's own new reproducer, added and executed against the
still-unfixed `Cargo.toml` before any fix was applied — see MRE below.)

## Minimum Reproducible Example

**What:** the invariant violated is that `browser_log`'s only documented level control is its
own RUNTIME `Config`/`Level` mechanism (`log::setup::setup`) — no Cargo-feature-driven
COMPILE-TIME cap should be able to silently override a caller's runtime request for
`Debug`/`Trace` output, in browser_log's own logging or in any consumer's.

A synthetic crate depending only on plain upstream `log`, with the exact feature combination
`browser_log`'s `Cargo.toml` enables, reproduces the identical defect without needing this
repository at all — proving the defect lives entirely in that feature choice:

```bash
mkdir -p /tmp/mre354/src
cat > /tmp/mre354/Cargo.toml <<'EOF'
[package]
name = "mre354"
version = "0.1.0"
edition = "2021"

[dependencies]
log = { version = "0.4", features = [ "std", "release_max_level_info" ] }
EOF
cat > /tmp/mre354/src/main.rs <<'EOF'
fn main() {
    println!("STATIC_MAX_LEVEL={:?}", log::STATIC_MAX_LEVEL);
}
EOF
cd /tmp/mre354
cargo run --release 2>&1 | tail -1
cargo run 2>&1 | tail -1
```

**Expected:**
```
STATIC_MAX_LEVEL=Trace
STATIC_MAX_LEVEL=Trace
```

**Actual** (directly observed, both lines from the script above, in order):
```
STATIC_MAX_LEVEL=Info
STATIC_MAX_LEVEL=Trace
```

**Verify Command** (the permanent, repo-local regression test this bug's fix adds):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log
cargo test -p browser_log --release --no-fail-fast
```
**Expected** (fixed): all targets pass, including `static_max_level_test`'s 2 tests and
`debug_log_test`'s 1 test (`test result: ok` for every target, `-p browser_log` build succeeds
with no `error: N targets failed`).

**Actual** (pre-fix, directly observed):
```
test debug_records_reach_the_logger_at_current_build_profile ... FAILED
test static_max_level_is_not_capped_in_release_profile ... FAILED
...
test debug_log_methods_report_the_real_caller_location_and_module ... FAILED
error: 2 targets failed:
    `-p browser_log --test debug_log_test`
    `-p browser_log --test static_max_level_test`
```

A plain `cargo test -p browser_log` (no `--release`) cannot observe this symptom in either fix
state: `cargo test`'s default `test` profile inherits `dev` profile settings
(`debug_assertions = true`), and this workspace's root `Cargo.toml` carries no `[profile.*]`
override (confirmed by reading it) — `release_max_level_info`'s own `cfg(not(debug_assertions))`
gate simply never applies there. This is why the Verify Command above requires `--release`; see
`tests/static_max_level_test.rs`'s own module doc for the same caveat spelled out in-repo.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Cargo.toml`'s `release_max_level_info` feature on the `log` dependency sets `log::STATIC_MAX_LEVEL` to `Info` at compile time in release builds | ✅ Root Cause | `browser_log/Cargo.toml:42` (pre-fix) enables the feature; `log-0.4.33/src/lib.rs:1637-1650` shows it is read via `match cfg!(debug_assertions) { false if cfg!(feature = "release_max_level_info") => LevelFilter::Info, ... }` | E1, E2, E3 |
| H2 | `log::debug!`/`log::trace!` gate on `STATIC_MAX_LEVEL` before the runtime level is ever consulted, so `browser_log`'s runtime `Config`/`set_max_level` mechanism cannot override the cap | ✅ Root Cause | `log-0.4.33/src/macros.rs:135-146`: `if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level()` — the left operand alone can make the whole condition permanently false regardless of the right operand (the runtime level) | E4, E5, E6 |
| H3 | The cap only manifests in `cfg(not(debug_assertions))` builds (release profile by default); dev/test-profile builds are unaffected regardless of the feature list | ✅ Verified | `log-0.4.33/src/lib.rs:1637`'s `match cfg!(debug_assertions)` guards every `release_max_level_*` arm behind `false` (i.e. `not(debug_assertions)`) | E3, E7 |
| H4 | Release-profile builds are inherently capped at `Info`, independent of any specific Cargo feature choice | ❌ Disproved | A control crate with plain `log = "0.4.33"` (no feature flags) built `--release` reports `STATIC_MAX_LEVEL=Trace`, not `Info` — the cap is caused specifically by the `release_max_level_info` feature, not release profile in general | E8 |
| H5 | No existing test in `browser_log` exercised a release-profile build or asserted on `log::STATIC_MAX_LEVEL` before this bug was found | ✅ Verified | `find module/helper/browser_log/tests -name '*.rs'` (pre-fix) listed only `basic_test.rs`, `debug_log_test.rs`, `panic_hook_test.rs` — none reference `STATIC_MAX_LEVEL` or run under `--release`; `tests/readme.md`'s documented invocation (`cargo test -p browser_log --all-features`) never passes `--release` | E9 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_log/Cargo.toml:42` (pre-fix) | `log = { workspace = true, optional = true, features = [ "std", "release_max_level_info" ] }` — the feature is enabled | H1 ✅ |
| E2 | `log-0.4.33/src/lib.rs:1637-1650` | `pub const STATIC_MAX_LEVEL` selects `LevelFilter::Info` via `false if cfg!(feature = "release_max_level_info")` (the `false` arm matches `cfg!(debug_assertions) == false`) | H1 ✅ |
| E3 | Terminal output (this report, MRE section) | Real crate, exact pre-fix feature list: `cargo run --release` → `STATIC_MAX_LEVEL=Info`; `cargo run` (dev) → `STATIC_MAX_LEVEL=Trace` | H1 ✅, H3 ✅ |
| E4 | `log-0.4.33/src/macros.rs:135-146` (the `__log!` arm `debug!`/`trace!` expand through) | `if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level()` — `STATIC_MAX_LEVEL` is checked first; a capped constant short-circuits the whole condition to `false` regardless of the runtime `max_level()` | H2 ✅ |
| E5 | `module/helper/browser_log/src/log/setup.rs:179-192` | `pub fn setup(config: Config) { ... log::set_max_level(max_level.to_level_filter()) }` — confirms the crate's ONLY level-control mechanism is this runtime call, which E4 shows cannot override a compile-time `STATIC_MAX_LEVEL` cap | H2 ✅ (symptom) |
| E6 | Terminal output (this report, Symptom section) | `debug_log_test.rs`'s pre-existing test (calls `sample.debug_trace(...)`/`sample.debug_log(Level::Debug, ...)`, which resolve to `log::trace!`/`log::debug!`) captures only 3 of 5 records under `--release` pre-fix — the missing 2 are exactly the Trace- and Debug-level calls | H2 ✅ (symptom) |
| E7 | Terminal output (this report, MRE section) | `cargo test -p browser_log` (no `--release`, dev profile) passes cleanly in both pre-fix and post-fix `Cargo.toml` states — the defect is invisible outside release profile | H3 ✅ |
| E8 | Terminal output (this report, Symptom section) | Control crate, plain `log = "0.4.33"`, `cargo run --release` → `STATIC_MAX_LEVEL=Trace` (not capped) | H4 ❌ |
| E9 | `module/helper/browser_log/tests/readme.md:3` (pre-fix) + directory listing | Documented test invocation is `cargo test -p browser_log --all-features` (no `--release`); pre-fix directory listing showed only `basic_test.rs`, `debug_log_test.rs`, `panic_hook_test.rs` — no file referenced `STATIC_MAX_LEVEL` | H5 ✅ |

## Root Cause

```
log = { workspace = true, optional = true, features = [ "std", "release_max_level_info" ] }
                                                                 |
                                                                 +-- this feature is native to
                                                                     the `log` crate itself, not
                                                                     browser_log's own code

  log-0.4.33/src/lib.rs:1637-1650:
    pub const STATIC_MAX_LEVEL: LevelFilter = match cfg!(debug_assertions) {
        false if cfg!(feature = "release_max_level_info") => LevelFilter::Info,   <-- selected
        ...                                                                            here, in
        _ => LevelFilter::Trace,                                                       release
    };                                                                                 builds

  log-0.4.33/src/macros.rs:135-146 (debug!/trace! expand through __log!):
    if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() { ... }
           |
           +-- lvl = Level::Debug or Level::Trace (compile-time constant from the call site)
               STATIC_MAX_LEVEL = Info (from above, in release)
               Debug <= Info is false, Trace <= Info is false
               => the `if` body (the actual log dispatch) never executes, regardless of
                  max_level() (the runtime value browser_log::log::setup::setup sets)
```
H1 (`STATIC_MAX_LEVEL` capped by the feature) and H2 (`debug!`/`trace!` check that constant
before the runtime level) together are the compound root cause: H1 alone would be harmless if
the macros consulted the runtime level first, and H2 alone is inert without a capped constant
to short-circuit on. Combined, they explain every observed symptom: the constant being `Info`
in release (E2, E3) and the runtime-configured logger never receiving Debug/Trace records
despite requesting `Trace` (E4, E5, E6).

## Why Not Caught

No test in `browser_log/tests/` ever ran in release profile before this bug was found (H5,
confirmed by E9): the crate's own documented test invocation
(`tests/readme.md`: `cargo test -p browser_log --all-features`) never passes `--release`, and
nothing else in the workspace's tooling seen so far runs this crate's tests under `--release`
either. `debug_log_test.rs` DOES exercise `log::debug!`/`log::trace!` delivery end-to-end
(pinning BUG-167/BUG-229) and would have caught this the moment it ran in release profile — as
demonstrated directly in this report's Symptom section — but it had never been run that way.
This is a missing release-profile test axis, not a missing test case: the assertion coverage
already existed, it simply never executed under the one build configuration where
`STATIC_MAX_LEVEL` actually differs from its dev-profile value.

## Fix Location

**`module/helper/browser_log/Cargo.toml:42`** (pre-fix), now **`:58`** (before/after, after the
added 16-line fix-explanation comment block):

```toml
# Before:
log = { workspace = true, optional = true, features = [ "std", "release_max_level_info" ] }

# After:
log = { workspace = true, optional = true, features = [ "std" ] }
```
A `# Fix(BUG-354): ... Root cause: ... Pitfall: ...` comment block (the TOML-comment equivalent
of this repo's usual 3-field source comment, following the precedent already set by the
`# Fix(BUG-079): ...` block on the `test_tools`/`getrandom` dependency a few lines below) was
added immediately above the fixed line.

`module/helper/browser_log/tests/static_max_level_test.rs` (new): a permanent regression test
with two functions — one always-on (exercises the runtime logging path in every profile; only
its `debug_assertions = false` branch actually distinguishes pre-fix from post-fix) and one
`cfg(not(debug_assertions))`-gated (asserts directly on `log::STATIC_MAX_LEVEL`, compiled in
only under release profile). Both fail pre-fix and pass post-fix under
`cargo test -p browser_log --release`; both are inert (pass trivially, or don't compile in at
all) under plain `cargo test -p browser_log`, by design — see the file's own module doc.

## Prevention

Add a release-profile test run to whatever this crate's CI or verification tooling eventually
runs by default — `cargo test -p browser_log --release` — since this bug class (a compile-time
cap that depends on `cfg(debug_assertions)`) is invisible to any tooling that only ever builds
in dev/test profile. Detection command for the specific `log`-crate feature family that causes
this class of defect, across the whole workspace (any crate opting a `log`/`tracing`-family
dependency into a compile-time release cap):
```bash
grep -rn 'release_max_level_' --include=Cargo.toml .
```
Run against this fix's own final state, this correctly returns no matches for `browser_log`
(the only workspace crate found using this feature at filing time) — confirmed by direct
execution.

**Pitfall:** a Cargo feature enabled on a **library** crate's own dependency is not scoped to
that library — Cargo's feature unification means every crate in the same build that also
depends on the same underlying crate (here, `log`) gets the union of every enabled feature,
including ones a library's own `Cargo.toml` silently opts in on the consuming binary's behalf.
`release_max_level_*`/`max_level_*` looks like an innocuous "reduce log noise in production"
default, but a library should never make that call for its consumers — only a top-level binary
crate, which controls its own build profile and knows its own logging needs, is in a position
to decide whether capping `STATIC_MAX_LEVEL` is desired.

## Generalized Version

**Broken assumption:** enabling a `log`-crate `max_level_*`/`release_max_level_*` Cargo feature
on a library's own dependency only affects that library's internal logging.

Fails whenever:
1. A **library** crate (not a top-level binary) enables `max_level_*` or `release_max_level_*`
   on its `log` (or any crate using the same `STATIC_MAX_LEVEL`-style compile-time cap
   convention) dependency, AND
2. Any other crate in the same compiled dependency graph also depends on that same crate and
   calls its logging macros at a level the cap excludes — Cargo feature unification means that
   other crate's calls are silently capped too, even though its own `Cargo.toml` never opted in

**Detection invariant:**
```
for every library crate's Cargo.toml (not a `[[bin]]`-only crate):
  no `log` (or equivalent) dependency may enable a `max_level_*`/`release_max_level_*` feature
  — such a cap belongs only in a top-level binary's own Cargo.toml, if desired there at all
```
Single confirmed instance in this workspace: `grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .` (run from the workspace root, post-fix) returns no matches anywhere — `browser_log` was the only crate using this feature family, and this fix removed its only occurrence.
Not a duplicate of any prior bug in this repo's `task/bug/` history (dedup search:
`grep -rli "static_max_level\|release_max_level\|log::debug\|STATIC_MAX_LEVEL" task/bug/` found
no prior hits before this filing).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found by a prior investigation-stage agent during a workspace bug-hunt pass; re-confirmed independently while filing via a standalone `/tmp/mre354` crate and the real crate's own release-profile test run |
| 2026-08-18 | fix_applied | Removed `release_max_level_info` from the `log` dependency's `features` list, `module/helper/browser_log/Cargo.toml:42` (pre-fix line) |
| 2026-08-18 | verified | VERIFY Gate (Tier 2 dual-role self-check, 8/8 dimensions 🟢): `cargo nextest run -p browser_log --all-features` (dev, 10/10) and `--release` (11/11, including the release-only `static_max_level_is_not_capped_in_release_profile`) both re-run fresh and pass; workspace-wide `grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .` confirms the feature is genuinely gone everywhere. |
| 2026-08-18 | VERIFY Gate | Reproducer suite `cargo test -p browser_log --release --no-fail-fast` confirmed all passing (basic_test 2 passed, debug_log_test 1 passed, panic_hook_test 6 passed, static_max_level_test 2 passed, 10 doc-tests passed; 0 failed across all targets); fix confirmed present at `module/helper/browser_log/Cargo.toml:58` (`log = { workspace = true, optional = true, features = [ "std" ] }` -- `release_max_level_info` absent). state: Unverified -> Verified |

## Refs: tests/

- `tests/static_max_level_test.rs` — new permanent regression test: asserts `log::debug!()` reaches an installed logger and that `log::STATIC_MAX_LEVEL` is never capped below `Trace` under a release-profile build

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

*Note: this file's `## History` shows two independent same-day verification passes (this gate's own `verified` row, and a differently-worded `VERIFY Gate` row from a concurrent actor also operating on this bug tracker) -- both report a clean PASS on the same underlying fix; neither is retracted, per this repo's append-only History convention.*

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All 12 sections + header fields present; `**state:**` read `Unverified` despite the file already sitting in `verified/` by directory path -- pre-existing filing inconsistency this gate resolves; `## Refs: src/` and `## Refs: docs/` correctly omitted per FI009 (fix is Cargo.toml-only, no `src/` or `docs/` files touched) | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Verify Command re-run fresh: `cargo nextest run -p browser_log --all-features` (dev, 10/10 pass) AND `--release` (11/11 pass, including the release-only `static_max_level_is_not_capped_in_release_profile` and the release-branch of `debug_records_reach_the_logger_at_current_build_profile`) -- the ONLY profile that can distinguish pre-/post-fix per this bug's own documented constraint, confirmed by direct read of the workspace root `Cargo.toml`'s absence of any `[profile.*]` override | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 5 Hypothesis rows, 2 ✅ Root Cause (H1, H2); H1↔{E1,E2,E3}, H2↔{E4,E5,E6}, H3↔{E3,E7}, H4↔{E8}, H5↔{E9} bidirectional, re-checked both directions; `grep -rln "BUG-354"` confirms all cross-reference files (`Cargo.toml`, `static_max_level_test.rs`, `tests/readme.md`'s test responsibility table, `task/bug/readme.md`, this bug file) carry a matching reference | — |
| D4 | Root Cause Quality | — | 🟢 | Root Cause traces to H1+H2 both ✅; Fix Location (`Cargo.toml:42`, pre-fix line) independently re-verified: active `log` dependency line now reads `features = [ "std" ]`, `release_max_level_info` genuinely absent (not commented out); Generalized Version's detection invariant (`grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .`) independently re-run -- 0 matches outside this bug's own Fix-comment prose | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; Fix Location resolves inside `$SCOPE_DIR` (`module/helper/browser_log/Cargo.toml`) | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` = `module/helper/browser_log`; Fix Location resolves to that same crate's own manifest | — |
| D7 | Crate Locality | — | 🟢 | Fix lands in the owning library crate's own manifest (removing its own opt-in), not a workspace-root or consumer-side workaround -- matches the Generalized Version's own detection invariant (no library crate in the workspace may enable `max_level_*`/`release_max_level_*`) | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix only removes one Cargo feature flag; adds no new public surface, no expansion of the crate's responsibility | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — exit 0 both profiles (`cargo nextest run -p browser_log --all-features`: 10/10
dev-profile; `--release`: 11/11 release-profile, including `static_max_level_is_not_capped_in_release_profile`
and `debug_records_reach_the_logger_at_current_build_profile`'s release-profile branch), 2026-08-18.
Adversarial pass: independently re-ran `grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .`
from the workspace root -- 0 matches outside this bug's own Fix-comment prose, confirming the feature is
genuinely removed workspace-wide, not merely commented out or relocated; independently confirmed the
workspace root `Cargo.toml` carries no `[profile.*]` override (so `cargo test`/`nextest run` without
`--release` structurally cannot exercise this defect, matching the bug's own documented constraint), and
that `browser_log::log::setup::setup`'s runtime `log::set_max_level(...)` call (`src/log/setup.rs:188`) is
the crate's only level-control mechanism, independently re-read.
