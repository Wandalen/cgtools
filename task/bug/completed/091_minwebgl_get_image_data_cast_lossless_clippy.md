# BUG-091: `minwebgl::texture::d2`'s `get_image_data` call fails `clippy::cast_lossless` under `-D warnings`

- **Severity:** Medium
- **state:** Completed
- **Affects:** Any `-D warnings` clippy gate that reaches `module/min/minwebgl` with the `web-sys`/canvas texture path compiled in — concretely, `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` (task 087's own Invariant I2), and by the same mechanism any other crate's `--all-features`/`--workspace` clippy gate that pulls in `minwebgl` via an `adapter-webgl`-style optional dependency
- **Component:** `module/min/minwebgl` (`src/texture/d2.rs:363`)
- **repo_identity:** self
- **Filed:** 2026-08-11
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-11
- **Fixed:** 2026-08-11
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```
error: casting `f32` to `f64` may become silently lossy if types change
   --> module/min/minwebgl/src/texture/d2.rs:363:46
    |
363 |     let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();
    |                                              ^^^^^^^^^^^^^^^^ help: try: `f64::from(img_width)`
    |
    = note: `-D clippy::cast-lossless` implied by `-D warnings`

error: casting `f32` to `f64` may become silently lossy if types change
   --> module/min/minwebgl/src/texture/d2.rs:363:64
    |
363 |     let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();
    |                                                                ^^^^^^^^^^^^^^^^^ help: try: `f64::from(img_height)`

error: could not compile `minwebgl` (lib) due to 2 previous errors
```

Reproduced fresh this session (2026-08-11) two ways, isolating the fault to `minwebgl` itself
rather than any caller:

```bash
RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings
# exit 101 — task/verified/-0056_longrun.log

RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --all-targets --features adapter-webgl -- -D warnings
# exit 101 — task/verified/-0059_longrun.log, adapter-native NOT enabled, same 2 errors
```

The second run proves the failure is entirely inside `minwebgl` (reached via `tilemap_renderer`'s
`adapter-webgl` → `dep:minwebgl` feature edge) and has nothing to do with `adapter-native` or any
other feature.

## Impact

**Who is affected:** Anyone running a `-D warnings` clippy gate (this workspace's own Invariant/
Level-3-style checks) against a crate whose `--all-features` (or a single enabled feature, e.g.
`adapter-webgl`) pulls in `minwebgl`. Plain, non-strict `cargo check`/`cargo build`/`cargo clippy`
(without `-D warnings`) are unaffected — `cast_lossless` is a default-warn (not default-deny)
clippy lint; it only escalates to a hard error under an explicit `-D warnings`/`-D
clippy::cast_lossless` gate.

**What breaks:** `cargo clippy ... -- -D warnings` exits 101 for any target that compiles
`module/min/minwebgl/src/texture/d2.rs`. Concretely blocks task 087's own Invariant I2
(`cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`) — confirmed
independently of task 087's own `adapter-native` changes (both isolation runs above).

**Why Medium, not High:** unlike BUG-080 (breaks the plain, default `cargo check --workspace`),
this only breaks an opt-in strict-lint gate, and the fix is a single mechanical one-line rewrite
with no API or behavior change.

## How Discovered

Hit directly while completing task 087's own Invariant I2 clippy check
(`task/executing/087_tilemap_renderer_adapter_native_backend.md`). Isolated to `minwebgl` itself
(not `adapter-native`) by re-running the same clippy invocation with only `adapter-webgl` enabled —
identical 2 errors, same line, confirming task 087's own changes are not implicated.

## Minimum Reproducible Example

```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
RUSTFLAGS="-D warnings" cargo clippy -p minwebgl --lib -- -D warnings
```

**Verify Command:** as above; **Expected** (once fixed): exit 0; **Actual:** exit 101,
`error: casting `f32` to `f64` may become silently lossy if types change` ×2 (verbatim output in
`## Symptom`).

## Root Cause

`module/min/minwebgl/src/texture/d2.rs:363`, added by commit `9b71cf39` ("feat: add scene script
support and comprehensive testing across examples", 2026-08-10; git-blamed directly this session —
`git log -L 363,363:module/min/minwebgl/src/texture/d2.rs` →
`9b71cf398efd52337882c0577b352e0b3374a552 2026-08-10`):

```rust
let data = ctx.get_image_data( 0.0, 0.0, img_width as f64, img_height as f64 ).unwrap().data().to_vec();
```

`img_width`/`img_height` are `f32`; `as f64` is a widening cast that can never truncate, but
clippy's `cast_lossless` still flags any `as`-cast between numeric types where a lossless `From`
conversion exists (`f64: From<f32>`), on the principle that `as` silently stops being lossless if
either type ever changes — `f64::from(...)` is guaranteed to keep failing loudly (a compile error)
if that invariant is ever broken, whereas `as` would silently start truncating. Pre-existing since
2026-08-10, unrelated to any of this session's 2026-08-11 GPU HAL / `adapter-native` work.

## Why Not Caught

`cargo clippy -p minwebgl` alone (bare, no `-D warnings`) is clean-looking in normal development —
`cast_lossless` is warn-by-default, easy to miss in unfiltered clippy output, and no `--all-features`/
`-D warnings` gate had been run against a target that actually compiles `d2.rs` since the line
landed on 2026-08-10 until task 087's own Invariant I2 reached it today.

**Pitfall:** an optional dependency pulled in only under a specific feature combination
(`adapter-webgl` here) can carry a latent `-D warnings` failure invisible to every narrower
`--features <other>` gate — the failure only surfaces the first time some caller's own
`--all-features` (or that specific feature) check reaches it, and then looks (misleadingly) like a
regression in the caller's own change.

## Fix Applied

`module/min/minwebgl/src/texture/d2.rs` — mechanical one-line rewrite, matching clippy's own
suggestion exactly:

```rust
let data = ctx.get_image_data( 0.0, 0.0, f64::from( img_width ), f64::from( img_height ) ).unwrap().data().to_vec();
```

Applied 2026-08-11 by the task-058 all-warnings sweep lane (the standing "fix all clippy
errors/warnings" directive covers `minwebgl`, which is outside task 087/092's declared
`tilemap_renderer` unit — no collision with that lane's claim). Note the site had meanwhile been
wrapped by BUG-053's fix into a `#[ cfg( not( web_sys_unstable_apis ) ) ]` branch (its
`cfg( web_sys_unstable_apis )` sibling uses integer `dim_as_i32` casts), so the lint is only
reachable under a `RUSTFLAGS` override that suppresses `.cargo/config.toml`'s
`--cfg web_sys_unstable_apis` — exactly the configuration the MRE uses.

**Verification (2026-08-11):**
- Exact MRE / Verify Command: `RUSTFLAGS="-D warnings" cargo clippy -p minwebgl --lib -- -D warnings` → exit 0, `Finished`.
- Branch-activating clippy: `RUSTFLAGS="" cargo clippy -p minwebgl --all-features -- -D warnings` → exit 0.
- Normal config (branch compiled out): `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` → exit 0.

## Generalized Version

**Broken assumption:** "a `-D warnings`/`--all-features` clippy gate that passes for crate A's own
declared feature set will also pass once a downstream consumer enables A as an optional
dependency." False whenever A itself has never been clippy-gated at `-D warnings` under the
specific feature combination the consumer activates — the first consumer to reach that combination
inherits a pre-existing lint debt that looks like their own regression. Running `-D warnings`
clippy directly against the dependency crate in isolation (as the MRE above does) is the fast way
to confirm the fault lies upstream, not in the consumer's own diff.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-11 | filed | Discovered while running task 087's Invariant I2 (`cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`). Isolated to `minwebgl` itself via a second run with only `adapter-webgl` enabled (no `adapter-native`) — identical failure, confirming task 087's own changes are not implicated. Left in Draft/unfixed state — fixing requires a `src/` edit in `module/min/minwebgl`, outside task 087's own declared scope. |
| 2026-08-11 | fixed + completed | Fixed by the task-058 all-warnings sweep lane: `img_width as f64`/`img_height as f64` → `f64::from(...)` at the (now cfg-gated, post-BUG-053) `not( web_sys_unstable_apis )` branch of `d2.rs`. Verified three ways (exact MRE with `RUSTFLAGS="-D warnings"`, branch-activating `RUSTFLAGS=""` clippy, and normal-config clippy) — all exit 0. Closed same-session, Round 0, self-accepted per BUG-079 precedent. |
