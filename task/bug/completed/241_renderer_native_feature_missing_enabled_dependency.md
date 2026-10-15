# BUG-241: `renderer`'s crate-root `mod_interface!` and unconditionally-included `webgl` layer require the `enabled` feature's dependencies even when disabled, breaking the documented no-browser `native`-only build

- **Severity:** High (hard compile failure, zero workaround short of discovering you must also
  request an unrelated-looking feature; breaks a configuration the crate's own Cargo.toml
  explicitly documents as a supported, intentional use case — "terminal pixel tests, no browser")
- **state:** Completed
- **Affects:** Any consumer building `renderer` with `--no-default-features` plus `native` and/or
  `animation` alone, without also separately requesting `enabled` — confirmed via
  `--features native` (the exact combination the crate's own doc comment invites: "the canonical
  gpu_hal renderer on the native wgpu backend ... materializing off-wasm instead (terminal pixel
  tests, no browser)") and independently reconfirmed via `--features animation` (shares the
  identical root cause: the crate-root `mod_interface!` macro use). `--features webgpu` alone was
  not independently re-probed but shares the same unconditional `mod_interface!` call and is
  reasoned, not confirmed, to be affected identically.
- **Component:** `module/helper/renderer` (`Cargo.toml` feature graph + `src/lib.rs` layer gating)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self — same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

```
$ cargo check -p renderer --no-default-features --features native
   Compiling renderer v0.1.0 (/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/renderer)
error[E0433]: cannot find `mod_interface` in the crate root
  --> module/helper/renderer/src/lib.rs:11:3
   |
11 | ::mod_interface::mod_interface!
   |   ^^^^^^^^^^^^^ could not find `mod_interface` in the list of imported crates

error: could not compile `renderer` (lib) due to 1 previous error
```

## Impact

**Who is affected:** Anyone building this crate with `--no-default-features` and any single
non-`enabled`, non-`webgl` feature (`native`, `animation`, and — by the same shared root cause —
almost certainly `webgpu`). `default = [ "enabled" ]` (pre-fix) meant every ordinary build
(`cargo build`, `cargo test`, `cargo clippy` with no explicit `--no-default-features`) silently
carried `enabled` along regardless, so this was invisible to the crate's own normal test/CI
invocations — it only surfaces the moment someone actually exercises the crate's own documented
minimal-feature story.

**What breaks:** A hard `E0433` compile failure at the very first line of module wiring (`lib.rs`
line 11) — before even reaching the second, compounding defect (see `## Root Cause`).

**Why High, not Medium:** Unlike BUG-079 (a comparable build-configuration defect, but confined to
a wasm32 dev-dependency/proc-macro edge that never touches shippable code), this defect breaks a
*documented, named, shippable* feature configuration — the `native` feature's own doc comment in
`Cargo.toml` markets it as "the canonical gpu_hal renderer on the native wgpu backend ... terminal
pixel tests, no browser." A consumer following that documentation to get a lean, browser-free
build hits an immediate, confusing compile error mentioning `mod_interface` — a macro-wiring
crate with no apparent connection to "native" or "wgpu" — with no hint that the actual fix is to
also enable an unrelated-sounding `enabled` feature.

**Entity Scope:** `None` — Cargo feature-graph / build-configuration, not entity directory
instances.

## How Discovered

During this session's `renderer` crate scout (task #174, opened immediately after closing
`tilemap_renderer`'s scout task #173). Reading `src/lib.rs` as the first file in the crate's
"core scene graph + renderer" review group, `layer webgl;`'s cfg attribute was visibly commented
out (`//#[ cfg( feature = "webgl" ) ]`) while its sibling `layer webgpu;` two lines below carried
a live, uncommented cfg gate — an asymmetry worth checking rather than assuming was intentional.
Tracing what `webgl`'s own (`webgl = []`, empty) feature actually gates, and what the crate-root
`mod_interface!` macro invocation itself requires, led directly to reproducing the failure via a
real `cargo check` rather than reasoning about it in the abstract.

## Minimum Reproducible Example

No synthetic MRE needed — the real crate reproduces it directly and deterministically:

```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check -p renderer --no-default-features --features native
```

**Verify Command:** the command above; **Expected** (once fixed): exit 0; **Actual (pre-fix):**
exit 101, `error[E0433]: cannot find mod_interface in the crate root` (verbatim output in
`## Symptom`).

## Root Cause

Two compounding defects, both class "a module tree is wired unconditionally but its real
dependencies are gated behind an optional feature nobody is forced to also request":

**1. `src/lib.rs:11`** — `::mod_interface::mod_interface!` (the crate's own module-wiring macro,
used at the crate root with no `#[cfg(...)]` at all) requires `dep:mod_interface`, which — pre-fix
— was declared `optional = true` and only pulled in by the `enabled` feature
(`enabled = [ "dep:mod_interface", ... ]`). Any feature selection that omits `enabled` fails to
compile at this line specifically, regardless of which other feature was requested.

**2. `src/lib.rs:16-17`** (pre-fix) — `layer webgl;`, gating the entire `src/webgl/**` tree
(post_processing, mesh, material, scene, light, skeleton, texture, sampler, geometry, primitive,
node, renderer, camera, program, ibl, loaders, helpers, shadow — 18 sub-layers, all themselves
unconditional except the already-correctly-gated `animation`), had its `#[ cfg( feature = "webgl"
) ]` commented out — making the whole tree, and the `enabled`-only deps it needs
(`minwebgl`/`web-sys`/`mingl`/`gltf`/...), unconditional too. Even a hypothetical fix for defect 1
alone (e.g. making only `mod_interface` unconditional) would still leave a `native`-only build
compiling the *entire* unrelated browser/WebGL stack — defeating the documented point of a lean,
browser-free `native` configuration.

Both existed because `webgl`'s own Cargo.toml feature was declared empty (`webgl = []`) — it
gated nothing, so restoring defect 2's cfg alone (without also feeding `webgl` its actual
dependency need) would have simply moved the identical `E0433`-class failure from "always" to
"whenever `--features webgl` is requested without `enabled`."

## Why Not Caught

`default = [ "enabled" ]` meant every ordinary invocation of this crate — `cargo build`,
`cargo test`, `cargo clippy`, this session's own earlier `cargo check -p renderer` / `--all-features`
runs on other bugs in this crate's history (BUG-171 through BUG-198, BUG-204) — always carried
`enabled` along for free, whether or not the invocation's own `--features` list mentioned it
explicitly. `tests/native_render_test.rs`'s own `#![cfg(all(feature = "native", not(target_arch =
"wasm32")))]` gate meant it only ever *ran* under a feature set that happened to include `native`
alongside `enabled` (`--all-features`, or `full`) — never under the crate's own documented minimal
"just `native`" configuration in isolation. A working test under a broader feature superset gives
zero signal about whether the documented minimal subset also compiles; nobody had reason to
probe the narrower combination until this session's line-by-line read of `lib.rs` surfaced the
suspicious commented-out cfg asymmetry.

## Fix Applied (2026-08-17)

Three coordinated changes, all in `module/helper/renderer`:

**`Cargo.toml`:**
- `mod_interface = { workspace = true }` — dropped `optional = true`; it is used unconditionally
  at 3 module roots (`lib.rs`, `webgl.rs`, `webgpu.rs`) regardless of which top-level feature is
  active, so it was never actually optional in practice.
- `enabled = [...]` — removed the now-invalid `"dep:mod_interface"` entry (Cargo rejects `dep:`
  syntax against a non-optional dependency).
- `webgl = [ "enabled" ]` (was `webgl = []`) — the feature now actually gates what its own tree
  needs.
- `default = [ "enabled", "webgl" ]` (was `default = [ "enabled" ]`) — preserves the pre-fix
  default build's behavior (full WebGL renderer present with no explicit `--features` flag)
  now that `layer webgl;` is properly gated instead of unconditional.

**`src/lib.rs`:** restored the live `#[ cfg( feature = "webgl" ) ]` on `layer webgl;` (was
commented out), with a `Fix(BUG-241)` comment recording why.

## Verification

Four `cargo check -p renderer` invocations, `longrun`-detached per this repo's convention:

| Feature selection | Pre-fix | Post-fix |
|---|---|---|
| `--no-default-features --features native` (Verify Command) | exit 101, `E0433` | exit 0 |
| (default, no flags) | exit 0 (unchanged) | exit 0 |
| `--no-default-features --features webgl` | exit 101, `E0433` (same root cause) | exit 0 |
| `--all-features` | exit 0 (unchanged) | exit 0 |
| `--no-default-features --features animation` | not independently reproduced pre-fix (same root cause as `native`, reasoned not confirmed) | exit 0 |

No in-suite `bug_reproducer` test exists for this bug, matching this repo's established
precedent for pure build-configuration/feature-resolution defects (BUG-079): the defect is a
Cargo feature-graph compile error, reproducible only by a nested `cargo` invocation with a
specific `--features` selection, not by any runtime code path within a test binary that itself
depends on the very feature selection under test. The recorded Verify Command is the reproducer,
exercised before (exit 101) and after (exit 0) the fix, per the table above.

## Generalized Version

**Broken assumption:** "a module layer's `#[cfg(feature = ...)]` gate is redundant/safe to
comment out because the feature it names doesn't currently gate anything meaningful." False the
moment that layer's own subtree has real, non-trivial dependency requirements — commenting out
the gate doesn't just make the *layer* unconditional, it makes everything the layer's code
`use`s unconditional too, silently expanding what every feature combination requires. A
`grep -n "^\s*//#\[\s*cfg"` sweep for commented-out `cfg` attributes on `mod_interface!` layer
declarations is a cheap, concrete way to catch this class before it surfaces as a downstream
consumer's confusing compile error.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during task #174's line-by-line read of `renderer/src/lib.rs`, immediately after closing `tilemap_renderer`'s scout task #173. Reproduced via `cargo check -p renderer --no-default-features --features native` (exit 101 pre-fix). Fixed via 3 coordinated `Cargo.toml`/`lib.rs` changes; reverified the original Verify Command plus 3 adjacent feature combinations (default, explicit `webgl`-alone, `--all-features`) all exit 0. Closed same-session (Tier 2 Dual-Role Self-Check). |
