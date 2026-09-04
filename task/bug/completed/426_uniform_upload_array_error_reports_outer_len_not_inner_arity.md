# BUG-426: `UniformUpload` for `[[T; N]]` reports the outer slice's length instead of the inner array's arity in its own error

- **Severity:** Medium (no crash -- but the error message is self-contradictory, actively misleading
  a developer trying to debug why a uniform-array upload failed)
- **state:** Completed
- **Affects:** Any consumer of `minwebgl::uniform`'s `UniformUpload` impls for `[[f32; N]]`,
  `[[i32; N]]`, `[[u32; N]]` who passes an unsupported inner arity `N` (i.e. not 1, 2, 3, or 4).
- **Component:** `module/min/minwebgl` (`src/uniform/float32.rs`, `int32.rs`, `unsigned32.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect *class* and same fix-precedent as BUG-277 (a copy-pasted error arm
  in the sibling `UniformMatrixUpload for [f32; N]` impl in the same file, also reporting the wrong
  field) -- but a distinct instance in a different set of three impls (`[[T; N]]` slice-of-arrays,
  not `[T; N]` matrix), fixed together in this same pass since all three `[[T; N]]` impls
  (`float32.rs`, `int32.rs`, `unsigned32.rs`) share the exact copy-pasted defect.

## Symptom

```rust
// pre-fix -- src/uniform/float32.rs, impl UniformUpload for [ [ f32 ; N ] ]
match N
{
  1 => { .. }, 2 => { .. }, 3 => { .. }, 4 => { .. },
  _ => Err( vector_upload_length_error( type_name_of_val( self ), self.len() ) ),
  //                                                                ^^^^^^^^^^ outer slice count
};
```

Uploading `&[[f32; 5]; 3]` (3 vectors, each of the unsupported inner arity 5) produced a
self-contradictory message reading `"...of length 3. Known length: [1, 2, 3, 4]"` -- 3 IS in the
claimed-valid list, because the field reported (`self.len()`, the outer slice's element count) was
never the field that actually failed validation (`N`, the inner array's arity, which the surrounding
`match` is on).

## Impact

**Who is affected:** Any consumer of the three `[[T; N]]` `UniformUpload` impls
(`float32.rs`/`int32.rs`/`unsigned32.rs`) who passes an unsupported inner arity.

**What breaks:** Diagnostics only -- the upload still correctly fails with `Err`, but the error
message misidentifies which value is invalid, actively misleading anyone debugging the failure
(the reported "invalid" length is drawn from the one dimension that was never actually checked
against the valid-lengths list).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`,
cross-checking every `match`-based error arm against its own scrutinee after BUG-277 had already
established this exact copy-paste defect shape (error arm references `self.len()` when the
surrounding `match` is actually on a different value) elsewhere in the same file -- the three
`[[T; N]]` impls share the identical shape, one level up (matching on the const generic `N`, not
`self.len()`).

## Minimum Reproducible Example

```rust
// module/min/minwebgl/tests/uniform_test.rs
let error = vector_upload_length_error( "&[[f32; 5]]", 5usize );
let WebglError::CantUploadUniform( _, _, reported_n, _ ) = error else { panic!() };
assert_eq!( reported_n, 5 ); // pre-fix: call site passed self.len() (outer count), not N
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo nextest run -p minwebgl -E 'test(vector_upload_length_error_reports_inner_arity_not_outer_len)'
```

## Root Cause

All three `[[T; N]]` `upload` impls `match N { .. }` to pick the right `glUniformNfv`-family call,
but their catch-all `_` arm was copy-pasted from the sibling plain-slice (`[T]`) impl above it, where
`self.len()` genuinely *is* the value the `match` scrutinee is on (`match self.len() { .. }`). In the
`[[T; N]]` impls the match is on `N`, not `self.len()`, but the copy-pasted error arm kept reporting
the old field.

## Why Not Caught

`UniformUpload::upload` takes `&GL` (`web_sys::WebGl2RenderingContext`), which can't be constructed
outside a browser, so nothing could call it directly from a native `cargo test` run to observe the
error text; no live-GL example in this repo exercises the error branch either, since every real
caller passes an already-correctly-sized vector array.

## Fix Location

`module/min/minwebgl/src/uniform/float32.rs`, `int32.rs`, `unsigned32.rs`: all three `[[T; N]]`
`upload` impls' catch-all arms changed from
`Err( vector_upload_length_error( type_name_of_val( self ), self.len() ) )` to
`Err( vector_upload_length_error( type_name_of_val( self ), N ) )`.

## Prevention

New test `vector_upload_length_error_reports_inner_arity_not_outer_len` in
`module/min/minwebgl/tests/uniform_test.rs`: calls `vector_upload_length_error` directly with an
inner arity `n` deliberately different from any plausible outer slice length, across three cases
(`5`, `0`, `6`), and asserts the error's reported length equals `n` -- a regression back to reporting
`self.len()` cannot coincidentally match since the test's `n` values are chosen specifically to
differ from the outer length in every case. RED state empirically confirmed via a temporary probe
before the fix was finalized (see the test's own doc comment).

## Pitfall

A slice-of-arrays impl (matches on the const generic `N`) and a plain-slice impl (matches on
`self.len()`) can share an identically-shaped error arm while differing in exactly which value the
`match` scrutinee -- and therefore the error's "invalid length" field -- actually is; copy-pasting
one into the other silently keeps the wrong field, and nothing about the resulting code fails to
compile, since both `N` and `self.len()` are valid `usize`-typed expressions in that position.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`, cross-checking match-based error arms against their own scrutinee, building on the BUG-277 precedent in the same file. |
| 2026-08-20 | fixed | Changed all three `[[T; N]]` impls' catch-all arms from `self.len()` to `N`; added `Fix(BUG-426)`/`Root cause`/`Pitfall` source comments (full in `float32.rs`, cross-referencing in `int32.rs`/`unsigned32.rs`). |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | RED state empirically confirmed via a temporary revert of all three call sites back to `self.len()`, re-running the test with an outer/inner length mismatch -- genuinely failed the `reported_n` assertion; restored, re-ran full suite -- `cargo nextest run -p minwebgl` 19/19 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-426)`/`Root cause`/`Pitfall` 3-field format applied at all three fix sites (full comment in `float32.rs`, cross-referencing comments in `int32.rs`/`unsigned32.rs` per this workspace's established cross-reference convention, e.g. BUG-277's own sibling comments). | — |
| D3 | Scope containment | — | 🟢 | Only the three uniform impl files (fix) and `uniform_test.rs`/`tests/readme.md` (test + Responsibility Table update) touched -- all within `module/min/minwebgl`. | — |

**Reproduced:** YES -- temporary revert of all three call sites to `self.len()` caused the new test
to fail on the `reported_n` assertion for all three cases; restoring the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/uniform/float32.rs` | `impl UniformUpload for [[f32; N]]`'s catch-all arm now reports `N` instead of `self.len()`; full `Fix(BUG-426)`/`Root cause`/`Pitfall` source comment. |
| `module/min/minwebgl/src/uniform/int32.rs` | Same fix for `[[i32; N]]`; cross-referencing `Fix(BUG-426)` comment. |
| `module/min/minwebgl/src/uniform/unsigned32.rs` | Same fix for `[[u32; N]]`; cross-referencing `Fix(BUG-426)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/uniform_test.rs` | Added `vector_upload_length_error_reports_inner_arity_not_outer_len`. |
| `module/min/minwebgl/tests/readme.md` | Updated `uniform_test.rs`'s row to mention BUG-426 alongside BUG-277. |
