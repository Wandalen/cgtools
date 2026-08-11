# BUG-079: `getrandom` resolves to two incompatible major versions on `wasm32-unknown-unknown`, breaking every `--all-targets` build that pulls in `test_tools` or the `derive_tools`→`strum`→`phf` proc-macro chain

- **Severity:** Medium
- **state:** 📝 (Draft)
- **Affects:** every crate whose `wasm32-unknown-unknown` `--all-targets` build reaches `test_tools` (dev-dependency) or `derive_tools`/`strum`/`phf_generator` (proc-macro chain) — confirmed on `module/min/mingl`, `module/min/minwebgpu`; both dependency paths are workspace-wide (`test_tools` is a near-universal dev-dependency), so the practical blast radius is most of `module/`
- **Component:** workspace root `Cargo.lock` dependency resolution + `.cargo/config.toml`
- **repo_identity:** self
- **Filed:** 2026-08-11
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** null
- **verification_date:** null
- **Fixed:** null
- **Accepted By:** null

## Symptom

```bash
$ cargo clippy -p mingl --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings
    Checking getrandom v0.2.17
error: the wasm*-unknown-unknown targets are not supported by default, you may need to enable the "js" feature. For more information see: https://docs.rs/getrandom/#webassembly-support
   --> /home/user1/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/getrandom-0.2.17/src/lib.rs:346:9
    |
346 | /         compile_error!("the wasm*-unknown-unknown targets are not supported by \
347 | |                         default, you may need to enable the \"js\" feature. \
348 | |                         For more information see: \
349 | |                         https://docs.rs/getrandom/#webassembly-support");
    | |________________________________________________________________________^

error: could not compile `getrandom` (lib) due to 1 previous error
```

Reproduced fresh this session (2026-08-11); `Cargo.lock` has zero diff for the entire session
(`git diff --stat -- Cargo.lock` produces no output), so this is not a regression from any
change made this session — it is a pre-existing, latent gap that this session's `--all-targets`
wasm32 clippy checks are the first thing to have actually exercised.

## Impact

**Who is affected:** Any `--all-targets` (or otherwise dev-dependency-inclusive) build/clippy/test
invocation for `wasm32-unknown-unknown` on a crate that reaches either of two dependency paths
(`## Root Cause`). `test_tools` is used as a dev-dependency by most crates under `module/`, so
this is a workspace-wide gap in the wasm32 test/lint story, not a one-crate defect.

**What breaks:** A hard compile failure of the `getrandom` crate itself — not a lint, not a
warning, a `compile_error!` — the instant `--all-targets` (or any other invocation pulling in a
dev-dependency closure that reaches `rand 0.8`) is used for `wasm32-unknown-unknown`.
`--lib`-only invocations for the same crate/target are unaffected (confirmed: `mingl`, `minwebgpu`
both clippy-clean on wasm32 `--lib`), since `--lib` excludes the `test_tools` dev-dependency edge
and most (not all) proc-macro-chain consumers are still reachable through `--lib` — the specific
trigger is which edge of the dependency graph gets compiled, not the target itself.

**Why Medium, not High:** No production/runtime behavior is affected — this only blocks
`--all-targets` wasm32 invocations, and every crate checked this session has a working `--lib`-only
wasm32 fallback that covers 100% of shippable (non-test) source. Not Low, because this workspace
has dedicated wasm32 test infrastructure (`.cargo/config.toml`'s `[target.wasm32-unknown-unknown]
runner = "script/wasm_test_runner.sh"`) that this defect makes entirely unusable for any crate
matching the trigger — a deliberately-built capability is silently non-functional, not merely
untested.

**Entity Scope:** `None` — dependency resolution and build configuration, not entity directory
instances.

## How Discovered

While independently verifying a background agent's wasm32 clippy fix for `mingl` this session,
scoping the check to `--lib` (to isolate the crate's shippable surface from unrelated
dev-dependency issues) was necessary to get a clean run. Investigating *why* `--all-targets`
failed on the same crate/target combination, when `--lib` didn't, led to `cargo tree -i getrandom`
and the version split below. Independence from the current clippy-cleanup work was confirmed by
reproducing the identical failure on `minwebgpu` (an unrelated crate, also `--lib`-clean /
`--all-targets`-broken on wasm32) and by the zero-diff `Cargo.lock` check above.

## Minimum Reproducible Example

No synthetic MRE needed — the real workspace reproduces it directly and deterministically:

```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo clippy -p mingl --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings
```

**Verify Command:** the command above; **Expected** (once fixed): exit 0; **Actual:** exit 101,
`error: could not compile getrandom (lib) due to 1 previous error` (verbatim output in `## Symptom`).

## Root Cause

`Cargo.lock` resolves **three simultaneous major versions** of `getrandom` workspace-wide:

```
$ grep -n '^name = "getrandom"' -A2 Cargo.lock
name = "getrandom"   version = "0.2.17"
name = "getrandom"   version = "0.3.4"
name = "getrandom"   version = "0.4.3"
```

`.cargo/config.toml`'s `[target.wasm32-unknown-unknown]` sets
`--cfg getrandom_backend="wasm_js"` — this is `getrandom` 0.3/0.4's **cfg-based** backend-selection
mechanism (a deliberate, intentional choice, matching the newer versions' documented API). But
`getrandom 0.2.17` predates that mechanism entirely; it only understands the older **Cargo
feature**-based selection (`features = ["js"]`), so the cfg flag is invisible to it and it falls
straight through to its own unconditional `compile_error!` for any `wasm32-*-unknown` target.

Two independent dependency paths pull the old `0.2.17` in, confirmed via `cargo tree -i getrandom
--target wasm32-unknown-unknown`:

```
getrandom v0.2.17
└── rand_core v0.6.4
    ├── rand v0.8.7
    │   └── phf_generator v0.10.0 → phf_macros → phf v0.10.1 → strum v0.25.0 → derive_tools v0.36.0
    │       └── (derive_tools is a direct dependency of mingl AND ndarray_cg)
    └── rand_chacha v0.3.1 → rand v0.8.7 (*)

getrandom v0.2.17
└── rand_core v0.6.4 → rand v0.8.7 → test_tools v0.16.0 [dev-dependencies] → mingl
```

Meanwhile `getrandom 0.3.4` is pulled in separately, on the very same target, by an unrelated
chain (`ahash → hashbrown → hecs`/`wfc → tiles_tools`/`wfc_example` etc.) — confirming the
workspace's `getrandom_backend="wasm_js"` rustflag is correctly consumed by *that* path; it is
specifically the `rand 0.8.7`-rooted 0.2.x resolution that is stranded.

Both trigger paths are proc-macro/dev-dependency edges, not runtime/shippable-code edges — which
is exactly why `--lib`-only invocations avoid them (dev-dependencies are excluded entirely; and
where `--lib` still reaches `derive_tools`, apparently the proc-macro side of that dependency is
resolved and executed on the **host** target during macro expansion, not the crate's own compile
target, so it never hits this wasm32-target check at all — only when `--all-targets` additionally
compiles wasm32 **test binaries** that link against `test_tools` does the wasm32-target getrandom
resolution actually get exercised).

## Why Not Caught

`ndarray_cg`'s own pre-existing clippy backlog (fixed earlier this session) transitively blocked
full `--all-features` compilation for `mingl`, `minwebgpu`, `minwebgl`, and other `math`-feature
consumers for the entire session up to this point — so no `--all-targets` wasm32 build of any of
these crates had ever successfully reached the actual dev-dependency/proc-macro compilation step
before now. This is the same "domino effect" pattern that also surfaced fresh, previously-
unreachable clippy backlogs in `minwebgl` and `minwebgpu` this session: fixing one blocking crate
makes a previously-invisible downstream gap visible for the first time, rather than introducing
a new one.

**Pitfall:** a crate/target combination that has never successfully completed a full
`--all-targets` compilation gives zero signal about whether it *can* — a transitively-blocked
build and a genuinely clean one produce an identical "never tested" absence of evidence. Treat
"first time this has ever compiled that far" as a specific trigger to re-check breadth
(`tsk/longrun.rulebook.md § Long-Run Execution : Breadth Selection`), not just breadth-appropriate
detachment.

## Fix Location (not yet applied — options, not a decision)

The fix is a workspace-wide dependency-resolution change, outside a per-crate clippy-cleanup's
blast radius; recorded here rather than applied unilaterally. Candidate approaches, roughly
increasing in invasiveness:

1. **Pin `getrandom`'s Cargo-feature explicitly for the old path:** add `getrandom = { version =
   "0.2", features = ["js"] }` as an explicit workspace dependency. Cargo unifies same-major-version
   resolutions, so this can make the existing `0.2.17` resolution pick up the `js` feature workspace-
   wide — cheapest option if it doesn't conflict with anything already pinning `getrandom` 0.2
   without that feature.
2. **Upgrade the proc-macro chain:** `phf`/`phf_macros`/`phf_generator`/`strum`/`derive_tools` to
   versions that depend on `rand 0.9`+ (→ `getrandom 0.3`), if such versions exist and are
   otherwise compatible — removes the 0.2 resolution at the source instead of accommodating it.
3. **Drop or replace `test_tools`'s `rand 0.8` dependency** for the dev-dependency edge
   specifically, if `test_tools` doesn't actually need `rand` for wasm32 test builds.

Any of these needs a full-workspace `--all-targets` wasm32 re-verification before being considered
safe (this workspace has ~100+ crates; a `getrandom`/`rand` version bump has wide reach) — that
verification cost is why this bug is filed rather than fixed inline during a clippy-cleanup pass.

## Generalized Version

**Broken assumption:** "a `--cfg` flag set in `.cargo/config.toml` for a target applies uniformly
to every crate resolved for that target." False whenever a dependency graph resolves multiple
major versions of the same crate side-by-side and the flag's meaning is version-specific (here:
`getrandom_backend="wasm_js"` is meaningful only to `getrandom` ≥0.3). A `cargo tree -i <crate>`
check for duplicate major versions is the concrete way to catch this before it surfaces as a
build failure.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-11 | filed | Discovered while independently verifying a background agent's `mingl` wasm32 clippy fix; confirmed pre-existing (zero `Cargo.lock` diff this session) and workspace-wide (reproduces identically on the unrelated `minwebgpu` crate). Left in Draft/unfixed state — the fix is a workspace-wide dependency-resolution change outside this session's clippy-cleanup scope, and needs its own full-workspace wasm32 re-verification before landing. |
