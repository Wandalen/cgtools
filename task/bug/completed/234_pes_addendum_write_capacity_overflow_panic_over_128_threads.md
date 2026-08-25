# BUG-234: `pes_addendum_write` panics with "capacity overflow" for designs with more than 128 threads

- **Severity:** High (a legitimate, format-valid input -- a design with 129-255
  thread/color-change entries, explicitly allowed by this very file's own sibling bounds
  check -- crashes the writer with an unhandled panic instead of returning a catchable
  error)
- **state:** Completed
- **Affects:** `pes::write( ..., PESVersion::V6 )` for any `EmbroideryFile` whose
  `threads().len()` (equivalently, `pec::content_write`'s returned `color_indices.len()`)
  falls in `129..=255`. `PESVersion::V1` is unaffected (`version1_write` never calls
  `pes_addendum_write`).
- **Component:** `module/helper/embroidery_tools` (`src/format/pes/writer.rs`,
  `pes_addendum_write`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** None directly, but shares this crate's established convention (see
  BUG-152's `add_value >= 255` precedent in `pec_header_write`) of reporting "too many X
  for this format" via an explicit bounds check and `EmbroideryError::CompatibilityError`
  -- `pes_addendum_write` was the one place in this file that convention wasn't followed.

## Symptom

```rust
// pre-fix
fn pes_addendum_write< W >( writer : &mut W, color_indices : &[ usize ], rgb_list : &[ Color ] )
-> Result< (), EmbroideryError >
where W : Write
{
  let count = color_indices.len();
  let color_indices : Vec< _ > = color_indices.iter().map( | v | *v as u8 ).collect();
  let spaces = vec![ 0x20_u8; 128_usize.wrapping_sub( count ) ]; // underflows for count > 128
  ...
}
```

`color_indices` comes from `pec::content_write`'s return value, whose length equals
`emb.threads().len()` (one entry per thread, via `unique_palette_build`'s final
`for thread in threadlist` loop -- not deduplicated to unique colors). The only existing
guard on this length is `pec_header_write`'s own `if add_value >= 255 { return Err(...) }`
(`add_value = current_thread_count - 1`), which allows up to 255. For any `count` in
`129..=255`, `128_usize.wrapping_sub( count )` underflows to a value in
`[usize::MAX - 126, usize::MAX]` -- every one of which lands far past `isize::MAX`, so
`vec![0x20_u8; ...]` immediately panics with `"capacity overflow"` (Rust's `RawVec`
capacity check) instead of returning a `Result::Err`.

## Impact

**Who is affected:** Any caller writing an `EmbroideryFile` with 129-255 distinct
thread/color-change entries to PES v6 -- not a pathological edge case: gradient/photo-stitch
embroidery designs routinely exceed 128 color changes.

**What breaks:** `pes::write( ..., PESVersion::V6 )` panics instead of returning
`Result::Err`, crashing (or, in a multi-threaded host, poisoning) whatever thread called it,
with no chance for the caller to handle the condition gracefully -- unlike every other
"value exceeds this format's capacity" case in the same file, which reports through
`try_from`/an explicit bounds check and a real `EmbroideryError`.

**Magnitude:** 1 function (`pes_addendum_write`), 1 missing bounds check.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `embroidery_tools`'s `format/pes/writer.rs`, reading
`pes_addendum_write` in full and noting it was the only bounds-sensitive computation in the
file using `wrapping_sub` fed directly into an allocation size, instead of the `try_from`+
explicit-error convention used by every numeric conversion elsewhere in the same file
(including this function's own sibling `pec_header_write`'s "too many color changes" check).

## Minimum Reproducible Example

```rust
let mut emb = EmbroideryFile::new();
emb.stitch( 0, 0 );
emb.end();
let default_palette = pec::pec_threads();
for i in 0..129
{
  emb.thread_add( default_palette[ 1 + ( i % ( default_palette.len() - 1 ) ) ].clone() );
}
let mut memory = vec![ 0_u8; 4096 ];
let mut writer = Cursor::new( &mut memory );
let result = pes::write( &mut emb, &mut writer, pes::PESVersion::V6 );
// pre-fix: panics with "capacity overflow" instead of returning this Err
assert!( matches!( result, Err( EmbroideryError::CompatibilityError( _ ) ) ) );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo nextest run --all-features -E 'test(version6_write_with_more_than_128_threads_errors_instead_of_panicking)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `pes_addendum_write`'s `128_usize.wrapping_sub( count )` underflows whenever `count > 128`, and since `pec_header_write`'s own guard allows `count` up to 255, this is reachable via ordinary, format-valid input, producing a "capacity overflow" panic instead of a `Result::Err`. | ✅ Root Cause | Direct read plus arithmetic trace confirms `count` can reach 255 without erroring upstream, and that the wrapped value always exceeds `isize::MAX`, guaranteeing the panic path (not a slow/hanging allocation attempt). Confirmed empirically via temporary-revert-and-rerun (exact "capacity overflow" panic reproduced at the predicted line). | E1, E2, E3, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/embroidery_tools/src/format/pes/writer.rs`, `pes_addendum_write` (pre-fix, direct read) | `let spaces = vec![ 0x20_u8; 128_usize.wrapping_sub( count ) ];` -- no bounds check on `count` before the subtraction. | H1 ✅ |
| E2 | `module/helper/embroidery_tools/src/thread.rs`, `unique_palette_build` (direct read) | The function's final loop is `for thread in threadlist { palette.push( ... ) }` -- one output entry per thread in `threadlist`, not per unique color, so the returned `Vec`'s length equals `emb.threads().len()` exactly. | H1 ✅ |
| E3 | `module/helper/embroidery_tools/src/format/pec/writer.rs`, `pec_header_write` (direct read) | `let add_value = current_thread_count - 1; if add_value >= 255 { return Err(...) }` -- the only upstream guard, allowing `current_thread_count` (== `color_indices.len()`) up to 255, well past `pes_addendum_write`'s unstated 128 limit. | H1 ✅ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting the `count > 128` guard and re-running the new test with 129 threads reproduced `thread '...' panicked at .../raw_vec/mod.rs:28:5: capacity overflow` exactly, confirming both the underflow and its panic (not silent-corruption or hang) failure mode. | H1 ✅ |

## Root Cause

`pes_addendum_write` writes PES v6's fixed 128-byte addendum color-index field by padding
`color_indices` (one byte per thread/color-change entry) with spaces up to 128 bytes total,
computing the pad length as `128_usize.wrapping_sub( count )`. The function assumed `count`
would never exceed 128, but the only bounds check actually enforced upstream
(`pec_header_write`'s "too many color changes" guard) allows up to 255 -- a design with
129-255 threads passes that check cleanly and reaches `pes_addendum_write` with an
un-representable `count`. `wrapping_sub` silently wraps instead of erroring, producing a
`usize` value near `usize::MAX`; passed to `vec![0x20_u8; ...]`, this exceeds the maximum
allocation size the standard library permits (`isize::MAX` bytes), tripping `RawVec`'s
"capacity overflow" panic.

## Why Not Caught

No existing test wrote a design with more than a handful of threads -- the largest,
`write_v6_matches_reference_fixture`, uses only 2. The 128-byte addendum limit is a PES v6
format constraint entirely separate from (and stricter than) PEC's own 255-thread limit,
and nothing in `pec_header_write`'s guard comment flagged that a second, stricter downstream
limit existed.

## Fix Location

`module/helper/embroidery_tools/src/format/pes/writer.rs`: `pes_addendum_write` now returns
`EmbroideryError::CompatibilityError` when `count > 128`, before the subtraction, mirroring
`pec_header_write`'s own "too many color changes" convention. The subsequent
`128_usize.wrapping_sub( count )` was changed to a plain `128 - count`, now provably safe
under the new guard.

## Prevention

`tests/pes_test.rs::version6_write_with_more_than_128_threads_errors_instead_of_panicking`
writes a design with 129 threads to PES v6 and asserts a `CompatibilityError` comes back
instead of a panic.

## Pitfall

Every other "value exceeds this format's capacity" case in `format/pes/writer.rs` reports
through `try_from`/an explicit bounds check and a real `EmbroideryError` -- `wrapping_sub`
fed straight into an allocation size was the one place that convention wasn't followed. A
downstream function must never assume an upstream bounds check enforces *its own* stricter
limit; each format-capacity constraint needs its own explicit guard at the point it actually
applies, not an inherited assumption from a sibling check with a looser threshold.

## Generalized Version

**Broken assumption:** "the caller already validated this value is small enough, because
some bounds check exists somewhere upstream in the call chain."

**Confirmed general rule:** When two different downstream consumers of the same value have
different capacity limits (here: PEC's 255-thread limit vs. PES v6 addendum's 128-byte
limit), the looser upstream check does not substitute for the stricter one -- each consumer
must enforce its own limit explicitly, at the point where exceeding it would otherwise
corrupt output or crash. `wrapping_sub`/`wrapping_*` arithmetic feeding directly into an
allocation size or array index is a red flag distinct from ordinary unsigned-subtraction
underflow risk: it silently converts a bounds violation into a value that looks like "just a
very large number" until it reaches something (an allocator, an indexing op) that turns it
into a crash.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `embroidery_tools` scouting pass (Batch 2), reading `format/pes/writer.rs` in full and noting `pes_addendum_write`'s `wrapping_sub`-into-allocation-size as the file's one exception to its own established `try_from`+explicit-error convention. |
| 2026-08-17 | fixed | `pes_addendum_write` now returns `EmbroideryError::CompatibilityError` for `count > 128`, checked before the (now-safe, non-wrapping) subtraction. |
| 2026-08-17 | verified | `cargo nextest run -p embroidery_tools --all-features`: 14/14 passed, 0 skipped. `cargo test --doc -p embroidery_tools --all-features`: 0 doctests (crate has none). `cargo clippy -p embroidery_tools --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (exact `capacity overflow` panic reproduced pre-fix at `raw_vec/mod.rs:28:5`, passed cleanly post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, `matches!` against a specific error variant is an exact, non-flaky check. Adversarial pass: checked whether the panic could instead manifest as a slow/hanging multi-exabyte allocation attempt rather than a fast panic (which would make the regression test itself dangerous to run repeatedly) -- traced that every reachable wrapped value (`count` in `129..=255`) lands in `[usize::MAX-126, usize::MAX]`, all provably past `isize::MAX`, so Rust's `RawVec` capacity check rejects it before any real allocation is attempted; confirmed empirically (sub-20ms test run, no memory pressure observed). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified `pec_header_write`'s `add_value >= 255` check (BUG-152's file) as the precedent convention this fix now follows, and confirmed it does NOT itself already guard the stricter 128 limit. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `pes_addendum_write`, `unique_palette_build`, and `pec_header_write`, plus empirical revert-rerun proof matching the predicted panic location and message exactly. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to `pes_addendum_write`'s bounds check and the now-safe subtraction. Adversarial pass: grepped `format/pes/writer.rs` and `format/pec/writer.rs` for other `wrapping_*` uses -- none found; this was the only instance of the pattern in the crate. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `pes_addendum_write`; its signature (still `Result< (), EmbroideryError >`) and all callers are unchanged -- `version6_write` already propagates the `?` from this call, so the new error path requires no caller update. | — |

**Reproduced:** Confirmed via `cargo nextest` (exact "capacity overflow" panic pre-fix,
clean `CompatibilityError` post-fix) and temporary direct-source-edit revert-and-rerun.
2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/format/pes/writer.rs` | `pes_addendum_write` now returns `EmbroideryError::CompatibilityError` for `count > 128` before the subtraction; `wrapping_sub` replaced with a plain, now-safe `128 - count` (full `Fix(BUG-234)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/pes_test.rs` | Added `version6_write_with_more_than_128_threads_errors_instead_of_panicking` (`bug_reproducer(BUG-234)`, 5-section doc comment), placed after `v6_roundtrip_preserves_metadata_and_threads`; added `EmbroideryError` import. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects. |
