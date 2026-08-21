# BUG-453: `.get_extension(name).expect(msg)` only unwraps the outer `Result`, not the inner `Option`, across 4 demo crates

- **Severity:** Medium (no crash on supported hardware -- these sites already fail closed once a
  render call actually needs the missing extension -- but on unsupported hardware the failure
  surfaces far from its cause, with a misleading or generic message instead of a clear
  "extension not supported" panic at startup)
- **state:** Completed
- **Affects:** `examples/minwebgl/animation_amplitude_change`, `examples/minwebgl/animation_surface_rendering`,
  `examples/minwebgl/character_control`, `examples/minwebgl/curve_surface_rendering` -- 4 independent
  example crates, 5 `get_extension` call sites total.
- **Component:** `examples/minwebgl/{animation_amplitude_change,animation_surface_rendering,character_control,curve_surface_rendering}/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- `examples/minwebgl/area_light/src/main.rs` already used the correct
  2-layer pattern this fix copies; not itself a bug, just the reference implementation.

## Symptom

```rust
// pre-fix -- animation_amplitude_change/src/main.rs:228-229
let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );
```

`get_extension` returns `Result< Option< Object >, JsValue >`. A single `.expect()` only unwraps
the outer `Result` -- when a browser doesn't support the extension, `get_extension` returns
`Ok( None )`, which is not an `Err`, so `.expect()` never fires. The `_`-bound `Option< Object >`
is then silently discarded, and execution continues as though the extension were available.

## Impact

**Who is affected:** Any user running one of these 4 demos on a browser/GPU combination lacking
`EXT_color_buffer_float` and/or `EXT_shader_image_load_store`.

**What breaks:** Diagnosability only -- all 5 sites already fail closed eventually (nothing
renders correctly without the extension actually being usable), just with a worse error surfacing
later and further from the true cause (a framebuffer-completeness error, a garbled/blank render,
or a panic deep inside unrelated rendering code) instead of a clear
`"EXT_color_buffer_float extension is not supported"` panic right at startup.

**Magnitude:** 5 call sites across 4 crates -- `animation_amplitude_change` and `character_control`
each check 2 extensions (`EXT_color_buffer_float`, `EXT_shader_image_load_store`);
`animation_surface_rendering` and `curve_surface_rendering` each check 1
(`EXT_color_buffer_float`).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, comparing every
`get_extension` call site against `area_light/src/main.rs`'s existing 2-layer
`.expect().expect()` pattern, which already correctly handles both failure layers.

## Minimum Reproducible Example

`get_extension` requires a live `WebGl2RenderingContext`, which has no native (non-browser)
construction path -- this is a structural defect visible directly from the pre-fix source rather
than something a native test can execute:

```rust
// Result<Option<T>, E>::expect(msg) on Ok(None) returns None -- does not panic.
let _ : Option< Object > = Ok::< Option< Object >, JsValue >( None ).expect( "..." );
```

**Verify Command:** N/A -- no native harness exists for `WebGl2RenderingContext`; verified via
`cargo check --target wasm32-unknown-unknown` (all 4 crates compile cleanly) plus direct source
inspection of the corrected 2-layer chain at each site (see Refs below).

## Root Cause

`web_sys::WebGl2RenderingContext::get_extension()` returns `Result< Option< Object >, JsValue >` --
two independent failure layers: the outer `Result::Err` for a JS-level exception, and the inner
`Option::None` for "the call itself succeeded, but this extension isn't supported." A single
`.expect()` unwraps only the outer layer; the inner `None` is a legitimate `Ok` value, so
`.expect()` never fires for it.

## Why Not Caught

Example crates carry no `tests/` requirement (`health.md`), and the failure mode is
browser/GPU-dependent -- it never manifests on the WebGL2-conformant setups these demos are
normally developed against, so ordinary manual testing gives no signal either way.
`area_light/src/main.rs` already had the correct pattern, but nothing enforced it as the template
other 4 crates should have copied.

## Fix Location

Applied `area_light`'s existing 2-layer pattern at all 5 call sites:
- `animation_amplitude_change/src/main.rs:228-239` (`EXT_color_buffer_float`, `EXT_shader_image_load_store`)
- `animation_surface_rendering/src/main.rs:84-92` (`EXT_color_buffer_float`)
- `character_control/src/main.rs:366-377` (`EXT_color_buffer_float`, `EXT_shader_image_load_store`)
- `curve_surface_rendering/src/main.rs:57-65` (`EXT_color_buffer_float`)

```rust
let _ = gl.get_extension( "EXT_color_buffer_float" )
.expect( "Failed to query EXT_color_buffer_float extension" )
.expect( "EXT_color_buffer_float extension is not supported" );
```

## Prevention

No native regression test is practical (browser-only API, no native `WebGl2RenderingContext`
construction path). The fix itself is the durable guard going forward: any future `get_extension`
call site copy-pasted from one of these 4 files now carries the correct 2-layer pattern instead of
the single-layer one. Verified via `cargo check --target wasm32-unknown-unknown` across all 4
crates (see Verification Record) -- confirms the chain still type-checks correctly end-to-end.

## Pitfall

`Result< Option< T >, E >` has two independent failure layers -- unwrapping only the outer
`Result` silently passes the inner `None` through as success. Any API shaped this way (not just
`get_extension`) needs two `.expect()`/`.unwrap()` calls, or an explicit `match`, to actually fail
on both layers.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery and fix landed together in one session. |
| 2026-08-20 | fixed | Applied `area_light`'s existing 2-layer `.expect().expect()` pattern at all 5 sites across 4 crates. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 4/4

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | All 5 sites use the 2-layer pattern | — | 🟢 | Adversarial pass: re-grepped `get_extension` across all 4 crates post-edit -- confirmed zero remaining single-layer `.expect()`/`.unwrap()` call sites, and confirmed each of the 5 sites chains exactly two `.expect()` calls with distinct, accurate messages. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-453)`/`Root cause`/`Pitfall` 3-field format applied at all 4 file sites. | — |
| D3 | Compiles for wasm32 target | — | 🟢 | `cargo check --target wasm32-unknown-unknown -p animation_blending -p animation_surface_rendering -p character_control -p curve_surface_rendering` (combined with the other 4 touched crates in one invocation) -- exit 0, zero errors, zero warnings. | — |
| D4 | Scope containment | — | 🟢 | `git status`/source inspection confirms only the intended `get_extension` call sites changed in each file. | — |

**Reproduced:** N/A (browser-only API; no native reproduction harness) -- pre-fix source inspected
directly to confirm the single-layer defect; post-fix source inspected directly to confirm both
layers are now checked at all 5 sites. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/animation_amplitude_change/src/main.rs` | 2-layer `.expect()` chain for `EXT_color_buffer_float`, `EXT_shader_image_load_store`. |
| `examples/minwebgl/animation_surface_rendering/src/main.rs` | 2-layer `.expect()` chain for `EXT_color_buffer_float`. |
| `examples/minwebgl/character_control/src/main.rs` | 2-layer `.expect()` chain for `EXT_color_buffer_float`, `EXT_shader_image_load_store`. |
| `examples/minwebgl/curve_surface_rendering/src/main.rs` | 2-layer `.expect()` chain for `EXT_color_buffer_float`. |

## Refs: tests/

| File | Change |
|------|--------|
| — | No native test practical (browser-only API); verified via `cargo check --target wasm32-unknown-unknown` across all 4 crates instead. |
