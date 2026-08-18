#!/usr/bin/env bash
# BUG-315 task/bug/315_gallery_html_escape_ampersand_backreference.md -- reproducer for
# _html_escape() in action/gallery leaving <, >, " unescaped (and appending garbage entity
# text) because bash treats an unescaped & in a ${var/pattern/replacement} replacement as a
# backreference to the matched text, not a literal ampersand.
# test_kind: bug_reproducer(BUG-315)
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
