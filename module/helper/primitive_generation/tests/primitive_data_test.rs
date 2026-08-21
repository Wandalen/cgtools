//! Integration tests for `primitive_generation::primitives_parent_graph_validate`.
//!
//! Covers BUG-499: `primitives_data_to_gltf`'s parent/child wiring loop linked
//! `Rc<RefCell<Node>>` pointers directly from `PrimitiveData::parent` indices
//! with no acyclic check, so a self-referencing or cyclic parent index built a
//! broken/leaking node graph with no error surfaced.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::{ PrimitiveData, Transform, primitives_parent_graph_validate };
  use minwebgl::F32x4;

  /// Builds a minimal `PrimitiveData` carrying only a `parent` link -- the rest
  /// of the fields are irrelevant to `primitives_parent_graph_validate`, which
  /// reads nothing but `parent`.
  fn primitive_with_parent( parent : Option< usize > ) -> PrimitiveData
  {
    PrimitiveData
    {
      name : None,
      parent,
      attributes : None,
      color : F32x4::default(),
      transform : Transform::default(),
    }
  }

  // test_kind: bug_reproducer(BUG-499)
  /// ## Root Cause
  /// `primitives_data_to_gltf`'s parent/child wiring loop read
  /// `primitive.parent` and, whenever the index was in bounds, wired
  /// `Rc<RefCell<Node>>` parent/child pointers directly -- with no check that
  /// walking the resulting parent chain ever terminates. A primitive whose own
  /// `parent` index points back at itself creates a `Node` that is its own
  /// ancestor: a one-node reference cycle wired with no error.
  ///
  /// Reproducer: `primitives_data_to_gltf` never runs in this native test (it
  /// requires a live `WebGl2RenderingContext`), but the acyclic check it now
  /// calls first is a pure function over `&[PrimitiveData]` -- calling
  /// `primitives_parent_graph_validate` directly on a single primitive whose
  /// `parent` is `Some( 0 )` (itself) reproduces the same self-referencing
  /// index the wiring loop would otherwise have accepted silently.
  ///
  /// ## Why Not Caught
  /// `primitives_data_to_gltf` requires a real `WebGl2RenderingContext` to run
  /// at all, so nothing about its parent-wiring loop could be natively unit
  /// tested in isolation before this fix -- the only place a cycle could have
  /// been observed was a live browser render producing a silently broken or
  /// leaking scene graph, with no diagnostic pointing back at the bad index.
  ///
  /// ## Fix Applied
  /// Added `primitives_parent_graph_validate`, a pure function over
  /// `&[PrimitiveData]` that walks each primitive's parent chain with a
  /// visited-index set, returning `Err` on the first revisited index (covers
  /// both direct self-reference and longer cycles). `primitives_data_to_gltf`
  /// now calls it before doing any GL work and panics with the returned
  /// message on `Err`, consistent with the function's existing panic-based
  /// error idiom (it already `.unwrap()`s buffer creation).
  ///
  /// ## Prevention
  /// Index-based parent links look like inert plain data right up until
  /// they're wired into live `Rc`/`RefCell` graph pointers -- validating the
  /// indices *before* wiring is the only point where a cycle is cheap to
  /// detect and cheap to test (a plain slice, no GL context or live node graph
  /// required); after wiring, finding the same cycle means walking live `Rc`
  /// pointers instead.
  ///
  /// ## Pitfall
  /// A parent index that happens to be in-bounds passes every check a naive
  /// wiring loop performs (`nodes.get( parent_index )` succeeds) -- "resolves
  /// to a real node" and "terminates in a finite number of hops" are two
  /// different properties, and only the first was ever checked.
  #[ test ]
  fn self_referencing_parent_is_rejected()
  {
    let primitives = vec![ primitive_with_parent( Some( 0 ) ) ];

    let result = primitives_parent_graph_validate( &primitives );

    assert!
    (
      result.is_err(),
      "expected a self-referencing parent (primitive 0's parent is itself) to be \
       rejected as a cycle, got Ok(())"
    );
  }

  #[ test ]
  fn two_cycle_parent_chain_is_rejected()
  {
    let primitives = vec!
    [
      primitive_with_parent( Some( 1 ) ),
      primitive_with_parent( Some( 0 ) ),
    ];

    let result = primitives_parent_graph_validate( &primitives );

    assert!
    (
      result.is_err(),
      "expected a 2-cycle parent chain (0 -> 1 -> 0) to be rejected, got Ok(())"
    );
  }

  #[ test ]
  fn acyclic_parent_chain_is_accepted()
  {
    let primitives = vec!
    [
      primitive_with_parent( None ),
      primitive_with_parent( Some( 0 ) ),
      primitive_with_parent( Some( 1 ) ),
    ];

    let result = primitives_parent_graph_validate( &primitives );

    assert!
    (
      result.is_ok(),
      "expected a genuine tree (0 <- 1 <- 2) to be accepted, got {result:?}"
    );
  }

  #[ test ]
  fn out_of_bounds_parent_is_treated_as_no_parent()
  {
    // Matches `primitives_data_to_gltf`'s own wiring fallback: an out-of-bounds
    // parent index roots that node at the scene instead of erroring, so
    // validation must not treat it as a cycle either.
    let primitives = vec![ primitive_with_parent( Some( 99 ) ) ];

    let result = primitives_parent_graph_validate( &primitives );

    assert!
    (
      result.is_ok(),
      "expected an out-of-bounds parent index to be treated as \"no parent\", got {result:?}"
    );
  }
}
