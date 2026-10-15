# BUG-161: `sprite_upload`'s row/column computation panics via divide-by-zero when `sprites_in_row` is 0

- **Severity:** Medium (crashes via an unattributed arithmetic panic rather than corrupting data
  silently, but `SpriteSheet` is a fully-public, constructor-less struct with no validation
  choke point, so any consumer computing `sprites_in_row` from other data -- e.g. a sprite wider
  than the source image -- can reach this with ordinary, non-adversarial input)
- **state:** Completed
- **Affects:** `sprite_upload` -- any caller whose `SpriteSheet.sprites_in_row` is `0`
- **Component:** `module/min/minwebgl` (`src/texture/d2.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Co-located in the same function (`sprite_upload`) as BUG-160, but a distinct
  root cause (a divide-by-zero vs. mip-level math) -- filed and fixed separately.

## Symptom

```rust
// pre-fix, inline in sprite_upload's per-sprite loop, no zero-guard:
let col = index % sprites_in_row * sprite_width; // sprites_in_row = 0 -> panics
let row = index / sprites_in_row * sprite_height; // "attempt to calculate the remainder with a divisor of zero"
```

## Impact

**Who is affected:** Any caller constructing a `SpriteSheet` with `sprites_in_row: 0` -- e.g.
derived from `image_width / sprite_width` where a sprite is wider than the source image, or any
other computed-rather-than-literal construction path. `SpriteSheet` has no constructor
(deliberately kept exhaustive, non-`#[non_exhaustive]`, so external crates can struct-literal
-construct it directly -- documented in its own doc comment citing the
`examples/minwebgl/sprite_animation` call site), so there is no single choke point where this
could be rejected before reaching `sprite_upload`.

**What breaks:** The process panics with Rust's raw `"attempt to calculate the remainder with a
divisor of zero"` message, attributing the failure to the arithmetic operation itself rather
than to the actual precondition (`sprites_in_row` must be nonzero).

**Magnitude:** Total for the affected input -- immediate panic, no silent corruption, but an
unattributed crash instead of a clear error.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Independent re-derivation: dispatched a background Explore agent to read 100% of `minwebgl`'s
`src/` (36 files) from scratch. It flagged the unguarded `%`/`/` by `sprites_in_row` in
`sprite_upload`'s per-sprite loop as a candidate; independently re-verified by reading
`src/texture/d2.rs` directly and confirming `SpriteSheet` has no constructor to validate this
field at construction time.

## Minimum Reproducible Example

```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests sprite_upload_test::sprite_position_rejects_zero_sprites_in_row 2>&1 | tail -6
```

**Expected** (post-fix):
```
test sprite_upload_test::sprite_position_rejects_zero_sprites_in_row ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the extracted guard):
```
thread 'sprite_position_rejects_zero_sprites_in_row' panicked at module/min/minwebgl/src/texture/d2.rs:292:13:
attempt to calculate the remainder with a divisor of zero
Summary [ 0.012s] 1 test run: 0 passed, 1 failed, 0 skipped
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests sprite_upload_test::sprite_position
# 2 "ok" = fixed; a raw divisor-of-zero panic = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `sprite_upload`'s row/column computation divides and modulos by `sprites_in_row` with no zero-guard, and `SpriteSheet` has no construction-time choke point to prevent `sprites_in_row: 0` from reaching it. | ✅ Root Cause | Read `src/texture/d2.rs` directly: the per-sprite loop computes `index % sprites_in_row` / `index / sprites_in_row` unconditionally; `SpriteSheet`'s own doc comment confirms it is deliberately kept exhaustive/constructor-less for external struct-literal construction. | E1, E2 |
| H2 | The crate's real caller always supplies a hand-written nonzero literal, so this can't occur via any exercised path. | Confirmed (not a defense) | True for the one existing example caller, but doesn't change that `SpriteSheet`'s public, constructor-less shape permits any future or external caller to construct it with a computed (and possibly zero) value -- consistent with H1, not a rebuttal of it. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/texture/d2.rs` (pre-fix, unedited) | `let col = index % sprite_sheet.sprites_in_row * sprite_width; let row = index / sprite_sheet.sprites_in_row * sprite_height;` -- no `sprites_in_row == 0` check anywhere in the function. | H1 ✅ |
| E2 | `SpriteSheet`'s doc comment (`src/texture/d2.rs`) | States the struct is deliberately kept exhaustive (non-`#[non_exhaustive]`) so `examples/minwebgl/sprite_animation` can construct it via struct literal -- confirming there is no constructor to validate fields at. | H1 ✅ |
| E3 | `tests/sprite_upload_test.rs::sprite_position_rejects_zero_sprites_in_row` (real run) | Reverting `sprite_position`'s body to the unguarded pre-fix formula and re-running: real panic `attempt to calculate the remainder with a divisor of zero` at the exact division site. | H1 ✅ |

## Root Cause

```rust
// before (inline in sprite_upload's per-sprite loop, no zero-guard)
for i in 0..sprite_sheet.amount
{
  let col = i % sprite_sheet.sprites_in_row * sprite_sheet.sprite_width;
  let row = i / sprite_sheet.sprites_in_row * sprite_sheet.sprite_height;
  // ...
}
```

`SpriteSheet` has no constructor and is deliberately kept exhaustive for external
struct-literal construction, so a caller-computed `sprites_in_row: 0` (e.g. derived from a
sprite wider than the source image) panicked via integer division-by-zero instead of a clear
message.

## Why Not Caught

The computation lived inline in `sprite_upload` with no standalone function to call directly,
and this crate's only real caller always supplies a nonzero, hand-written literal.

## Fix Location

`module/min/minwebgl/src/texture/d2.rs`.

```rust
// after: extracted into a standalone, Result-returning pure fn
pub fn sprite_position( index : u32, sprites_in_row : u32, sprite_width : u32, sprite_height : u32 ) -> Result< ( u32, u32 ), WebglError >
{
  if sprites_in_row == 0
  {
    return Err( WebglError::NotSupportedForType( "SpriteSheet::sprites_in_row must be > 0" ) );
  }
  let col = index % sprites_in_row * sprite_width;
  let row = index / sprites_in_row * sprite_height;
  Ok( ( col, row ) )
}

// sprite_upload's per-sprite loop now propagates via `?` (it already returns Result)
let ( col, row ) = sprite_position( i, sprite_sheet.sprites_in_row, sprite_sheet.sprite_width, sprite_sheet.sprite_height )?;
```

## Prevention

Added `tests/sprite_upload_test.rs` (shared file with BUG-160, includes `bug_reproducer(BUG
-161)`): a happy-path 4-sprite/2-per-row case, plus a regression test asserting
`sprites_in_row: 0` returns `Err(WebglError::NotSupportedForType(_))` instead of panicking.

## Pitfall

A fully-public, constructor-less struct (kept exhaustive so external crates can
struct-literal-construct it, see `SpriteSheet`'s own doc comment) has no single choke point to
validate fields at construction time -- every consumer of a field must guard independently.

## Generalized Version

**Broken assumption:** "the struct's only real caller always supplies valid field values, so no
validation is needed downstream." False once a struct is public and constructor-less by design
-- any future or external caller can construct it with computed, possibly-degenerate values, and
there is no single point to catch that short of guarding every consumer of the field
independently.

**Confirmed general rule:** when a struct is deliberately kept exhaustive/constructor-less for
external struct-literal construction, treat every function that consumes one of its fields in a
division, modulo, or other partial operation as needing its own guard -- the struct's own shape
guarantees no upstream validation will ever exist.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by an independent Explore-agent re-derivation of minwebgl's `src/` (36 files, task #93); independently re-verified by reading `src/texture/d2.rs` directly and confirming `SpriteSheet`'s constructor-less, exhaustive shape. |
| 2026-08-16 | fixed | Extracted `sprite_position`, returning `Result<(u32,u32), WebglError>` (`WebglError::NotSupportedForType`) instead of panicking when `sprites_in_row == 0`; `sprite_upload` propagates via `?`. |
| 2026-08-16 | verified | Added `tests/sprite_upload_test.rs` tests via in-place revert-test-restore against the real guard: captured the real pre-fix divide-by-zero panic, restored, confirmed passing. Scoped crate suite (13 tests) + `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against the actual extracted function; adversarial pass performed a real in-place revert-test-restore (removed the zero-guard), capturing the actual divide-by-zero panic before restoring. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-159/BUG-160 (same investigation batch; BUG-160 shares this function but a distinct root cause) -- no cross-dependency. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading and `SpriteSheet`'s own doc comment confirming its deliberate constructor-less, exhaustive shape. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only the row/column computation in `sprite_upload`'s per-sprite loop changed; rest of the function untouched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `minwebgl` src + test + bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is a function extraction plus a one-line call-site substitution (`?` propagation); no signature change to `sprite_upload` beyond its existing `Result` return type. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | New public fn (`sprite_position`) has one responsibility (position computation with validation), exported as a plain top-level `pub fn` matching `texture/d2.rs`'s existing non-`mod_interface` convention. | — |

**Reproduced:** YES -- `sprite_position_rejects_zero_sprites_in_row` was confirmed to fail with
the exact predicted divide-by-zero panic (`attempt to calculate the remainder with a divisor of
zero`) when `sprite_position`'s zero-guard was temporarily removed; restoring the guard returns
the test to passing. Scoped crate suite (13 tests) + `cargo clippy -p minwebgl --all-targets
--all-features -- -D warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/texture/d2.rs` | Extracted `sprite_position`, returning `Result<(u32,u32), WebglError>`; `sprite_upload`'s per-sprite loop now calls it via `?` (full `Fix(BUG-161)` root cause/pitfall comment). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/sprite_upload_test.rs` | New file (shared with BUG-160): includes the happy-path case and the `bug_reproducer(BUG-161)` zero-guard regression test. |
| `module/min/minwebgl/tests/readme.md` | Added Responsibility Table row for `sprite_upload_test.rs` (shared with BUG-160). |
