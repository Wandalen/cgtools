# BUG-499: `primitives_data_to_gltf` performs no acyclic validation of `PrimitiveData::parent`, silently building a broken node graph on cyclic input

- **Severity:** Low (requires a caller to construct `PrimitiveData` with a self-referencing or
  cyclic `parent` index -- not reachable from any of this crate's own generators, which never set
  `parent` to anything but `None` -- but produces a genuine `Rc`/`RefCell` reference cycle and
  memory leak with zero error when it does occur)
- **state:** Completed
- **Affects:** Any caller of `primitives_data_to_gltf` passing a `&[PrimitiveData]` whose `parent`
  indices form a cycle (including direct self-reference).
- **Component:** `module/helper/primitive_generation` (`src/primitive_data.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None. Found in the same sweep as BUG-500 (same crate) but a different
  mechanism and a different source file -- filed separately.

## Symptom

```rust
// pre-fix -- src/primitive_data.rs, primitives_data_to_gltf
for ( i, node ) in nodes.iter().enumerate()
{
  let primitive = &primitives_data[ i ];
  if let Some( parent_index ) = primitive.parent
  {
    if let Some( parent_node ) = nodes.get( parent_index )
    {
      parent_node.borrow_mut().child_add( node.clone() );
      node.borrow_mut().parent_set( Some( parent_node.clone() ) ); // no cycle check
    }
    else { scenes[ 0 ].borrow_mut().children.push( node.clone() ); }
  }
  else { scenes[ 0 ].borrow_mut().children.push( node.clone() ); }
}
```

If `primitives_data[ 2 ].parent == Some( 2 )` (self-reference), or a longer cycle like `0 -> 1 ->
0`, this loop wires `Rc<RefCell<Node>>` parent/child pointers that reference each other in a
cycle, with no error surfaced.

## Impact

**Who is affected:** Any caller constructing `PrimitiveData` with a cyclic `parent` chain --
today, entirely hypothetical for this crate's own generators (`solid.rs`, `primitive.rs`,
`text/ufo.rs` never set `parent` to anything but the struct's own default `None`), but the
function accepts an arbitrary `&[PrimitiveData]` from any caller, including future ones.

**What breaks:** A `Node` graph containing a genuine `Rc`/`RefCell` reference cycle -- besides
producing an unbounded/broken scene hierarchy for any downstream traversal (e.g. a recursive
world-matrix update walking `parent` links would loop forever), `Rc`'s reference-counting cannot
collect a cycle, so this is also a real, permanent memory leak for the lifetime of the returned
`GLTF`.

**Consumer audit:** Grepped all 6+ call sites of `primitives_data_to_gltf` (all in `examples/`,
outside this fix's scope) -- none currently construct `PrimitiveData` with a non-`None` `parent`
at all, so no live caller is affected today; this is a hardening fix against future/malformed
input, not a currently-triggered defect.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/primitive_generation`.

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/primitive_data_test.rs
let primitives = vec![ primitive_with_parent( Some( 0 ) ) ]; // parent points at itself
let result = primitives_parent_graph_validate( &primitives );
assert!( result.is_err() ); // pre-fix: no such function existed; the wiring loop accepted this silently
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run -E 'binary(primitive_data_test)'
```

## Root Cause

The parent/child wiring loop linked `Rc<RefCell<Node>>` pointers directly from
`PrimitiveData::parent` indices, checking only that the index was in-bounds
(`nodes.get( parent_index )`) -- "resolves to a real node" and "the resulting parent chain
terminates in a finite number of hops" are two different properties, and only the first was ever
checked.

## Why Not Caught

`primitives_data_to_gltf` requires a real `WebGl2RenderingContext` to run at all, so nothing
about its parent-wiring loop could be natively unit tested in isolation before this fix -- the
only place a cycle could have been observed was a live browser render producing a silently broken
or leaking scene graph, with no diagnostic pointing back at the bad index.

## Fix Location

`module/helper/primitive_generation/src/primitive_data.rs`: added
`primitives_parent_graph_validate( primitives_data : &[ PrimitiveData ] ) -> Result< (), String >`,
a pure function requiring no GL context -- walks each primitive's parent chain with a per-start
visited-index `HashSet`, returning `Err` on the first revisited index (covers both direct
self-reference and longer cycles), and treating an out-of-bounds parent index as "no parent" to
match the wiring loop's own existing fallback behavior. `primitives_data_to_gltf` now calls this
function first and `panic!`s with the returned message on `Err`, consistent with the function's
existing panic-based error idiom (it already `.unwrap()`s buffer creation). Exposed as `pub` via
`mod_interface` so it is independently testable without a GL context, matching this crate's own
`Font::max_size()` (BUG-216) precedent for extracting pure logic out of otherwise GL/IO-bound
functions.

## Prevention

New test file `primitive_data_test.rs` with 4 tests: `self_referencing_parent_is_rejected`,
`two_cycle_parent_chain_is_rejected`, `acyclic_parent_chain_is_accepted` (a genuine 3-node tree),
and `out_of_bounds_parent_is_treated_as_no_parent` (confirming the fallback-compatibility
behavior is preserved, not just the cycle-rejection behavior).

## Pitfall

Index-based parent links look like inert plain data right up until they're wired into live
`Rc`/`RefCell` graph pointers -- validating the indices *before* wiring is the only point where a
cycle is cheap to detect and cheap to test (a plain slice, no GL context or live node graph
required); after wiring, finding the same cycle means walking live `Rc` pointers instead, and by
then the leak has already happened.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/primitive_generation`. |
| 2026-08-20 | fixed | Added `primitives_parent_graph_validate`; wired into `primitives_data_to_gltf` with panic-on-cycle. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `primitives_parent_graph_validate` to an unconditional `Ok(())` stub (simulating pre-fix "no validation at all") and confirmed `two_cycle_parent_chain_is_rejected` and `self_referencing_parent_is_rejected` both fail; restored the fix and confirmed 35/35 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-499)`/`Root cause`/`Pitfall` 3-field comment applied at the new function's definition; `primitives_data_to_gltf`'s `# Panics` doc updated to document the new cycle/self-reference panic condition. | — |
| D3 | Scope containment | — | 🟢 | Panic-on-cycle chosen (not a `Result`-returning signature change) specifically to avoid touching any of the 6+ out-of-scope `examples/` call sites -- confirmed no call site required edits via `git diff` / grep. | — |

**Reproduced:** YES -- temporarily reverted `primitives_parent_graph_validate` to
`Ok( () )` unconditionally; `two_cycle_parent_chain_is_rejected` and
`self_referencing_parent_is_rejected` both failed (`expected ... to be rejected as a cycle, got
Ok(())`). Restored the fix; full crate suite (35/35) passes with 0 warnings. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/primitive_data.rs` | Added `primitives_parent_graph_validate`; `primitives_data_to_gltf` now calls it and panics on `Err`; `# Panics` doc updated; exposed via `mod_interface`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/primitive_data_test.rs` | New file: 4 tests covering self-reference, a 2-cycle, a genuine acyclic tree, and out-of-bounds-parent fallback compatibility. |
| `module/helper/primitive_generation/tests/readme.md` | New file: Responsibility Table for the crate's 10 test files (8 pre-existing + this + `ufo_font_scale_test.rs`, BUG-500). |
