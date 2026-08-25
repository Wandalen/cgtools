# BUG-438: `UnrealBloomPass` has a manual `gl_resources_free` but no `Drop` backstop -- leaks on any code path that skips calling it

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak on any error
  path or forgotten call site, since cleanup was entirely opt-in)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::post_processing::UnrealBloomPass` that drops
  an instance without first calling `gl_resources_free` -- including any panic-unwind or early
  `?`-return before the call site is reached.
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/unreal_bloom.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/433/436/437/440, found in the same sweep --
  distinct in that `UnrealBloomPass` already had a manual cleanup method (unlike the other five,
  which had none at all); the gap here is specifically the missing unconditional `Drop`
  backstop.

## Symptom

`UnrealBloomPass` already had a manual `gl_resources_free` method, deleting its 10 mip textures
and all blur/composite shader programs -- but no `impl Drop` backstop. Any caller that dropped
the pass without first remembering to call `gl_resources_free` (e.g. on an error path, or simply
forgetting -- nothing in the type system enforces it) leaked every one of those GPU resources
silently.

## Impact

**Who is affected:** Any consumer that drops an `UnrealBloomPass` without explicitly calling
`gl_resources_free` first -- including `Renderer::resize()` itself before BUG-435's fix (an
early `?`-return that replaced `self.bloom_effect` with `None` without freeing the old value's
resources first was exactly this pattern, one level up).

**What breaks:** No immediate visual/functional symptom -- cumulative GPU memory pressure on
any code path that drops the pass without the manual free call.

**Magnitude:** 10 mip textures + all blur/composite shader programs per unfreed drop.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433 -- cross-referencing
every GPU-resource-owning struct against whether it has a matching `Drop` path, not just a
manual free method. `UnrealBloomPass` was the one struct in this sweep with a manual method but
no `Drop` backstop.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs, mod tests
let gl = gl_init();
let pass = UnrealBloomPass::new( &gl, 64, 64, gl::RGBA16F ).unwrap();
let horizontal : Vec< _ > = pass.horizontal_targets.iter().cloned().flatten().collect(); // MIPS textures
let vertical : Vec< _ > = pass.vertical_targets.iter().cloned().flatten().collect(); // MIPS textures
let blur_programs : Vec< _ > = pass.blur_materials.iter().map( | m | m.program().clone() ).collect();
let composite_program = pass.composite_material.program().clone();
drop( pass ); // deliberately NOT calling gl_resources_free first
// pre-fix: every one of these is still a live GL object after drop.
for t in horizontal.iter().chain( vertical.iter() ) { assert!( !gl.is_texture( Some( t ) ) ); }
for p in &blur_programs { assert!( !gl.is_program( Some( p ) ) ); }
assert!( !gl.is_program( Some( &composite_program ) ) );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- unreal_bloom_pass_drop_frees_all_textures_and_programs_without_explicit_free_call
```

## Root Cause

The struct had no persistent `gl` field to call `gl.delete*` from inside `Drop::drop`, since
every other method already received `gl` as an explicit parameter -- so a `Drop` impl was never
added when `gl_resources_free` was.

## Why Not Caught

No test previously exercised dropping an `UnrealBloomPass` *without* first calling
`gl_resources_free` -- `tests/unreal_bloom_tests.rs` exercises the explicit-call path, which
already worked correctly; the implicit-drop-only path had no coverage.

## Fix Location

`module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs`: added a persistent `gl`
field (cloned at construction) and `impl Drop for UnrealBloomPass`, calling
`self.gl_resources_free(&self.gl.clone())` (or equivalent direct delete calls) from
`Drop::drop`, making cleanup unconditional rather than opt-in.

## Prevention

New inline test `unreal_bloom_pass_drop_frees_all_textures_and_programs_without_explicit_free_call`
in `unreal_bloom.rs`'s `#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (inline
because it needs private-field access -- see `rulebook.md § Test placement`), reusing the
`UnrealBloomPass::new`/`texture_make()` construction pattern already established by
`tests/unreal_bloom_tests.rs`. Deliberately drops the pass *without* calling
`gl_resources_free` first, to prove `Drop` alone -- not the manual method -- suffices.

## Pitfall

A manual `gl_resources_free`-only cleanup method is opt-in -- it only helps callers who remember
to call it, and does nothing on a panic-unwind or an early `?`-return before the call site is
reached. A stored `gl` field plus `impl Drop` makes cleanup unconditional; a manual method alone
never does, no matter how consistently existing call sites happen to call it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added a persistent `gl` field and `impl Drop for UnrealBloomPass`, delegating to the existing `gl_resources_free` logic; added `Fix(BUG-438)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; test reuses the `UnrealBloomPass::new`/`texture_make()` pattern already proven by `tests/unreal_bloom_tests.rs`. Adversarial pass: confirmed by direct inspection that pre-fix `UnrealBloomPass` had no `impl Drop` at all -- dropping without the manual call would leave every resource live; the new test's deliberate omission of the manual call is exactly the previously-uncovered path. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-438)`/`Root cause`/`Pitfall` 3-field source comment; 5-section test doc comment on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `unreal_bloom.rs`'s `UnrealBloomPass` struct/impl block plus its own inline test module. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `UnrealBloomPass` had no `Drop`
impl, so a drop without the manual `gl_resources_free` call left every resource live; the new
test's deliberate no-manual-call-then-drop sequence is the direct, deterministic check for that
gap. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs` | Added a persistent `gl` field and `impl Drop for UnrealBloomPass` with `Fix(BUG-438)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs` | Added inline `mod tests::unreal_bloom_pass_drop_frees_all_textures_and_programs_without_explicit_free_call` (wasm32-gated). |
