# BUG-478: `ECSInspector::json_export` emits invalid JSON for any string containing a quote/backslash/control character, and silently omits per-entity data

- **Severity:** Medium (no crash -- but output claiming to be JSON was invalid JSON for
  realistic input, and silently covered less ground than the sibling `report_generate` method)
- **state:** Completed
- **Affects:** Any consumer of `ECSInspector::json_export` piping output to a JSON parser, or
  relying on it for the same entity-level detail `report_generate` provides.
- **Component:** module/helper/tiles_tools (`src/debug.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-347 (a different `ECSInspector` defect -- `entity_record` component-count
  inflation, already fixed, unrelated mechanism).

## Symptom

```rust
// pre-fix -- src/debug.rs, ECSInspector::json_export (hand-rolled string building)
format!("\"{}\": {}", component_name, count) // component_name interpolated raw, unescaped
```

A component name, system name, or entity data value containing `"`, `\`, a newline, or another
JSON control character produced output that was not valid JSON -- feeding it to any JSON
parser (e.g. `serde_json::from_str`) would fail or, worse, silently parse to a different
structure than intended. Separately, `json_export` never emitted anything about individual
entities (ids, components, position, custom data) -- `report_generate`, the sibling
human-readable export, did.

## Impact

**Who is affected:** Any consumer treating `json_export`'s output as machine-readable JSON, or
expecting entity-level parity with `report_generate`.

**What breaks:** Malformed/rejected JSON for any component/system name or data value containing
a quote, backslash, or control character -- plausible for user-authored component/entity names
in a real game. Missing entity data made the export strictly less informative than its
human-readable sibling despite sharing the "export debug state" purpose.

**Consumer audit:** `json_export` is a public method with no call sites elsewhere in the
workspace (`grep -rln 'json_export' --include="*.rs" .` from the repo root, excluding
`tiles_tools` itself, returns none) -- confirmed via direct audit.

**Magnitude:** One method (`json_export`), plus a new shared helper (`utils::json_string_escape`)
reused at every string-interpolation site within it.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/debug.rs` end to end and comparing `json_export`
against its sibling `report_generate` for scope parity.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/debug_test.rs
inspector.entity_record(entity_id, &["a \"quoted\" component".to_string()]);
let json = inspector.json_export();
let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
// pre-fix: serde_json::from_str fails -- the unescaped `"` inside the component name
// terminates the JSON string literal early, corrupting the surrounding structure
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(debug_test) and test(json_export_escapes)'
```

## Root Cause

`json_export` was hand-rolled with `format!`/string interpolation directly against raw
component/system/entity-data strings, with no escaping step -- every other string-building path
in this module (`report_generate`, `csv_export`) has no JSON-specific correctness requirement,
so this was the only site where raw interpolation was actually unsafe, and it was never singled
out for special handling.

## Why Not Caught

No existing test fed `json_export` a component/system name containing a JSON special character
-- all prior tests used simple alphanumeric identifiers, which never exercised the missing
escaping. No existing test compared `json_export`'s scope against `report_generate`'s, so the
missing entity data was never flagged as an inconsistency.

## Fix Location

`module/helper/tiles_tools/src/debug.rs`: added `pub fn json_string_escape(s: &str) -> String`
to the existing `pub mod utils { ... }` block, implementing JSON quote/backslash/control-character
escaping per the JSON spec. Rewrote `ECSInspector::json_export` to escape every interpolated
string value via this helper; sort `component_counts`/`system_timings`/entities for deterministic
output; and add a new `"entities"` array with per-entity `id`/`components`/`position`/`data`
records, aligning its scope with `report_generate`. Judgment call: did **not** wire `serde_json`
into `debug.rs`'s `src/` code for this -- `serde_json` is only available as a `src/`-usable
dependency behind this crate's `serialization` feature, while `debug.rs` is gated only by the
narrower `enabled` feature; adding a `serialization`-feature dependency to a module that doesn't
otherwise require it would be a feature-gating regression. Hand-rolled escaping via the new
`utils::json_string_escape` avoids that coupling while still producing valid JSON.

## Prevention

New test `test_ecs_inspector_json_export_escapes_and_includes_entities` in
`tests/debug_test.rs` records an entity with a component name containing `"` and a data value
containing `\n`/`\\`, a system timing with `\\` in its name, calls `json_export()`, parses the
result via `serde_json::from_str` (asserting valid JSON -- `serde_json` is available
unconditionally as a `[dev-dependencies]` entry, not feature-gated, so this test can validate
JSON correctness without requiring `debug.rs`'s own `src/` code to depend on it), and asserts
escaped values round-trip correctly plus the new `entities` array is present and correct.

## Pitfall

Hand-rolled string interpolation into a format with its own escaping rules (JSON, in this case)
is unsafe the moment the interpolated value is not fully controlled by the code doing the
interpolating -- component/system names and entity data values are effectively user input in a
game engine context. A test suite that only ever exercises "nice" alphanumeric identifiers gives
zero signal about this class of defect; testing with the format's own special characters
(quotes, backslashes, control characters) is required to catch it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, comparing `json_export` against `report_generate` for scope and correctness. |
| 2026-08-20 | fixed | Added `utils::json_string_escape`; rewrote `json_export` to escape all interpolated strings, sort output for determinism, and include per-entity data matching `report_generate`'s scope. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | JSON validity under adversarial input | — | 🟢 | Adversarial pass: crafted input specifically containing `"`, `\n`, `\\` in both component names and data values (not just one) -- confirmed `serde_json::from_str` parses the result and every escaped value round-trips to its original unescaped form. | — |
| D2 | Feature-gating correctness | — | 🟢 | Confirmed `json_string_escape` uses no `serde_json` symbols (hand-rolled char-by-char escaping) so `debug.rs`'s `enabled`-gated `src/` code introduces no new dependency on the `serialization` feature; confirmed the test file's use of `serde_json` is a `[dev-dependencies]`-only, unconditional dependency, not gated by `serialization`. | — |
| D3 | Full-crate regression | — | 🟢 | `cargo nextest run -p tiles_tools --all-features` -- 286/286 pass; `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` clean. | — |

**Reproduced:** YES -- `test_ecs_inspector_json_export_escapes_and_includes_entities`'s
`serde_json::from_str(&json).unwrap()` call fails against the pre-fix hand-rolled interpolation
for the crafted quote-containing component name (verified by inspection of the pre-fix
interpolation code, which had no escaping step at all) and passes against the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/debug.rs` | Added `utils::json_string_escape`; rewrote `ECSInspector::json_export` to escape all string values, sort output, and include a new `"entities"` array; `Fix(BUG-478)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/debug_test.rs` | Added `test_ecs_inspector_json_export_escapes_and_includes_entities`, validating JSON parseability under adversarial input and correct escaped round-tripping plus entity-array presence. |
