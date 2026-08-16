# BUG-200: `Queue::buffer_write`'s WebGL arm silently no-ops on oversized data

- **Severity:** High (silent data corruption -- the buffer keeps stale/uninitialized contents
  while the call reports success, with no error anywhere in the chain)
- **state:** Completed
- **Affects:** Every WebGL-backend caller of `gpu_hal::Queue::buffer_write` that reuses a buffer
  sized for earlier, smaller data without recreating it.
- **Component:** `module/helper/gpu_hal` (`src/device.rs`, `src/webgl.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Same "WebGL error flag never checked, function returns `Ok` regardless"
  pattern already documented for BUG-160 (`sprite_upload` hardcoded mip levels) and BUG-176
  (`texture_create`'s WebGL arm silently no-ops on `INVALID_VALUE`) -- this is the third
  occurrence of the pattern in this crate/workspace, now on the buffer-write path instead of
  texture creation.

## Symptom

```rust
// pre-fix -- gpu_hal/src/device.rs, Queue::buffer_write, WebGl arm
Self::WebGl( context ) =>
{
  let raw = buffer.expect_webgl();
  context.bind_buffer( raw.target, Some( &raw.buffer ) );
  context.buffer_sub_data_with_i32_and_u8_array( raw.target, 0, data );
  Ok( () )
}
```

`context.buffer_sub_data_with_i32_and_u8_array` -- the `web-sys` binding for WebGL2's
`bufferSubData` -- returns `()`, not a `Result`. Per the WebGL2 spec, `bufferSubData` "generates
`INVALID_VALUE` and does nothing" when `data` would overflow the destination buffer's allocated
size -- but that error lands only on the GL context's internal error flag, which this function
never reads. `buffer_write` returned `Ok(())` unconditionally, regardless of whether the write
actually happened.

## Impact

**Who is affected:** Any WebGL-backend caller that writes to a buffer sized smaller than the
data it later tries to write into it -- e.g. a buffer originally created for a small uniform,
later reused (not recreated) for a larger payload after a scene/asset change.

**What breaks:** The destination buffer keeps its old or uninitialized GPU-side contents. Every
subsequent draw reading that buffer operates on silently wrong data, with `buffer_write`'s own
return value (`Ok(())`) giving no indication anything went wrong. Contrasted directly against
the sibling `texture_write`'s WebGL arm in the same file, which calls
`tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array` -- a binding that DOES
return `Result<(), JsValue>` and IS correctly `.map_err(...)?`-propagated -- this asymmetry
within the very same file is itself evidence the buffer path was simply missed, not a
deliberate design choice.

**Magnitude:** Every `buffer_write` call against a WebGL buffer was exposed identically, since
the defect is in the single shared implementation, not any one caller.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Surfaced by task #121's dedicated review pass of `gpu_hal` (this session), which specifically
checked the actual `web-sys-0.3.104` binding source for
`buffer_sub_data_with_i32_and_u8_array` and confirmed its return type is `()`, then cross-checked
against `texture_write`'s sibling binding (`tex_sub_image_2d_with_...`) which DOES return a
`Result` and is correctly handled -- establishing the asymmetry as the core evidence.

## Minimum Reproducible Example

```rust
// gpu_hal-backed WebGL buffer, allocated with 4 bytes
let small_buffer = device.buffer_create( 4, BufferUsage::UNIFORM | BufferUsage::COPY_DST )?;
let oversized = [ 1.0f32, 2.0, 3.0, 4.0 ].iter().flat_map( |v| v.to_le_bytes() ).collect::<Vec<_>>(); // 16 bytes
let result = queue.buffer_write( &small_buffer, &oversized );
// pre-fix: result == Ok(()) -- the buffer's 4 bytes are untouched, no error anywhere
```

**Verify Command** (real browser, no mocking -- WebGL has no offscreen-readback native
counterpart for this path, per `tests/manual/readme.md`'s own established convention):
```bash
cd examples/gpu_hal/triangle_browser && trunk serve --release --no-default-features --features webgl --port 8080 &
browsee .launch session::t url::http://127.0.0.1:8080/ features::webgpu window::800x600 && browsee .wait for::render timeout::60 session::t && browsee .pixel region::40x40x300,150 session::t
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `buffer_write`'s WebGL arm never validates `data`'s length against the destination buffer's allocated size, and the underlying `bufferSubData` call structurally cannot report the resulting `INVALID_VALUE` itself, so an oversized write silently no-ops while `buffer_write` still returns `Ok`. | ✅ Root Cause | Confirmed by reading the WebGL arm directly (no length check anywhere) and confirming `buffer_sub_data_with_i32_and_u8_array`'s binding signature returns `()`, matching the WebGL2 spec's documented "generates `INVALID_VALUE` and does nothing" behavior for this exact case. | E1, E2, E3 |
| H2 | `BufferWebGl` already tracked its allocated size somewhere, making this validation trivial to have added but simply omitted -- i.e. the fix requires no new state, just a check. | ✅ Confirmed (informs fix, not a competing hypothesis) | Pre-fix, `BufferWebGl` had no size field at all (`{ buffer, target }`) -- the validation was not merely omitted, the data needed to perform it didn't exist yet. Both construction sites (`buffer_create`, `buffer_init_create`) needed updating once the field was added. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/gpu_hal/src/device.rs` (pre-fix, `buffer_write` WebGL arm) | No length validation anywhere before `buffer_sub_data_with_i32_and_u8_array`. | H1 ✅ |
| E2 | `web-sys-0.3.104`'s generated binding for `buffer_sub_data_with_i32_and_u8_array` | Return type is `()`, not `Result<(), JsValue>` -- structurally cannot surface a GL error. | H1 ✅ |
| E3 | `module/helper/gpu_hal/src/device.rs`, `texture_write`'s WebGL arm, `tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array` | Sibling upload call in the SAME file DOES return `Result<(), JsValue>` and IS `.map_err(...)?`-propagated -- proving the asymmetry is an omission on the buffer path, not a backend limitation applying uniformly. | H1 ✅ |
| E4 | `module/helper/gpu_hal/src/webgl.rs` (pre-fix, `BufferWebGl`) | Struct had only `{ buffer, target }` -- no size tracking existed to validate against. | H2 ✅ |

## Root Cause

```rust
// before -- no length check, and no size field to check against even if one existed
pub struct BufferWebGl { pub buffer : web_sys::WebGlBuffer, pub target : u32 }
// ...
context.buffer_sub_data_with_i32_and_u8_array( raw.target, 0, data );
Ok( () )
```

Two missing pieces compounded: no `size` tracked on the WebGL buffer handle, and no guard using
it even if it had been tracked -- and the underlying GL call gives no error signal to fall back
on, unlike `texture_write`'s sibling call in the same file.

## Why Not Caught

`buffer_write` had no test exercising an oversized write on any backend prior to this bug. The
crate's WebGL backend has no automated test coverage at all (per `gpu_hal/readme.md`'s own `##
Verify` section -- WebGL/WebGPU are verified manually via `browsee` against a real browser,
since they present to a canvas with no offscreen readback to assert on), so this asymmetry
against `texture_write`'s correctly-handled sibling call went unnoticed until this session's
dedicated review pass.

## Fix Location

- `module/helper/gpu_hal/src/webgl.rs`: added `pub size : u64` to `BufferWebGl`, tracking the
  buffer's allocated size at creation.
- `module/helper/gpu_hal/src/device.rs`: both construction sites updated --
  `buffer_create`'s WebGL arm now populates `size` from its own `size : u64` parameter;
  `buffer_init_create`'s WebGL arm now populates `size` from `data.len() as u64` (this variant
  sizes the buffer directly from the data it's initialized with, no separate parameter).
  `buffer_write`'s WebGL arm now checks `data.len() as u64 > raw.size` before calling
  `bind_buffer`/`buffer_sub_data_with_i32_and_u8_array`, returning `Error::InvalidInput`
  (reusing the variant added by BUG-176) instead of proceeding.

This is a purely additive fix -- `buffer_write`'s signature (`Result<(), Error>`) is unchanged,
since the function was already fallible (its WebGPU arm can already fail). A workspace-wide
audit of `BufferWebGl {` construction (`grep -rn "BufferWebGl {" module/helper/gpu_hal/src/
module/helper/gpu_hal/tests/`) confirmed exactly these 2 sites exist -- both updated, no third
site missed.

## Prevention

A real-browser scenario added to `examples/gpu_hal/triangle_browser/src/main.rs`
(`triangle_draw`, `#[cfg(feature = "webgl")]`-gated): after the existing uniform-buffer write,
attempts a 16-byte write into a freshly-created 4-byte buffer and switches the render's clear
color to cyan if that write does NOT return `Err` -- turning a `Result` assertion into an
observable pixel difference, consistent with this crate's own established real-browser
verification convention (no `wasm_bindgen_test`/mocking introduced, matching
`tests/manual/readme.md`'s explicit statement that this crate's browser backends "cannot be
`cargo test`-automated"). Documented as new Scenario 4 in `tests/manual/readme.md`, with the
Test Matrix updated.

## Pitfall

A WebGL binding returning `()` instead of `Result` is not evidence the underlying GL call
cannot fail -- it means the call's failure mode (the GL error flag) is invisible unless the
caller separately validates its own preconditions before making the call. This is the second
time this exact class of asymmetry has been found in this crate's WebGL arms (BUG-176's
`texture_create` vs. this bug's `buffer_write`) -- when one call in a match arm handles this
correctly (`texture_write`'s `Result`-returning sibling) and another doesn't, that inconsistency
within the same file is itself a signal worth checking, not just each call site in isolation.

## Generalized Version

**Broken assumption:** "if the wasm-bindgen binding compiles and returns `()`, the underlying
browser call either can't fail or fails loudly (exception/panic)."

**Confirmed general rule:** A binding's Rust return type reflects how the *browser API* itself
reports errors, not whether errors are possible -- WebGL's error-flag model (distinct from
WebGPU's promise-rejection/validation-error model) means many WebGL2 calls that CAN fail per
spec still bind to `()` in `web-sys`, and the caller must know the spec's own failure behavior
("generates X and does nothing") for each specific call rather than trusting the Rust type
signature to encode it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Surfaced by task #121's dedicated `gpu_hal` review pass, cross-checking `buffer_write`'s WebGL arm against `texture_write`'s correctly-`Result`-propagated sibling in the same file. |
| 2026-08-16 | fixed | Added `size` tracking to `BufferWebGl`; updated both construction sites (`buffer_create`, `buffer_init_create`); added the oversized-write guard to `buffer_write`'s WebGL arm, reusing BUG-176's `Error::InvalidInput` variant. |
| 2026-08-16 | verified | `cargo nextest run -p gpu_hal --all-features`: 9/9 passed (includes BUG-199's tests, unaffected). `cargo check --target wasm32-unknown-unknown --no-default-features --features "enabled,webgpu,webgl" -p gpu_hal`: clean. `cargo clippy -p gpu_hal --all-targets --features native -- -D warnings`: clean. `cargo clippy --target wasm32-unknown-unknown -p gpu_hal --all-targets --no-default-features --features "enabled,webgl" -- -D warnings`: clean. `cargo clippy --target wasm32-unknown-unknown --no-default-features --features webgl` on `examples/gpu_hal/triangle_browser`: 1 pre-existing `clippy::wildcard_imports` warning at `src/main.rs:18` (`use gpu_hal::*;`) -- confirmed via `git show HEAD -- src/main.rs` to already exist verbatim in the last commit, untouched by this fix's diff; not introduced or fixed here (out of scope -- unrelated to BUG-200, left for its own owner). Real-browser empirical fail-then-pass via `trunk serve` + `browsee` against the same example (webgl build, Firefox): guard temporarily reverted (direct source edit) -> clear band read pure cyan `(0, 255, 255)`, reproducing the defect; guard restored, server rebuilt, fresh browser session -> clear band read black (`4, 0, 0`, negligible antialiasing residue), triangle band pure red (`255, 0, 0`), confirmed by both `.pixel` sampling and a full `.shot` screenshot. Encountered and self-corrected an infinite trunk rebuild loop mid-verification (Durable Log written inside the watched crate directory triggering the file watcher every ~1.4s) -- killed the looping job by exact PID (confirmed via its own launch-prologue PID and `ps`/`pgrep` cross-check, per this session's established never-pkill-by-pattern discipline) and relaunched `longrun .launch` from the parent directory instead. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 7/7

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass ran the real-browser fail-then-pass cycle end to end. Adversarial pass specifically distrusted the first post-fix `.pixel` reading (`249,249,251`, neither black nor cyan) rather than accepting it as a pass — took a full `.shot` screenshot to visually confirm the actual render, identified the reading as a chrome-band sampling artifact (matching this crate's own already-documented offset-fragility caveat), then re-sampled at screenshot-verified clean coordinates before accepting the result. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Correctly cross-referenced against BUG-160 and BUG-176 as the same "WebGL error flag unchecked" pattern, third occurrence — not a duplicate (disjoint call path: buffer write vs. texture upload/creation). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct inspection of the actual `web-sys` binding signature (confirmed `()` return type) and the WebGL2 spec's documented behavior for this exact call, not inferred from symptom alone; corroborated by the in-file asymmetry against `texture_write`'s correctly-handled sibling. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a new struct field, two construction-site updates, and one validation guard; no unrelated refactor. The manual-test-doc and example additions are the established, narrowest verification mechanism this crate already uses for WebGL, not new infrastructure. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Fix lives entirely in `gpu_hal` (`device.rs`, `webgl.rs`); the `triangle_browser` example addition is verification-only (an existing manual-test fixture extended, not a new production code path) and documented as such in `tests/manual/readme.md`. | — |
| D7 | Environment Self-Correction | 🟢 | 🟢 | Mid-verification infinite rebuild loop (Durable Log written inside the trunk-watched crate directory) was identified from the log's own repeating `starting build` lines, root-caused correctly (matches this session's own prior-established pitfall), and resolved by killing the exact confirmed PID and relaunching from outside the watched directory — no destructive action taken against any other concurrent actor's process, confirmed via `ps`/`pgrep` cross-check of the specific PID before every kill. | — |

**Reproduced:** YES -- pre-fix (guard reverted), a real Firefox/WebGL2 session renders the
verification scenario's cyan fallback clear color, proving the oversized write silently
succeeded; post-fix (guard restored), the same scenario renders the correct black clear with
the triangle intact, confirmed via both `.pixel` sampling at screenshot-verified coordinates and
a full `.shot` screenshot. 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_hal/src/webgl.rs` | Added `pub size : u64` to `BufferWebGl`. |
| `module/helper/gpu_hal/src/device.rs` | `buffer_create`'s WebGL arm: populate `size` from its own parameter. `buffer_init_create`'s WebGL arm: populate `size` from `data.len() as u64`. `buffer_write`'s WebGL arm: added the `Fix(BUG-200)` oversized-data guard returning `Error::InvalidInput`; updated the `# Errors` doc section. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/gpu_hal/triangle_browser/src/main.rs` | `triangle_draw`: added a `#[cfg(feature = "webgl")]`-gated verification block attempting an oversized `buffer_write` and switching the clear color to cyan if the guard fails to reject it. |
| `module/helper/gpu_hal/tests/manual/readme.md` | Added Scenario 4 (`buffer_write rejects oversized WebGL data`) and a corresponding Test Matrix row. |
