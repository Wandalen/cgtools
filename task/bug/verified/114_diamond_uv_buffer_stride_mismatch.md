# BUG-114: `diamond` example's uv `BufferDescriptor` uses `.stride( 3 )` on tightly-packed `[f32;2]` texture-coordinate data

- **Severity:** High
- **state:** Verified
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
**Actual:** Two live attempts made 2026-08-15 — both inconclusive, see `## Impact`'s live-verification-attempt
notes. The stride hypothesis itself remains neither confirmed nor refuted.

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

**Live-verification attempt (2026-08-15) — inconclusive, environment gap:** `trunk serve --release` was
launched for this crate and reached a real browser (`browsee`, chromium, port 45231) successfully — build
succeeded, page served 200. But the page crashed before reaching the buffer/attribute-pointer code at all:
`panicked at examples/minwebgl/diamond/src/main.rs:226:49: called \`Result::unwrap()\` on an \`Err\` value:
DomError(ContextRetrievingError("No webgl2 context"))` — `gl::context::retrieve_or_make()` (main.rs:94)
never obtained a WebGL2 context, so execution never reached line 124. A control test against the
already-fixed, already-known-working `obj_load` crate (served from its own prebuilt `dist/`) hit the
**identical** `ContextRetrievingError("No webgl2 context")` panic in the same browsee session — strongly
indicating (single control test, same shared `gl::context::retrieve_or_make` library call in both crates,
not application code) the failure is an environment-wide WebGL2-context-acquisition gap in that particular sandboxed
browser/X11/GPU session (plausibly swiftshader/GPU resource contention from other concurrent browser
processes on this shared host), not anything specific to `diamond`'s code. This neither confirms nor
refutes the stride hypothesis — Dimension 2 (MRE Validity & Reproducibility) of the bug VERIFY Gate cannot
be marked PASS from this attempt; the byte-math argument above remains the only evidence for this bug.
State is left at Unverified rather than advanced, per `bugs/file.rulebook.md § Report New Bug : Step 9 -
VERIFY Gate` Substep 5's outcome (b) (tool/environment unavailable → surface the gap, never fabricate a
verdict). Whoever next attempts live verification should first confirm a real WebGL2 context is obtainable
in their environment at all (e.g. via a known-good example) before treating a blank canvas as confirmation
of this bug.

**Second live-verification attempt (2026-08-15) — same failure, "other browsee sessions" hypothesis ruled
out:** Re-attempted after first confirming via `browsee .list` that every previously-launched browsee
session on this host was already `dead` — i.e., no other browsee-managed browser held a live GPU context
at launch time, which the first attempt's causal guess above hadn't ruled out. A fully fresh `trunk serve
--release` (port 47331) plus a fresh `browsee` chromium session against a clean URL hit the
**byte-identical** panic: `DomError(ContextRetrievingError("No webgl2 context"))` at the same
`main.rs:226:49` site. Since no other browsee session was alive this time, sibling-browsee-session
contention specifically is ruled out as the cause. The host was running very high general load at the time
(`uptime` load average ~8.3/8.8/9.7, 34 concurrent `claude` processes, many active `cargo test`/build
jobs per `ps`) — consistent with broader host-level resource exhaustion affecting GPU/EGL context
acquisition, but this session did not directly test that narrower hypothesis either (e.g. by freeing host
load and retrying under confirmed-light conditions); it is the best available explanation given what's now
ruled out, not a directly confirmed one. Two independent, freshly-cleaned attempts reproducing the
identical error is a reasonable stopping point — a third immediate attempt was judged unlikely to add new
information and was not made. Both browsee session and trunk server were torn down and confirmed dead
afterward (`browsee .list`, `ss -ltn`). Whoever retries next should do so when overall host load is
confirmed materially lower, not merely when other browsee sessions are dead.

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

## Minimum Reproducible Example

```bash
mkdir -p /tmp/mre114
cat > /tmp/mre114/repro.html <<'HTML'
<!DOCTYPE html>
<html><head><title>mre114</title></head>
<body>
<canvas id="c" width="64" height="64"></canvas>
<script>
function compile(gl, type, src) {
  const s = gl.createShader(type);
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    console.log('MRE114: SHADER_COMPILE_FAIL ' + gl.getShaderInfoLog(s));
  }
  return s;
}

function drawWithStride(gl, loc, byteStride, label) {
  gl.getError(); // clear
  gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, byteStride, 0);
  gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0); // 2 triangles = 4 distinct vertices via index buffer
  const err = gl.getError();
  console.log('MRE114: ' + label + ' byteStride=' + byteStride + ' err_after_drawElements=' + err + ' (INVALID_OPERATION=' + gl.INVALID_OPERATION + ', NO_ERROR=' + gl.NO_ERROR + ')');
}

function run() {
  const canvas = document.getElementById('c');
  const gl = canvas.getContext('webgl2');
  if (!gl) { console.log('MRE114: no_webgl2_context'); return; }
  console.log('MRE114: context_ok version=' + gl.getParameter(gl.VERSION));

  // Tightly-packed [f32;2] uv data, 4 vertices -> 8 bytes/vertex, 32 bytes total (mirrors diamond's uv_buffer).
  const uvData = new Float32Array([0,0, 1,0, 1,1, 0,1]);
  const uvBuf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, uvBuf);
  gl.bufferData(gl.ARRAY_BUFFER, uvData, gl.STATIC_DRAW);
  console.log('MRE114: uv_buffer_bytes=' + uvData.byteLength);

  // Index buffer: 2 triangles over a 4-vertex quad (indices 0..3), same shape as a real indexed mesh draw.
  const indices = new Uint16Array([0,1,2, 0,2,3]);
  const idxBuf = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, idxBuf);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);

  const vs = compile(gl, gl.VERTEX_SHADER, '#version 300 es\nin vec2 p; void main(){ gl_Position = vec4(p,0.0,1.0); }');
  const fs = compile(gl, gl.FRAGMENT_SHADER, '#version 300 es\nprecision mediump float; out vec4 o; void main(){ o = vec4(1.0); }');
  const prog = gl.createProgram();
  gl.attachShader(prog, vs); gl.attachShader(prog, fs); gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    console.log('MRE114: PROGRAM_LINK_FAIL ' + gl.getProgramInfoLog(prog));
    return;
  }
  gl.useProgram(prog);
  const loc = gl.getAttribLocation(prog, 'p');
  console.log('MRE114: attrib_location=' + loc);
  gl.enableVertexAttribArray(loc);
  gl.bindBuffer(gl.ARRAY_BUFFER, uvBuf);

  // Buggy: BUG-114's actual defect — stride arg 3 elements -> byte stride 3*4=12, but buffer only has 8 bytes/vertex.
  drawWithStride(gl, loc, 3 * 4, 'buggy_stride');
  // Fixed: stride arg 2 elements -> byte stride 2*4=8, matches actual tightly-packed layout.
  drawWithStride(gl, loc, 2 * 4, 'fixed_stride');

  // Control A: getError() plumbing itself, isolated from any buffer/draw path — must report INVALID_VALUE.
  gl.getError();
  gl.vertexAttribPointer(loc, 5, gl.FLOAT, false, 8, 0);
  console.log('MRE114: control_A_invalid_size err=' + gl.getError() + ' (INVALID_VALUE=' + gl.INVALID_VALUE + ')');
  gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 8, 0);

  // Control B: 252-byte stride (<=255 WebGL cap, 4-aligned) against the 32-byte buffer -> needs 764 bytes,
  // a 24x unambiguous overrun. If this also reports NO_ERROR, the backend performs no drawElements-time
  // vertex-buffer bounds check at all (see Actual block below).
  drawWithStride(gl, loc, 252, 'control_B_max_stride_overrun');
}
run();
</script>
</body></html>
HTML
# Serve and execute via any WebGL2-capable browser automation tool, e.g. this repo's own `browsee`:
browsee .run /tmp/mre114/repro.html
# NOTE: snap-packaged browsers deny /tmp reads (AppArmor) — if using browsee under a snap install,
# copy repro.html to a non-hidden $HOME path first (browsee/docs/pitfall/002_snap_confinement.md).
```

**Expected** (per WebGL2's vertex-buffer bounds validation, and the loud `GL_INVALID_OPERATION` task 097 observed live for the byte-identical `obj_load` defect — see `## Evidence Table` E6):
```
MRE114: buggy_stride byteStride=12 err_after_drawElements=1282 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: fixed_stride byteStride=8 err_after_drawElements=0 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: control_A_invalid_size err=1281 (INVALID_VALUE=1281)
MRE114: control_B_max_stride_overrun byteStride=252 err_after_drawElements=1282 (INVALID_OPERATION=1282, NO_ERROR=0)
```

**Actual** (this session, headless Chromium via `browsee .run`, 2026-08-16 — see `## Evidence Table` E7):
```
MRE114: context_ok version=WebGL 2.0 (OpenGL ES 3.0 Chromium)
MRE114: uv_buffer_bytes=32
MRE114: attrib_location=0
MRE114: buggy_stride byteStride=12 err_after_drawElements=0 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: fixed_stride byteStride=8 err_after_drawElements=0 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: control_A_invalid_size err=1281 (INVALID_VALUE=1281)
MRE114: control_B_max_stride_overrun byteStride=252 err_after_drawElements=0 (INVALID_OPERATION=1282, NO_ERROR=0)
[console] Automatic fallback to software WebGL has been deprecated. Please use the --enable-unsafe-swiftshader flag...
```
`control_A` fired correctly (proves `getError()` itself works in this sandbox); `control_B`'s 24x unambiguous overrun (764 bytes needed vs. 32 available) *also* returned `NO_ERROR` — this sandbox's software-rendered (SwiftShader-fallback) WebGL2 backend performs no `drawElements`-time vertex-buffer bounds validation at all, independent of BUG-114's specific stride value. `buggy_stride`'s `err=0` is this environment characteristic, not evidence the underlying stride mismatch is harmless — see `## Root Cause` and `## Generalized Version`.

**Actual** (Firefox instead of Chromium, `browsee .launch browser::firefox` + `.console`, 2026-08-17 — see `## Evidence Table` E9), re-run twice (once pre-compaction, once fresh in this same session, byte-identical results both times):
```
MRE114: context_ok version=WebGL 2.0
MRE114: uv_buffer_bytes=32
MRE114: attrib_location=0
MRE114: buggy_stride byteStride=12 err_after_drawElements=1282 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: fixed_stride byteStride=8 err_after_drawElements=0 (INVALID_OPERATION=1282, NO_ERROR=0)
MRE114: control_A_invalid_size err=1281 (INVALID_VALUE=1281)
MRE114: control_B_max_stride_overrun byteStride=252 err_after_drawElements=1282 (INVALID_OPERATION=1282, NO_ERROR=0)
```
Exact line-for-line match to this section's own `Expected` block above: `buggy_stride` correctly raises `GL_INVALID_OPERATION`, `fixed_stride` correctly raises `NO_ERROR`, `control_A` confirms `getError()` plumbing, and — critically — `control_B`'s same 24x overrun that Chromium/SwiftShader let through now correctly raises `GL_INVALID_OPERATION`. Firefox's software WebGL2 fallback is architecturally distinct from Chromium's SwiftShader path (`browsee/docs/pitfall/004_webgl2_blocklist_fatal.md`: "Firefox needs nothing: its software fallback engages automatically") and, unlike it, genuinely performs `drawElements`-time vertex-buffer bounds validation — this is a trustworthy, non-backend-artifact reproduction of both the original defect and the fix's correctness. Resolves Dimension 2's prior blocking finding (see `## History`).

**Verify Command:**
```bash
grep -q '\[ f32; 2 \] >().stride( 2 )' examples/minwebgl/diamond/src/main.rs && echo PASS || echo FAIL
```
**What:** Statically asserts the uv `BufferDescriptor`'s `.stride()` argument equals its vector's own element count (2) — the invariant whose violation caused this bug. Deliberately a static source check, not a live GL-error check: this session's own MRE execution (above) proved this sandbox's software WebGL backend does not raise `GL_INVALID_OPERATION` for this violation, making a live-error CI check unreliable across backends.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The uv `BufferDescriptor`'s `.stride(3)` is a copy-paste error from the preceding 3-component position/normal lines, telling WebGL 12 bytes/vertex against the uv buffer's actual 8 bytes/vertex. | ✅ Root Cause | `main.rs:131` (pre-fix) passed `.stride(3)` for `[f32;2]` data; `buffer.rs:200` multiplies by `sz=4`, giving byte stride 12 vs. actual 8. | E3, E6, E8, E9, E10 |
| H2 | `attribute_pointer` (the shared library helper) itself miscalculates the byte-stride argument passed to WebGL, independent of caller input. | ❌ Disproved | `buffer.rs:200`'s `self.stride * sz` is a straightforward multiply; `buffer.rs:164`'s `sz` derives only from the vector's scalar type; sibling call sites (`main.rs:122-123`) using the same function with correct stride values produce correct byte strides. | E3, E4, E5 |
| H3 | `uv_buffer`'s actual per-vertex layout has a padding/extra component, making a 3-element stride the objectively correct value (i.e. the buffer, not the stride argument, is wrong). | ❌ Disproved | `main.rs:62` types `tex_coords` as `Vec<[f32;2]>`; `main.rs:83` collects glTF tex-coord data directly into that 2-component type with no padding. | E1, E2 |
| H4 | Any spec-compliant WebGL2 implementation always validates vertex-buffer bounds at draw time, so a live browser test that raises no error refutes the stride-mismatch defect. | ❌ Disproved | This session's controlled MRE run (`/tmp/mre114/repro.html` via `browsee .run`) shows a deliberate 24x buffer overrun also returns `NO_ERROR` in this sandbox's software-rendered WebGL backend, while an unrelated control call correctly raises `INVALID_VALUE` — proving the backend itself, not the test harness, is permissive for this validation class. | E7 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `examples/minwebgl/diamond/src/main.rs:62` | `type Geometry = ( Vec<[f32;3]>, Vec<[f32;3]>, Vec<[f32;2]>, Vec<u32> )` — `tex_coords` is declared as tightly-packed 2-component data, no third/padding component. | H3 ❌ |
| E2 | `examples/minwebgl/diamond/src/main.rs:83` | `let tex_coords = tex_iter.into_f32().collect();` collects glTF's own tex-coord reader output directly into the `Vec<[f32;2]>` slot — runtime data matches the declared 2-component layout. | H3 ❌ |
| E3 | `module/min/minwebgl/src/buffer.rs:200` | In the `nelements()==1` branch (the one a `[f32;2]` vector takes), `self.stride * sz` is passed as WebGL's byte-stride argument; for `main.rs:131`'s pre-fix `.stride(3)` and `sz=4`, this evaluates to 12. | H1 ✅, H2 ❌ |
| E4 | `module/min/minwebgl/src/buffer.rs:164` | `let sz = self.vector.scalar.byte_size();` — `sz` derives solely from the vector's own scalar type, never from `self.stride`; the library has no independent path that could miscalculate the stride argument itself. | H2 ❌ |
| E5 | `examples/minwebgl/diamond/src/main.rs:122-123` | Sibling `[f32;3]` position/normal `BufferDescriptor`s use `.stride(3)` against genuinely 3-component tightly-packed data through the same `attribute_pointer` function and render correctly — isolates the defect to the uv line's own argument, not the shared function. | H2 ❌ |
| E6 | `examples/minwebgl/obj_load/src/main.rs:48` | The byte-identical `.stride(2)` fix, already applied and structurally verified for the same `[f32;2]` uv-attribute pattern via the same `attribute_pointer` call chain (task 097). | H1 ✅ (symptom) |
| E7 | `/tmp/mre114/repro.html` console output, this session (`browsee .run`, 2026-08-16) | Isolated MRE reproducing diamond's exact draw shape (indexed `drawElements`, tightly-packed `[f32;2]` 32-byte buffer) returns `NO_ERROR` for `byteStride=12` (buggy) AND for a 24x overrun positive control (`byteStride=252`, needs 764 bytes); an unrelated isolated call (`vertexAttribPointer` size=5) correctly raises `INVALID_VALUE`, proving `getError()` itself functions — this sandbox's software/SwiftShader WebGL backend performs no `drawElements`-time buffer-bounds validation at all. | H4 ❌ |
| E8 | `examples/minwebgl/diamond/dist/` rebuilt this session (`trunk build --release`, exit 0) + `browsee .shot` screenshot, post-fix | Live pixel-verified render of the post-fix example shows a correctly-shaded faceted diamond gem against a non-blank background (rgb 248,248,250 at center), no console errors — confirms the applied `.stride(2)` fix produces correct visible output in this same sandbox. | H1 ✅ (symptom) |
| E9 | `/home/user1/mre114/repro.html` console output (byte-identical to `## Minimum Reproducible Example`'s documented script, diff-verified), Firefox software WebGL2 backend via `browsee .launch browser::firefox` + `.console`, 2026-08-17 | Exact line-for-line match to the MRE's own `Expected` block: `buggy_stride` raises `GL_INVALID_OPERATION`, `fixed_stride` raises `NO_ERROR`, `control_A` raises `INVALID_VALUE`, and `control_B`'s 24x overrun ALSO correctly raises `GL_INVALID_OPERATION` — proving Firefox's backend (unlike Chromium/SwiftShader, see E7) performs genuine `drawElements`-time bounds validation, so this reproduction is trustworthy rather than a backend artifact. Resolves Dimension 2's prior blocking finding. | H1 ✅ (live reproduction), H4 (unaffected — still disproved; Chromium's permissiveness alone already refuted "any" backend always validates) |
| E10 | `examples/minwebgl/diamond`'s existing post-fix `dist/` build (unchanged since E8, confirmed newer than `src/main.rs`'s last edit), served locally and rendered via Firefox, 2026-08-17 | Clean render — faceted diamond gem with skybox-style reflection, visually matching the crate's own reference `showcase.webp` (same speckled-facet appearance, confirmed by direct comparison) — no console errors beyond harmless libEGL/GTK desktop-integration warnings. Corroborates E8's Chromium-based post-fix confirmation via a second, independent browser engine. | H1 ✅ (symptom, second engine) |

## Root Cause

```
main.rs:131 (pre-fix)      BufferDescriptor::new::<[f32;2]>().stride(3)     <- copy-pasted from
                                                                                pos/normal lines above
  |
  v attribute_pointer()  (buffer.rs:194-202, nelements()==1 branch)
buffer.rs:200               vertex_attrib_pointer_with_i32(..., self.stride * sz, ...)
                             = 3 * 4 = 12 bytes/vertex told to WebGL
  |
  v  but actual uv_buffer layout
main.rs:62,83 + buffer.rs:41-46   tex_coords: Vec<[f32;2]>, uploaded via Data::as_bytes()
                                    = 2 * 4 = 8 bytes/vertex actually present, tightly packed
  |
  v
WebGL told to read 12 bytes/vertex from a buffer that only has 8 bytes/vertex per the
`offset + stride*(count-1) + element_size <= buffer.byteLength` bound WebGL2 itself defines.
```

Traceable to H1 (✅ Root Cause): the uv `BufferDescriptor`'s `.stride(3)` was copy-pasted from the two preceding `[f32;3]` position/normal lines (`main.rs:122-123`) without adjusting for uv's own 2-component layout. `attribute_pointer` (`buffer.rs:200`) faithfully multiplies whatever `stride` it's given by the scalar byte size — the library function itself is correct (H2, disproved via E3-E5); the mismatch originates entirely at the caller's argument choice, not the shared helper.

## Why Not Caught

`examples/minwebgl/diamond` has no `tests/` directory and no automated coverage of any kind (`find examples/minwebgl/diamond -iname '*test*'` returns nothing) — no test exercises the crate's vertex-attribute setup at all. No invariant asserts, for any `BufferDescriptor::new::<[f32;N]>().stride(S)` call uploaded from a tightly-packed `Vec<[f32;N]>`, that `S == N` — the exact check that would have caught this copy-paste error at review or CI time. Task 097's fix to the byte-identical `obj_load` defect corrected only that one call site, adding no general-purpose assertion or lint that would have caught this sibling occurrence too (see `## Prevention`).

## Fix Location

`examples/minwebgl/diamond/src/main.rs:131`  (was line 124 pre-fix, before the `Fix(BUG-114)` comment block at lines 124-130 shifted it):

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

## Prevention

**Safeguard:** when copy-pasting a `BufferDescriptor::new::<[f32;N]>()...stride(S)` chain for a new attribute, always set `S` from that attribute's OWN element count, never leave it copied from a neighboring attribute's chain.

**Detection command:**
```bash
grep -rn 'BufferDescriptor::new::< \[ f32; 2 \] >().stride( 3 )' examples/ module/
```
Flags any remaining `[f32;2]`-typed `BufferDescriptor` using a 3-element stride — the exact copy-paste shape both this bug and the already-fixed `obj_load` sibling took. Returns empty post-fix (both known occurrences corrected).

**Pitfall:** when copying a `BufferDescriptor` chain for a differently-sized attribute, `.stride()` must match that attribute's OWN element count, not the neighboring attribute's — matches the source comment at `main.rs:129-130` verbatim.

## Generalized Version

**Broken assumption:** that a `BufferDescriptor::new::<[f32;N]>().stride(S)` call is self-evidently consistent just because it compiles and follows the visual pattern of adjacent lines — the type parameter `N` and the `.stride(S)` argument are independently settable and nothing in the type system enforces `S == N` for tightly-packed data.

**Failure conditions:** any call site where (a) a `BufferDescriptor` is constructed by copy-pasting an adjacent chain for a differently-shaped attribute, and (b) the `.stride()` argument is left unchanged from the copied source. Applies to any element type/size combination, not just `[f32;2]` vs `[f32;3]`.

**Detection invariant:** for every `BufferDescriptor::new::<[f32;N]>()...stride(S)...` call site uploading from a tightly-packed `Vec<[f32;N]>` (no interleaving/padding), `S` must equal `N`. A lint or debug-assertion enforcing this at `attribute_pointer` call time (comparing the type parameter's own element count against the runtime `stride` field) would catch this entire defect class structurally, rather than relying on a case-by-case grep per known-bad pattern.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Filed as a disclosed follow-up from task 097's round-3 adversarial pass; root cause identified via byte-math re-derivation and structural match to the already-fixed `obj_load` defect, not yet live-reproduced for this crate. |
| 2026-08-15 | live-verify attempted (inconclusive) | `trunk serve` + browsee reached the page but hit `ContextRetrievingError("No webgl2 context")` before the buffer code ran; control test against known-good `obj_load` hit the identical error, strongly indicating an environment-wide WebGL2-context gap in that browsee session, not a diamond-specific finding. State left at Unverified; stride hypothesis still untested live. |
| 2026-08-15 | live-verify retried (same result, cause narrowed) | Second attempt on a fresh trunk server + fresh browsee session, after confirming all prior browsee sessions were dead, hit the byte-identical `ContextRetrievingError`. Rules out "other browsee sessions" as the cause; points to broad host-level load (uptime ~8-9, 34 concurrent claude processes) as the more likely explanation, though not directly confirmed. Stopped after 2 clean attempts rather than retrying further. State left at Unverified; stride hypothesis still untested live. |
| 2026-08-16 | fix applied, live verification still blocked | Independently re-confirmed the byte-math root cause against current `module/min/minwebgl/src/buffer.rs:162-213` (`self.stride * sz` passed as WebGL's byte stride, `sz=4` for `f32`) and current `main.rs:124` (defect still present, unchanged from filing). Checked host load before considering a third live-verification attempt: `uptime` showed load average 34.27/29.10/29.16 with 90 concurrent `claude` processes and 39 concurrent `cargo test/build/nextest` jobs — materially *worse* than both prior attempts (~8.3 avg, 34 processes), so per this file's own guidance a third live attempt was deliberately not made (would very likely reproduce the same environment-caused `ContextRetrievingError` inconclusively, adding no information). Applied the documented Fix Location fix (`.stride( 3 )` → `.stride( 2 )` on the uv attribute only) with a `Fix(BUG-114)`/Root cause/Pitfall source comment; `cargo check`/`cargo clippy --all-features -- -D warnings` for `minwebgl_diamond` on `wasm32-unknown-unknown` both clean post-fix — the only verification achievable without a live WebGL2 context. Live pixel/console confirmation remains unavailable in this environment; state deliberately left at Unverified rather than fabricating a VERIFY Gate PASS for Dimension 2 (MRE Validity & Reproducibility), per `bugs/file.rulebook.md § Report New Bug : Step 9 - VERIFY Gate` Substep 5 outcome (b). Whoever next confirms a real WebGL2 context is obtainable (host load materially lower than 34.27) should complete the live before/after check this file's Fix Location section already specifies. |
| 2026-08-16 | report repaired, VERIFY Gate run | Repaired the report to full FI048 completeness (added Minimum Reproducible Example, Hypothesis Table, Evidence Table, Root Cause, Why Not Caught, Prevention, Generalized Version — previously only 5/12 required sections were present). Built a rigorous, iteratively-corrected positive-control MRE (`/tmp/mre114/repro.html`) proving this sandbox's software/SwiftShader WebGL2 backend performs no `drawElements`-time vertex-buffer bounds validation at all (not merely leniency toward this bug's specific 12-vs-8-byte mismatch): a deliberate 24x buffer overrun (`byteStride=252`) also returned `NO_ERROR`, while an isolated unrelated control (`vertexAttribPointer` size=5) correctly raised `INVALID_VALUE`, proving the test harness's own error plumbing works and the permissiveness is a genuine backend characteristic. Also confirmed live pixel-verified rendering of the post-fix example (`browsee .shot`, correctly-shaded diamond, no console errors). Ran the formal VERIFY Gate (8-dimension Tier 2 Dual-Role Self-Check) — see round 1 outcome below (superseded; former `## Verification Findings` content, removed on this round's PASS per `bugs/file.rulebook.md § Report New Bug : Step 9 - VERIFY Gate` Substep 3): 7/8 🟢, Dimension 2 (MRE Validity & Reproducibility) 🔴 BLOCKING — the MRE executed cleanly and a positive control (`control_B`, 24x overrun) also returned `NO_ERROR`, proving Chromium/SwiftShader's software WebGL2 backend performs no `drawElements`-time bounds validation at all; the predicted `GL_INVALID_OPERATION` therefore could not manifest regardless of whether the underlying defect is real. VERIFY_FAIL fired; bug stayed `❓ Unverified` per Substep 5 outcome (b) (environment capability gap, never fabricate a verdict). |
| 2026-08-17 | new evidence gathered (Firefox), VERIFY Gate re-run, PASS | Recreated the byte-identical MRE at `/home/user1/mre114/repro.html` (diff-verified against this file's own documented script; the original `/tmp/mre114/` copy had been cleaned up between sessions) and re-ran it via Firefox instead of Chromium (`browsee .launch browser::firefox` + `.console`), twice independently (once pre-compaction, once fresh within this same session, session `bug114_verify2`, byte-identical results both times) — see `## Minimum Reproducible Example`'s second `Actual` block and `## Evidence Table` E9. Firefox's software WebGL2 fallback is architecturally distinct from Chromium's SwiftShader path (`browsee/docs/pitfall/004_webgl2_blocklist_fatal.md`) and, unlike it, genuinely performs `drawElements`-time bounds validation — proven by `control_B`'s 24x overrun now correctly raising `GL_INVALID_OPERATION` where Chromium let it through. The full predicted symptom reproduced exactly: `buggy_stride` → `GL_INVALID_OPERATION`, `fixed_stride` → `NO_ERROR`. Also independently re-rendered the actual post-fix `diamond` example (existing `dist/`, unchanged since E8) via Firefox — clean render, visually matching the crate's own reference `showcase.webp` (see E10), no console errors. Re-ran the file's own static Verify Command (`grep` for `.stride( 2 )` on `main.rs`) fresh: exit 0/PASS. Re-ran the formal VERIFY Gate (8-dimension Tier 2 Dual-Role Self-Check): all 8 dimensions 🟢, Dimension 2's prior blocking finding resolved by the Firefox reproduction. VERIFY_PASS fired — see `## Verification Record`. State → `🎯 Verified`; file moved to `bug/verified/`. |

## Verification Record

**VERIFY Gate (2026-08-17) — Tier 2 Dual-Role Self-Check, 8 dimensions, verdict: PASS (8/8).** Supersedes round 1 (2026-08-16, FAIL on Dimension 2 — see `## History`'s 2026-08-16 entry for that round's full findings, preserved there per this gate's own Substep 3 instruction to remove `## Verification Findings` on PASS).

| Gate | Name | Prev | Now | Issues | Fixes |
|---|---|---|---|---|---|
| 1 | Completeness | 🟢 | 🟢 | — | — |
| 2 | MRE Validity & Reproducibility | 🔴 | 🟢 | — | Firefox re-run (session `bug114_verify2`) reproduced the full predicted symptom exactly, including `control_B`'s 24x overrun now correctly raising `GL_INVALID_OPERATION` — proves Firefox's backend performs genuine bounds validation, resolving round 1's Chromium/SwiftShader environment-gap blocker. |
| 3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| 4 | Root Cause Quality | 🟢 | 🟢 | — | — |
| 5 | Execution Scope | 🟢 | 🟢 | — | — |
| 6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| 7 | Crate Locality | 🟢 | 🟢 | — | — |
| 8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |

**Reproduced:** YES — exit 0 (static Verify Command, `grep` for `.stride( 2 )` on `main.rs`, 2026-08-17); live MRE console reproduction (Firefox, `browsee`, session `bug114_verify2`) exact line-for-line match to `## Minimum Reproducible Example`'s `Expected` block, same date.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/diamond/src/main.rs` | Line 131: uv `BufferDescriptor`'s `.stride( 3 )` → `.stride( 2 )`, matching its `[f32;2]` element count. `Fix(BUG-114)`/`Root cause`/`Pitfall` comment added at lines 124-130. |
