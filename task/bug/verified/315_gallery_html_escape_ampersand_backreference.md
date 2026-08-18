# BUG-315: `action/gallery`'s `_html_escape` leaves `<`/`>`/`"` unescaped and appends garbage entity text, because bash treats an unescaped `&` in a substitution's replacement as a match backreference

- **Severity:** Medium (no crash, no attacker-controlled input -- readme content is
  repo-local -- but a real, currently-manifesting content-correctness defect: the function
  whose entire purpose is making embedded text safe for HTML markup fails to escape 3 of its
  4 target characters, and the corrupted output is the public GitHub Pages gallery page)
- **state:** Verified
- **Affects:** every regeneration of `examples/index.html` via `action/gallery` (default
  write mode or `verify::1` diff-check mode) for any example whose `readme.md` title or
  extracted description paragraph contains a literal `<`, `>`, or `"` character
- **Component:** `action` (the `gallery` script)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18
- **Fix Task:** [366](../../verifying/366_gallery_html_escape_ampersand_backreference_fix_registration.md) (renumbered from 357→366; ID collision with a concurrently-running `bug_promote BUG-298` session, caught via live disk scan and resolved before the task reached its Tasks Index entry, 2026-08-18)

## Symptom

`_html_escape()` (`action/gallery:83-91` pre-fix, now `:83-100` with the fix's added comment
block) exists to HTML-entity-escape a string's `&`, `<`,
`>`, `"` characters before it is embedded into `index.html` markup (a `<h3>` title, an
`alt=`/`aria-label=` attribute value, or a `<p class="demo-desc">` body). It performs 4
sequential bash parameter substitutions of the shape
`_s="${_s//<pattern>/&<entity-name>;}"`.

Bash's `${parameter/pattern/replacement}` construct treats an *unescaped* `&` inside
`replacement` as a backreference to the text that matched `pattern` -- the same convention
`sed` uses for `&` in its own replacement text -- not as a literal ampersand. 3 of the 4
replacement strings here (`&lt;`, `&gt;`, `&quot;`) begin with a literal, unescaped `&` that
was never written as `\&`, so each of those 3 substitutions inserts the *original matched
character* immediately followed by the literal entity-name text, instead of replacing it
with the entity. The 4th substitution (`&` -> `&amp;`, line 86) looks correct only by
coincidence: its matched text IS `&`, so the backreference happens to reproduce exactly the
same output a literal replacement would have produced.

Concretely: `<` becomes `<lt;` (not `&lt;`), `>` becomes `>gt;` (not `&gt;`), and `"` becomes
`"quot;` (not `&quot;`) -- in every case the original, unsafe character is left completely
unescaped in the output, with garbage entity-name text appended immediately after it.

## Impact

**Who is affected:** anyone viewing the generated `examples/index.html` gallery page (the
public GitHub Pages site `action/build_site` assembles from it) for any example whose
`readme.md` title or description paragraph contains a literal `<`, `>`, or `"` character;
anyone relying on `action/gallery verify::1` as a drift-detection gate.

**What breaks:** the escaped text renders with visible, garbled markup fragments
(`<lt;`, `>gt;`, `"quot;...quot;"`) instead of the intended literal characters, and -- more
importantly -- the original `<`/`>`/`"` characters reach the page's HTML markup completely
unescaped, defeating the one job this function has. `action/gallery verify::1` is a diff
check against whatever is already committed: it only detects *changes* between a fresh
regeneration and the on-disk file, not correctness, so once a corrupted regeneration is
committed once, every subsequent `verify::1` run passes cleanly against that corrupted
baseline -- silently normalizing the corruption instead of catching it.

This is not hypothetical, and it is not only a future risk: the corruption was already
present in the **committed** `examples/index.html` before this fix. `grep -n
"quot;Hello Triangle" examples/index.html` (pre-fix) found 4 occurrences (2 demo cards, each
rendered twice into different tag groups) reading `This demo is the classic "quot;Hello
Triangle"quot; in WebGPU...` where the intended text was `&quot;Hello Triangle&quot;` --
confirming this defect had already corrupted live, generated, committed markup, not merely a
latent risk waiting for a trigger.

Separately, as of this filing, 3 examples' `readme.md` files (`examples/minwebgl/filter`,
`examples/minwebgl/minimize_wasm`, `examples/minwebgl/jewelry_site`) had description-paragraph
text containing `<!-- ... -->`-delimited comments and `"..."`-quoted words (from another
contributor's in-progress documentation edits, unrelated to this bug), so regenerating the
gallery before this fix would have baked further instances of this exact corruption into
`examples/index.html` for those 3 demo cards too.

**Entity Scope:** `None` -- source-level generator defect, not entity directory instances.

## How Discovered

While investigating a report that `action/gallery verify::1` was failing (stale generated
gallery pages), the live diff for `examples/index.html` showed 5 lines containing garbled
text patterns (`<lt;`, `-->gt;`, `"quot;...quot;"`) inside 3 examples' description
paragraphs. Comparing the same underlying description text in the `index.md` diff (which
showed clean, correctly-unescaped `<!-- ... -->` text) confirmed the corruption was specific
to the HTML-generation code path, not present in the source readmes. Isolating and directly
executing `_html_escape()`'s exact logic in a bare `bash` shell reproduced the corruption
outside the full script, which led to identifying bash's `&`-backreference behavior in
`${parameter/pattern/replacement}` as the root cause.

## Minimum Reproducible Example

**Verify Command**:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
bash action/tests/gallery_test.sh
```
**Expected** (fixed): `PASS: _html_escape produces correct HTML entities`, exit 0.

**Actual** (pre-fix, directly observed by extracting and running the unmodified function):
```bash
$ bash -c '
_html_escape()
{
  local _s="$1"
  _s="${_s//&/&amp;}"
  _s="${_s//</&lt;}"
  _s="${_s//>/&gt;}"
  _s="${_s//\"/&quot;}"
  printf "%s" "$_s"
}
_html_escape "test <!-- comment \"works\" --> end"
'
test <lt;!-- comment "quot;works"quot; -->gt; end
```
Expected output was `test &lt;!-- comment &quot;works&quot; --&gt; end`. Isolating each
substitution individually confirmed the corruption comes from the pattern-substitution
construct itself, not from interaction between the 4 lines:
```bash
$ bash -c 'x="a<b"; y="${x//</&lt;}"; echo "[$y]"'
[a<lt;b]
```
And confirms bash treats the replacement's `&` as a whole-match backreference (the same
convention `sed` uses), not a literal character:
```bash
$ bash -c 'x="foo<>bar"; y="${x//<>/[&]}"; echo "[$y]"'
[foo[<>]bar]
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `_html_escape`'s `<`/`>`/`"` substitutions produce corrupted, not escaped, output | ✅ Verified | Direct execution of the exact function body reproduces `<lt;`/`>gt;`/`"quot;...quot;"` | E1 |
| H2 | The cause is bash treating unescaped `&` in the replacement as a match backreference (sed-style), not a literal character | ✅ Root Cause | Isolated single-substitution test (`${x//</&lt;}` alone) reproduces the same corruption; escaping the `&` as `\&` produces correct output | E2, E3 |
| H3 | The `&`->`&amp;` substitution (line 86) looks correct only by coincidence, not because it is written differently | ✅ Verified | Same unescaped-`&` construct as the other 3 lines; happens to work because the matched text there equals the literal character being inserted | E1 |
| H4 | `index.md` generation is unaffected (uses raw, unescaped description text) | ✅ Verified | `index.md` diff shows clean, correctly-formed `<!-- ... -->` text for the same 3 examples where `index.html` shows corruption; confirmed by code comment at `action/gallery:120-122` and by `grep` showing `_html_escape`/`_desc_esc` used only in the HTML-emitting code path | E4 |
| H5 | No existing test exercises `_html_escape` or any other `action/*` script | ✅ Verified | No `action/tests/` directory or shell-test file existed anywhere in the repo before this fix | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | Terminal output (this report, MRE section) | Direct execution of `_html_escape`'s unmodified body on `test <!-- comment "works" --> end` produces `test <lt;!-- comment "quot;works"quot; -->gt; end` | H1, H3 |
| E2 | Terminal output (this report, MRE section) | Isolated single-line test `x="a<b"; echo "${x//</&lt;}"` alone (no interaction with the other 3 substitutions) already prints `a<lt;b` | H2 |
| E3 | Terminal output (this report, MRE section) | `${x//<>/[&]}` on `x="foo<>bar"` prints `foo[<>]bar` -- confirms `&` in the replacement is substituted with the *matched text*; escaping to `\&` (`${_s//</\&lt;}`) produces the correct `&lt;`, verified directly against the fixed `_html_escape()` via `action/tests/gallery_test.sh` | H2 |
| E4 | `action/gallery:120-122` (comment) + `action/gallery:176-179` | Comment states index.md "uses the raw, unstripped description instead"; `_html_escape`/`_desc_esc` appear only in the block building `index.html`'s `<p class="demo-desc">` | H4 |
| E5 | `find action -iname "*test*"` (pre-fix) | No output -- no test file or `tests/` directory existed under `action/` | H5 |

## Root Cause

```
_s="${_s//</&lt;}"
         |    |
         |    +-- replacement text: an UNESCAPED & here is not literal --
         |        bash reads it as "insert the text that matched <pattern>"
         |        (the sed `&` = whole-match convention), same as:
         |          sed 's/</\&lt;/'   vs.   sed 's/</\\&lt;/'
         +-- pattern: <

  matched text for this substitution is exactly "<", so the backreference
  inserts "<" where the intended literal "&" was meant to go, followed by
  the remaining literal characters "lt;" -- producing "<lt;" instead of "&lt;"
```
The same construct on line 86 (`_s="${_s//&/&amp;}"`) has matched text `&`, which happens to
be identical to the literal character the author intended to insert at that position -- so
its output is correct by coincidence, not because that line differs in any structural way
from lines 87-89.

## Why Not Caught

No existing `readme.md` title or description paragraph, across every example the gallery has
ever generated from, happened to contain a literal `<`, `>`, or `"` character before today --
ordinary demo-description prose rarely needs those characters, so this substitution bug had
no historical trigger. It first manifested when a different, concurrent contributor's
in-progress bug-fix documentation added `<!-- Fix(BUG-XXX): ... -->`-style HTML comments and
`"..."`-quoted words directly inside 3 examples' description-paragraph text (unrelated
readme edits still in progress at filing time; see Impact). Separately, `action/*` had zero
test coverage of any kind before this fix (confirmed via E5) -- `_html_escape` was never
exercised by an automated check that could have caught the corruption independent of readme
content.

## Fix Location

**`action/gallery:86-89`** (pre-fix), now **`:95-98`** (before/after):

```bash
# Before:
_s="${_s//&/&amp;}"
_s="${_s//</&lt;}"
_s="${_s//>/&gt;}"
_s="${_s//\"/&quot;}"

# After:
_s="${_s//&/\&amp;}"
_s="${_s//</\&lt;}"
_s="${_s//>/\&gt;}"
_s="${_s//\"/\&quot;}"
```
Source comment (`Fix(BUG-315)`/`Root cause`/`Pitfall`) added immediately above, inside
`_html_escape()`.

**`action/tests/gallery_test.sh`** (new): extracts `_html_escape()` verbatim from the real
`action/gallery` file at test-run time (via `sed`, so the test always exercises the actual
current implementation rather than a copy that could drift stale) and asserts it produces
correctly-escaped output for a string containing all 4 target characters together.

`examples/index.html` and `examples/index.md` were regenerated (`action/gallery`, default
write mode) after the fix, which both corrects the previously-committed `"quot;Hello
Triangle"quot;` corruption (4 occurrences, see Impact) and resolves the ordinary content
drift `action/gallery verify::1` had been reporting (stat counts, several `scene_script`/
`tiles_tools` description updates, a broken `minwebgpu/hello_triangle_quickstart` link, an
`orrery/flexible` link/description update) -- confirmed by `action/gallery verify::1`
transitioning from exit 1 to exit 0 ("gallery is up to date") after regeneration.

## Prevention

Detection command for the general pattern (a bash `${var//pattern/replacement}` or
`${var/pattern/replacement}` construct whose replacement text contains `&`):
```bash
grep -rnE '\$\{[a-zA-Z_][a-zA-Z0-9_]*//?[^}]*/[^\\}][^}]*&[^}]*\}' --include="*.sh" --include="action/*" .
```
This is a starting point for human review, not a precise check, and it cannot by itself
distinguish a fixed occurrence from a broken one -- run against this fix's own final code it
still matches 3 of the 4 lines (`action/gallery:95-97`), since `\&amp;`/`\&lt;`/`\&gt;` still
contain the character `&`, just correctly escaped now. The 4th line (`98`, the `"`->`&quot;`
substitution) is missed entirely by this regex, pre-fix or post-fix alike -- the embedded
`\"` in that line's pattern confuses the character classes used above. Confirmed by direct
execution against the current file. Reviewing each `${var/.../...}` construct by eye for a
leading `\` before every `&` is still required; this command narrows the search, it does not
confirm correctness or find every candidate line.

**Pitfall:** in bash's `${parameter/pattern/replacement}`/`${parameter//pattern/replacement}`,
an unescaped `&` in `replacement` is a backreference to the matched text, exactly like
`sed`'s `&` -- it is easy to write this construct expecting shell-only literal-string
semantics (as ordinary variable assignment would give) and not realize the replacement text
needs its own `&` -> `\&` escaping, especially when (as happened here) one of several
substitutions in the same function coincidentally produces correct output despite sharing
the same unescaped-`&` mistake.

## Generalized Version

**Broken assumption:** the replacement text of a bash `${parameter/pattern/replacement}`
substitution is inserted literally, with no special characters of its own.

Fails whenever:
1. A bash `${var/pattern/replacement}` (or `//` global form) substitution's `replacement`
   text contains an unescaped `&`, AND
2. The matched `pattern` text differs from the character(s) immediately following that `&`
   in `replacement` (the one case where the bug is invisible is when the matched text and
   the literal character the author intended coincide, as on this file's own line 86)

**Detection invariant:**
```
for every `${var/pattern/replacement}` construct:
  any literal & in `replacement` must be written as `\&`,
  unless the author specifically intends bash's sed-style whole-match backreference
```
Single confirmed instance in this workspace (grep swept every shell-shebang file for the same
unescaped-`&`-in-replacement shape; only `action/gallery`'s own 4 lines matched). Not a
duplicate of any prior bug in this repo's `task/bug/` history (dedup search:
`grep -rli "gallery\|html_escape\|html escape\|index\.html\|_html_escape" task/bug/` found one
unrelated hit,
`task/bug/completed/109_file_load_resolves_relative_paths_against_origin_not_page_dir.md`,
which only mentions an unrelated `dist/index.html` path from a `trunk build --public-url`
defect).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found while investigating a stale-gallery-pages report; root-caused via direct isolated execution of `_html_escape()` outside the full script |
| 2026-08-18 | fix_applied | Escaped `&` -> `\&` in `_html_escape`'s 4 replacement strings; regenerated `examples/index.html`/`index.md` (`action/gallery` write mode) |
| 2026-08-18 | verified | Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after correcting 3 imprecise citations in Symptom, Fix Location, and Prevention (D4) |
| 2026-08-18 | promoted_to_fix_task | Linked to [Task 366](../../verifying/366_gallery_html_escape_ampersand_backreference_fix_registration.md) via the `bug_promote` skill (PROC12) — formal task-system registration of this bug's already-applied, already-verified fix. Task 366 (renumbered from 357, ID collision with a concurrently-running `bug_promote BUG-298` session) reached its own Readiness Verification Gate PASS 8/8 and is blocked on `tsk .verify_pass`'s same-actor guard (identical to this bug's own filing/verifying actor), same standing pattern as this backlog's other same-actor-blocked tasks (e.g. task 254 for BUG-114). |

## Refs: src/

- `action/gallery` — escaped the `&` in each of `_html_escape`'s 4 replacement strings as `\&` so bash treats them as literal text instead of a match backreference
- `examples/index.html`, `examples/index.md` — regenerated (generated artifacts, not hand-edited) after the fix; corrects the previously-committed `"quot;Hello Triangle"quot;` corruption and the unrelated ordinary content drift `verify::1` had been reporting

## Refs: tests/

- `action/tests/gallery_test.sh` — new reproducer: extracts `_html_escape()` verbatim from `action/gallery` and asserts correct entity output for a string containing `<`, `>`, `"`, and `&` together

## Verification Record

**Tier 2 (Dual-Role Self-Check)** — 8-dimension check (Completeness, MRE Validity &
Reproducibility, Cross-Reference Integrity, Root Cause Quality, Execution Scope, Crate Scope
Unity, Crate Locality, Crate Single Responsibility), reused unchanged from the BUG-311..314
checks earlier this pass; D6-D8's "crate" framing is applied to `action/` as a component
boundary since this fix has no Cargo crate of its own.

*Single emoji per cell — see `governance/maav.rulebook.md § MAAV : Surface Rule` for the
🟢🔴🟡🟠 legend.*

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟡 | 🟢 | Symptom/Fix Location cited only the pre-fix `action/gallery:83-91`/`:86-89` line ranges without noting the fix's added comment block shifted them to `:83-100`/`:95-98`; Prevention claimed its detection grep "matches every one of the 4 lines" when it actually matches only 3 of 4 (the `"`->`&quot;` line's embedded `\"` breaks the regex) | Added explicit pre-fix/post-fix line ranges to Symptom and Fix Location; corrected Prevention's claim to state exactly which 3 lines match and why the 4th is missed |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 3 issues | 3 fixes |

**Confirming pass notes:** all 12 FI008 sections + 2 Refs present, matching BUG-314's exact
section structure (`grep -n "^##\|^- \*\*"` diff-checked); MRE verify command
(`action/tests/gallery_test.sh`) executed live both pre-fix (FAIL, exact output matches what
is cited) and post-fix (PASS, exit 0); both FI027 backreferences
(`action/gallery:85`, `action/tests/gallery_test.sh:2`) use the established bare
`task/bug/NNN_....md` path form; `git status --short` confirmed only the 6 intended paths
were touched (`action/gallery`, `action/readme.md`, `action/tests/`, `examples/index.html`,
`examples/index.md`, `task/bug/draft/`) against a working tree that also has ~19 unrelated
paths modified/untracked by the concurrent actor, none of which this fix touches; `action/gallery
verify::1` reconfirmed exit 0 ("gallery is up to date") after the fix and regeneration; the
Prevention detection command was actually executed against the final fixed code, not merely
asserted to work.

**Adversarial pass notes:** re-traced every file:line citation against a fresh read of the
current (post-fix) source rather than trusting the draft's original citations — caught that
`_html_escape`'s line range shifted from `83-91`/`86-89` (pre-fix) to `83-100`/`95-98`
(post-fix, after the added comment block) and that Symptom/Fix Location only cited the
pre-fix numbers; re-executed the Prevention section's own detection grep against the fixed
file and found it matches only 3 of the 4 target lines (95-97, not 98) -- the draft's "matches
every one of the 4 lines" claim was wrong, corrected to name the exact matched lines and why
the 4th is missed. Attempted to falsify H3 ("line 86 works by coincidence") by checking
whether the `&`->`&amp;` substitution's matched text could ever differ from `&` itself -- it
cannot, since the pattern IS `&`; claim holds. Attempted to falsify the "single confirmed
instance in this workspace" claim in Generalized Version by re-running the sibling-occurrence
sweep after the fix was applied (not just before) to confirm no second broken site was missed
by the first pass -- still only the 4 already-fixed lines in `action/gallery` match. No
independence: this is a single authoring entity's own two-pass check, not a dispatched second
opinion.
