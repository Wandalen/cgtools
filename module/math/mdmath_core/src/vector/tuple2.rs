#[ cfg( debug_assertions ) ]
use core::mem::{ align_of_val, size_of_val };

use super::{Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, VectorIter, VectorIteratorRef, VectorIterMut, VectorIterator};

// = 2

impl< E > Collection for ( E, E )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, E )
{
  const LEN : usize = 2;
}

impl< E > IntoArray< E, 2 > for ( E, E )
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 2 ]
  {
    [ self.0, self.1 ]
  }
}

impl< E > ArrayRef< E, 2 > for ( E, E )
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; 2 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `(E,)`
    // into a reference to an array `[E; N]`. This is safe because:
    // 1. The tuple `(E,)` and the array `[E; N]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    let result : &[ E; 2 ] = unsafe { &*( std::ptr::from_ref::< ( E, E ) >( self ).cast::< [ E; 2 ] >() ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // Check size and alignment of the first component
    debug_assert_eq!( size_of_val( &self.1 ), size_of_val( &result[ 1 ] ), "Component size should be the same" );
    debug_assert_eq!( align_of_val( &self.1 ), align_of_val( &result[ 1 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl< E > ArrayMut< E, 2 > for ( E, E )
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 2 ]
  {
    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );
    #[ cfg( debug_assertions ) ]
    let size_component = size_of_val( &self.1 );
    #[ cfg( debug_assertions ) ]
    let align_component = align_of_val( &self.1 );

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `(E,)`
    // into a reference to an array `[E; 1]`. This is safe because:
    // 1. The tuple `(E,)` and the array `[E; 1]` have the same memory layout.
    //    - Both contain a single element of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    let result : &mut [ E; 2 ] = unsafe { &mut *( std::ptr::from_mut::< ( E, E ) >( self ).cast::< [ E; 2 ] >() ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_component, size_of_val( &result[ 1 ] ), "Component size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_component, align_of_val( &result[ 1 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

// Fix(BUG-122): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — after a `next()` call, `next_back()` reinterpreted the
// resulting `index` as if counted from the back, returning the same field `next()` already
// returned (as a second, harmless-but-wrong `&E`) while the other field was never yielded.
// Root cause: same shared-single-cursor shape as the already-fixed `Tuple2IterMut` (BUG-050),
// just never itself updated when that fix landed.
// Pitfall: a hand-rolled `DoubleEndedIterator` needs independent front/back cursors even when
// the yielded references are shared and aliasing-safe — a single shared counter is a
// correctness bug, not just a soundness one, under mixed `.next()`/`.next_back()` sequences.
#[ derive( Clone ) ]
struct Tuple2Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple2Iter< 'tuple_ref, E >
{
  type Item = &'tuple_ref E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    let index = self.front;
    self.front += 1;

    match index
    {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple2Iter< '_, E > {}

impl< E > DoubleEndedIterator for Tuple2Iter< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back
    {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      _ => unreachable!(),
    }
  }
}

// Fix(BUG-050): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — mixing the two calls on one iterator (e.g. `.next()`
// then `.next_back()`) double-yielded the same tuple field as two simultaneously-live `&mut E`
// references instead of two disjoint ones.
// Root cause: copy-pasted from the immutable `Tuple2Iter` above (where aliasing `&E` is
// harmless) into a `&mut` context without redesigning the cursor for unique-borrow safety.
// Pitfall: a hand-rolled `DoubleEndedIterator` yielding `&mut` references needs independent
// front/back cursors (mirrors `core::slice::IterMut`), never a single shared counter — always
// test a mixed `.next()`/`.next_back()` sequence, not just pure-forward or pure-`.rev()`.
struct Tuple2IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple2IterMut< 'tuple_ref, E >
{
  type Item = &'tuple_ref mut E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    let index = self.front;
    self.front += 1;

    match index
    {
      0 =>
      {
        // SAFETY: `front` and `back` never cross (guarded above), so this field is
        // reborrowed at most once across the whole iteration — either here, from the
        // front, or in `next_back`, from the back, but never both — so this can never
        // alias a mutable reference already handed out by a previous call.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple2IterMut< '_, E > {}

impl< E > DoubleEndedIterator for Tuple2IterMut< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back
    {
      0 =>
      {
        // SAFETY: see `next` — `front`/`back` never cross, so each field is reborrowed
        // at most once across the whole iteration.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      _ => unreachable!(),
    }
  }
}

impl< E: Clone > VectorIter< E, 2 > for ( E, E )
{
  #[ inline ]
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    Tuple2Iter
    {
      tuple : self,
      front : 0,
      back : 2,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 2 > for ( E, E )
{
  #[ inline ]
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut E >
  where
    E : 'tuple_ref,
  {
    Tuple2IterMut
    {
      tuple : self,
      front : 0,
      back : 2,
    }
  }
}
