# BUG-101: `animation_surface_rendering`'s pinned `kurbo`/`peniko` versions no longer match the unpinned `interpoli` git dependency's current requirements, breaking `cargo check --workspace`

- **Severity:** High
- **state:** Completed
- **Affects:** The default, unscoped `cargo check --workspace --all-features` / `cargo build --workspace` — breaks for anyone running a full-workspace gate, not just `animation_surface_rendering`'s own maintainers
- **Component:** `examples/minwebgl/animation_surface_rendering` (`src/animation/model.rs`, `src/animation/animation.rs`, `Cargo.toml`), root `Cargo.toml`
- **repo_identity:** self
- **Filed:** 2026-08-12
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-13
- **Fixed:** 2026-08-13

## Symptom

```
error[E0277]: the trait bound `kurbo::Point: Tween` is not satisfied
  --> examples/minwebgl/animation_surface_rendering/src/animation/model.rs:97:18
   |
97 |     pub anchor : Value< kurbo::Point >,
   |                  ^^^^^^^^^^^^^^^^^^^^^ the trait `Tween` is not implemented for `kurbo::Point`
   |
   = help: the following other types implement trait `Tween`:
             AlphaColor<color::Srgb>
             f32
             f64
             kurbo::point::Point
             kurbo::size::Size
             kurbo::vec2::Vec2
   = note: there are multiple different versions of crate `kurbo` in the dependency graph
note: required by a bound in `interpoli::Value`
  --> /home/user1/.cargo/git/checkouts/interpoli-ca5b1e4fe33ed67c/04ae4a4/src/value.rs:9:19
 9 | pub enum Value<T: Tween> {
   |                   ^^^^^ required by this bound in `Value`
```

Plus matching `E0277`/`E0308` errors for `Vec2: Tween`, `peniko::Brush` vs `Brush<I,G>`, `kurbo::bezpath::PathEl` vs `PathEl`, and `kurbo::affine::Affine` vs `Affine` (all "multiple different versions of crate `X` in the dependency graph") — 75 errors total, all in `animation_surface_rendering`, none in any other workspace crate. Full output: `task/-0011_longrun.log`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check --workspace --all-features
```
**Expected:** exit 0. **Actual:** exit 101, `error: could not compile \`animation_surface_rendering\` (bin "animation_surface_rendering") due to 75 previous errors`.

**Isolating command** (confirms the rest of the workspace is unaffected):
```bash
cargo check --workspace --all-features --exclude animation_surface_rendering
# exit 0 — task/-0012_longrun.log
```

## Impact

**Who is affected:** Anyone running the default, unscoped `cargo check --workspace` / `cargo build --workspace` / `cargo test --workspace` — this is not an opt-in strict-lint gate (unlike BUG-091), it is the plain default build.

**What breaks:** The workspace-wide build gate any task's own "final full-workspace verification" step relies on (e.g. this session's task 100 Acceptance Criterion "`cargo check --workspace` exits 0"). Any other crate's own `-p <crate>` / package-scoped build is unaffected.

**Why High:** breaks the plain, default `cargo check --workspace` (same class as BUG-080), not merely an opt-in `-D warnings` gate.

## How Discovered

Hit while running task 100's own final workspace-verification step (`cargo check --workspace --all-features`, `task/-0011_longrun.log`) after `module/shader/shader_chunks_cli` (task 100's own deliverable) was already compiling, testing, and clippy-clean in isolation. Confirmed zero relation to task 100's changes via `git status --short` (no uncommitted modification to `animation_surface_rendering`, `interpoli`, `kurbo`, or `peniko` — the failure is not caused by any in-flight edit) and via the `--exclude animation_surface_rendering` isolation run above (rest of the workspace, including `shader_chunks_cli`'s own integration, checks clean).

## Minimum Reproducible Example

```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check -p animation_surface_rendering --all-features
```

## Root Cause

`examples/minwebgl/animation_surface_rendering/Cargo.toml` pins `kurbo`/`peniko`/`color` away from the workspace defaults, with a comment explaining why:

```toml
# interpoli (git, no rev — tracks master) requires kurbo ^0.11 — matched for type interop.
kurbo = { version = "0.11" }
...
# interpoli's peniko ^0.4 bundles color 0.3 — both matched for type interop.
peniko = { version = "0.4" }
color = { version = "0.3" }
```

but the workspace root `Cargo.toml` now pins:

```toml
[workspace.dependencies.kurbo]
version = "0.13.1"

[workspace.dependencies.peniko]
version = "0.6.1"
```

`interpoli` is declared `git = "https://github.com/linebender/interpoli"` with **no `rev`/`tag`/`branch`** (confirmed: `grep -n "interpoli" Cargo.toml` shows only the `git =` line, no pin), and this workspace has **no committed `Cargo.lock`** (`git ls-files Cargo.lock` returns empty) — so dependency resolution is not reproducible across sessions. The commit resolved this session, `interpoli#04ae4a48`, apparently now depends on `kurbo`/`peniko` versions compatible with the workspace's current 0.13.1/0.6.1 pins (not the 0.11/0.4 the local comment assumes) — Cargo's resolver ends up with two non-unified copies of `kurbo` (0.11.3 to satisfy `animation_surface_rendering`'s explicit local override, 0.13.1 to satisfy everything using `kurbo.workspace = true`) and two of `peniko` (0.4.1 / 0.6.1). Since Rust treats distinct semver-major copies of the same crate as distinct types, `animation_surface_rendering`'s own `kurbo::Point`/`Vec2` (resolved against its local 0.11 override) no longer satisfies `interpoli::Value<T: Tween>`'s `Tween` bound (implemented against interpoli's own, now-0.13-versioned, `kurbo::Point`/`Vec2`) — the exact "multiple different versions of crate `kurbo`/`peniko` in the dependency graph" note the compiler emits on every one of the 75 errors.

**Not related to this session's task 100 work**: `module/shader/shader_chunks_cli` shares no file, dependency, or transitive edge with `animation_surface_rendering`/`interpoli`/`kurbo`/`peniko` — confirmed via `git status --short` (zero uncommitted changes to any of those paths) and the `--exclude` isolation run (§ Symptom).

## Why Not Caught

`cargo check -p animation_surface_rendering` in isolation, run right after the local `kurbo = "0.11"`/`peniko = "0.4"` pins were written, would have resolved cleanly against whatever `interpoli` commit existed at that time. Because `interpoli` is an unpinned git dependency in a workspace with no committed `Cargo.lock`, the *same* command can silently start failing later purely from upstream `interpoli` history moving forward — no local edit is needed to trigger it, and nothing in this repository's own diff shows the break coming.

**Pitfall:** a local dependency-version override with a comment explaining "matched for type interop" against an *unpinned* transitive dependency is a time bomb — the match is only true at the moment the comment was written. Either pin the transitive dependency itself (here: give `interpoli` an explicit `rev =`) or accept that the override needs re-verification every time `cargo update`/a fresh resolve could move the unpinned dependency.

## Generalized Version

**Broken assumption:** "a locally-pinned dependency version, chosen to match an unpinned transitive git dependency's *current* requirement, stays matched over time." False whenever nothing pins the transitive dependency itself (no `rev`/`tag`, no committed `Cargo.lock`) — the upstream default branch can move independently of this repository, silently invalidating the local override with no corresponding local diff to explain why a previously-clean build now fails.

## Fix

Re-investigated from scratch rather than trusting the filed snapshot, since `interpoli` is an unpinned git dependency and this workspace carries no committed `Cargo.lock` (the bug's own diagnosed non-determinism cause). Two findings changed the picture:

1. **The local override was never actually the mismatch.** `cargo tree -p animation_surface_rendering -i kurbo@0.11.3` shows `animation_surface_rendering`'s own `kurbo = "0.11"` override and `interpoli`'s internal `kurbo 0.11.3` requirement already unify to the *same* crate instance (same name+version+source) — they were never in conflict. The real double-version split (`kurbo@0.13.1` alongside `kurbo@0.11.3` in this crate's own dependency graph) traces to `primitive_generation` → `norad` (UFO font/glyph processing, via `animation_surface_rendering`'s `font-processing` feature on `primitive_generation`) needing the workspace's `kurbo = "0.13.1"` pin — a code path entirely disjoint from `model.rs`/`animation.rs`'s interpoli-facing usage.
2. **The exact filed symptom is not currently reproducible.** `cargo check -p animation_surface_rendering --all-features` and the bug's own literal Verify Command (`cargo check --workspace --all-features`) both exit 0 cleanly right now — at the *same* `interpoli` commit (`04ae4a48`) this bug was filed against (confirmed via `Cargo.lock`, unchanged). Since the git dependency is unpinned and there is no committed lock, dependency resolution is not deterministic across resolves — the 75-error state captured in Symptom apparently resolved itself on a later `cargo` invocation, with no local edit either way. This cuts both directions from the bug's own Pitfall: an unpinned transitive dependency can silently break *or* silently un-break.

Rather than close as a no-op "could not reproduce," applied this bug's own prescribed root-cause fix (§ Root Cause: "pin the transitive dependency itself") to remove the non-determinism going forward:

- Root `Cargo.toml`: added `rev = "04ae4a485c1ec95678f40363c3250cc2a1dd354c"` to `[workspace.dependencies.interpoli]` — pins to the exact commit already confirmed working, so resolution is reproducible from here on regardless of upstream `interpoli` master moving.
- `examples/minwebgl/animation_surface_rendering/Cargo.toml`: updated the now-stale "no rev — tracks master" comment on the local `kurbo` override to reflect the pin.
- No change to `model.rs`/`animation.rs` — no API breakage exists at the pinned commit, so none needed fixing.
- `Cargo.lock` remains uncommitted, unchanged by this fix — a workspace-wide policy decision on committing it is out of scope for this bug.

**Verify:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check --workspace --all-features   # exit 0 — task/-0003_longrun.log
cargo clippy -p animation_surface_rendering --all-targets --all-features -- -D warnings   # exit 0, 0 warnings — task/-0004_longrun.log
```
`animation_surface_rendering` carries no test files of its own (bin-only example crate; confirmed via repo-wide grep for `#[ test ]`/`#[test]` under its path — zero matches), so no test-suite re-run applies beyond the workspace check above.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-12 | filed | Discovered while running task 100's final `cargo check --workspace --all-features` step. Isolated to `animation_surface_rendering` alone via `--exclude animation_surface_rendering` (exit 0) and confirmed unrelated to task 100's own `shader_chunks_cli` changes via `git status --short` (zero uncommitted modification to any file in the failure's dependency chain). Left in Draft/unfixed — fixing requires either pinning `interpoli` to a compatible `rev` or bumping `animation_surface_rendering`'s local `kurbo`/`peniko`/`color` overrides to match `interpoli`'s current requirements (and fixing any resulting API breakage in `model.rs`/`animation.rs`), both outside task 100's own `shader_chunks_cli` scope. |
| 2026-08-13 | fixed | Re-investigated: the local `kurbo`/`peniko` overrides already matched `interpoli`'s real requirement (the double-version split was `primitive_generation`→`norad`, an unrelated code path); the exact filed symptom no longer reproduces at the same `interpoli` commit, consistent with the bug's own unpinned-dependency root cause. Applied the bug's own prescribed fix anyway (pin `interpoli` to `rev = "04ae4a48..."` in root `Cargo.toml`) to close the non-determinism rather than leave it to recur silently. `cargo check --workspace --all-features` and `cargo clippy -p animation_surface_rendering --all-targets --all-features -- -D warnings` both exit 0. Closed via self-verification per this registry's established convention (§ `task/bug/completed/` precedent — bugs in this project are closed by direct file/table edit, not the `tsk` CLI state machine). |
