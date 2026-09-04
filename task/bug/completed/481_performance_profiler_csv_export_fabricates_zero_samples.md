# BUG-481: `PerformanceProfiler::csv_export` writes fabricated `0` memory values for frames with no matching sample, indistinguishable from a real zero reading

- **Severity:** Low (no crash, no incorrect frame-time data -- but memory columns silently
  fabricate data for rows where none was ever recorded)
- **state:** Completed
- **Affects:** Any consumer of `PerformanceProfiler::csv_export` where `frame_times` and
  `memory_samples` have different lengths (the common case, since they're recorded
  independently and at different cadences).
- **Component:** module/helper/tiles_tools (`src/debug.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-478 (a different `debug.rs` export defect -- `json_export` escaping,
  same file, unrelated mechanism).

## Symptom

```rust
// pre-fix -- src/debug.rs, PerformanceProfiler::csv_export
let memory = self.memory_samples.get(i).copied().unwrap_or(MemorySample { heap_used: 0, heap_total: 0, .. });
// writes memory.heap_used, memory.heap_total as "0,0" when no sample exists at index i
```

For any frame index `i` beyond `memory_samples.len()`, the CSV row wrote literal `0` for both
memory columns -- indistinguishable in the output from a genuine zero-byte memory reading.
A consumer parsing the CSV cannot tell "no measurement was taken this frame" apart from "memory
usage was measured as exactly zero this frame".

## Impact

**Who is affected:** Any consumer of `csv_export`'s output parsing memory columns, especially
tooling that computes aggregate statistics (averages, totals) across the exported rows --
fabricated zeros silently skew any such aggregate downward.

**What breaks:** Data integrity of the exported CSV's memory columns for any frame without a
matching sample -- which is the common case, since `frame_times` and `memory_samples` are
recorded independently (frame timing typically sampled every frame; memory sampled less often).

**Consumer audit:** `csv_export` is a public method; `grep -rln 'csv_export' --include="*.rs" .`
from the repo root, excluding `tiles_tools` itself, returns no external call sites -- confirmed
via direct audit.

**Magnitude:** Single zip-by-index loop; see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/debug.rs` end to end -- the `.unwrap_or(MemorySample
{ heap_used: 0, .. })` pattern is a fabricated-default anti-pattern indistinguishable from a
real reading of zero.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/debug_test.rs
profiler.frame_time_record(Duration::from_millis(16));
profiler.frame_time_record(Duration::from_millis(17)); // 2 frame times
profiler.memory_sample_record(MemorySample { heap_used: 1024, heap_total: 2048 }); // 1 memory sample
let csv = profiler.csv_export();
// row 1 (frame 0): real memory values
// row 2 (frame 1): pre-fix wrote "...,0,0" -- indistinguishable from a genuine zero reading
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(debug_test) and test(csv_export_blanks_unmatched)'
```

## Root Cause

The zip-by-index loop used `.get(i).copied().unwrap_or(MemorySample { heap_used: 0, heap_total:
0, .. })` to handle the common case of `memory_samples.len() < frame_times.len()` -- picking a
fabricated zero-valued default instead of representing "no data" as an actually-empty field,
conflating "measured zero" with "not measured" in the output format.

## Why Not Caught

No existing test exercised `csv_export` with mismatched `frame_times`/`memory_samples` lengths
-- all prior test fixtures recorded equal counts of both, where the `.unwrap_or` branch is never
taken and the defect is invisible.

## Fix Location

`module/helper/tiles_tools/src/debug.rs`: `csv_export`'s zip-by-index logic changed from
`self.memory_samples.get(i).copied().unwrap_or(MemorySample{..0..})` to a `match
self.memory_samples.get(i) { Some(sample) => write real values, None => write blank fields }`
pattern -- rows without a matching memory sample now have genuinely empty memory columns instead
of fabricated zeros.

## Prevention

New test `test_performance_profiler_csv_export_blanks_unmatched_memory_rows` in
`tests/debug_test.rs` records 2 frame_times but only 1 memory_sample, writes the CSV to a temp
file, and asserts row 0 has real values while row 1 ends with `",,"` (empty fields) and does
**not** contain `",0,0"` (the old fabricated-zero pattern).

## Pitfall

`.unwrap_or(<zero-valued-default>)` is a convenient way to avoid handling the `None` case
explicitly, but it silently converts "no data" into "a real reading of zero" wherever the
default happens to be the zero value of the type -- this is invisible in the type system (both
are valid `MemorySample` values) and only shows up as a data-integrity defect in the consumer's
own downstream aggregation. Any export/serialization format needs an explicit way to represent
"no data" (an empty field, a `null`, an omitted column) distinct from a real zero.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/debug.rs` end to end. |
| 2026-08-20 | fixed | `csv_export`'s memory-column logic changed from a fabricated-zero default to genuinely blank fields for unmatched rows. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: confirmed the test's assertion (`row 1` does not contain `",0,0"`) genuinely fails against the pre-fix `.unwrap_or(MemorySample{..0..})` pattern (which would write exactly `",0,0"` for the unmatched row) and passes against the fix. | — |
| D2 | Matched-row behavior preserved | — | 🟢 | Confirmed row 0 (which has a real matching memory sample) still writes its real recorded values unchanged -- the fix only changes behavior for genuinely unmatched rows, not matched ones. `cargo nextest run -p tiles_tools --all-features` -- 286/286 pass. | — |

**Reproduced:** YES -- `test_performance_profiler_csv_export_blanks_unmatched_memory_rows`'s
assertion that row 1 does not contain `",0,0"` is false against the pre-fix
`.unwrap_or(MemorySample{..0..})` default (verified by direct inspection of the pre-fix
zip-by-index logic) and true against the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/debug.rs` | `PerformanceProfiler::csv_export`'s memory-column logic changed from a fabricated-zero `.unwrap_or` default to genuinely blank fields for frames with no matching memory sample; `Fix(BUG-481)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/debug_test.rs` | Added `test_performance_profiler_csv_export_blanks_unmatched_memory_rows`, exercising mismatched `frame_times`/`memory_samples` lengths and asserting blank (not fabricated-zero) output for unmatched rows. |
