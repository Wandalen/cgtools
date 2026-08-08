# BUG-007: csgrs's mandatory core2 dependency is permanently yanked, breaking all workspace cargo resolution

- **Severity:** Critical
- **state:** Verified
- **Affects:** Any `cargo metadata`/`build`/`test`/`nextest run` invocation anywhere in the workspace (no committed `Cargo.lock` — every invocation re-resolves the full graph from scratch)
- **Component:** workspace root `Cargo.toml` — `[workspace.dependencies.csgrs]` / `[patch.crates-io]` (the only manifest where a crates.io patch is honored; consumer crates listed in `## Impact`)
- **repo_identity:** self
- **Filed:** 2026-08-08
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **Reproducer:** `cd /tmp/mre007 && cargo metadata --format-version=1` (see `## Minimum Reproducible Example`)

## Symptom

```bash
# cargo nextest run --all-features   (wrong — from module/helper/animation, before fix; that crate has
# zero dependency on csgrs itself — failure is collateral damage from workspace-wide resolution)
error: failed to select a version for the requirement `core2 = "^0.4"`
  version 0.4.0 is yanked
required by package `csgrs v0.20.1`
    ... which satisfies dependency `csgrs = "^0.20.1"` of package `primitive_generation v0.1.0`
error: command `cargo metadata '--format-version=1' --all-features --filter-platform aarch64-unknown-linux-gnu` failed with exit status: 101

# cargo nextest run --all-features   (correct — identical command, after fix)
Finished 'test' profile [unoptimized + debuginfo] target(s) in 50.48s
Summary [0.037s] 21 tests run: 21 passed, 0 skipped
```

## Impact

Blocks every `cargo metadata`/`build`/`test`/`nextest run` invocation anywhere in this workspace, not
just the `--all-features` path that first surfaced it: `examples/minwebgl/narrow_outline/Cargo.toml:21`
and `examples/minwebgl/text_rendering/Cargo.toml:19` both declare `csgrs = { workspace = true }` with
no `optional = true` (E5), so those two crates cannot resolve under **any** feature combination, default
or otherwise. Failure is loud (cargo exits 101 with an explicit resolver error, not a silently wrong
result) but total — a workspace-wide dependency-resolution failure is collateral damage even for crates
with zero relation to csgrs, because Cargo solves one graph for the entire workspace and this repo has
no committed `Cargo.lock` (`.gitignore:11`, `.gitignore:25` — E6) to shield an existing resolution from
a newly-yanked transitive dependency. Every fresh checkout, CI run, or local cache-clear re-triggers the
failure from scratch, for every contributor.

Entity Scope: `None` — the affected files are ordinary source-tree `Cargo.toml` manifests, not entity
directory instances; `## Affected Entity Collections` does not apply.

## How Discovered

```bash
$ cd module/helper/animation && cargo nextest run --all-features --no-fail-fast --hide-progress-bar \
    --no-tests=pass --status-level=fail --final-status-level=fail
error: failed to select a version for the requirement `core2 = "^0.4"`
  version 0.4.0 is yanked
required by package `csgrs v0.20.1`
    ... which satisfies dependency `csgrs = "^0.20.1"` of package `primitive_generation v0.1.0`
error: command `cargo metadata '--format-version=1' --all-features --filter-platform aarch64-unknown-linux-gnu` failed with exit status: 101
```

## Minimum Reproducible Example

Fully self-contained — depends only on the real, public `csgrs` crate on crates.io; no cgtools-specific
paths, crate names, or registry state involved.

```bash
mkdir -p /tmp/mre007/src
cat > /tmp/mre007/Cargo.toml <<'EOF'
[package]
name = "mre007"
version = "0.1.0"
edition = "2021"

[dependencies]
csgrs = "0.20.1"
EOF
echo 'fn main() {}' > /tmp/mre007/src/main.rs
cd /tmp/mre007
cargo metadata --format-version=1
```

**Expected:**
```
(exit 0 — resolved dependency graph as JSON on stdout)
```

**Actual:**
```
    Updating crates.io index
error: failed to select a version for the requirement `core2 = "^0.4"`
  version 0.4.0 is yanked
location searched: crates.io index
required by package `csgrs v0.20.1`
    ... which satisfies dependency `csgrs = "^0.20.1"` of package `mre007 v0.1.0 (/tmp/mre007)`
```

Executed 3 independent times; failed identically every time.

**Verify Command:**
```bash
cd /tmp/mre007 && cargo metadata --format-version=1 2>&1 | grep -q "is yanked" && echo "BUG REPRODUCED"
```
**What:** Violates the invariant that `cargo metadata` must exit 0 for any dependency graph containing
only non-yanked, resolvable crate versions — `core2 0.4.0` (the only version satisfying csgrs 0.20.1's
`^0.4` requirement) is permanently yanked, so resolution fails deterministically for any consumer of
csgrs ≥0.16 without a registry override.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | Failure is a transient registry/network issue or a stale local index cache, not a genuine permanent yank | ❌ Disproved | `Cargo.toml:320` requires `csgrs = "0.20.1"` — crates.io reports ALL 7 published `core2` versions yanked (0.0.0 through 0.4.0), a permanent deprecation, not a transient blip | E1, E2 |
| H2 | `csgrs 0.20.1`'s unconditional dependency on the permanently-yanked `core2 ^0.4`, combined with no committed `Cargo.lock`, makes every fresh resolution fail | ✅ Root Cause | `Cargo.toml:320-323` requires `csgrs = "0.20.1"`; csgrs's own manifest declares `core2 = "^0.4"` with `optional: false, target: null` — mandatory for every consumer, not feature-gated | E1, E2, E3, E4, E6, E7 |
| H3 | A workspace-level feature-unification quirk in this repo's root `Cargo.toml` accidentally activates `csgrs`/`core2` when it wouldn't otherwise be needed | ❌ Disproved | `examples/minwebgl/narrow_outline/Cargo.toml:21`, `examples/minwebgl/text_rendering/Cargo.toml:19` both declare `csgrs = { workspace = true }` with no `optional = true` — activation is deliberate, real usage, not a unification accident | E5 |
| H4 | `csgrs` itself (not just its `core2` dependency) was yanked, and the fix is to pin an older `csgrs` | ❌ Disproved | crates.io confirms `csgrs 0.20.1` is NOT yanked (34 published versions, none yanked); real, working `csgrs::sketch`/`csgrs::mesh` API usage in example source requires the ≥0.16 API surface, ruling out a pre-0.16 downgrade | E8, E9 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | Terminal output | `cargo nextest run --all-features` from `module/helper/animation` fails: `failed to select a version for the requirement core2 = "^0.4" — version 0.4.0 is yanked — required by package csgrs v0.20.1` | H1 ❌, H2 ✅ |
| E2 | Terminal output (`crates.io` API `/api/v1/crates/core2`) | All 7 published `core2` versions (0.4.0, 0.3.3, 0.3.2, 0.3.1, 0.3.0, 0.3.0-alpha.1, 0.0.0) carry `"yanked": true` — the entire crate is deprecated, not one bad release | H1 ❌, H2 ✅ |
| E3 | Terminal output (`crates.io` API `/api/v1/crates/csgrs/0.20.1/dependencies`) | `core2`'s dependency entry shows `"optional": false, "target": null` — unconditional for every consumer of csgrs 0.20.1, not feature-gated | H2 ✅ |
| E4 | Terminal output (`crates.io` API version bisection) | `csgrs 0.15.0`'s dependency list has no `core2` entry; `csgrs 0.16.0`'s does — the dependency was introduced between those two releases | H2 ✅ (symptom) |
| E5 | `examples/minwebgl/narrow_outline/Cargo.toml:21`, `examples/minwebgl/text_rendering/Cargo.toml:19` | `csgrs = { workspace = true }` with no `optional = true` in either file — non-optional, always-active dependency | H3 ❌ |
| E6 | `.gitignore:11`, `.gitignore:25` | `Cargo.lock` is explicitly gitignored and not tracked (`git ls-files` confirms absence) — nothing shields a fresh checkout from re-resolving the full graph | H2 ✅ |
| E7 | Terminal output (post-fix verification) | After adding `[patch.crates-io]` pinning `core2` to commit `545e84bcb0f235b12e21351e0c69767958efe2a7` in root `Cargo.toml:427-433`: `cargo metadata --all-features` exits 0 (8s), and the user's exact original command exits 0 with `21 tests run: 21 passed, 0 skipped` (51s) | H2 ✅ |
| E8 | Terminal output (`crates.io` API `/api/v1/crates/csgrs`) | `csgrs` has 34 published versions, none yanked, including the current `0.20.1` | H4 ❌ |
| E9 | `examples/minwebgl/narrow_outline/src/main.rs:87,90-91,582,1069`, `examples/minwebgl/text_rendering/src/text.rs:821,823-824` | Real, working calls to `csgrs::traits::CSG`, `csgrs::sketch::Sketch<()>`, `csgrs::mesh::Mesh<()>` — API surface that only exists in csgrs ≥0.16, ruling out a pre-0.16 downgrade | H4 ❌ |

## Root Cause

```
cargo resolves the workspace graph (no Cargo.lock committed → full fresh resolution every run)
  primitive_generation's optional "csg" feature          (Cargo.toml:47, primitive_generation)
  examples/minwebgl/{narrow_outline,text_rendering}      (unconditional, Cargo.toml:21 / :19)
       |
       v
  csgrs = "0.20.1"                                       (Cargo.toml:320-323, workspace root)
       |
       v
  core2 = "^0.4"  [optional: false, target: null]        (csgrs 0.20.1's own manifest)
       |
       v
  ALL published core2 versions yanked                     <- resolution FAILS here
```

csgrs added an unconditional dependency on `core2` between v0.15.0 and v0.16.0 (H2 — ✅ Root Cause;
E4); upstream
deprecated `core2` in favor of using `core`/`std::io` directly and yanked every release including 0.4.0
(E2), but csgrs never dropped the dependency. Because this workspace has no committed `Cargo.lock`
(E6), Cargo re-resolves the full graph from scratch on every invocation and always hits the
permanently-yanked `core2` — an unrelated crate's test run (`module/helper/animation`, which has zero
dependency on csgrs) fails purely as collateral damage from atomic, workspace-wide resolution (E1, E7).

## Why Not Caught

No CI pipeline exists in this repo (no `.github/` directory) to run the existing `ctest1`-`ctest5` /
`wtest1`-`wtest5` Makefile targets (`readme.md:107`; `ctest4`/`ctest5` already run `cargo +nightly
audit`) on a fresh checkout, and no committed `Cargo.lock` means a contributor's warm local cache can
mask the break indefinitely. No unit or integration test can assert "this external registry dependency
stays non-yanked forever" — that is an external-registry fact, not code logic — but a CI job on a clean
checkout, or a periodic `cargo update --dry-run`, would have caught the transition the moment upstream
yanked `core2`.

## Fix Location

`Cargo.toml:424-433` (workspace root):

```toml
# Before:
[workspace.dependencies.ron]
version = "0.12.1"

# After:
[workspace.dependencies.ron]
version = "0.12.1"

# csgrs (see [workspace.dependencies.csgrs] above) unconditionally depends on core2 ^0.4.
# Every published core2 version on crates.io is yanked — upstream deprecated the crate in
# favor of using `core`/`std::io` directly, and removed Cargo.toml from HEAD entirely, so a
# bare `git` dependency won't resolve either. This pins the last commit before deprecation,
# which still has the real 0.4.0 manifest. Remove this patch if csgrs ever drops core2.
[patch.crates-io]
core2 = { git = "https://github.com/bbqsrc/core2", rev = "545e84bcb0f235b12e21351e0c69767958efe2a7" }
```

## Prevention

Commit a `Cargo.lock` at the workspace root (currently gitignored) so a routine `cargo build` doesn't
silently re-resolve into a freshly-yanked transitive dependency the moment one appears anywhere in the
graph — a locked, deliberately-`cargo update`d lockfile turns "breaks whenever any upstream crate
anywhere in the graph gets yanked" into "breaks only when a maintainer deliberately updates." Wire the
existing `ctest4`/`ctest5` Makefile targets (already run `cargo +nightly audit`) into a CI workflow that
runs on a clean checkout — none currently runs automatically.

**Pitfall:** An unconditional (non-optional) dependency on a small, single-purpose crate is a single
point of failure for an entire workspace's buildability the moment that crate is deprecated and yanked
— patch the offending transitive dependency via `[patch.crates-io]` the moment this class of break is
discovered, rather than waiting for the direct dependency to drop it upstream.

## Generalized Version

**Broken assumption:** Any version of a crate published to crates.io remains resolvable indefinitely, so
re-running dependency resolution without a lockfile is always safe.

**Failure conditions:**
1. A workspace has no committed `Cargo.lock` (or the lock is being regenerated), AND
2. Some dependency in the resolved graph — direct or transitive, optional or not — has every version
   satisfying its requirers' constraints yanked, AND
3. No `[patch]` override exists for that dependency.

**Detection invariant:**
```
cargo metadata --all-features   # from a clean cache / no-lockfile state
exit code == 0
```
Any resolver failure citing "is yanked" violates this invariant.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-08 | filed | Initial report — csgrs 0.20.1's `core2 ^0.4` dependency is entirely yanked, breaking all workspace cargo resolution |
| 2026-08-08 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions 🟢 (2 findings caught and fixed in-loop: D4 Root Cause label, D6 Component scope); MRE executed 3× fresh; fix already applied and independently verified (root `Cargo.toml:424-433`; `cargo metadata --all-features` exit 0; user's original command exit 0, 21/21 tests passed) |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟡 | 🟢 | Root Cause prose cited H2 by ID but not its `✅ Root Cause` label explicitly | Labeled the citation "(H2 — ✅ Root Cause; E4)" |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟡 | 🟢 | `Component` named 4 crates when Fix Location resolves to exactly one (workspace root manifest) | Narrowed `Component` to the root manifest; consumers left in `## Impact` |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 found, 2 fixed | 2/2 |

**Reproduced:** YES — Verify Command exit 0, 2026-08-08 (underlying `cargo metadata` itself exits 101, matching the documented Actual block).
