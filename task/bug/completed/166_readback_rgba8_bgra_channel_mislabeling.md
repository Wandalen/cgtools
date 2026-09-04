# BUG-166: `readback::rgba8` silently mislabels channel order for `Bgra8Unorm`/`Bgra8UnormSrgb` textures

- **Severity:** High (silent wrong data, not a crash -- a caller gets a clean `Ok` result with
  red and blue channels swapped, with no error, panic, or other signal that anything is wrong)
- **state:** Completed
- **Affects:** `readback::rgba8` -- any caller reading back a `Bgra8Unorm`/`Bgra8UnormSrgb`
  texture, including the common case of reading back a surface-configured render target (this
  crate's own `surface::preferred_format` routinely selects `Bgra8UnormSrgb`)
- **Component:** `module/min/minwgpu` (`src/readback.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** None directly, but shares this session's `module/min/minwgpu` review (task
  #95) with BUG-165; unrelated root cause and unrelated code path.

## Symptom

```rust
// pre-fix
let format = texture.format();
if format.block_copy_size( None ) != Some( 4 )
{
  return Err( .. ); // Bgra8Unorm/Bgra8UnormSrgb pass this check -- both are 4 bytes/pixel
}
..
let pixels = rows_unpad( &data, ( width, height ) ); // no channel reordering anywhere
Ok( ( pixels, ( width, height ) ) ) // "RGBA8 pixels" that are actually still BGRA-ordered
```

## Impact

**Who is affected:** Any caller of `readback::rgba8` on a `Bgra8Unorm`/`Bgra8UnormSrgb`
texture -- explicitly named as accepted input in this very function's own doc comment ("e.g.
`Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8UnormSrgb`"). This is not a rare combination: this same
crate's `surface::preferred_format` (exercised by `surface_test.rs`'s own
`preferred_format_picks_first_srgb_when_present`) routinely selects `Bgra8UnormSrgb`, the
common swapchain format on several real `wgpu` backends -- so "configure a surface, then read
back what was rendered to it" is a natural, in-repo-supported pattern this bug breaks silently.

**What breaks:** The returned "RGBA8 pixels" have their red and blue channels swapped for
`Bgra8*` input -- e.g. a fully red source pixel `(255,0,0,255)` in BGRA byte order
(`B=0,G=0,R=255,A=255`) is returned unchanged as raw bytes, which a caller correctly treating
the result as RGBA reads as blue `(R=0,G=0,B=255,A=255)`. No error, panic, or other signal --
the function returns a clean `Ok`.

**Magnitude:** One call, silent -- the worst-case failure mode for a data-correctness bug: it
produces plausible-looking, fully-formed, wrong output rather than failing loudly.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, not speculative: a from-scratch Explore review of `module/min/minwgpu` (task #95)
compared `rgba8`'s doc comment (explicitly listing `Bgra8UnormSrgb` as accepted input) against
its implementation (a byte-size-only check, `format.block_copy_size( None ) != Some( 4 )`, with
no channel-order handling anywhere in the file) and found the two contradict each other.
Confirmed directly against `wgpu-types-30.0.0/src/texture/format.rs` in the local cargo
registry cache that several other 4-byte-per-pixel formats (`Rg16*`, `R32*`, `Rgb9e5Ufloat`,
`Rgb10a2*`, `Rg11b10Ufloat`) also pass the same byte-size check despite not being RGBA8-shaped
at all -- the check was never a format check, only a size check.

## Minimum Reproducible Example

```bash
cd module/min/minwgpu && cargo test -p minwgpu --test readback_test bgra_to_rgba_swizzle
```

**Expected** (post-fix): a `[B,G,R,A]`-ordered input buffer becomes `[R,G,B,A]`-ordered after
`bgra_to_rgba_swizzle`.

**Actual** (pre-fix): no swizzle function existed; `rgba8` returned the raw BGRA byte order
unchanged, mislabeled as "RGBA8 pixels".

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwgpu && cargo test -p minwgpu --test readback_test
# all "ok" = fixed; a manually-inspected Bgra8UnormSrgb readback with swapped R/B = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The format-validity check is a byte-size test, not a channel-order/format test, so it silently accepts `Bgra8Unorm`/`Bgra8UnormSrgb` (and other non-RGBA-shaped 4-byte formats) without ever correcting their channel order. | ✅ Root Cause | Read the implementation directly: the only check is `block_copy_size( None ) != Some( 4 )`; no `match` or swizzle on `format` exists anywhere in the file pre-fix. | E1 |
| H2 | This is unreachable in practice because no in-repo caller currently passes a `Bgra8*` texture to `rgba8`. | ❌ Falsified (as grounds to not fix) | True today (all 3 current call sites use `Rgba8Unorm`/`Rgba8UnormSrgb`), but `rgba8`'s own doc explicitly advertises `Bgra8UnormSrgb` as supported, and this crate's own `preferred_format` routinely selects exactly that format -- a future or external caller following the documented contract hits this immediately. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/readback.rs` (pre-fix), `rgba8`'s doc comment vs. its `block_copy_size` check | The doc lists `Bgra8UnormSrgb` as accepted input; the implementation has no code path that ever reorders bytes for it. | H1 ✅ |
| E2 | `tests/surface_test.rs::preferred_format_picks_first_srgb_when_present` | Proves this crate's own `preferred_format` selects `Bgra8UnormSrgb` given a mix including it -- the exact format this bug mishandles is one this crate itself is likely to produce. | H2 ❌ |
| E3 | `wgpu-types-30.0.0/src/texture/format.rs` (local cargo registry cache) | Confirms `Rg16*`, `R32*`, `Rgb9e5Ufloat`, `Rgb10a2*`, `Rg11b10Ufloat` are also 4 bytes/pixel, proving the byte-size check was never format-specific. | H1 ✅ |

## Root Cause

```rust
// before -- byte-size-only check, no channel-order handling
let format = texture.format();
if format.block_copy_size( None ) != Some( 4 )
{
  return Err( .. );
}
..
let pixels = rows_unpad( &data, ( width, height ) );
Ok( ( pixels, ( width, height ) ) )
```

`block_copy_size` reports bytes-per-pixel, not channel order or semantic layout -- treating it
as a stand-in for "is this an RGBA8 format" silently let through both BGRA-ordered formats
(needing a swizzle) and entirely different 4-byte pixel encodings (needing outright rejection).

## Why Not Caught

No test exercised `rgba8` (or any pure logic backing it) with a `Bgra8*` input; every existing
test in `readback_test.rs` used a format-agnostic byte buffer with no channel semantics, so
nothing ever asserted the actual byte order of the returned pixels.

## Fix Location

`module/min/minwgpu/src/readback.rs`.

```rust
// after -- explicit allowlist, swizzle for the two BGRA formats
let is_bgra = matches!( format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb );
let is_rgba = matches!( format, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb );
if !is_rgba && !is_bgra { return Err( .. ); }
..
let mut pixels = rows_unpad( &data, ( width, height ) );
if is_bgra { bgra_to_rgba_swizzle( &mut pixels ); }
```

Replaced the byte-size check with an explicit 4-format allowlist (`Rgba8Unorm`,
`Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`) -- every other 4-byte format is now rejected
outright rather than silently mislabeled. New `bgra_to_rgba_swizzle` helper swaps the red/blue
byte of every pixel in place for the two BGRA formats, fulfilling the doc's original "RGBA8
pixels" contract for real instead of narrowing it. `bgra_to_rgba_swizzle` is exposed via
`mod_interface!` specifically so this pure swizzle logic is unit-testable without a real GPU
device/texture.

## Prevention

Added 3 tests to `tests/readback_test.rs` (`bug_reproducer(BUG-166)` on the primary):
`bgra_to_rgba_swizzle_swaps_red_and_blue_per_pixel` (2-pixel buffer, asserts exact byte
reordering with green/alpha untouched), `bgra_to_rgba_swizzle_empty_is_noop`, and
`bgra_to_rgba_swizzle_panics_on_non_multiple_of_4` (malformed-input precondition).

## Pitfall

A byte-size check is not a format check -- "4 bytes per pixel" says nothing about channel
count, order, or bit layout. Several real `wgpu` formats (`Rg16*`, `R32*`, `Rgb9e5Ufloat`,
`Rgb10a2*`, `Rg11b10Ufloat`) are also 4 bytes per pixel but are not RGBA8-shaped at all; only an
explicit format allowlist can assert a function's caller-visible "returns RGBA8 pixels"
contract. When a function's own doc names specific accepted formats, the implementation must be
checked against that exact list, not a looser proxy condition that happens to also pass them.

## Generalized Version

**Broken assumption:** "if two GPU texture formats have the same byte size, a generic
byte-oriented copy between them is safe."

**Confirmed general rule:** byte size alone never implies channel-compatible layout across GPU
texture formats -- code that promises a specific channel order (e.g. "RGBA8 pixels") must
either restrict its accepted-format allowlist to formats it explicitly handles the layout of,
or perform an explicit conversion (swizzle, unpack, etc.) for every format whose native layout
differs from the promised one.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during a from-scratch Explore review of `module/min/minwgpu` (task #95); confirmed by contrasting `rgba8`'s own doc against its implementation, and cross-checking `wgpu-types` source for other formats sharing the same byte-size loophole. |
| 2026-08-16 | fixed | Replaced the byte-size check with an explicit 4-format allowlist; added `bgra_to_rgba_swizzle`, applied for the two BGRA formats after unpadding. |
| 2026-08-16 | verified | Added 3 tests to `tests/readback_test.rs`. Scoped native `cargo nextest`/`cargo clippy` clean across `minwgpu` + 4 downstream crates (all 3 real `rgba8` call sites confirmed to already use `Rgba8Unorm`/`Rgba8UnormSrgb`, so behavior is unchanged for them), 45/45 tests passing, 0 failures. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote a unit test asserting the exact byte reordering; adversarial pass independently re-derived which other `wgpu` formats share the same byte-size loophole (via `wgpu-types` source) to confirm the fix's allowlist, not just the swizzle, was necessary. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Checked and confirmed no coupling to BUG-165 (same review pass, unrelated code path and root cause) or BUG-162/163/164 (different crate). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source inspection (both this crate's implementation and `wgpu-types`' format table) plus explicit rejection of "not reachable in practice" as grounds to skip the fix (H2), using this crate's own `preferred_format` test as proof of realistic reachability. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is exactly the allowlist + swizzle needed to make the existing doc contract true; no broader refactor of `rgba8`'s buffer/mapping logic. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `minwgpu` src + 1 test file + this bug file touched; the 3 real call sites needed no changes since none pass a `Bgra8*` texture today. | — |
| D7 | Crate Locality | 🟢 | 🟢 | All 3 real call sites of `rgba8` in this workspace were identified via `grep` and their texture formats confirmed unaffected; no call site missed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `bgra_to_rgba_swizzle` is a single-purpose pure byte-swap helper, exposed via `mod_interface!` only because it needed to be unit-testable from `tests/`. | — |

**Reproduced:** YES -- pre-fix, `bgra_to_rgba_swizzle` did not exist and `rgba8` had no
channel-order handling at all, so a `Bgra8*` readback's raw bytes would be returned unchanged
under an "RGBA8 pixels" label, silently swapping red and blue relative to the documented
contract. Post-fix, `bgra_to_rgba_swizzle_swaps_red_and_blue_per_pixel` confirms the exact
byte-level correction. Scoped native `cargo nextest`/`cargo clippy` clean across `minwgpu` + 4
downstream crates, 45/45 tests passing, 0 failures, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwgpu/src/readback.rs` | `rgba8`'s format check replaced with an explicit 4-format allowlist (full `Fix(BUG-166)` comment); new `bgra_to_rgba_swizzle` helper applied for `Bgra8Unorm`/`Bgra8UnormSrgb`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwgpu/tests/readback_test.rs` | 3 new tests: `bgra_to_rgba_swizzle_swaps_red_and_blue_per_pixel` (`bug_reproducer(BUG-166)`), `bgra_to_rgba_swizzle_empty_is_noop`, `bgra_to_rgba_swizzle_panics_on_non_multiple_of_4`. |
