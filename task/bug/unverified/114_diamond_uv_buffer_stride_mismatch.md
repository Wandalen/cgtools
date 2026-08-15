# BUG-114: `diamond` example's uv `BufferDescriptor` uses `.stride( 3 )` on tightly-packed `[f32;2]` texture-coordinate data

- **Severity:** High
- **state:** Unverified
- **Affects:** `examples/minwebgl/diamond`'s sole rendering path — every `trunk serve` run of this example uploads and binds its uv attribute with a stride mismatched to the underlying buffer
- **Component:** `examples/minwebgl/diamond` (`src/main.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/

## Symptom

```rust
// examples/minwebgl/diamond/src/main.rs:122-124
gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer )?;
gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 1, &normal_buffer )?;
gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;
//                        ^^^^^^^^^^^^^^^          ^^^ wrong — copied from the 3-component lines above;
//                        2-component data                uv_buffer is tightly packed at 2 floats/vertex
```

**Predicted failure** (byte math, re-derived fresh from `module/min/minwebgl/src/buffer.rs:162-211`'s
`attribute_pointer` — the same source location task 097 used to confirm the byte-identical `obj_load`
bug): `attribute_pointer` passes `self.stride * sz` as WebGL's `vertexAttribPointer` stride argument in
bytes, where `sz = self.vector.scalar.byte_size()` = 4 for `f32`. `.stride( 3 )` on `[f32;2]` data means
WebGL is told each uv vertex is `3 * 4 = 12` bytes apart, but `uv_buffer` — uploaded directly from
`Vec<[f32;2]>` via `gl::buffer::upload`, tightly packed — actually has only `2 * 4 = 8` bytes per vertex.
WebGL validates `offset + stride*(count-1) + element_size ≤ buffer.byteLength` at draw time; substituting
stride=12 against an 8-bytes/vertex buffer violates this for any mesh with ≥2 vertices. Expected error
class: `GL_INVALID_OPERATION: glDrawElements: Vertex buffer is not big enough` — the exact error task 097
observed live for the byte-identical `obj_load` defect before its fix.

**Verify Command** (not yet run against this crate — see `## Impact`):
```bash
cd examples/minwebgl/diamond
trunk serve --release
# open the served URL in a browser, check devtools console
```
**Expected:** diamond model renders with skybox reflection/refraction, no console errors.
**Actual:** not yet empirically confirmed for this crate — see `## How Discovered` and `## Impact`.

## Impact

**Who is affected:** Anyone running the `diamond` example (`cd examples/minwebgl/diamond && trunk serve`)
— the sole example demonstrating cubemap-reflective/refractive gemstone rendering.

**What breaks:** Per the byte math above, the uv attribute's vertex stride is registered as 1.5× the
buffer's actual per-vertex size. Silent vs. loud: WebGL's own bounds validation makes this a loud failure
(`GL_INVALID_OPERATION` on the draw call) for any mesh with ≥2 vertices — a gltf-sourced diamond/gem mesh
will have far more than 2 vertices, so the example is expected to fail to draw at all, not merely render
with distorted UVs.

**Magnitude:** Every invocation of this example, unconditionally — the defect is in one-time buffer/VAO
setup code that runs on every load, not input- or data-dependent beyond "more than 1 vertex."

**Not yet live-confirmed for this crate specifically** — this filing is based on (a) direct re-derivation
of the byte math against current `buffer.rs` source, and (b) exact structural identity with `obj_load`'s
already-fixed, live-confirmed defect (task 097: same `BufferDescriptor::new::<[f32;2]>().stride(3)`
pattern, same causal mechanism, same predicted error class). A live browser reproduction specifically for
`diamond` has not been performed this session — filing captures the code-level defect now rather than
deferring it; live confirmation is left to whoever verifies/fixes this bug.

## How Discovered

Not discovered via this session's own investigation of `diamond` — surfaced as a byproduct of task 097's
round-3 adversarial pass (fixing the identical defect in `obj_load`), which grepped the workspace for the
same pattern:

```bash
$ grep -rn '\[ *f32; *2 *\]>().stride( *3 *)' examples/
examples/minwebgl/obj_load/src/main.rs:48     (fixed by task 097)
examples/minwebgl/diamond/src/main.rs:124     (this bug — left unfixed, out of task 097's scope)
```

Task 097 explicitly flagged this second occurrence as "a separate, likely-real latent bug in a sibling
example" rather than silently fixing or silently dropping it (see
`task/completed/097_obj_load_adopt_existing_helpers.md`'s Round 3 adversarial-pass notes). This filing is
that disclosed follow-up, re-verified independently against current `main.rs`/`buffer.rs` source this
session rather than taken on trust from task 097's own narrative alone.

## Fix Location

`examples/minwebgl/diamond/src/main.rs:124`:

```rust
// Before:
gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;

// After:
gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;
```

Single scalar argument change (3→2), confined to the uv attribute's own call — identical shape to task
097's fix in `obj_load/src/main.rs:48`. Position/normal attributes (lines 122-123) are unaffected; both
already correctly use `.stride( 3 )` against genuinely 3-component tightly-packed data.

**Verification (for whoever picks this up):** live browser run (`trunk serve --release` in
`examples/minwebgl/diamond/`) before and after, confirming (a) pre-fix reproduces a console error (or
otherwise confirms/disconfirms the predicted `GL_INVALID_OPERATION`), and (b) post-fix the diamond renders
with no console errors, matching the example's documented refraction/reflection appearance
(`showcase.webp`).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Filed as a disclosed follow-up from task 097's round-3 adversarial pass; root cause identified via byte-math re-derivation and structural match to the already-fixed `obj_load` defect, not yet live-reproduced for this crate. |
