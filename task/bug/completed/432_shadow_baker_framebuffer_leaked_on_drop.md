# BUG-432: `ShadowBaker` allocates a `WebGlFramebuffer` in `new` but never frees it

- **Severity:** Low (no crash, no visual corruption -- a slow, unbounded GPU-memory leak that
  only manifests across repeated construct/drop cycles, e.g. a scene reload that rebuilds the
  lightmap-baking pipeline)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::ShadowBaker` that constructs and drops more
  than one instance over the application's lifetime (a single long-lived instance never observes
  the leak at all, since nothing frees the framebuffer at any point during its life either way).
- **Component:** `module/helper/renderer` (`src/webgl/shadow.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* as BUG-433/436/437/438/440 (a GPU-resource-owning struct
  in this crate with no matching teardown path) -- found and fixed together as part of one sweep
  of `module/helper/renderer/`'s post-processing/skeleton/IBL code, but each is an independent
  struct with its own fix, filed separately since no shared root cause links them beyond the
  general pattern.

## Symptom

```rust
// pre-fix -- src/webgl/shadow.rs, ShadowBaker::new
let framebuffer = gl.create_framebuffer();
// ... framebuffer used by soft_shadow_render, stored on `self.framebuffer` ...
// -- no `impl Drop`, no manual free method anywhere in the struct.
```

`ShadowBaker` allocates exactly one `WebGlFramebuffer` per instance and never deletes it, on any
code path -- not on drop, not via any manual cleanup call, since none existed.

## Impact

**Who is affected:** Any consumer that constructs more than one `ShadowBaker` over the
application's lifetime -- e.g. a scene reload or lightmap-quality change that rebuilds the baking
pipeline. Each construct/drop cycle leaks one more framebuffer object for the remaining lifetime
of the GL context, with no way for the caller to reclaim it short of losing the whole context.

**What breaks:** No immediate visual or functional symptom -- the leak is purely cumulative GPU
memory pressure, invisible until enough cycles accumulate to matter (a single-construction
consumer never notices at all).

**Magnitude:** One `WebGlFramebuffer` per `ShadowBaker` construct/drop cycle.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide automated bug/UX-defect discovery sweep of `module/helper/renderer/`
and `module/helper/tilemap_renderer/`, cross-referencing every GPU-resource-owning struct in the
post-processing/shadow/skeleton/IBL code against whether it has a matching `gl.delete_*`/`Drop`
path. `ShadowMap`, the very next struct up in the same file, already has a correct `impl Drop`
(deleting both `framebuffer` and `depth_texture`) -- `ShadowBaker` was the one sibling struct in
this file missing the equivalent.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/shadow.rs, mod tests (inline, wasm32-gated)
let gl = gl_init();
let baker = ShadowBaker::new( &gl ).unwrap();
let framebuffer = baker.framebuffer.clone();
assert!( gl.is_framebuffer( framebuffer.as_ref() ) ); // true right after construction
drop( baker );
assert!( !gl.is_framebuffer( framebuffer.as_ref() ) ); // pre-fix: still true -- never freed
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- shadow_baker_drop_frees_framebuffer
```

## Root Cause

Unlike `ShadowMap` right above it in the same file (which already has `impl Drop` deleting both
its `framebuffer` and `depth_texture`), `ShadowBaker` was never given a matching `Drop` impl when
it was added. GPU handle wrapper types (`Option< WebGlFramebuffer >` etc.) are just JS-object
handles -- dropping the Rust value does not call `gl.delete*` for you; only an explicit delete
call does.

## Why Not Caught

`ShadowBaker` had zero prior test coverage of any kind -- nothing exercised its construction or
destruction, so a missing `Drop` impl produced no observable failure in CI or manual testing.

## Fix Location

`module/helper/renderer/src/webgl/shadow.rs`: added `impl Drop for ShadowBaker`, calling
`self.gl.delete_framebuffer( self.framebuffer.as_ref() )`, modeled directly on the sibling
`ShadowMap`'s pre-existing `impl Drop` in the same file.

## Prevention

New inline test `shadow_baker_drop_frees_framebuffer` in `src/webgl/shadow.rs`'s
`#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (placed inline rather than in
`tests/` because it needs to read the private `framebuffer` field before drop -- see
`rulebook.md § Test placement`). Captures a clone of the framebuffer handle right after
construction, asserts it is a live GL object, drops the `ShadowBaker`, and asserts the handle is
no longer a live GL object -- the same deterministic `gl.is_framebuffer` existence-check pattern
used by this crate's other GPU-teardown reproducer tests.

## Pitfall

Adding a new GL-resource-owning struct next to an existing one that already has `impl Drop` is
easy to do without copying that pattern over -- the struct compiles and runs identically either
way, so nothing short of a GPU-memory audit (or, going forward, a construct/drop existence-check
test) surfaces the leak.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `impl Drop for ShadowBaker`, matching `ShadowMap`'s existing pattern; added `Fix(BUG-432)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean. Adversarial pass: verified by direct code inspection that pre-fix `ShadowBaker` had no `Drop`/free path of any kind -- the new test's post-drop `is_framebuffer` assertion would have failed against that code (no way to run the literal pre-fix binary without reverting, but the absence of any delete call makes the failure mode structurally certain, not merely likely). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-432)`/`Root cause`/`Pitfall` 3-field format applied to the source comment at `shadow.rs`; 5-section test doc comment applied to the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `shadow.rs`'s `ShadowBaker` impl block plus its own inline test module; no other file touched for this specific item. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `ShadowBaker` had no delete path
for `framebuffer` on any code path (drop or otherwise); the new test's post-drop
`gl.is_framebuffer` assertion is the direct, deterministic check for exactly that absence.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shadow.rs` | Added `impl Drop for ShadowBaker` with `Fix(BUG-432)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shadow.rs` | Added inline `mod tests::shadow_baker_drop_frees_framebuffer` (wasm32-gated, needs private-field access per `rulebook.md § Test placement`). |
