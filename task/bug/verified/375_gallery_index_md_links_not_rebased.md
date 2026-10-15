# BUG-375: `action/gallery` copies readme descriptions into `examples/index.md` verbatim, so relative links written for an example's own directory dangle one level up

- **Severity:** Low (no code malfunction; but the committed, generated `examples/index.md` — a
  public docs surface — carried 3 dangling link occurrences, and the defect re-breaks the file
  on every regeneration for any readme description that uses a relative link)
- **state:** Verified
- **Affects:** every regeneration of `examples/index.md` via `action/gallery` (default write
  mode or `verify::1` baseline), for any example whose extracted description paragraph
  contains a relative markdown link
- **Component:** `action` (the `gallery` script, index.md emission path)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_core/games_prototyping/default/space_sandbox/action
- **Fix Task:** [390](../../verifying/390_register_action_gallery_indexmd_link_rebasing_fix_closes_bug375.md)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_core/games_prototyping/default/space_sandbox/action (self)
- **verification_date:** 2026-08-18

## Symptom

`action/gallery` builds each `examples/index.md` row from the example readme's extracted
description paragraph. Pre-fix, the md-row builder (`action/gallery:229` pre-fix,
`_md_desc="${_desc//|/\\|}"`) performed only pipe-escaping on that text — every markdown
link inside it was copied byte-for-byte.

A relative link target is resolved against the file it appears in. The description was
written for `examples/<category>/<example>/readme.md`, but `index.md` lives at `examples/` —
two directory levels higher — so every relative target in a copied description resolves
against the wrong base and dangles. The committed `examples/index.md` carried 3 such
occurrences:

- `../hello_triangle/readme.md` ×2 (the `minwebgpu/hello_triangle_quickstart` row, rendered
  in two tag-group sections) — correct from the readme's own directory, nonexistent from
  `examples/` (would resolve to the repo root's nonexistent `hello_triangle/`).
- `../../../docs/pattern/005_script_as_glue.md` ×1 (the `scene_script/f32x2_vector_arithmetic`
  row) — correct from the readme's directory, escapes above the repo root from `examples/`.

## Impact

**Who is affected:** anyone reading `examples/index.md` (rendered on GitHub as the examples
catalog) and clicking a description's link; any link-integrity tooling pointed at the repo.

**What breaks:** the 3 committed link occurrences 404. Worse, the defect is structural, not
one-off: any future readme description using a relative link re-introduces a dangling link
into `index.md` on the next regeneration, and `action/gallery verify::1` — a drift check
against the committed baseline — then *defends* the broken output as up-to-date.

The HTML side (`examples/index.html`) is unaffected: `_strip_markdown` removes link syntax
entirely from card descriptions, so no link target ever reaches the HTML output.

**Entity Scope:** `None` — source-level generator defect, not entity directory instances.

## How Discovered

Running the dangling-link lint (`cgtools_lint`, M01 `DanglingLinkCheck`) over `examples/`
during a corpus-wide lint triage reported exactly 3 dangling targets, all in
`examples/index.md`. Tracing them to their source readmes showed each link was *correct in
the readme it was written in* — only the generated copy dangled — which localized the defect
to the generator's md-row emission rather than to any readme.

## Minimum Reproducible Example

**Verify Command**:
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
bash action/tests/gallery_test.sh
```
**Expected** (fixed): both `PASS:` lines, exit 0 — including
`PASS: _rebase_links rebases relative targets and preserves absolute ones`.

**Actual** (pre-fix): the test's extraction step fails (`_rebase_links()` did not exist —
no rebasing code of any kind was present), and the corpus-level evidence was directly
observable in the committed file:

```bash
$ grep -c '](\.\./hello_triangle/readme.md)' examples/index.md   # pre-fix
2
$ grep -c '](\.\./\.\./\.\./docs/pattern/005_script_as_glue.md)' examples/index.md   # pre-fix
1
```

Post-fix regeneration rewrites these to `./minwebgpu/hello_triangle/readme.md` and
`../docs/pattern/005_script_as_glue.md` (both verified to exist from `examples/`), and both
grep counts drop to 0.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The 3 dangling targets in `index.md` are byte-identical copies of links that are valid in their source readmes | ✅ Verified | Each target resolves correctly from its readme's own directory; only the `index.md` copy dangles | E1, E2 |
| H2 | The md-row builder performs no link processing at all — only pipe-escaping | ✅ Root Cause | Pre-fix `action/gallery:227` is a bare `${_desc//|/\\|}` substitution; no other code touches link targets on the md path | E3 |
| H3 | The HTML path is unaffected | ✅ Verified | `_strip_markdown` drops link syntax from HTML card descriptions; no `](` survives into `index.html` demo cards | E4 |
| H4 | `verify::1` cannot catch this class of defect | ✅ Verified | It diffs a fresh regeneration against the committed file — both sides carry the same dangling links, so it passes | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `git diff examples/index.md` (fix commit's regeneration) | The 3 removed pre-fix lines carry `../hello_triangle/readme.md` ×2 and `../../../docs/pattern/005_script_as_glue.md` ×1; the replacement lines carry the rebased forms | H1 |
| E2 | `ls examples/minwebgpu/hello_triangle/readme.md docs/pattern/005_script_as_glue.md` | Both intended targets exist — the links were correct in situ, wrong only after relocation | H1 |
| E3 | `action/gallery:229` (pre-fix, confirmed via `git show HEAD:action/gallery`) | `_md_desc="${_desc//|/\\|}"` — verbatim copy modulo pipe-escaping | H2 |
| E4 | `grep -c '](' examples/index.html` demo-card descriptions | No markdown link syntax reaches HTML output (`_strip_markdown` removes it) | H3 |
| E5 | `action/gallery verify::1` pre-fix | Exit 0 against the committed baseline that itself contained the dangling links | H4 |

## Root Cause

```
readme.md (examples/minwebgpu/hello_triangle_quickstart/)
  "Same triangle as [Hello Triangle](../hello_triangle/readme.md)"
                                     ^^^ correct HERE: sibling directory

        | copied verbatim by the md-row builder (pipe-escape only)
        v

index.md (examples/)   -- two levels higher in the tree
  "... [Hello Triangle](../hello_triangle/readme.md)"
                        ^^^ now resolves to <repo>/hello_triangle/ -- nonexistent
```

Markdown text is not location-independent: every relative link is an address relative to
the file that contains it. A generator that relocates prose between tree depths must rebase
each link target against the destination, exactly as it must escape delimiters for the
destination syntax. The builder did the escaping and skipped the rebasing.

## Why Not Caught

No test covered `index.md` generation at all — BUG-315's test (the only `action/` test)
covers the HTML-escaping path. `verify::1` structurally cannot catch it (E5/H4: both diff
sides share the defect). The dangling-link lint that finally surfaced it had not previously
been pointed at `examples/` — it entered this corpus's lint set only when `cgtools_lint`
was assembled, and the first full corpus run is what flagged the 3 occurrences.

## Fix Location

**`action/gallery:158-215`** (post-fix): two new helpers — `_normalize_path()` (lexical
`.`/`..`/empty-segment resolution, preserving leading `..` runs that climb above the base)
and `_rebase_links()` (rewrites every inline-link target in a description against the
example's `examples/`-relative directory; absolute `http(s)://`, `mailto:`, `#anchor`, and
root-relative `/...` targets pass through untouched; processes right-to-left via anchored
greedy match so each link is rewritten exactly once).

**`action/gallery:288`** (post-fix; pre-fix `:229`): the md-row builder now runs
`_md_desc="$( _rebase_links "$_desc" "$_relpath_short" )"` before pipe-escaping, where
`_relpath_short` is the example's `examples/`-relative path (e.g.
`minwebgpu/hello_triangle_quickstart`). A rebased relative target gains a `./` prefix
(`./minwebgpu/hello_triangle/readme.md`); a target climbing above `examples/` keeps its
leading `..` (`../docs/pattern/005_script_as_glue.md`).

**`action/gallery:3-11`**: header comment extended to document the rebasing contract and
why the HTML path needs none.

**`action/tests/gallery_test.sh`**: second test block (this bug's reproducer) extracts
`_normalize_path()`/`_rebase_links()` verbatim from the live script and asserts: the two
committed dangling-link shapes rebase to their correct forms, same-directory links gain the
example prefix, absolute/anchor/root/mailto targets stay byte-identical, and linkless text
(including bare parentheses) passes through unchanged.

`examples/index.md` was regenerated after the fix (write mode, then `verify::1` reconfirmed
exit 0 — idempotent), which is what actually removes the 3 committed occurrences.

## Prevention

Two standing guards, one per level:

- **Function level:** `action/tests/gallery_test.sh` extracts the rebase functions from the
  live script — deleting or renaming the rebase step fails the extraction check loudly, and
  regressing its behavior fails the assertions.
- **Corpus level:** the dangling-link lint (M01) over `examples/` re-checks every committed
  link target, including everything `index.md` emits, on every corpus lint run:
  ```bash
  cd /home/user1/pro/lib/yrd_gamedev/linter && ./target/debug/cgtools_lint .lint target::../cgtools/examples/
  ```

**Pitfall:** markdown copied between files at different tree depths silently re-scopes every
relative link — text that is correct in situ is wrong anywhere else. Any generator that
relocates prose must rebase link targets, not just escape delimiters; and a drift check
(`verify::1`-style diff against a committed baseline) can never catch a defect the baseline
itself already contains.

## Generalized Version

**Broken assumption:** description text extracted from a source file can be embedded into a
generated file at a different path without transformation.

Fails whenever:
1. A generator copies markdown (or any text with relative addressing) from file A into
   generated file B, AND
2. A and B sit at different directory depths (or different subtrees), AND
3. The copied text contains at least one relative link/include/path.

**Detection invariant:**
```
for every generator emitting relocated prose:
  every relative address in the emitted copy must be rebased
  source-dir -> destination-dir, with absolute/anchored addresses exempt
```
Single confirmed emission site in this repo (`action/gallery`'s md-row builder; the HTML
builder strips links instead — checked both). Not a duplicate of any prior bug
(dedup search: `grep -rli 'dangling\|rebase\|index\.md' task/bug/` — hits are the unrelated
BUG-315 HTML-escaping defect, SVG asset bugs, and animation-timing bugs; none concern link
relocation).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via the first full `cgtools_lint` corpus run: M01 reported 3 dangling targets, all in the generated `examples/index.md`, each valid in its source readme |
| 2026-08-18 | fix_applied | Added `_normalize_path`/`_rebase_links` to `action/gallery`, wired into the md-row builder; regenerated `examples/index.md`/`index.html`; `verify::1` exit 0 post-regeneration |
| 2026-08-18 | verified | Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS after resolving a pre-fix line-citation contradiction (hunk-header arithmetic said 227, draft said 229; direct `git show HEAD:action/gallery` read confirmed 229) |

## Refs: src/

- `action/gallery` — added `_normalize_path()`/`_rebase_links()` and rebased each md-row description's link targets against the example's `examples/`-relative directory before pipe-escaping; header comment documents the contract
- `examples/index.md` — regenerated (generated artifact, not hand-edited); the 3 dangling occurrences now carry rebased, existing targets

## Refs: tests/

- `action/tests/gallery_test.sh` — new second test block: extracts `_normalize_path()`/`_rebase_links()` verbatim from `action/gallery` and asserts the two committed dangling-link shapes rebase correctly, plus prefix/pass-through/linkless edge cases

## Verification Record

**Tier 2 (Dual-Role Self-Check)** — same 8-dimension check as BUG-315's record (Completeness,
MRE Validity & Reproducibility, Cross-Reference Integrity, Root Cause Quality, Execution
Scope, Crate Scope Unity, Crate Locality, Crate Single Responsibility); D6-D8's "crate"
framing applied to `action/` as a component boundary, as this fix has no Cargo crate.

*Single emoji per cell — see `governance/maav.rulebook.md § MAAV : Surface Rule` for the
🟢🔴🟡🟠 legend.*

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟡 | 🟢 | Adversarial re-derivation of the pre-fix builder line from the fix diff's hunk header (`@@ -226,7 +285,8 @@`) yielded 227, contradicting the draft's 229 — a contradiction between two read methods that required cross-verification rather than picking either | Resolved by a direct `git show HEAD:action/gallery` read: line 229 confirmed (leading context is 226-228; the arithmetic error was in the adversarial pass, not the draft); all citations kept at `:229` |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 issue | 1 fix |

**Confirming pass notes:** MRE verify command executed live post-fix (`action/tests/gallery_test.sh`,
both PASS lines, exit 0, via detached launch); both rebased targets confirmed to exist on
disk (`ls examples/minwebgpu/hello_triangle/readme.md docs/pattern/005_script_as_glue.md`);
regeneration idempotency confirmed (`verify::1` exit 0 after write-mode run); the 3 pre-fix
occurrences and their exact spellings confirmed from `git diff examples/index.md`'s removed
lines rather than from memory; post-fix function/wiring line numbers (158, 187, 288)
confirmed via `grep -n` against the current file.

**Adversarial pass notes:** attempted to falsify H3 (HTML path unaffected) by grepping the
regenerated `index.html`'s `demo-desc` content for `](` — zero matches
(`_strip_markdown` removes link syntax before `_html_escape`); attempted to find a second
emission site for un-rebased prose (the tag-group HTML cards and the stats line) — none
emits raw description markdown; re-derived the pre-fix builder line number from the diff
hunk header as a check on the draft's `229`, got 227, and resolved the contradiction with a
direct `git show HEAD:action/gallery` read (229 confirmed — the arithmetic, not the draft,
was wrong; recorded under D4 rather than silently trusting either method); checked
the rebase edge case where a target climbs exactly to the repo root (`../..` from a category
dir) — `_normalize_path` preserves the leading `..` run rather than swallowing it, covered
by the above-examples docs-link assertion in the test. No independence: this is a single
authoring entity's own two-pass check, not a dispatched second opinion.
