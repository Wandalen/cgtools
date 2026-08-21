# BUG-433: `GBuffer` leaks its VAO, framebuffer, depth renderbuffer, and every attachment texture

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak across
  repeated construct/drop cycles, e.g. a canvas resize that rebuilds the geometry pass at a new
  resolution)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::post_processing::GBuffer` that constructs and
  drops more than one instance over the application's lifetime -- notably `Renderer::resize()`,
  which rebuilds the `GBuffer` on every resolution change.
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/gbuffer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/436/437/438/440 (a GPU-resource-owning struct
  with no matching teardown path), found in the same sweep; each fixed and filed independently.

## Symptom

`GBuffer::new` allocates a VAO, a color framebuffer, a depth `WebGlRenderbuffer`, and a set of
attachment textures -- but had no `impl Drop` and no manual free method of any kind. The local
`depthbuffer` binding created inside `new` was never even stored on the struct in a way that let
it be freed later.

## Impact

**Who is affected:** Any consumer that constructs more than one `GBuffer` over the application's
lifetime. `Renderer::resize()` is the primary in-tree caller that does this routinely -- every
resize rebuilds the geometry pass's `GBuffer`, so every resize permanently leaked one VAO, one
framebuffer, one renderbuffer, and every attachment texture from the prior resolution.

**What breaks:** No immediate visual/functional symptom -- purely cumulative GPU memory pressure
that grows with every resize, invisible until enough resizes accumulate.

**Magnitude:** Per construct/drop cycle: 1 VAO + 1 framebuffer + 1 depth renderbuffer + N
attachment textures (N = however many attachments the `GBuffer` was constructed with).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/` that
found BUG-432 -- cross-referencing every GPU-resource-owning struct in the post-processing code
against whether it has a matching `gl.delete_*`/`Drop` path. `GBuffer` had none.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/post_processing/gbuffer.rs, mod tests (inline, wasm32-gated)
let gl = gl_init();
let mut attachment_buffers = FxHashMap::default();
attachment_buffers.insert( GBufferAttachment::Albedo, vec![] );
let gbuffer = GBuffer::new( &gl, 64, 64, attachment_buffers ).unwrap();
let ( vao, fb, depth ) = ( gbuffer.vao.clone(), gbuffer.framebuffer.clone(), gbuffer.depthbuffer.clone() );
let textures : Vec< _ > = gbuffer.textures.values().cloned().collect();
drop( gbuffer );
// pre-fix: every one of these is still a live GL object after drop.
assert!( !gl.is_vertex_array( Some( &vao ) ) );
assert!( !gl.is_framebuffer( Some( &fb ) ) );
assert!( !gl.is_renderbuffer( Some( &depth ) ) );
for t in &textures { assert!( !gl.is_texture( Some( t ) ) ); }
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- gbuffer_drop_frees_vao_framebuffer_depthbuffer_and_textures
```

## Root Cause

`GBuffer` never had an `impl Drop` at all. The local `depthbuffer` variable created inside `new`
was dropped as a plain Rust value at end-of-scope (save for the one field it got assigned to),
and the struct itself carried no cleanup path for any of its five owned GL object families --
GPU handle wrapper types are just JS-object references; letting the Rust value go out of scope
never calls `gl.delete*`.

## Why Not Caught

`GBuffer` had no prior test coverage of its construction/destruction lifecycle -- existing tests
(e.g. `fbo_pass_cycle_test.rs`) exercise its rendering behavior, not resource teardown.

## Fix Location

`module/helper/renderer/src/webgl/post_processing/gbuffer.rs`: added `impl Drop for GBuffer`
(immediately before the new inline test module), deleting the VAO, framebuffer, depth
renderbuffer, and every attachment texture.

## Prevention

New inline test `gbuffer_drop_frees_vao_framebuffer_depthbuffer_and_textures` in
`gbuffer.rs`'s `#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (inline because it
needs the private fields -- see `rulebook.md § Test placement`). Uses the minimal
`Albedo`/`PbrInfo`/`Uv1` attachment set already established as a working construction pattern by
`tests/fbo_pass_cycle_test.rs`. Captures all four handle families right after construction,
drops the `GBuffer`, and asserts none remain live GL objects.

## Pitfall

A struct accumulating GL-owned fields incrementally over several edits (VAO first, then a
framebuffer, then a depth renderbuffer, then attachment textures) has no single moment that
forces a teardown audit -- each individual addition compiles and runs fine without a matching
delete call, so the leak only shows up under sustained GPU-memory pressure, not functional
testing.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `impl Drop for GBuffer` covering all five owned GL object families; added `Fix(BUG-433)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean, and the test reuses `fbo_pass_cycle_test.rs`'s already-proven `GBuffer::new` construction pattern. Adversarial pass: confirmed by direct inspection that pre-fix `GBuffer` had zero delete calls for any of the four handle families the test checks -- the assertions would have failed against that code. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-433)`/`Root cause`/`Pitfall` 3-field source comment; 5-section test doc comment on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `gbuffer.rs`'s `GBuffer` impl block plus its own inline test module. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `GBuffer` had no delete path for
any of the VAO/framebuffer/depth-renderbuffer/attachment-texture handles; the new test's
post-drop existence checks are the direct, deterministic check for exactly that absence.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` | Added `impl Drop for GBuffer` with `Fix(BUG-433)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` | Added inline `mod tests::gbuffer_drop_frees_vao_framebuffer_depthbuffer_and_textures` (wasm32-gated). |
