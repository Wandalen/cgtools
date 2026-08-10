use super::*;

#[ test ]
fn test_const_length_tuple()
{
  use the_module::ConstLength;
  assert_eq!( <( i32, i32 ) as ConstLength>::LEN, 2 );
}

#[ test ]
fn test_vector_ref_tuple()
{
  use the_module::ArrayRef;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let array_ref : &[ i32; 2 ] = tuple.array_ref();
  assert_eq!( array_ref, &[ 42, 43 ] );
}

#[ test ]
fn test_vector_mut_tuple()
{
  use the_module::ArrayMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let vector_mut : &mut [ i32; 2 ] = tuple.vector_mut();
    vector_mut[ 0 ] = 100;
    vector_mut[ 1 ] = 200;
  }
  assert_eq!( tuple, ( 100, 200 ) );
}

#[ test ]
fn test_vector_iter_tuple()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
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
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200 ) );
}

#[ test ]
fn test_vector_iter_rev_tuple()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let mut iter = tuple.vector_iter_mut().rev();
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
  assert_eq!( tuple, ( 100, 200 ) );
}

// test_kind: bug_reproducer(BUG-050)
/// ## Root Cause
/// `Tuple2IterMut::next()`/`next_back()` (`vector/tuple2.rs`) shared a single monotonically
/// increasing `index` field with per-value match arms hardcoded for one iteration direction —
/// mixing `.next()` then `.next_back()` on the same (non-`.rev()`) iterator yielded the SAME
/// tuple field twice as two simultaneously-live `&mut E` references, violating Rust's
/// unique-mutable-reference aliasing guarantee (confirmed under Miri's Stacked Borrows checker).
/// ## Why Not Caught
/// Every existing `vector_iter_mut` test called only `.next()` repeatedly, or only `.rev()` then
/// `.next()` repeatedly (fully reversed) — never mixed `.next()`/`.next_back()` calls on the same
/// unwrapped iterator, the exact trigger condition for the double-yield.
/// ## Fix Applied
/// Replaced the shared `index` field with independent `front`/`back` cursors (mirrors
/// `core::slice::IterMut`'s own two-cursor design), so they converge but provably never cross.
/// ## Prevention
/// Any hand-rolled `DoubleEndedIterator` yielding `&mut` references must be tested with at least
/// one mixed-direction sequence (`.next()` then `.next_back()`) asserting the final values match
/// what a correct front/back traversal would produce.
/// ## Pitfall
/// A `DoubleEndedIterator` backed by one shared index counter is sound only under
/// single-direction iteration — mixing directions silently double-yields the same element as two
/// live aliased `&mut` references.
#[ test ]
fn test_vector_iter_mut_mixed_direction_no_aliasing_tuple2()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let mut iter = tuple.vector_iter_mut();
    let a = iter.next().unwrap();
    let b = iter.next_back().unwrap();
    *a = 100;
    *b = 200;
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200 ) );
}
