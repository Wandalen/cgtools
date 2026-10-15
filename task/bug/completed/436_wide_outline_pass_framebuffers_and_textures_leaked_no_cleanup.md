# BUG-436: `WideOutlinePass` leaks 4 framebuffers and 4 owned textures -- no cleanup path of any kind

- **Severity:** Low (no crash, no visual corruption -- an unbounded GPU-memory leak across
  repeated construct/drop cycles, e.g. a canvas resize that rebuilds the outline pipeline at a
  new resolution)
- **state:** Completed
- **Affects:** Every consumer of `renderer::webgl::post_processing::outline::WideOutlinePass`
  that constructs and drops more than one instance over the application's lifetime.
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/outline/wide_outline.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-432/433/437/438/440, found in the same sweep; each
  fixed and filed independently.

## Symptom

`WideOutlinePass` creates 4 framebuffers and 4 owned intermediate textures across its
construction/rendering pipeline (a 5th texture, `object_color`, is supplied by and belongs to
the caller) -- but had no `gl_resources_free` method and no `impl Drop`, unlike sibling passes in
the same module (`SwapFramebuffer`, `UnrealBloomPass`) that already had one or both.

## Impact

**Who is affected:** Any consumer that constructs more than one `WideOutlinePass` over the
application's lifetime -- e.g. a canvas resize that rebuilds the outline pipeline at a new
resolution.

**What breaks:** No immediate visual/functional symptom -- cumulative GPU memory pressure that
only shows up after enough construct/drop cycles.

**Magnitude:** 4 framebuffers + 4 owned textures per construct/drop cycle.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433 -- cross-referencing
every GPU-resource-owning struct in `post_processing/` against whether it has a matching
`gl.delete_*`/`Drop` path. `WideOutlinePass`'s sibling passes in the same module already had a
`gl_resources_free` and/or `Drop`; `WideOutlinePass` had neither.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs, mod tests
let gl = gl_init();
let object_color = texture_make( &gl, 8, 8 );
let mut pass = WideOutlinePass::new( &gl, object_color.clone(), 3.0, 8, 8 ).unwrap();
let framebuffers : Vec< _ > = pass.framebuffers.values().cloned().collect(); // 4 framebuffers
let owned_textures : Vec< _ > = pass.textures.iter()
  .filter( | ( name, _ ) | *name != "object_color" ).map( | ( _, t ) | t.clone() ).collect();
pass.gl_resources_free( &gl );
// pre-fix: gl_resources_free did not exist as a method at all (hard compile error).
for fb in &framebuffers { assert!( !gl.is_framebuffer( Some( fb ) ) ); }
for t in &owned_textures { assert!( !gl.is_texture( Some( t ) ) ); }
assert!( gl.is_texture( Some( &object_color ) ) ); // caller-owned, must remain live
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- wide_outline_pass_gl_resources_free_frees_owned_resources_but_not_object_color
```

## Root Cause

No cleanup path existed at all for this struct's owned GL resources -- neither a manual
`gl_resources_free` nor an `impl Drop` backstop, unlike sibling passes in the same module
(`SwapFramebuffer`, `UnrealBloomPass`) that already have one or both.

## Why Not Caught

`WideOutlinePass` had no prior test coverage of its construction/destruction lifecycle --
existing tests (`tests/webgl/wide_outline.rs`) exercise its rendering output, not resource
teardown.

## Fix Location

`module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`: added
`pub fn gl_resources_free(&mut self, gl: &GL)`, deleting the 4 owned framebuffers and 4 owned
intermediate textures -- explicitly excluding `object_color`, which is supplied by and remains
owned by the caller via the `object_color_texture` constructor parameter.

## Prevention

New inline test `wide_outline_pass_gl_resources_free_frees_owned_resources_but_not_object_color`
in `wide_outline.rs`'s `#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (inline
because it needs the private fields -- see `rulebook.md § Test placement`), reusing the
`texture_make()`/construction pattern already established by `tests/webgl/wide_outline.rs`.
Captures all 4 framebuffer and 4 owned-texture handles before the call, asserts all 8 are
deleted afterward, and additionally asserts the caller-supplied `object_color` texture is *still*
a live GL object -- directly guarding the ownership-boundary distinction the fix's Pitfall calls
out.

## Pitfall

`object_color`'s presence in the same `textures` map as the 4 owned intermediate textures makes
"delete everything in `textures`" the wrong rule -- that texture is supplied by the caller via
the `object_color_texture` constructor parameter and remains the caller's to free; deleting it
here would be a use-after-free the moment the caller's own copy of the handle is next used. Any
future edit to this method must preserve that exclusion explicitly, not derive it implicitly
from map membership.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `WideOutlinePass::gl_resources_free`, freeing the 4 owned framebuffers and 4 owned textures while explicitly excluding the caller-owned `object_color`; added `Fix(BUG-436)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; reuses `texture_make()` construction pattern already proven by `tests/webgl/wide_outline.rs`. Adversarial pass: confirmed by direct inspection that pre-fix `WideOutlinePass` had zero delete calls for any of the 4 framebuffers or 4 owned textures, and separately confirmed the fix's exclusion list correctly omits `object_color` (the test's dedicated `is_texture` assertion on `object_color` after the call would catch a regression that deleted it too). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-436)`/`Root cause`/`Pitfall` 3-field source comment; 5-section test doc comment on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `wide_outline.rs`'s `WideOutlinePass` impl block plus its own inline test module. | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `WideOutlinePass` had no
`gl_resources_free` method and no `Drop` impl at all (the method literally did not exist, a hard
compile error for any caller attempting to call it); the new test's post-call existence checks
on all 8 owned handles, plus the dedicated caller-ownership check on `object_color`, are the
direct, deterministic check for the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` | Added `WideOutlinePass::gl_resources_free` with `Fix(BUG-436)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` | Added inline `mod tests::wide_outline_pass_gl_resources_free_frees_owned_resources_but_not_object_color` (wasm32-gated). |
