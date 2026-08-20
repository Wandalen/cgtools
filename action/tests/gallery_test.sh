#!/usr/bin/env bash
# BUG-315 task/bug/315_gallery_html_escape_ampersand_backreference.md -- reproducer for
# _html_escape() in action/gallery leaving <, >, " unescaped (and appending garbage entity
# text) because bash treats an unescaped & in a ${var/pattern/replacement} replacement as a
# backreference to the matched text, not a literal ampersand.
# test_kind: bug_reproducer(BUG-315)
#
# BUG-375 task/bug/375_gallery_index_md_links_not_rebased.md -- reproducer for index.md rows
# copying description text verbatim, so relative links written for an example readme's own
# directory (e.g. ../hello_triangle/readme.md) dangle when emitted into examples/index.md
# one level up. Covers _rebase_links()/_normalize_path(), which rebase each inline-link
# target against the example's examples/-relative directory.
# test_kind: bug_reproducer(BUG-375)
set -euo pipefail

_repo_root="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../.." && pwd )"

# Extract _html_escape() verbatim from the real script under test, so this test always
# exercises the actual current implementation rather than a copy that could drift stale.
_fn_src="$( sed -n '/^_html_escape()$/,/^}$/p' "$_repo_root/action/gallery" )"
if [[ -z "$_fn_src" ]]
then
  printf 'FAIL: could not extract _html_escape() from action/gallery -- function renamed or moved?\n' >&2
  exit 1
fi
eval "$_fn_src"

_result="$( _html_escape 'a < b > c "d" & e' )"
_expected='a &lt; b &gt; c &quot;d&quot; &amp; e'

if [[ "$_result" != "$_expected" ]]
then
  printf 'FAIL: _html_escape corrupted HTML entities\n  got:      %s\n  expected: %s\n' \
    "$_result" "$_expected" >&2
  exit 1
fi

printf 'PASS: _html_escape produces correct HTML entities\n'

# ---- BUG-375: index.md relative-link rebasing ----------------------------------------
# Root Cause: the md-row builder copied each readme description verbatim into
#   examples/index.md, one directory level above the readme the links were written for,
#   so every relative link target dangled (3 committed instances found by the
#   dangling-link lint: ../hello_triangle/readme.md, ../../../docs/pattern/... twice).
# Why Not Caught: no test covered index.md generation at all (BUG-315 added coverage for
#   the HTML path only), and the dangling-link lint that finally caught it was only
#   pointed at examples/ after the gallery had already been committed.
# Fix Applied: _rebase_links() rewrites every inline-link target against the example's
#   examples/-relative directory (via lexical _normalize_path), leaving absolute URLs,
#   mailto:, #anchors, and root-relative paths untouched.
# Prevention: this test extracts both functions verbatim from action/gallery, so renaming
#   or dropping the rebase step fails the extraction check loudly; the dangling-link lint
#   over examples/ remains the corpus-level guard.
# Pitfall: markdown copied between files at different tree depths silently re-scopes every
#   relative link -- text that is correct in situ is wrong anywhere else; any generator
#   that relocates prose must rebase link targets, not just escape delimiters.

for _fn in _normalize_path _rebase_links
do
  _fn_src="$( sed -n "/^${_fn}()\$/,/^}\$/p" "$_repo_root/action/gallery" )"
  if [[ -z "$_fn_src" ]]
  then
    printf 'FAIL: could not extract %s() from action/gallery -- function renamed or moved?\n' "$_fn" >&2
    exit 1
  fi
  eval "$_fn_src"
done

_check_rebase()
{
  local _label="$1" _text="$2" _base="$3" _expected="$4" _got
  _got="$( _rebase_links "$_text" "$_base" )"
  if [[ "$_got" != "$_expected" ]]
  then
    printf 'FAIL: _rebase_links %s\n  got:      %s\n  expected: %s\n' "$_label" "$_got" "$_expected" >&2
    exit 1
  fi
}

# The two committed dangling-link shapes this bug manifested as:
_check_rebase 'sibling-example link' \
  'See [Hello Triangle](../hello_triangle/readme.md) first.' \
  'minwebgpu/hello_triangle_quickstart' \
  'See [Hello Triangle](./minwebgpu/hello_triangle/readme.md) first.'
_check_rebase 'above-examples docs link' \
  'Per [script-as-glue](../../../docs/pattern/005_script_as_glue.md).' \
  'scene_script/f32x2_vector_arithmetic' \
  'Per [script-as-glue](../docs/pattern/005_script_as_glue.md).'
# Same-directory link gains the example prefix:
_check_rebase 'same-dir link' \
  '[demo](./sub/x.md) and [again](other.md)' \
  'cat/ex' \
  '[demo](./cat/ex/sub/x.md) and [again](./cat/ex/other.md)'
# Pass-through targets stay byte-identical:
_check_rebase 'absolute/anchor/root pass-through' \
  '[a](https://example.com/x) [b](#anchor) [c](/root/p.md) [d](mailto:x@y.z)' \
  'cat/ex' \
  '[a](https://example.com/x) [b](#anchor) [c](/root/p.md) [d](mailto:x@y.z)'
# No links at all -- text unchanged:
_check_rebase 'linkless text' 'plain (parens) text' 'cat/ex' 'plain (parens) text'

printf 'PASS: _rebase_links rebases relative targets and preserves absolute ones\n'
