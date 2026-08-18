# BUG-284: `shader_chunks_query`/`sch tree`'s no-argument forest view (aligned format) renders spurious blank lines and marks every root as the "last sibling"

- **Severity:** Low (CLI output-formatting defect only -- no data loss, no crash, no wrong query
  results; affects only the human-readability of the `tree` command's no-argument forest view in
  its default `aligned` rendering shape)
- **state:** Completed
- **Affects:** `shader_chunks_query_core::chunk_tree`'s `TreeFormat::Aligned` branch -- reached by
  `shader_chunks_query tree` / `sch tree` (no `name` argument) and by direct calls to
  `chunk_tree(None, _, TreeFormat::Aligned)`. Does not affect `tree <name>` (single-chunk trees
  have only one root, so the defect never triggers) or the `dot`/`mermaid` shapes (which already
  combine every root into one shared render).
- **Component:** `module/shader/shader_chunks_query_core` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`chunk_tree(None, false, TreeFormat::Aligned)` -- the engine behind `shader_chunks_query tree`
with no `name` (the "show every root chunk" forest view) -- rendered every top-level root chunk
with the tree-art "last sibling" connector (`└── `), even when more roots followed it, and
inserted a spurious blank line between every pair of consecutive root blocks. A user reading the
output would reasonably interpret `└── ` as "no more siblings," which is wrong for every root
except the true last one, and the output was roughly twice as long as necessary (half the lines
were blank padding with no content).

## Impact

**Who is affected:** any user or script running `shader_chunks_query tree` / `sch tree` (or the
aggregator's equivalent) with no chunk name -- the documented "forest of every root chunk" mode,
which is one of `tree`'s own two headline examples (`{binary} tree fbm3` / `{binary} tree`).
`tree <name>` (single chunk) and `tree ... shape::dot`/`shape::mermaid` are unaffected.

**What breaks:** No incorrect data -- every chunk name, tag, and nesting relationship was still
present and correctly indented at the right depth. The defect is purely in the tree-art connector
glyph chosen for top-level rows and in extraneous blank-line padding, both of which make the
forest harder to scan and misrepresent sibling structure at a glance.

**Entity Scope:** `None` -- CLI output-rendering defect, not entity directory instances.

## How Discovered

While reviewing `shader_chunks_query_core::chunk_tree`'s `TreeFormat::Aligned` branch (part of
this session's assigned tree-formatting review scope), noted it builds a *separate*
`invisible_parent` per root and calls `formatter.format_aligned` once per root, joining the
results with `"\n"` -- unlike the `TreeFormat::Dot`/`TreeFormat::Mermaid` branch's
`collect_edges`, which combines every root into one shared `edges` list for a single render. Built
the `shader_chunks_query` binary and ran `./target/debug/shader_chunks_query tree` directly to
confirm empirically: every root line was prefixed `└── ` and a blank line separated every pair of
roots, while the single-root `tree fbm3` path and the `dot`/`mermaid` forest paths were both clean.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo build -p shader_chunks_query --bins
./target/debug/shader_chunks_query tree | head -4
```
**Expected** (fixed): the first (non-last) root uses the "more siblings follow" connector, and no
blank line separates it from the next root:
```
├── fullscreen_triangle          category:vertex
├── hash33                       category:hash
├── value_noise3                 category:noise
│   └── hash13                   category:hash
```
**Actual** (pre-fix, confirmed via temporary manual revert of just the fix's lines -- see
Verification):
```
└── fullscreen_triangle  category:vertex

└── hash33  category:hash

```
(every root gets `└── `, and a blank line follows every root block; full forest was 106 lines
pre-fix vs. 65 lines post-fix, for the same 50-chunk registry.)

## Root Cause

`chunk_tree`'s `TreeFormat::Aligned` branch (pre-fix):
```rust
let formatter = TreeFormatter::new();
Ok( roots.iter().map( | &chunk |
{
  let mut invisible_parent = TreeNode::new( String::new(), None );
  invisible_parent.children.push( dep_tree_node( chunk, &children_of ) );
  formatter.format_aligned( &invisible_parent )
}).collect::< Vec< _ > >().join( "\n" ) )
```
Two compounding defects from the same design choice (one `invisible_parent`, and one
`format_aligned` call, *per root* rather than shared across the whole forest):
1. **Wrong connector:** each real root was always the *sole* child of its own throwaway
   `invisible_parent`, so `format_aligned` -- which correctly computes `├── ` vs. `└── ` from a
   node's position among its **own** parent's children (confirmed correct for real multi-child
   siblings, e.g. `sdf_op_union`'s two direct dependencies render `├── d2_sdf_circle` /
   `└── d2_sdf_box` correctly) -- always saw exactly one child and rendered it as the last (and
   only) sibling, regardless of how many other roots followed in the outer `roots.iter()` loop.
2. **Spurious blank lines:** each per-root call to `format_aligned` returns a string already
   ending in its own trailing `"\n"`. Joining N such strings with an additional `"\n"` via
   `.join("\n")` doubles that newline at every boundary, producing a blank line after every root
   block except the last.

## Why Not Caught

Every existing forest-mode test (`tree_chunk_with_no_name_shows_forest_of_every_root_chunk`,
`tree_reverse_with_no_name_shows_forest_of_every_leaf_chunk`) only asserted that specific chunk
names appeared somewhere in the output (`output.contains(name)`), never on blank-line structure or
on which connector glyph prefixed a given line -- both true regardless of the defect, so they
passed either way. The `dot`/`mermaid` forest tests (`tree_dot_and_mermaid_with_no_name_...`)
exercise the *same* "no name" forest case but through the unaffected `collect_edges` code path,
so they gave no signal about the `aligned` path's own per-root-join logic. The single-chunk tests
(`tree_chunk_shows_fbm3_dependency_chain_in_order`, etc.) never invoke the multi-root join at all
(`roots.len() == 1`), so they could not have surfaced this either.

## Fix Applied (2026-08-17)

Changed the `TreeFormat::Aligned` branch to build **one shared** `invisible_parent`, push every
real root as one of its children, and call `formatter.format_aligned` **exactly once** over that
shared parent -- the same one-call-per-whole-forest shape the `dot`/`mermaid` branch already used
via `collect_edges`'s shared `edges` vec for the identical "no name" case:
```rust
let mut invisible_parent = TreeNode::new( String::new(), None );
for &chunk in &roots
{
  invisible_parent.children.push( dep_tree_node( chunk, &children_of ) );
}
Ok( formatter.format_aligned( &invisible_parent ) )
```
`data_fmt`'s own `format_aligned` then computes correct `├── `/`└── ` connectors across every real
root (verified: it already does this correctly for genuine same-parent siblings), and the
per-root `"\n"` join -- the source of the doubled blank lines -- is gone entirely. The
single-chunk path (`roots.len() == 1`) is untouched byte-for-byte, since a one-element loop
pushing into one shared parent is identical to the old one-element `.map()` in that case.

**`shader_chunks_query_core/tests/shader_chunks_query_core_test.rs`** (new test):
`tree_forest_aligned_format_has_no_blank_lines_and_uses_correct_sibling_connectors` asserts the
forest output contains no `"\n\n"` (no blank-line gaps between roots) and that the first root
(`fullscreen_triangle`, not the last of several) uses `├── `, not `└── `.

## Verification

`longrun`-detached, from repo root:
- **Pre-fix (RED):** copied `shader_chunks_query_core/src/lib.rs` to scratchpad before editing
  (never `git stash`, per this session's standing concurrent-fork stash-collision guidance);
  confirmed the file carried no other fork's unrelated changes via `git diff --stat` (clean) before
  editing. With the new test added but the fix not yet applied, `cargo test -p
  shader_chunks_query_core --test shader_chunks_query_core_test
  tree_forest_aligned_format_has_no_blank_lines_and_uses_correct_sibling_connectors`: **FAILED**,
  output showed every root prefixed `└── ` and blank-line-separated, exactly as diagnosed.
- **Post-fix (GREEN):** same test command: **1 passed**, 0 failed.
- Full crate test file: `cargo test -p shader_chunks_query_core --test
  shader_chunks_query_core_test`: **45 passed**, 0 failed (44 pre-existing + 1 new), 0 ignored.
- Full assigned-crate suite (`shader_chunks_query`, `shader_chunks_query_core`,
  `shader_chunks_validate`, `shader_chunks_validate_core`, `--all-features`, unit + integration +
  doc tests): **57 passed**, 0 failed, 0 ignored.
- `cargo clippy -p shader_chunks_query -p shader_chunks_query_core -p shader_chunks_validate -p
  shader_chunks_validate_core --all-targets --all-features -- -D warnings`: clean, exit 0, zero
  warnings.

## Generalized Version

**Broken assumption:** when a formatter itself is responsible for computing structural
relationships between siblings (here: `├── ` vs. `└── ` tree-art connectors, derived from a
node's position among its own parent's children), calling that formatter once per top-level item
and stitching the resulting strings together defeats the very sibling-awareness the formatter was
chosen for -- each call only ever sees a "family of one," so it can never render anything but "last
child." Whenever several logically-equal top-level items need one combined structural render,
give the formatter all of them in a single call under one shared parent/root, exactly as this same
function's own `dot`/`mermaid` branch already did for the identical case one branch over.
Separately, when stitching multiple formatter outputs together with a string join, always check
whether each individual output already carries its own trailing copy of the separator -- joining
already-terminated strings with that same separator doubles it into a blank line.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's review of `shader_chunks_query`/`shader_chunks_query_core`/`shader_chunks_validate`/`shader_chunks_validate_core` (4 assigned crates, 9 files read in full), prompted by this session's own domain hint to look hard at tree-formatting bugs. Root cause: `chunk_tree`'s `TreeFormat::Aligned` branch called `format_aligned` once per forest root (each wrapped as the sole child of its own throwaway invisible parent) and joined the per-root strings with `"\n"`, instead of giving every root to one shared invisible parent in a single call the way the `dot`/`mermaid` branch already did via `collect_edges`. This made every root render with the "last sibling" connector regardless of position, and doubled each per-root call's own trailing newline into a blank line between roots. Fixed by restructuring the branch to build one shared parent and call `format_aligned` exactly once. Verified via 1 new test (confirmed fail pre-fix / pass post-fix via a scratchpad-backed, non-git-stash manual revert of just the new lines, since concurrent forks sharing this repo have corrupted each other's git stash via interleaved push/pop earlier this same session) plus the full 4-crate `--all-features` suite (57/57) and clean clippy. A related, more severe latent issue was also noted but deliberately NOT filed as a separate bug: `dep_tree_node` (the same file's Aligned-mode recursive tree-node builder) has no cycle/ancestor guard, unlike its sibling `collect_edges` (which tracks an `expanded` set), so a hypothetical future cyclic `depends_on` in the bundled registry would stack-overflow `tree <name>` in `aligned` mode instead of erroring gracefully -- but the real bundled registry is confirmed acyclic by `shader_chunks_validate_core`'s own `check_dependency_cycle` (its `validate_registry_reports_nothing_for_the_current_bundled_registry` test passes), `chunk_tree` is hardwired to that real static registry with no way to inject a fixture cycle through the public API, `dep_tree_node` is a private function unreachable from `tests/`, and an actual stack overflow cannot be captured as a clean pass/fail `#[test]` in stable Rust -- so this session's mandatory regression-test-first workflow could not be honestly satisfied for it, and it was left as a documented observation rather than forced through as a bug filing. |
| 2026-08-17 | RENUMBERED 283→284 | Filed as BUG-283 first, but an immediate post-write collision re-check (mandatory before touching the shared readmes) found a second, unrelated `283_*.md` had landed in `task/bug/completed/` from a concurrent sibling fork (`shader_chunks_cli_core`/`shader_chunks_compose` scope) roughly 7 seconds later -- a genuine TOCTOU race between two live actors, neither side wrong. Since that sibling fork's background task was still actively running at detection time, this bug (the side with zero external cross-references yet, per this repo's own documented `highest_id` collision-resolution convention) was renumbered to the next verified-free ID, 284, rather than risk a concurrent write against the other fork's in-flight file. All internal references (`Fix(BUG-283)` source comment, `bug_reproducer(BUG-283)` test doc-comment tag, this file's own name and title) were updated to 284; no other section of this report referenced the old number in body text. |
