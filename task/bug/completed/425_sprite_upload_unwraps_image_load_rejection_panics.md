# BUG-425: `sprite_upload` panics on a rejected image-load promise instead of returning its own declared `Result::Err`

- **Severity:** Medium (no memory corruption -- a clean panic, but a panic nonetheless, in an `async
  fn` that already declares and otherwise correctly uses a `Result` return type for exactly this
  kind of recoverable failure)
- **state:** Completed
- **Affects:** Any consumer of `minwebgl::texture::d2::sprite_upload` loading a sprite sheet from a
  URL that can fail to load (broken path, network failure, unreachable host, CORS rejection, etc.).
- **Component:** `module/min/minwebgl` (`src/texture/d2.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- isolated error-propagation gap in this one function, no shared root cause
  with any other bug filed this sweep.

## Symptom

```rust
// pre-fix -- src/texture/d2.rs, sprite_upload
let ( promise, resolve, reject ) = /* ... */;
image.set_onerror( Some( reject_closure... ) ); // wires rejection on image load failure
// ...
JsFuture::from( load_promise ).await.unwrap(); // discards the rejection, panics instead
```

`sprite_upload` is declared `async fn sprite_upload( .. ) -> Result< WebGlTexture, WebglError >` and
correctly wires an `on_error` handler that rejects `load_promise` when the browser's `error` event
fires on the image element (e.g. a broken image URL) -- but then discards that rejection via
`.await.unwrap()`, panicking instead of returning it through the function's own already-declared
`Result` type.

## Impact

**Who is affected:** Any consumer passing a sprite-sheet URL to `sprite_upload` that can fail to
load -- a broken path, a network failure, a CORS-rejected cross-origin image, or any other browser
`error` event on the `<img>` element.

**What breaks:** The `async fn` panics instead of returning `Err`, even though it already declares
`Result< WebGlTexture, WebglError >` and every *other* fallible step inside it correctly propagates
via `?`. A panic in an async wasm context typically aborts the whole panic-catching boundary the
caller set up (or, with no such boundary, the whole wasm instance), rather than being recoverable the
way the function's own signature promises callers it will be.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`,
auditing every `async fn` returning `Result` for `.await` call sites that don't use `?` -- this
function wires a rejection handler explicitly (a clear signal the author anticipated the failure
case) but then discards what it delivers, a distinct and more surprising pattern than a `.await` with
no rejection handling wired at all.

## Minimum Reproducible Example

Live reproduction needs a real `WebGl2RenderingContext` and a real `HtmlImageElement` loading a
genuinely broken image URL inside an actual browser -- neither constructible from a native
`cargo test` run (no JS engine). See `sprite_upload_test.rs`'s own doc comment for the full reasoning
behind the source-inspection fallback used instead, the same one already established for BUG-290
(`minvulkan/tests/context_test.rs`) and BUG-424 (`minvulkan/tests/swapchain_test.rs`) in this
workspace for defects that are real but structurally unreachable from the available native/in-scope
test surface.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo nextest run -p minwebgl -E 'test(sprite_upload_propagates_image_load_rejection_instead_of_panicking)'
```

## Root Cause

`sprite_upload` bridges a JS `Promise`/`onerror` callback pair into an `async fn` via `JsFuture`, and
wired the rejection side correctly (`on_error` rejects the promise), but the `.await` consuming that
promise used `.unwrap()` instead of propagating the `Result< JsValue, JsValue >` `JsFuture::from(..)`
produces -- wiring a rejection handler is a separate step from actually propagating what it delivers,
and doing the first without the second silently converts every wired rejection back into a panic.

## Why Not Caught

`sprite_upload` needs a live `WebGl2RenderingContext` and a real `HtmlImageElement`, neither
constructible from a native `cargo test` run. Reproducing the panic live would need a real browser
loading a real broken image URL through a dedicated test page; no such page exists in this crate's
`tests/manual/` browser procedure (which currently covers only `context::from_canvas` + a draw call,
via `examples/minwebgl/context_triangle_smoke`), and adding one is out of reach here since it would
require creating or modifying an `examples/` crate, outside this fix's edit scope.

## Fix Location

`module/min/minwebgl/src/texture/d2.rs`: replaced
`JsFuture::from( load_promise ).await.unwrap();` with
`JsFuture::from( load_promise ).await.map_err( | _ | WebglError::Other( "image failed to load" ) )?;`.
Doc comment updated to move "image fails to load" from `# Panics` into `# Errors`.

## Prevention

New source-inspection test `sprite_upload_propagates_image_load_rejection_instead_of_panicking`,
appended to the existing `module/min/minwebgl/tests/sprite_upload_test.rs` (already the correctly-
scoped home for this function's other extracted-helper tests): asserts 0 occurrences of the old
`.unwrap()` form and exactly 1 occurrence of the fixed `.map_err(..)?` line, both via
`include_str!("../src/texture/d2.rs")`. No pure-logic extraction was possible here (unlike
BUG-160/161/277/426's precedents, which all had genuine computable formulas) since
`WebglError::Other("image failed to load")` takes no parameters, making a message-content test
tautological -- the source-inspection assertion is therefore the test's full content, not a
supplement to a runtime unit test.

## Pitfall

An `async fn` already returning `Result< _, WebglError >`, with every *other* fallible step inside it
correctly using `?`, can still hide a single `.unwrap()` on one particular await -- the function's
overall shape reads as fallible-safe, but a per-call-site audit is still needed since Rust does not
require every await in a `Result`-returning `async fn` to use `?`. Wiring a `Promise`/`JsFuture`
bridge's rejection handler is not the same step as propagating what it delivers on the awaiting end.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`, auditing `async fn -> Result` call sites for `.await`s not using `?`. |
| 2026-08-20 | fixed | Replaced `.unwrap()` with `.map_err(..)?`; moved the failure case from `# Panics` to `# Errors` in the doc comment; added `Fix(BUG-425)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Source-inspection test asserts both absence of the old `.unwrap()` form (0 occurrences) and presence of the exact fixed line (1 occurrence) -- a partial or reworded revert would fail at least one assertion. Full-crate pass: `cargo nextest run -p minwebgl` -- 19/19 pass; `cargo check -p minwebgl --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-425)`/`Root cause`/`Pitfall` 3-field format applied at the fix site; test carries the mandated 5-section doc block (`bug_reproducer(BUG-425)`). | — |
| D3 | Scope containment | — | 🟢 | Only `d2.rs` (fix) and `sprite_upload_test.rs`/`tests/readme.md` (test + Responsibility Table update) touched -- confirmed via `git diff`, all within `module/min/minwebgl`. | — |

**Reproduced:** Source-inspection only -- live reproduction is not achievable from this crate's
native test surface (no JS engine; see Minimum Reproducible Example above). The test's own
construction (asserting both absence of the pre-fix `.unwrap()` and presence of the exact fixed
`.map_err(..)?` line) is the closest available substitute for a RED/GREEN cycle. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/texture/d2.rs` | Replaced `.await.unwrap()` with `.await.map_err(..)?` on the image-load promise; moved the failure case from `# Panics` to `# Errors`; added `Fix(BUG-425)`/`Root cause`/`Pitfall` source comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/sprite_upload_test.rs` | Appended `sprite_upload_propagates_image_load_rejection_instead_of_panicking`; updated the file's top doc comment to mention BUG-425 coverage. |
| `module/min/minwebgl/tests/readme.md` | Updated `sprite_upload_test.rs`'s row to mention BUG-425 alongside BUG-160/161. |
