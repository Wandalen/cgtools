# BUG-468: `wfc`'s `pattern_set` panics via a 9-`unwrap()` chain on any malformed or non-CSV TMX upload

- **Severity:** Medium (no memory-safety impact, but a user-facing file-upload path panics the
  entire wasm module on ordinary bad input -- not a crafted edge case: Tiled's own default export
  encoding is not CSV, so exporting a map without manually switching Layer Format is enough to
  trigger it)
- **state:** Completed
- **Affects:** `examples/minwebgl/wfc`
- **Component:** `examples/minwebgl/wfc/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-469 (same function, `pattern_set` -- separate defect: a correctness/data
  bug in the GID-to-pixel mapping this fix also touches, not a panic-safety issue).

## Symptom

```rust
// pre-fix -- wfc/src/main.rs, pattern_set (condensed)
fn pattern_set( tmx_content : &str, app_state : &mut ApplicationState )
{
  let elem : xml::Element = tmx_content.parse().unwrap();                       // 1
  let layer = elem.get_child( "layer", None ).unwrap();                        // 2
  let width : u32 = layer.attributes.get( &( "width".into(), None ) ).unwrap().parse().unwrap();   // 3, 4
  let height : u32 = layer.attributes.get( &( "height".into(), None ) ).unwrap().parse().unwrap(); // 5, 6
  let data = layer.get_children( "data", None ).next().unwrap();               // 7
  // .. per-tile GID parsing, each `.parse::<u32>().unwrap()`                   // 8
  let pattern_buf = ImageBuffer::from_vec( width, height, pattern_raw ).unwrap(); // 9
  // ...
}
```

9 separate `.unwrap()` calls chained across XML parsing, child-element lookup, attribute lookup,
attribute parsing, and per-tile GID parsing -- any one of them panics the entire wasm module on
malformed input, with no graceful degradation and no user-visible error message.

## Impact

**Who is affected:** Any user of the file-upload control (`#file-input` in `index.html`) who
selects a TMX file that isn't both well-formed XML *and* CSV-encoded with the specific structure
this function assumes.

**What breaks:** The wasm module panics (via `unwrap()` on `None`/`Err`), which in a
`wasm-bindgen`-driven browser app aborts the entire running instance -- not just the upload
handler. The whole demo becomes unresponsive until the page is reloaded. Concretely reachable via:
Tiled's own default export encoding is Base64 (optionally zlib/gzip-compressed), not CSV -- so
exporting a map without manually setting Layer Format to "CSV" first (call site 7, no matching
`<data encoding="csv">` child found by the un-filtered lookup) already triggers this. A
flipped-tile GID (Tiled sets high bits far beyond any `u8` range for horizontal/vertical/diagonal
flip flags) triggers call site 8/9 via `ImageBuffer::from_vec`'s implicit expectation of the
already-validated size, or an earlier `u8::try_from` in the fixed version.

**Magnitude:** 9 unwrap call sites in 1 function, all on the same public-facing upload path.

**Entity Scope:** None -- a code-level defect confined to this crate's own TMX parsing.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, auditing user-facing
file-upload handlers for `unwrap()`/`expect()` chains reachable from untrusted input -- contrasted
against this same file's sibling `default_pattern_load`, which already handles its own (trusted,
bundled) TMX asset without panicking on any of the same operations.

## Minimum Reproducible Example

```rust
// examples/minwebgl/wfc/src/main.rs, inline #[cfg(test)] mod tests (this crate is a
// fn main()-only WebGL demo binary with no [lib] target -- see the local rulebook's Test
// Placement rule).
let tmx = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" orientation="orthogonal" width="2" height="1">
 <layer id="1" name="Layer 1" width="2" height="1">
  <data encoding="base64">AAAAAA==</data>
 </layer>
</map>"#;
let mut state = ApplicationState { map : None, pattern_image : None };
pattern_set( tmx, &mut state ); // pre-fix: panics inside the un-filtered data-child unwrap
```

**Verify Command** (<=3 lines, standalone):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p wfc_example -- tests::pattern_set_rejects_non_csv_encoding_without_panicking tests::pattern_set_rejects_malformed_xml_without_panicking tests::pattern_set_rejects_out_of_range_gid_without_panicking
```

## Root Cause

`pattern_set` was written assuming every input is a well-formed, CSV-encoded TMX file matching the
bundled default asset's exact shape, with no validation layer between untrusted user file-upload
input and the parsing chain. Its sibling `default_pattern_load` (for the crate's own trusted,
bundled asset) has the same shape of parsing but never needs to defend against malformed input, so
`pattern_set` copied the unwrap-chained style without adding the graceful-failure handling its
different (untrusted) input source requires.

## Why Not Caught

No test file existed for this crate before this fix -- it is a `fn main()`-only WebGL demo binary
with no lib target, and the file-upload path only runs via real DOM `<input type="file">` events
inside an actual browser, so it was never exercised against malformed input outside of interactive
manual testing (which naturally tends to use well-formed test files).

## Fix Location

`examples/minwebgl/wfc/src/main.rs`: rewrote `pattern_set` to use `let Ok(..)/Some(..) else { ..;
return; }` at every step that previously unwrapped, logging a specific, actionable message via
`gl::warn!` for each distinct failure mode (invalid XML, missing `<layer>`, missing/invalid
`width`/`height`, no CSV-encoded `<data>` child, non-numeric tile value, out-of-range GID, or a
tile count that doesn't match `width * height`) and leaving `app_state.pattern_image` untouched
instead of panicking. This same fix also resolves BUG-469's GID 0/1 pixel-value collision (see that
report) as part of the same rewritten function.

## Prevention

Added 3 new tests covering the panic-safety fix specifically: `pattern_set_rejects_non_csv_encoding_without_panicking`,
`pattern_set_rejects_malformed_xml_without_panicking`, `pattern_set_rejects_out_of_range_gid_without_panicking`
-- each asserts the function returns normally (no panic) and leaves `app_state.pattern_image` as
`None` for its specific malformed-input case. (A 4th test, covering BUG-469's separate GID
collision fix, is documented in that report.)

## Pitfall

Copying a parsing function's structure for a *different* input source (untrusted user upload vs.
trusted bundled asset) without re-auditing whether panic-on-error is still acceptable is an easy
trap -- the two functions look identical in shape, but only one of them can assume its input is
well-formed. Any user-reachable file-upload/paste/network-response handler should default to
graceful `Option`/`Result` propagation with a user-visible message, never a bare `unwrap()` chain,
regardless of how trusted a sibling function's identical-looking parsing code is.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery, fix, and tests landed together in one session. |
| 2026-08-20 | fixed | Rewrote `pattern_set` to use `let..else` graceful-failure returns at every step previously unwrapped, with a specific `gl::warn!` message per failure mode. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `pattern_set`'s function body to the pre-fix 9-unwrap version (tests left in place); `cargo test -p wfc_example --bin wfc_example` confirmed all 3 panic-safety tests failed (panicked, as expected for the reverted code). Restored the fix; all tests pass. Final combined pass: `cargo test -p wfc_example && cargo clippy -p wfc_example --all-targets --all-features --no-deps -- -D warnings && cargo check -p wfc_example --target wasm32-unknown-unknown`, all clean (exit 0). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-468)` context documented via this report; the rewritten function's per-branch `gl::warn!` messages serve as the in-code failure-mode documentation (no single fix-site source comment applies -- the fix restructures the whole function rather than patching one line). | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `pattern_set`'s body; no other function touched (`default_pattern_load` deliberately left unchanged -- its trusted bundled-asset input was never the concern here). Confirmed via re-reading the diff before verification. | — |

**Reproduced:** YES -- temporarily reverting `pattern_set` to its pre-fix unwrap-chained body caused
all 3 panic-safety regression tests to panic (fail); restoring the fix passes all 3 without a panic.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/wfc/src/main.rs` | `pattern_set` rewritten from a 9-`unwrap()` chain to graceful `let..else`-based error handling with per-failure-mode `gl::warn!` messages. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/wfc/src/main.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `pattern_set_rejects_non_csv_encoding_without_panicking`, `pattern_set_rejects_malformed_xml_without_panicking`, `pattern_set_rejects_out_of_range_gid_without_panicking`. |
