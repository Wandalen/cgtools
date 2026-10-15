use super::*;

#[ test ]
fn test_const_length_tuple3()
{
  use the_module::ConstLength;
  assert_eq!( <( i32, i32, i32 ) as ConstLength>::LEN, 3 );
}

#[ test ]
fn test_vector_ref_tuple3()
{
  use the_module::ArrayRef;
  let tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  let array_ref : &[ i32; 3 ] = tuple.array_ref();
  assert_eq!( array_ref, &[ 42, 43, 44 ] );
}

#[ test ]
fn test_vector_mut_tuple3()
{
  use the_module::ArrayMut;
  let mut tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  {
    let vector_mut : &mut [ i32; 3 ] = tuple.vector_mut();
    vector_mut[ 0 ] = 100;
    vector_mut[ 1 ] = 200;
    vector_mut[ 2 ] = 300;
  }
  assert_eq!( tuple, ( 100, 200, 300 ) );
}

#[ test ]
fn test_vector_iter_tuple3()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple3()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  {
    let mut iter = tuple.vector_iter_mut();
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    if let Some( x ) = iter.next()
    {
      *x = 200;
    }
    if let Some( x ) = iter.next()
    {
      *x = 300;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200, 300 ) );
}

#[ test ]
fn test_vector_iter_rev_tuple3()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple3()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  {
    let mut iter = tuple.vector_iter_mut().rev();
    if let Some( x ) = iter.next()
    {
      *x = 300;
    }
    if let Some( x ) = iter.next()
    {
      *x = 200;
    }
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200, 300 ) );
}

// test_kind: bug_reproducer(BUG-050)
/// ## Root Cause
/// `Tuple3IterMut::next()`/`next_back()` (`vector/tuple3.rs`) shared a single monotonically
/// increasing `index` field with per-value match arms hardcoded for one iteration direction —
/// two `.next()` calls followed by `.next_back()` on the same (non-`.rev()`) iterator
/// re-yielded the first element (`tuple.0`) as a second, simultaneously-live `&mut E`
/// reference while the last element (`tuple.2`) was never reached at all, violating Rust's
/// unique-mutable-reference aliasing guarantee (confirmed under Miri's Stacked Borrows checker).
/// ## Why Not Caught
/// Every existing `vector_iter_mut` test called only `.next()` repeatedly, or only `.rev()`
/// then `.next()` repeatedly (fully reversed) — never mixed `.next()`/`.next_back()` calls on
/// the same unwrapped iterator, the exact trigger condition for the double-yield.
/// ## Fix Applied
/// Replaced the shared `index` field with independent `front`/`back` cursors (mirrors
/// `core::slice::IterMut`'s own two-cursor design), so they converge but provably never cross.
/// ## Prevention
/// Any hand-rolled `DoubleEndedIterator` yielding `&mut` references must be tested with at
/// least one mixed-direction sequence (`.next()`/`.next()`/`.next_back()` or similar) asserting
/// the final values match what a correct front/back traversal would produce.
/// ## Pitfall
/// A `DoubleEndedIterator` backed by one shared index counter is sound only under
/// single-direction iteration — mixing directions silently double-yields an already-returned
/// element as a second live aliased `&mut` reference while leaving another element unreached.
#[ test ]
fn test_vector_iter_mut_next_and_next_back_disjoint_tuple3()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  {
    let mut iter = tuple.vector_iter_mut();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    let c = iter.next_back().unwrap();
    *a = 100;
    *b = 200;
    *c = 300;
    assert_eq!( iter.next(), None );
    assert_eq!( iter.next_back(), None );
  }
  assert_eq!( tuple, ( 100, 200, 300 ), "next() and next_back() must yield disjoint elements, not alias the same slot" );
}

// test_kind: bug_reproducer(BUG-122)
/// ## Root Cause
/// `Tuple3Iter::next()`/`next_back()` (`vector/tuple3.rs`) shared a single monotonically
/// increasing `index` field with per-value match arms hardcoded for one iteration direction —
/// identical shape to BUG-050 (fixed only for the `*Mut` sibling), so a `.next()` call followed
/// by `.next_back()` on the same (non-`.rev()`) iterator reinterpreted the post-`next()` index
/// as if counted from the back, returning the wrong element instead of the true back of the
/// remaining range.
/// ## Why Not Caught
/// Every existing `vector_iter` test called only `.next()` repeatedly, or only `.rev()` then
/// `.next()` repeatedly (fully reversed) — never mixed `.next()`/`.next_back()` on the same
/// unwrapped iterator, the exact trigger condition. BUG-050's own fix only updated the `*Mut`
/// iterators, since the immutable ones don't alias unsafely and so never tripped Miri.
/// ## Fix Applied
/// Replaced the shared `index` field with independent `front`/`back` cursors, mirroring the
/// already-fixed `Tuple3IterMut` (BUG-050) and `core::slice::Iter`'s own two-cursor design.
/// ## Prevention
/// This test mixes `.next()`/`.next_back()` on `vector_iter()` (not `vector_iter_mut()`) and
/// asserts every yielded value against what a correct front/back traversal produces.
/// ## Pitfall
/// A shared-index `DoubleEndedIterator` is only correct under single-direction iteration even
/// when its yielded references are shared (`&E`) rather than exclusive (`&mut E`) — the
/// aliasing-safety argument that excuses skipping Miri does not excuse skipping a mixed-order
/// correctness test.
#[ test ]
fn test_vector_iter_next_and_next_back_disjoint_tuple3()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ), "front element" );
  assert_eq!( iter.next_back(), Some( &44 ), "back of the remaining [43,44] range" );
  assert_eq!( iter.next(), Some( &43 ), "only element left" );
  assert_eq!( iter.next(), None );
  assert_eq!( iter.next_back(), None );
}
