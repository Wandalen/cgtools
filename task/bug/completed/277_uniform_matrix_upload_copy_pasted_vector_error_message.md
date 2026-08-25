# BUG-277: `UniformMatrixUpload::matrix_upload` for `[f32]`/`[f32; N]` reports a copy-pasted "vector" error with wrong known-lengths on an unsupported matrix length

- **Severity:** Low (no memory-safety or silent-corruption defect -- an already-erroring code
  path returns a message with wrong content, actively misleading a caller debugging the error)
- **state:** Completed
- **Affects:** `minwebgl::uniform::UniformMatrixUpload::matrix_upload` for `[ f32 ]` and
  `[ f32 ; N ]` (`src/uniform/float32.rs`)
- **Component:** `module/min/minwebgl` (`src/uniform/float32.rs`, `src/uniform.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Both `UniformMatrixUpload::matrix_upload` implementations for `[ f32 ]` and `[ f32 ; N ]`
(`src/uniform/float32.rs`) build their "unsupported flat length" error via the catch-all `_`
match arm below the `4 | 9 | 16` cases. That arm constructed
`WebglError::CantUploadUniform( "vector", type_name_of_val( self ), self.len(), "1, 2, 3, 4" )`
-- item kind `"vector"` and known-lengths `"1, 2, 3, 4"` are `UniformUpload::upload`'s
*vector*-error literals, copy-pasted verbatim into the *matrix* upload path. A caller passing an
unsupported flat length (e.g. 3, 5, or 6 elements) to `matrix_upload` received a `Display`
message reading `"Cant upload uniform vector with ... Known length : [ 1, 2, 3, 4 ]"` -- wrong on
both fields, and actively misleading for a length like 3, which the message claims is valid yet
was still rejected.

## Impact

**Who is affected:** any caller of `UniformMatrixUpload::matrix_upload` (directly, or via
`minwebgl::uniform::matrix_upload`) that passes flat matrix data of an unsupported length (i.e.
not 4, 9, or 16 elements) and reads or logs the resulting `WebglError`'s message. The error is
still returned correctly as `Err(..)` -- control flow is unaffected -- only the message content
was wrong, so no example or existing test (none assert on this specific error's text) observed
incorrect behavior.

**What breaks:** `WebglError::CantUploadUniform`'s rendered `Display` message
(`"Cant upload uniform {0} with {1} of length {2}.\nKnown length : [ {3} ]"`, per
`src/context.rs`'s `#[error(...)]` derive) names the wrong item kind and lists the wrong set of
valid lengths, misleading anyone debugging a real matrix-upload failure.

**Entity Scope:** `None` -- source-level error-message defect, not entity directory instances.

## How Discovered

During this session's assigned review of `minwebgl`'s texture/uniform/shader/vao layer (17
files: `shader.rs`, `texture.rs`, `texture/{cube,d2}.rs`, `ubo.rs`, `uniform.rs`,
`uniform/{float32,int32,unsigned32}.rs`, `vao.rs`, `webgl.rs`, `web.rs`, plus 5 existing test
files read for style calibration). `int32.rs`/`unsigned32.rs` were confirmed to have no matrix
uniform variant (WebGL2 has no integer matrix uniforms), so `float32.rs`'s matrix path is the
only one with this error-construction shape; reading the two `matrix_upload` catch-all arms
side-by-side against `UniformUpload::upload`'s vector catch-all arm directly above them in the
same file made the copy-paste visible on inspection, then confirmed by checking
`WebglError::CantUploadUniform`'s field semantics in `src/context.rs` (item-kind string / valid
count-list string) against what a 2x2/3x3/4x4 matrix upload can actually accept.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl --all-features f32_matrix_length_error
```
**Expected** (fixed): both tests pass -- the constructed error reports item kind `"matrix"` and
known lengths `"4, 9, 16"`.
**Actual** (pre-fix, confirmed via temporary revert-and-rerun of only
`f32_matrix_length_error`'s body):
```
thread 'f32_matrix_length_error_reports_matrix_not_vector' panicked:
item kind must be "matrix", not the vector error's "vector" (len 0)
thread 'f32_matrix_length_error_display_mentions_matrix_and_valid_lengths' panicked:
message must mention "matrix", got: Cant upload uniform vector with &[f32; 5] of length 5.
Known length : [ 1, 2, 3, 4 ]
```

## Root Cause

`src/uniform/float32.rs` (pre-fix, both `[ f32 ]` and `[ f32 ; N ]` `matrix_upload` impls):
```rust
match self.len()
{
  4  => { gl.uniform_matrix2fv_with_f32_array( ... ); Ok( () ) },
  9  => { gl.uniform_matrix3fv_with_f32_array( ... ); Ok( () ) },
  16 => { gl.uniform_matrix4fv_with_f32_array( ... ); Ok( () ) },
  _ => Err
  (
    WebglError::CantUploadUniform
    (
      "vector",                    // <- copy-pasted from UniformUpload::upload's vector arm
      type_name_of_val( self ),
      self.len(),
      "1, 2, 3, 4",                 // <- ditto; matrix's real valid lengths are 4, 9, 16
    ),
  )
}
```
Both `matrix_upload` catch-all arms were written by copying `UniformUpload::upload`'s
vector-length catch-all arm (same file, directly above) and changing only the matched values (4,
9, 16 vs. 1, 2, 3, 4) -- the two string literals inside the error constructor were never updated
to match the new context. `WebglError::CantUploadUniform`'s constant string arguments carry no
compiler-enforced link to the surrounding match arm, so nothing caught the stale content.

## Why Not Caught

`matrix_upload` takes `&GL` (`web_sys::WebGl2RenderingContext`), which cannot be constructed
outside a browser, so no native `cargo test` run could previously call it to observe the error
text; no live-GL example in this repo exercises the catch-all error branch either (every real
caller passes an already-correctly-sized matrix). No existing test asserted on
`WebglError::CantUploadUniform`'s message content for this path.

## Fix Applied (2026-08-17)

**`src/uniform.rs`:** added a new pure helper function `f32_matrix_length_error( type_name,
len )` inside `mod private`, returning
`WebglError::CantUploadUniform( "matrix", type_name, len, "4, 9, 16" )` -- the corrected item
kind and known-lengths values. Exported it via `mod_interface!`'s `own use { upload,
matrix_upload, f32_matrix_length_error };` (alongside the pre-existing `upload`/`matrix_upload`
re-exports), making it reachable as `minwebgl::uniform::f32_matrix_length_error`, matching this
crate's existing pattern for exposing pure logic extracted for testability.

**`src/uniform/float32.rs`:** both `matrix_upload` catch-all arms (`[ f32 ]` and `[ f32 ; N ]`)
now call `f32_matrix_length_error( type_name_of_val( self ), self.len() )` instead of
constructing `WebglError::CantUploadUniform` inline with the stale literals. Added an explicit
`use crate::uniform::f32_matrix_length_error;` import -- `own use`-exposed items land at their
module path but do not bubble into the crate-root wildcard glob the file already imports (unlike
`prelude use`-marked items), so the explicit import is required.

**`tests/uniform_test.rs`** (new file): two new tests --
`f32_matrix_length_error_reports_matrix_not_vector` (sweeps 10 unsupported lengths, asserts item
kind `"matrix"` and known-lengths `"4, 9, 16"`) and
`f32_matrix_length_error_display_mentions_matrix_and_valid_lengths` (asserts the rendered
`Display` string contains `"matrix"` and `"4, 9, 16"`, and does not contain the old `"1, 2, 3,
4"`).

## Verification

`longrun`-detached, from the `minwebgl` crate directory:
- Pre-fix (temporary Edit-based revert of only `f32_matrix_length_error`'s body back to
  `WebglError::CantUploadUniform( "vector", type_name, len, "1, 2, 3, 4" )`, new tests left in
  place): both new tests fail with the exact messages shown in the Minimum Reproducible Example
  above. Restored to the fix immediately after confirming.
- Post-fix: `cargo test -p minwebgl --all-features`: 17 passed / 0 failed across all test
  binaries (`clean_test`, `data_type_test`, `diagnostics_test`, `drawbuffers_test`,
  `geometry_test`, `sprite_upload_test`, `uniform_test`) plus 1 passed doctest, 7 ignored
  (GL-context-bound tests requiring a browser).
- `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings`: clean, exit 0 (this
  included satisfying clippy's `#[must_use]` requirement on the new pure
  `f32_matrix_length_error` function).

## Generalized Version

**Broken assumption:** copying a sibling match arm's error-construction code as a starting point
for a new, structurally-similar arm (matrix catch-all from vector catch-all, same file, same
enum variant) is safe as long as the *matched values* are updated. But an error type built from
free-standing string/count arguments (like `WebglError::CantUploadUniform( kind, type_name, len,
known_lengths )`) has no type-level link between those arguments and the branch they describe --
copying the call site correctly updates control flow while silently carrying over stale
message content that only a reader, or a test asserting on the message itself, will ever catch.
This generalizes to any error-builder pattern using positional string/literal arguments rather
than a per-context constructor: `f32_matrix_length_error`'s extraction into its own named
function (this fix's approach) closes the gap by giving each error *context* exactly one
call site to get right, rather than N inlined copies that can drift independently.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's assigned review of `minwebgl`'s texture/uniform/shader/vao layer (17 files: `shader.rs`, `texture.rs`, `texture/{cube,d2}.rs`, `ubo.rs`, `uniform.rs`, `uniform/{float32,int32,unsigned32}.rs`, `vao.rs`, `webgl.rs`, `web.rs` + 5 existing test files read for style calibration; all other files confirmed clean, including a full 559/559 mechanical diff of `webgl.rs`'s constants against the actual `web-sys` 0.3.104 crate source). Root cause: both `UniformMatrixUpload::matrix_upload` catch-all error arms (`[ f32 ]` and `[ f32 ; N ]`, `src/uniform/float32.rs`) were copy-pasted from `UniformUpload::upload`'s vector-length catch-all arm in the same file, carrying over the literal item-kind `"vector"` and known-lengths `"1, 2, 3, 4"` strings unchanged -- both wrong for the matrix context (valid flat lengths are 4, 9, 16). Fixed by extracting a dedicated `f32_matrix_length_error` helper (`src/uniform.rs`) with the corrected literals, called from both catch-all arms. Verified via 2 new native unit tests (confirmed fail pre-fix / pass post-fix via temporary Edit-based revert-and-rerun -- `git stash` avoided per this session's git-whitelist constraint, which permits only `status`/`log`/`diff`/`show`/bare `pull`), the full `--all-features` suite (17 passed / 0 failed + 1 doctest), and clean clippy. Filed as BUG-277, not the provisionally-used BUG-273, after this session's first on-disk scan (272 highest) went stale under heavy concurrent bug-filing across this workspace's other parallel review forks: by filing time, BUG-273 had already been claimed twice over by two unrelated forks (`273_report_obj_model_num_faces_zero_for_triangulated_meshes.md`, `273_storage_texture_binding_layout_default_format_not_storage_capable.md`), BUG-274 and BUG-276 were also already filed, and BUG-275 was independently in the process of being claimed by a third concurrent fork (renumbering its own report away from an earlier BUG-273 collision) -- a fresh full re-scan immediately before writing this report found 277 the first genuinely free id, confirmed clear a second time immediately before this file was written. All four provisional `BUG-273` references in this fix's own source/test comments and `tests/readme.md` row were renumbered to `BUG-277` accordingly. Closed same-session (Tier 2 Dual-Role Self-Check). |
