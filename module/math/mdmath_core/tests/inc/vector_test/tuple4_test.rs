use super::*;

#[ test ]
fn test_const_length_tuple4()
{
  use the_module::ConstLength;
  assert_eq!( <( i32, i32, i32, i32 ) as ConstLength>::LEN, 4 );
}

#[ test ]
fn test_vector_ref_tuple4()
{
  use the_module::ArrayRef;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let array_ref : &[ i32; 4 ] = tuple.array_ref();
  assert_eq!( array_ref, &[ 42, 43, 44, 45 ] );
}

#[ test ]
fn test_vector_mut_tuple4()
{
  use the_module::ArrayMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  {
    let vector_mut : &mut [ i32; 4 ] = tuple.vector_mut();
    vector_mut[ 0 ] = 100;
    vector_mut[ 1 ] = 200;
    vector_mut[ 2 ] = 300;
    vector_mut[ 3 ] = 400;
  }
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

#[ test ]
fn test_vector_iter_tuple4()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), Some( &45 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple4()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
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
    if let Some( x ) = iter.next()
    {
      *x = 400;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

#[ test ]
fn test_vector_iter_rev_tuple4()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some( &45 ) );
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple4()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  {
    let mut iter = tuple.vector_iter_mut().rev();
    if let Some( x ) = iter.next()
    {
      *x = 400;
    }
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
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

// test_kind: bug_reproducer(BUG-050)
/// ## Root Cause
/// `Tuple4IterMut::next()`/`next_back()` (`vector/tuple4.rs`) shared a single monotonically
/// increasing `index` field with per-value match arms hardcoded for one iteration direction —
/// alternating `.next()`/`.next_back()` calls on the same (non-`.rev()`) iterator re-yielded
/// two already-returned elements (`tuple.0` and `tuple.2`) as second, simultaneously-live
/// `&mut E` references while `tuple.1` and `tuple.3` were never reached, violating Rust's
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
/// least one mixed-direction sequence (alternating `.next()`/`.next_back()`) asserting the
/// final values match what a correct front/back traversal would produce.
/// ## Pitfall
/// A `DoubleEndedIterator` backed by one shared index counter is sound only under
/// single-direction iteration — mixing directions silently double-yields already-returned
/// elements as live aliased `&mut` references while leaving others unreached.
#[ test ]
fn test_vector_iter_mut_next_and_next_back_disjoint_tuple4()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  {
    let mut iter = tuple.vector_iter_mut();
    let a = iter.next().unwrap();
    let b = iter.next_back().unwrap();
    let c = iter.next().unwrap();
    let d = iter.next_back().unwrap();
    *a = 100;
    *b = 200;
    *c = 300;
    *d = 400;
    assert_eq!( iter.next(), None );
    assert_eq!( iter.next_back(), None );
  }
  assert_eq!( tuple, ( 100, 300, 400, 200 ), "next() and next_back() must yield disjoint elements, not alias the same slot" );
}

// test_kind: bug_reproducer(BUG-122)
/// ## Root Cause
/// `Tuple4Iter::next()`/`next_back()` (`vector/tuple4.rs`) shared a single monotonically
/// increasing `index` field with per-value match arms hardcoded for one iteration direction —
/// identical shape to BUG-050 (fixed only for the `*Mut` sibling), so alternating
/// `.next()`/`.next_back()` calls on the same (non-`.rev()`) iterator reinterpreted the shared
/// index inconsistently between the two directions, yielding wrong elements instead of a true
/// front/back traversal.
/// ## Why Not Caught
/// Every existing `vector_iter` test called only `.next()` repeatedly, or only `.rev()` then
/// `.next()` repeatedly (fully reversed) — never alternated `.next()`/`.next_back()` on the
/// same unwrapped iterator, the exact trigger condition. BUG-050's own fix only updated the
/// `*Mut` iterators, since the immutable ones don't alias unsafely and so never tripped Miri.
/// ## Fix Applied
/// Replaced the shared `index` field with independent `front`/`back` cursors, mirroring the
/// already-fixed `Tuple4IterMut` (BUG-050).
/// ## Prevention
/// This test alternates `.next()`/`.next_back()` on `vector_iter()` and asserts each call
/// yields the element a correct front/back traversal would produce.
/// ## Pitfall
/// A shared-index `DoubleEndedIterator` is only correct under single-direction iteration even
/// when its yielded references are shared (`&E`) rather than exclusive (`&mut E`) — the
/// aliasing-safety argument that excuses skipping Miri does not excuse skipping a mixed-order
/// correctness test.
#[ test ]
fn test_vector_iter_next_and_next_back_disjoint_tuple4()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ), "front" );
  assert_eq!( iter.next_back(), Some( &45 ), "back of remaining [43,44,45]" );
  assert_eq!( iter.next(), Some( &43 ), "front of remaining [43,44]" );
  assert_eq!( iter.next_back(), Some( &44 ), "only element left" );
  assert_eq!( iter.next(), None );
  assert_eq!( iter.next_back(), None );
}
