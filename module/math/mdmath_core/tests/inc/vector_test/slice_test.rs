use super::*;

#[ test ]
fn test_vector_ref_slice()
{
  use the_module::ArrayRef;
  let slice : &[ i32 ] = &[];
  let array_ref : &[ i32; 0 ] = slice.array_ref();
  assert_eq!( array_ref, &[] as &[ i32; 0 ] );

  let slice : &[ i32 ] = &[ 42 ];
  let array_ref : &[ i32; 1 ] = slice.array_ref();
  assert_eq!( array_ref, &[ 42 ] );

  let slice : &[ i32 ] = &[ 1, 2, 3 ];
  let array_ref : &[ i32; 3 ] = slice.array_ref();
  assert_eq!( array_ref, &[ 1, 2, 3 ] );
}

// test_kind: bug_reproducer(BUG-054)
/// ## Root Cause
/// `ArrayMut<E,N>::vector_mut` for `[E]` cast `self.as_ptr()` (`SharedReadOnly`
/// provenance) to `*mut [E;N]` instead of `self.as_mut_ptr()` (`Unique`
/// provenance) before writing through it.
///
/// ## Why Not Caught
/// This already-existing functional test exercises the exact sequence that
/// exposes BUG-054, but the provenance violation is undefined behavior rather
/// than an observable failure under ordinary execution — a plain `cargo test`
/// run passes regardless, so nothing flagged it before Miri checked provenance.
///
/// ## Fix Applied
/// `vector_mut` now casts through `self.as_mut_ptr()` (`Unique` provenance)
/// instead of `self.as_ptr()` before writing through the resulting pointer.
///
/// ## Prevention
/// Verified under Miri Stacked Borrows: `cargo +nightly miri test -p mdmath_core
/// --all-features` runs this exact test to confirm the provenance violation is
/// gone.
///
/// ## Pitfall
/// Casting a shared (`&self`)-derived pointer to a `*mut` and writing through it
/// is UB under Stacked Borrows even when it "works" on real hardware — always
/// derive a mutable-cast pointer from `as_mut_ptr()`, never `as_ptr()`.
#[ test ]
fn test_vector_mut_slice()
{
  use the_module::ArrayMut;
  let slice : &mut [ i32 ] = &mut [];
  {
    let vector_mut : &mut [ i32; 0 ] = slice.vector_mut();
    assert_eq!( vector_mut, &mut [] as &mut [ i32; 0 ] );
  }

  let slice : &mut [ i32 ] = &mut [ 42 ];
  {
    let vector_mut : &mut [ i32; 1 ] = slice.vector_mut();
    vector_mut[ 0 ] = 100;
  }
  assert_eq!( slice, &[ 100 ] );

  let slice : &mut [ i32 ] = &mut [ 1, 2, 3 ];
  {
    let vector_mut : &mut [ i32; 3 ] = slice.vector_mut();
    vector_mut[ 0 ] = 10;
    vector_mut[ 1 ] = 20;
    vector_mut[ 2 ] = 30;
  }
  assert_eq!( slice, &[ 10, 20, 30 ] );
}

#[ test ]
fn test_vector_iter_slice()
{
  use the_module::VectorIter;
  let slice : &[ i32 ] = &[];
  let mut iter = <[ i32 ] as VectorIter< i32, 0 >>::vector_iter( slice );
  assert_eq!( iter.next(), None );

  let slice : &[ i32 ] = &[ 42 ];
  let mut iter = <[ i32 ] as VectorIter< i32, 1 >>::vector_iter( slice );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );

  let slice : &[ i32 ] = &[ 1, 2, 3 ];
  let mut iter = <[ i32 ] as VectorIter< i32, 3 >>::vector_iter( slice );
  assert_eq!( iter.next(), Some( &1 ) );
  assert_eq!( iter.next(), Some( &2 ) );
  assert_eq!( iter.next(), Some( &3 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_slice()
{
  use the_module::VectorIterMut;
  let slice : &mut [ i32 ] = &mut [];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 0 >>::vector_iter_mut( slice );
    assert_eq!( iter.next(), None );
  }

  let slice : &mut [ i32 ] = &mut [ 42 ];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 1 >>::vector_iter_mut( slice );
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( slice, &[ 100 ] );

  let slice : &mut [ i32 ] = &mut [ 1, 2, 3 ];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 3 >>::vector_iter_mut( slice );
    if let Some( x ) = iter.next()
    {
      *x = 10;
    }
    if let Some( x ) = iter.next()
    {
      *x = 20;
    }
    if let Some( x ) = iter.next()
    {
      *x = 30;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( slice, &[ 10, 20, 30 ] );
}
