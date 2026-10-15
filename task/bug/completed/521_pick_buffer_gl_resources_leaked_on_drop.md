# BUG-521: `PickBuffer` allocates a framebuffer, id texture, and depth renderbuffer but never frees them on drop

- **Severity:** Low (no crash, no visual corruption -- a slow, unbounded GPU-memory leak that
  only manifests across repeated construct/drop cycles, e.g. resizing a picking-enabled canvas by
  constructing a new `PickBuffer` rather than calling `resize`, or tearing down and rebuilding a
  scene's picking pipeline)
- **state:** Completed
- **Affects:** Every consumer of `gpu_picking::PickBuffer` that constructs and drops more than one
  instance over the application's lifetime (a single long-lived instance that only ever calls
  `resize` never observes this specific leak, since `resize` already frees its own previous
  texture/renderbuffer before replacing them -- see Related Bugs).
- **Component:** `module/helper/gpu_picking` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Related Bugs:** Same defect *class* as BUG-432 (`renderer::webgl::ShadowBaker` leaked its
  framebuffer on drop) -- same fix shape (add an owned `gl: GL` field plus `impl Drop`), found
  independently in a different crate; no shared root cause beyond the general "GPU-resource-owning
  struct with no matching teardown path" pattern. Also found and fixed alongside BUG-530
  (`PickBuffer::pick` accepting out-of-range coordinates) and BUG-513 (`IdProgram::draw_part`
  accepting negative pick ids) during the same sweep of `gpu_picking`'s ~244-line `src/lib.rs`.

## Symptom

```rust
// pre-fix -- src/lib.rs, PickBuffer
pub struct PickBuffer
{
  framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
  id_texture : Option< gl::web_sys::WebGlTexture >,
  depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
  width : i32,
  height : i32,
  readback : gl::js_sys::Int32Array,
}
// no `impl Drop`, no manual free method anywhere in the struct or its impl block.
```

`PickBuffer` allocates exactly one `WebGlFramebuffer`, one `WebGlTexture`, and one
`WebGlRenderbuffer` per instance in `new`, and never deletes any of them, on any code path --
not on drop, not via any manual cleanup call, since none existed. (`resize` *does* correctly
delete the texture/renderbuffer it is about to replace -- but nothing ever freed the *last* set,
nor the framebuffer itself, on final teardown.)

## Impact

**Who is affected:** Any consumer that constructs more than one `PickBuffer` over the
application's lifetime -- e.g. recreating the picking pipeline on a major scene change, or any
code path that constructs a fresh `PickBuffer` instead of calling `resize` on an existing one.
Each construct/drop cycle leaks one more framebuffer, texture, and renderbuffer for the remaining
lifetime of the GL context.

**What breaks:** No immediate visual or functional symptom -- the leak is purely cumulative GPU
memory pressure, invisible until enough cycles accumulate to matter.

**Magnitude:** One `WebGlFramebuffer` + one `WebGlTexture` + one `WebGlRenderbuffer` per
`PickBuffer` construct/drop cycle.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking`, checking every
GL-resource-owning struct in the crate against whether it has a matching `gl.delete_*`/`Drop`
path -- explicitly prompted by this same defect class already found and fixed in
`renderer::webgl::ShadowBaker` (BUG-432) and its siblings (BUG-433/436/437/438/440). `PickBuffer`
had no `gl` field at all (needed to call `delete_*` from `Drop::drop`, which takes no
parameters), confirming no teardown path had ever been possible.

## Minimum Reproducible Example

```rust
// module/helper/gpu_picking/src/lib.rs, mod live_gl_tests (inline, wasm32-gated)
let gl = gl_init();
let buffer = PickBuffer::new( &gl, 4, 4 );

let framebuffer = buffer.framebuffer.clone();
let id_texture = buffer.id_texture.clone();
let depth_renderbuffer = buffer.depth_renderbuffer.clone();

assert!( gl.is_framebuffer( framebuffer.as_ref() ) );      // true right after construction
assert!( gl.is_texture( id_texture.as_ref() ) );            // true right after construction
assert!( gl.is_renderbuffer( depth_renderbuffer.as_ref() ) ); // true right after construction

drop( buffer );

assert!( !gl.is_framebuffer( framebuffer.as_ref() ) );      // pre-fix: still true -- never freed
assert!( !gl.is_texture( id_texture.as_ref() ) );            // pre-fix: still true -- never freed
assert!( !gl.is_renderbuffer( depth_renderbuffer.as_ref() ) ); // pre-fix: still true -- never freed
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_picking && cargo test -p gpu_picking --lib --target wasm32-unknown-unknown -- pick_buffer_drop_frees_gl_resources
```

## Root Cause

`PickBuffer` had no `impl Drop` at all, and no `gl: GL` field to call `delete_*` from even if one
were added. GPU handle wrapper types (`Option<WebGlFramebuffer>` etc.) are just JS-object
handles -- dropping the Rust value does not call `gl.delete*` for you; only an explicit delete
call does. `resize` deletes the texture/renderbuffer it is about to replace (so mid-life resizes
don't leak), but nothing ever freed the framebuffer or the final texture/renderbuffer set on
actual teardown.

## Why Not Caught

`PickBuffer` had zero prior test coverage of any kind -- nothing exercised its construction or
destruction, so a missing `Drop` impl produced no observable failure in CI or manual testing.

## Fix Location

`module/helper/gpu_picking/src/lib.rs`: added an owned `gl: GL` field (populated in `new`,
matching `renderer::webgl::ShadowBaker`'s identical precedent in this workspace -- see BUG-432)
plus `impl Drop for PickBuffer`, deleting `framebuffer`, `id_texture`, and `depth_renderbuffer`.

## Prevention

New inline test `pick_buffer_drop_frees_gl_resources` in `src/lib.rs`'s
`#[cfg(all(test, target_arch = "wasm32"))] mod live_gl_tests` block (placed inline, wasm32-only,
because it needs a live GL context and access to `PickBuffer`'s private fields before drop -- see
`rulebook.md § Test placement`). Captures clones of all three private GL handles right after
construction, asserts each is a live GL object, drops the `PickBuffer`, then asserts none of the
three are live any more -- the same deterministic `gl.is_*` existence-check pattern used by this
workspace's other GPU-teardown reproducer tests (e.g. BUG-432's `shadow_baker_drop_frees_framebuffer`).

## Pitfall

A GPU handle wrapper (`Option<WebGlTexture>` etc.) is just a JS-object reference -- letting the
Rust value go out of scope does not call `gl.delete*` for you; only an explicit delete call (here,
via `impl Drop`) reclaims the actual GPU-side allocation. Adding a new GL-resource field to this
struct in the future without also extending this `Drop` impl reintroduces the same leak for that
field.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed | Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking`. |
| 2026-08-21 | fixed | Added an owned `gl: GL` field and `impl Drop for PickBuffer`, matching `ShadowBaker`'s (BUG-432) existing pattern; added `Fix(BUG-521)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-21 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo test -p gpu_picking --lib --target wasm32-unknown-unknown -- pick_buffer_drop_frees_gl_resources` passes in headless Firefox via the workspace's `wasm_test_runner.sh`. Adversarial pass: temporarily emptied `impl Drop for PickBuffer`'s body (reintroducing the exact leak) and confirmed the test failed at runtime with `panicked at ... PickBuffer::drop must delete its framebuffer` (captured in the pre-fix run's own log), then reverted and reconfirmed green. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-521)`/`Root cause`/`Pitfall` 3-field format applied to the source comment at `src/lib.rs`; 5-section test doc comment applied to the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `PickBuffer`'s struct definition (new `gl` field), `PickBuffer::new` (one new field initializer), and the new `impl Drop` block plus its own inline test; no other file touched for this specific item. | — |

**Reproduced:** YES -- adversarial pass reintroduced the exact leak (emptied `impl Drop`'s body)
and confirmed `pick_buffer_drop_frees_gl_resources` failed at runtime in a real headless-Firefox
WebGL2 context with the expected panic message, then reverted and reconfirmed the fix passes.
2026-08-21.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added `gl: GL` field to `PickBuffer`, populated in `new`; added `impl Drop for PickBuffer` with `Fix(BUG-521)`/`Root cause`/`Pitfall` comment. |
| `module/helper/gpu_picking/Cargo.toml` | Added `wasm-bindgen-test` dev-dependency (crate had none before; needed for the new live-GL-context test). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added inline `mod live_gl_tests::pick_buffer_drop_frees_gl_resources` (wasm32-gated, live GL context, needs private-field access per `rulebook.md § Test placement`). |
