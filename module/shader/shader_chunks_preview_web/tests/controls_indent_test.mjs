// Regression test for `controls.js`'s pure, DOM-free `blockIndent()` logic.
// Run directly with Node's built-in test runner -- zero external
// dependencies, no mocking, no DOM shim needed (this file imports nothing
// from `controls.js` except the one pure string-manipulation function):
//
//   node --test tests/controls_indent_test.mjs
//
// The rest of `controls.js` (DOM wiring, `initEditor`, the debounced
// `input`/`keydown` listeners) is intentionally out of scope for this file
// -- see readme.md's "Disclosed gap" for why the WebGPU-dependent half of
// this crate has no automated coverage. `blockIndent`/`sectionsSplit` are
// pure string logic with zero WebGPU/DOM dependency, so that disclosure
// does not extend to them; this file closes that specific coverage gap.

// test_kind: bug_reproducer(BUG-XXX)
/// ## Root Cause
/// `blockIndent()` slices the selected block as
/// `value.slice(blockStart, blockEnd)` then `.split('\n')`s it into "lines"
/// to indent/outdent one at a time. When the selection's own end lands
/// exactly at the start of the following line (`atLineStart` true -- e.g. a
/// triple-click whole-line selection, or any selection whose end sits at a
/// line boundary), the sliced block itself ends with a trailing `\n`, and
/// `"...\n".split('\n')` always appends one extra empty-string element that
/// does not correspond to any real line -- it is just the empty span
/// between the last real `\n` and the end of the slice. The old code
/// processed that phantom element as if it were a real line anyway. On
/// indent this unconditionally prepends 2 spaces to it; because nothing in
/// the reconstructed string separates that phantom element's edit from
/// `value.slice(blockEnd)` (the untouched remainder, which starts with the
/// very next line's first real character), the 2 spaces landed on the
/// first line *after* the selection instead of doing nothing. On outdent
/// the same phantom-element handling happens to be an accidental no-op --
/// `''.match(/^ {1,2}/)` is always `null`, so `removedLen` is always `0`
/// for an empty line -- which is why only the indent direction is
/// user-visibly broken, even though both directions shared the same
/// underlying defect (treating a non-line as a line).
/// ## Why Not Caught
/// No test ever exercised `blockIndent()` with a selection ending exactly
/// at a line boundary -- this crate has zero automated test coverage
/// (disclosed in readme.md), and `blockIndent` itself was not even
/// `export`ed until this fix, so nothing outside `controls.js` could reach
/// it to test it in the first place.
/// ## Fix Applied
/// `blockIndent()` now strips the trailing `\n` from the sliced block
/// *before* splitting (only when `atLineStart`), splits only the real line
/// content, then re-appends the stripped `\n` to the rejoined result -- so
/// `split('\n')` never manufactures a phantom trailing "line" for the
/// indent/outdent `.map()` to corrupt. The non-`atLineStart` path is
/// untouched: `trailingNewline` is `''` and `body` is the raw slice
/// unchanged there, so `newBlock` is byte-identical to the pre-fix
/// computation for every selection that doesn't end at a line boundary.
/// ## Prevention
/// `blockIndent` is now `export`ed specifically so this regression test
/// (and any future one) can reach it directly -- the same visibility
/// discipline a Rust `tests/` integration test already applies (it can
/// only call a crate's `pub` items), just expressed via ES module exports
/// instead of `pub fn`.
/// ## Pitfall
/// Any code that slices a text buffer with `value.slice(a, b).split('\n')`
/// and processes every element as "a line" must account for a trailing
/// separator manufacturing a phantom empty final element --
/// `"x\n".split('\n')` is `["x", ""]`, not `["x"]`. The phantom element is
/// falsy-looking (an empty string) but still very much present and
/// iterated, so it silently slips through any check that only guards
/// against `undefined`/missing array elements.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { blockIndent } from '../controls.js';

test( 'blockIndent: single full-line selection ending at a line boundary indents only the selected line, not the line after it', () => {
  const value = 'AAAA\nBBBB\nCCCC\n';
  const start = 5; // start of "BBBB"
  const end = 10;  // start of "CCCC" -- selection is "BBBB\n" exactly (e.g. triple-click)
  const result = blockIndent( value, start, end, false );
  assert.equal
  (
    result.value,
    'AAAA\n  BBBB\nCCCC\n',
    'only the selected line should gain 2 spaces; the line after the selection must stay untouched'
  );
});

test( 'blockIndent: multi-line whole-line selection ending at a line boundary indents only the selected lines, not the line after them', () => {
  const value = 'AAAA\nBBBB\nCCCC\nDDDD\n';
  const start = 5;  // start of "BBBB"
  const end = 15;   // start of "DDDD" -- selection is "BBBB\nCCCC\n" exactly
  const result = blockIndent( value, start, end, false );
  assert.equal
  (
    result.value,
    'AAAA\n  BBBB\n  CCCC\nDDDD\n',
    'both selected lines should gain 2 spaces; the line after the selection must stay untouched'
  );
});

test( 'blockIndent: whole-line selection ending at a line boundary outdents only the selected lines, not the line after them', () => {
  const value = 'AAAA\n  BBBB\n  CCCC\nDDDD\n';
  const start = 5;  // start of "  BBBB"
  const end = 19;   // start of "DDDD" -- selection is "  BBBB\n  CCCC\n" exactly
  const result = blockIndent( value, start, end, true );
  assert.equal
  (
    result.value,
    'AAAA\nBBBB\nCCCC\nDDDD\n',
    'both selected lines should lose 2 spaces; the line after the selection must stay untouched'
  );
});

test( 'blockIndent: selection NOT ending at a line boundary is unaffected by this fix (no regression)', () => {
  const value = 'AAAA\nBBBB\nCCCC';
  const start = 5; // start of "BBBB"
  const end = 8;   // mid "BBBB" -- not at a line boundary
  const result = blockIndent( value, start, end, false );
  assert.equal
  (
    result.value,
    'AAAA\n  BBBB\nCCCC',
    'a selection not ending at a line boundary should still pull in the whole partially-selected line and indent it, unchanged from prior behavior'
  );
});
