# BUG-163: `format_to_size` panics on 10 of `GpuVertexFormat`'s 41 spec-defined variants

- **Severity:** High (panics on ordinary, legally-constructible input with no documented
  `# Panics` contract at all -- a caller passing any single-component or `Bgra`-ordered vertex
  format crashes with no warning)
- **state:** Completed
- **Affects:** `layout::vertex_attribute::format_to_size` -- any caller passing `Uint8`, `Sint8`,
  `Unorm8`, `Snorm8`, `Uint16`, `Sint16`, `Unorm16`, `Snorm16`, `Float16`, or `Unorm8x4Bgra`
- **Component:** `module/min/minwebgpu` (`src/layout/vertex_attribute.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered in the same review pass as BUG-162 (task #94, `minwebgpu` code
  review), different file, no shared root cause.

## Symptom

```rust
// pre-fix
pub fn format_to_size( format : web_sys::GpuVertexFormat ) -> usize
{
  match format
  {
    GpuVertexFormat::Uint8x2 | ... => size_of::< [ u8; 2 ] >(),
    // ... 31 of 41 variants covered ...
    _ => panic!( "Unknown GpuVertexFormat variant" ), // Uint8, Sint8, Unorm8, Snorm8,
                                                        // Uint16, Sint16, Unorm16, Snorm16,
                                                        // Float16, Unorm8x4Bgra all hit this
  }
}
```

## Impact

**Who is affected:** Any caller of `format_to_size` (directly, or via
`VertexBufferLayout`'s `From` conversion) passing one of the 10 single-component or `Bgra`
formats -- all ordinary, spec-defined, legally-constructible `GpuVertexFormat` values a caller
can select via `VertexAttribute::format(..)`.

**What breaks:** The process panics with a generic "Unknown GpuVertexFormat variant" message,
even though every one of the 10 missing variants is a real, spec-valid format this crate already
imports and re-exports.

**Magnitude:** Any caller building a vertex layout with a single-channel attribute (e.g. a
per-vertex `u8` material index using `Uint8`, or a packed `Unorm8x4Bgra` color) hits this
unconditionally -- there is no workaround short of avoiding 10 of the 41 documented formats.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Direct source review of `module/min/minwebgpu/src/layout/vertex_attribute.rs` during task #94.
Cross-checked the match's covered variants against the full `GpuVertexFormat` enum (41 variants,
confirmed directly from the vendored `web-sys` source,
`~/.cargo/registry/.../web-sys-0.3.104/src/features/gen_GpuVertexFormat.rs`) and found 10
variants falling through to the wildcard panic arm.

## Minimum Reproducible Example

```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test vertex_attribute_tests 2>&1 | tail -6
```

**Expected** (post-fix):
```
test tests::previously_missing_variants_no_longer_panic_test ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 filtered out
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the completed match):
```
thread 'previously_missing_variants_no_longer_panic_test' panicked at .../vertex_attribute.rs:...:
Unknown GpuVertexFormat variant
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgpu && cargo test --target wasm32-unknown-unknown --all-features --test vertex_attribute_tests
# 2 "ok" = fixed; a panic on Uint8/Sint8/etc. = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The match was built by covering only the multi-component (`x2`/`x3`/`x4`) formats, missing all single-component and the one `Bgra`-ordered format. | ✅ Root Cause | Diffed the match's covered variant list against `web_sys::GpuVertexFormat`'s full 41-variant source: exactly the 9 single-component variants (`Uint8`, `Sint8`, `Unorm8`, `Snorm8`, `Uint16`, `Sint16`, `Unorm16`, `Snorm16`, `Float16`) plus `Unorm8x4Bgra` were absent. | E1 |
| H2 | `web_sys::GpuVertexFormat` is a plain, closed enum, so the wildcard arm after completing all 41 variants can be safely removed. | ❌ Falsified | rustc's `E0004` (`non-exhaustive patterns: '_' not covered`) fired even after all 41 named variants were matched -- the `#[wasm_bindgen]` macro expansion marks every JS-string enum `#[non_exhaustive]` regardless of how its `pub enum` source reads. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `~/.cargo/registry/.../web-sys-0.3.104/src/features/gen_GpuVertexFormat.rs` (vendored source, unedited) | Authoritative 41-variant list; diffed directly against the pre-fix match's covered arms to identify the exact 10 missing variants. | H1 ✅ |
| E2 | `cargo clippy --target wasm32-unknown-unknown` real compiler output | `error[E0004]: non-exhaustive patterns: '_' not covered`, with a note pointing at the `#[wasm_bindgen]` attribute macro as the origin of the non-exhaustiveness -- obtained when a first fix attempt removed the wildcard arm after covering all 41 named variants. | H2 ❌ |

## Root Cause

```rust
// before -- match covers only multi-component formats, 10 single-component/Bgra variants
// (Uint8, Sint8, Unorm8, Snorm8, Uint16, Sint16, Unorm16, Snorm16, Float16, Unorm8x4Bgra)
// fall through to an unconditional panic
match format
{
  GpuVertexFormat::Uint8x2 | GpuVertexFormat::Sint8x2 | ... => size_of::< [ u8; 2 ] >(),
  // ...
  _ => panic!( "Unknown GpuVertexFormat variant" ),
}
```

The match was built by covering the `x2`/`x3`/`x4` component-count families exhaustively but
never added the 9 single-component (no suffix) formats or the one alternate-channel-order
(`Unorm8x4Bgra`) format -- all real, spec-defined, equally reachable values of the same enum.

## Why Not Caught

No existing test exercised `format_to_size` at all; no existing call site ever constructed a
`VertexAttribute` using one of the 10 single-component/`Bgra` formats, so the gap was never
observed at runtime.

## Fix Location

`module/min/minwebgpu/src/layout/vertex_attribute.rs`.

```rust
// after -- all 41 variants covered; wildcard kept as a documented, unreachable internal-invariant
// panic (required by rustc's E0004 -- see Pitfall) rather than a silent wrong-size fallback
match format
{
  GpuVertexFormat::Uint8 | GpuVertexFormat::Sint8 | GpuVertexFormat::Unorm8 | GpuVertexFormat::Snorm8 => size_of::< [ u8; 1 ] >(),
  // ... (9 more previously-missing arms, correct WebGPU-spec byte sizes)
  _ => unreachable!( "GpuVertexFormat variant not recognized by this web_sys version" ),
}
```

The wildcard stays `unreachable!()` rather than becoming a `Result`-based error: `format_to_size`
is called from inside `layout/vertex_buffer.rs`'s `impl From<VertexBufferLayout> for ...`, whose
trait contract is strictly infallible -- converting to `Result` would require the much larger,
out-of-scope `From` → `TryFrom` refactor across every caller.

## Prevention

Added `tests/vertex_attribute_tests.rs` (new file, `bug_reproducer(BUG-163)`): exercises all 10
previously-missing variants directly, plus a regression test re-asserting several
previously-covered multi-component variants stay correct.

## Pitfall

`web_sys::GpuVertexFormat` is marked `#[non_exhaustive]` by the `#[wasm_bindgen]` macro's actual
expansion even though its own `pub enum` source declaration reads as a plain, closed 41-variant
enum -- don't assume a wasm-bindgen-generated enum's exhaustiveness from reading its source; the
macro's real expansion can differ, and rustc's `E0004` is the only reliable signal.

## Generalized Version

**Broken assumption:** "a match covering every named variant of an enum I can see in its source
is exhaustive, so no wildcard arm is needed."

**Confirmed general rule:** for any `#[wasm_bindgen]`-generated enum specifically, always keep a
wildcard arm regardless of how many named variants are covered -- the macro's real expansion
marks these `#[non_exhaustive]` unconditionally, and only the compiler's own `E0004` reliably
confirms this, not a reading of the enum's `pub enum` declaration.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via direct source review of `vertex_attribute.rs` during task #94, cross-checked against the vendored `web-sys` source's full 41-variant list. |
| 2026-08-16 | fixed | Completed the match with all 10 missing variants at their correct WebGPU-spec byte sizes; corrected an initial wrong assumption (non-`#[non_exhaustive]`) via real `E0004` compiler feedback, restoring a documented `unreachable!()` wildcard. |
| 2026-08-16 | verified | Added `tests/vertex_attribute_tests.rs` (2 tests), confirmed passing via real geckodriver execution. Scoped wasm32 clippy clean; full-workspace `verb/test` (native nextest+doctest+clippy, wasm32 Stage 1+2) clean, 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against the completed match; adversarial pass diffed the vendored `web-sys` source directly against the pre-fix match's covered arms to confirm exactly 10 variants were missing (not fewer, not more). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-162 (same review batch, different file, different root cause). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by a direct diff against the authoritative vendored enum source, not inference. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only the match's arm coverage changed; `format_to_size`'s signature (`usize`, not `Result`) deliberately unchanged, justified in Fix Location against the `From` impl's infallibility constraint. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `minwebgpu` src + test + bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix adds match arms only; no call site changed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing function's coverage completed. | — |

**Reproduced:** YES -- prior to completing the match, calling `format_to_size` with any of the
10 missing variants panicked with `Unknown GpuVertexFormat variant`; confirmed via in-place
revert-test-restore and via real geckodriver execution of
`previously_missing_variants_no_longer_panic_test`. Scoped wasm32 clippy + full-workspace
`verb/test` (native + wasm32 Stage 1/2) clean, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgpu/src/layout/vertex_attribute.rs` | `format_to_size` completed to all 41 `GpuVertexFormat` variants; wildcard kept as a documented `unreachable!()` (full `Fix(BUG-163)` comment). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgpu/tests/vertex_attribute_tests.rs` | New file: 2 tests (`bug_reproducer(BUG-163)` covering all 10 previously-missing variants, plus a regression test for previously-covered variants). |
| `module/min/minwebgpu/tests/readme.md` | New file: Responsibility Table for all 5 test files in this directory (crossed the 3-file threshold). |
