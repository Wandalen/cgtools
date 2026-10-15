# BUG-476: `data_decompress`'s corrupted-header length check adds an untrusted `u32` to a constant, overflow-prone on 32-bit targets

- **Severity:** Medium (unreachable on the 64-bit hosts this workspace develops/tests on, but
  real on `wasm32-unknown-unknown`, this crate's stated primary target, where a corrupted or
  malicious save file's header could wrap the bounds check meant to reject it)
- **state:** Completed
- **Affects:** Any `wasm32-unknown-unknown` consumer of `GameStateSerializer::game_state_deserialize`
  (or `SaveManager::game_state_load`) fed a corrupted or adversarial `.save` file with
  `with_compression(true)`.
- **Component:** module/helper/tiles_tools (`src/serialization.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-475 (the compression-stub fix this bug's resolution rode along with,
  structurally eliminating the vulnerable arithmetic as a side effect of the redesign).

## Symptom

```rust
// pre-fix -- src/serialization.rs
fn data_decompress(data: &[u8]) -> Result<Vec<u8>, SerializationError> {
  if data.len() < 7 || data[0..3] != [0xC0, 0x4D, 0x50] {
    return Err(SerializationError::InvalidCompressionFormat);
  }
  let original_size = u32::from_le_bytes([data[3], data[4], data[5], data[6]]) as usize;
  if data.len() != original_size + 7 {
    return Err(SerializationError::CorruptedData);
  }
  Ok(data[7..].to_vec())
}
```

`original_size` is parsed directly from the (potentially corrupted or adversarial) input bytes,
then added to the constant `7` with no overflow guard. On a target where `usize` is 32 bits
(wasm32-unknown-unknown), a crafted `original_size` near `u32::MAX` makes `original_size + 7`
wrap around past `usize::MAX`, corrupting the very check meant to reject malformed input.

## Impact

**Who is affected:** wasm32-unknown-unknown consumers only -- on a 64-bit host, `usize` is 64
bits and `u32::MAX + 7` is nowhere near 64-bit `usize::MAX`, so the addition cannot overflow
there regardless of input; this bug is categorically unreachable on the x86_64 development/test
host this crate is built and tested on.

**What breaks:** On a vulnerable 32-bit target, in a debug build the overflow panics
(`overflow-checks` is on by default in `dev`/`test` profiles); in a release build it silently
wraps, potentially letting a bogus length check pass and returning corrupted/undersized data
from a call site that expected either valid data or a clean `Err`.

**Consumer audit:** `data_decompress` is private, reached only via
`game_state_deserialize`/`SaveManager::game_state_load` with `with_compression(true)`. No
external call sites (see BUG-475's consumer audit, same code path).

**Magnitude:** Single arithmetic expression; see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/serialization.rs` end to end alongside BUG-475 --
noticed the unchecked `original_size + 7` addition on attacker-controllable input while
redesigning `data_compress`/`data_decompress` for real compression.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/serialization_test.rs
let mut corrupted = vec![0xC0, 0x4D, 0x50];
corrupted.extend_from_slice(&(u32::MAX - 1).to_le_bytes());
corrupted.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
let result = GameStateSerializer::new().with_compression(true).game_state_deserialize(&corrupted);
assert!(matches!(result, Err(SerializationError::CorruptedData)));
```

**Note:** this specific crafted input does **not** panic on the x86_64 host this test actually
runs on -- see Why Not Caught / Pitfall for why the overflow is architecture-specific and this
test cannot be a literal fail-before/pass-after reproducer here.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(serialization_test) and test(rejects_corrupted_header)'
```

## Root Cause

The old format could validate a compressed payload's exact on-disk length up front
(`data.len() != original_size + 7`) because the "compressed" bytes were literally an
unmodified copy of the input (see BUG-475) -- the addition was safe *in that format* only
because nothing about it depended on `original_size` being independently untrustworthy in a way
that mattered arithmetically. Once real compression (BUG-475's fix) would decouple the on-disk
length from the original size, re-deriving this check from scratch was required, not just
re-verifying it still compiled.

## Why Not Caught

No existing test fed `data_decompress`/`game_state_deserialize` a corrupted or adversarial
header before this fix -- every decompression test only ever round-tripped through
`data_compress`'s own well-formed output. Separately, and more fundamentally: this class of bug
is architecture-specific. `usize` is 64 bits on the x86_64 host this repo builds and tests on,
so `u32::MAX + 7` can never overflow here regardless of which code path runs -- no amount of
running `cargo test` on this host would ever have caught it. It is only reachable on a 32-bit
target such as wasm32-unknown-unknown, which this workspace does not run its test suite against
directly (native `cargo nextest` only).

## Fix Location

`module/helper/tiles_tools/src/serialization.rs`: fixed structurally as a side effect of
BUG-475's redesign, not via a `checked_add` patch on the old formula. The new
`data_decompress` performs no arithmetic on the untrusted `original_size` at all -- it inflates
the payload first via `flate2::read::DeflateDecoder`, then compares the *already-computed*
inflated length directly against `original_size`
(`if decompressed.len() != original_size { return Err(SerializationError::CorruptedData); }`)
-- an equality check between two independently-bounded `usize` values, with no addition of
untrusted input anywhere in the vulnerable path.

## Prevention

New test `test_deserialize_rejects_corrupted_header_with_near_max_original_size` in
`tests/serialization_test.rs` feeds a header claiming a near-`u32::MAX` decompressed size
paired with a short, invalid-as-DEFLATE payload, and asserts a clean
`Err(SerializationError::CorruptedData)` -- permanent regression coverage that a hostile header
is always rejected gracefully, on every target, even though the specific historical overflow
class cannot be triggered as a crash on this test host (see Pitfall).

## Pitfall

A length/bounds check written as `data.len() != untrusted_value + constant` needs to be
re-derived, not just re-verified, whenever the surrounding data format changes -- the addition
was only ever safe because the old format guaranteed a relationship between `untrusted_value`
and `data.len()` that no longer holds once the format changes. Separately: this overflow class
is real on the crate's stated wasm32-unknown-unknown target but categorically unreachable on
the x86_64 host this repo's `cargo nextest` runs on, since `usize` is 64 bits here -- a test
that "passes" on this host provides no evidence either way about the 32-bit-specific overflow;
verifying that class of fix directly requires either a wasm32 target run or, as done here, a
manual 32-bit arithmetic audit confirming the vulnerable addition no longer exists in the fixed
code path at all.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found alongside BUG-475 during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/serialization.rs` end to end. |
| 2026-08-20 | fixed | Fixed structurally as a side effect of BUG-475's redesign -- the new `data_decompress` compares inflated length against `original_size` directly, with no addition of untrusted input in the vulnerable path. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Overflow elimination confirmed | — | 🟢 | Adversarial pass: manually audited the new `data_decompress` body line by line for any remaining arithmetic on untrusted input -- confirmed the only operation on `original_size` is a direct equality comparison against `decompressed.len()`, no addition/subtraction/multiplication anywhere in the path. `original_size` itself is bounded to `u32::MAX` by construction (parsed from a 4-byte LE field), so even the comparison itself cannot overflow on any target. | — |
| D2 | Regression test honesty | — | 🟢 | Confirmed the test's own doc comment explicitly discloses it cannot reproduce a crash on this x86_64 host (both old and new code return `Err` here) -- avoiding a misleading fail-before/pass-after claim neither side of the fix actually exhibits on this host. | — |

**Reproduced:** NO on this host, by architectural necessity -- `usize` is 64 bits on the x86_64
test host, so `u32::MAX + 7` never overflows here regardless of which code (old or new) runs;
both return `Err(CorruptedData)` for the crafted input. The fix's correctness was instead
verified by direct code audit confirming the vulnerable addition is structurally absent from
the new code path (see D1 above), consistent with this bug's own Pitfall section. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/serialization.rs` | `data_decompress` rewritten (jointly with BUG-475) so no arithmetic is performed on the untrusted `original_size` field; `Fix(BUG-476)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/serialization_test.rs` | Added `test_deserialize_rejects_corrupted_header_with_near_max_original_size`, feeding a near-`u32::MAX`-claiming corrupted header and asserting a clean `Err`, with doc comments disclosing this cannot reproduce a crash on the x86_64 test host. |
