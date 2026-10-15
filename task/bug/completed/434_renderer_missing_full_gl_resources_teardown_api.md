# BUG-434: `Renderer` has no complete GPU-resource teardown path -- `resize()`'s cleanup covers only 3 of its 6+ owned resource groups

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak; only
  manifests when a `Renderer` (or its `blend_effect` field) is dropped, since nothing ever freed
  the resources this fix now covers)
- **state:** Completed
- **Affects:** Every consumer that drops a `Renderer` (e.g. tearing down a scene, navigating away
  from a render view) and expects its GPU resources to be reclaimed. Before this fix, no code
  path -- including `resize()`'s own narrower cleanup -- ever freed `blend_effect`,
  `compiled_programs`, `composite_shader`, or `skybox_shader`.
- **Component:** `module/helper/renderer` (`src/webgl/renderer.rs`,
  `src/webgl/post_processing/blend.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/433/436/437/438/440 (GPU-resource-owning
  structs with no matching teardown path), found in the same sweep. Distinct from those: this is
  an API-completeness gap in `Renderer` itself, not a missing `Drop` on a single leaf struct --
  `Renderer` deliberately has no `impl Drop` of its own (unlike the other 6 fixes), since the
  crate's convention for the top-level `Renderer` is an explicit, caller-invoked teardown method,
  not an implicit one.

## Symptom

Before this fix, `Renderer` had exactly one cleanup method, `resizable_resources_free`, invoked
only from inside `resize()`, which frees only the three fields `resize()` is about to recreate
(`framebuffer_ctx`, `bloom_effect`, `swap_buffer`). Nothing freed `blend_effect`'s compiled
program, any of `compiled_programs`' compiled material shaders, or the `composite_shader`/
`skybox_shader` programs -- on any code path, ever, including a full `Renderer` teardown.

## Impact

**Who is affected:** Any consumer dropping or tearing down a `Renderer` -- e.g. a scene switch,
navigating away from a render view, or an application shutdown path. Every such teardown leaked
one blend-pass program, every compiled material program, and the composite/skybox programs.

**What breaks:** No immediate visual/functional symptom -- cumulative GPU memory pressure that
only shows up after enough `Renderer` teardown cycles.

**Magnitude:** 1 (`blend_effect`'s program) + N (`compiled_programs`, one per distinct material)
+ 2 (`composite_shader`, `skybox_shader`) compiled WebGL programs per `Renderer` teardown.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433 -- auditing
`Renderer`'s full field list against its only existing cleanup method
(`resizable_resources_free`) surfaced that most of the struct's GPU-owning fields were outside
that method's scope, and no broader method existed to cover them.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/renderer.rs, mod tests (inline, wasm32-gated)
let gl = gl_init(); // requires EXT_color_buffer_float, per webgl_renderer_pass_cycle_test.rs
let mut renderer = Renderer::new( &gl, 64, 64, 4 ).unwrap();
let composite = renderer.composite_shader.program().clone();
let skybox = renderer.skybox_shader.program().clone();
renderer.gl_resources_free( &gl );
// pre-fix: gl_resources_free did not exist at all; even after adding it, an incomplete
// implementation would leave composite/skybox live.
assert!( !gl.is_program( Some( &composite ) ) );
assert!( !gl.is_program( Some( &skybox ) ) );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- gl_resources_free
```

## Root Cause

No method ever covered the *complete* set of GPU resources `Renderer` accumulates over its
lifetime. `resizable_resources_free` was written narrowly, scoped to exactly the three fields
`resize()` needs to recreate -- it was never intended as (and never grew into) a full-teardown
API, but no alternative full-teardown method existed either. `BlendPass` (owned by `Renderer` as
`blend_effect`) had the identical gap one level down: it compiled a program in `new` but had no
free method of its own for `Renderer::gl_resources_free` to delegate to.

## Why Not Caught

No test previously exercised a full `Renderer` teardown -- existing tests
(`webgl_renderer_pass_cycle_test.rs`) exercise the render/resize cycle, not resource reclamation
at end-of-life.

## Fix Location

- `module/helper/renderer/src/webgl/post_processing/blend.rs`: added
  `pub fn gl_resources_free(&mut self, gl: &GL)` to `BlendPass`, deleting `material`'s compiled
  program.
- `module/helper/renderer/src/webgl/renderer.rs`: added `pub fn gl_resources_free(&mut self, gl:
  &GL)` to `Renderer`, delegating to `resizable_resources_free` for the three already-covered
  fields, then additionally calling `self.blend_effect.gl_resources_free(gl)`, deleting every
  program in `compiled_programs`, and deleting `composite_shader`/`skybox_shader`'s programs.

## Prevention

Two new inline tests, one per fix site (both wasm32-gated, both needing private-field access --
see `rulebook.md § Test placement`):
- `renderer.rs`: `renderer_gl_resources_free_deletes_composite_and_skybox_programs` -- captures
  both program handles before the call, asserts both are deleted after.
- `blend.rs`: `blend_pass_gl_resources_free_deletes_material_program` -- same pattern, scoped to
  `BlendPass`'s own `material` program, since `renderer.rs`'s test cannot reach `BlendPass`'s
  private `material` field across the module boundary.

## Pitfall

When adding a new field that owns a GPU resource (a texture, a compiled program, a nested pass)
to a struct that already has a "cleanup" method, it is easy to assume that method is a complete
teardown path when it was actually written narrowly for a different purpose (here, resize-time
recreation) -- nothing enforces that every GPU-owning field gets added to every cleanup method
that exists.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `BlendPass::gl_resources_free` and `Renderer::gl_resources_free`; added `Fix(BUG-434)`/`Root cause`/`Pitfall` source comments and two inline reproducer tests. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; both tests reuse `Renderer::new(&gl,64,64,4)` / `EXT_color_buffer_float` construction pattern already proven by `webgl_renderer_pass_cycle_test.rs`. Adversarial pass: confirmed by direct inspection that neither `gl_resources_free` method existed pre-fix at all (a hard compile error, not merely a behavioral gap), which is the strongest possible confirmation the reproducer distinguishes pre/post-fix state. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-434)`/`Root cause`/`Pitfall` 3-field source comments at both fix sites; 5-section test doc comments on both reproducers. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `renderer.rs`'s `Renderer` impl block and `blend.rs`'s `BlendPass` impl block, plus their own inline test modules. | — |

**Reproduced:** YES -- pre-fix, `Renderer::gl_resources_free`/`BlendPass::gl_resources_free`
did not exist as methods at all (confirmed by direct source inspection), so any caller
attempting the reproducer's call sequence would fail to compile; post-fix, both methods exist
and the reproducers' post-call existence checks confirm they delete the targeted programs.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/blend.rs` | Added `BlendPass::gl_resources_free` with `Fix(BUG-434)`/`Root cause`/`Pitfall` comment. |
| `module/helper/renderer/src/webgl/renderer.rs` | Added `Renderer::gl_resources_free` with `Fix(BUG-434)`/`Root cause`/`Pitfall` comment, delegating to `resizable_resources_free` and `BlendPass::gl_resources_free`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/renderer.rs` | Added inline `mod tests::renderer_gl_resources_free_deletes_composite_and_skybox_programs` (wasm32-gated). |
| `module/helper/renderer/src/webgl/post_processing/blend.rs` | Added inline `mod tests::blend_pass_gl_resources_free_deletes_material_program` (wasm32-gated). |
