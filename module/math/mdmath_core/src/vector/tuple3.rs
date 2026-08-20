#[ cfg( debug_assertions ) ]
use core::mem::{ align_of_val, size_of_val };

use super::{Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, VectorIter, VectorIteratorRef, VectorIterMut, VectorIterator};

// = 3

impl< E > Collection for ( E, E, E )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, E, E )
{
  const LEN : usize = 3;
}

impl< E > IntoArray< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 3 ]
  {
    [ self.0, self.1, self.2 ]
  }
}

impl< E > ArrayRef< E, 3 > for ( E, E, E )
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; 3 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `(E, E, E)`
    // into a reference to an array `[E; 3]`. This is safe because:
    // 1. The tuple `(E, E, E)` and the array `[E; 3]` have the same memory layout.
    //    - Both contain 3 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    let result : &[ E; 3 ] = unsafe { &*( std::ptr::from_ref::< ( E, E, E ) >( self ).cast::< [ E; 3 ] >() ) };

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

impl< E > ArrayMut< E, 3 > for ( E, E, E )
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 3 ]
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

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `(E, E, E)`
    // into a reference to an array `[E; 3]`. This is safe because:
    // 1. The tuple `(E, E, E)` and the array `[E; 3]` have the same memory layout.
    //    - Both contain 3 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    let result : &mut [ E; 3 ] = unsafe { &mut *( std::ptr::from_mut::< ( E, E, E ) >( self ).cast::< [ E; 3 ] >() ) };

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
// arms were hardcoded per-direction — after any `next()` call, `next_back()` reinterpreted
// the resulting `index` as if it were counted from the back, yielding the wrong field (or,
// for the tuple2 sibling, the same field twice while dropping the other entirely) instead of
// the true back of the remaining range.
// Root cause: same shared-single-cursor shape as the already-fixed `Tuple3IterMut` (BUG-050),
// just never itself updated when that fix landed — aliasing `&E` is safe here, so the
// consequence is wrong values rather than UB, but the logic defect is identical.
// Pitfall: a hand-rolled `DoubleEndedIterator` needs independent front/back cursors (mirrors
// `core::slice::Iter`) even when the yielded references are shared and aliasing-safe — a
// single shared counter is a correctness bug, not just a soundness one, under mixed
// `.next()`/`.next_back()` sequences; pure-forward or pure-`.rev()` alone cannot catch it.
#[ derive( Clone ) ]
struct Tuple3Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple3Iter< 'tuple_ref, E >
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

    match index {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      2 => Some( &self.tuple.2 ),
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple3Iter< '_, E > {}

impl< E > DoubleEndedIterator for Tuple3Iter< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      2 => Some( &self.tuple.2 ),
      _ => unreachable!(),
    }
  }
}

// Fix(BUG-050): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — mixing the two calls on one iterator (e.g. two `.next()`
// then one `.next_back()`) re-yielded an already-returned tuple field as a second
// simultaneously-live `&mut E` reference instead of reaching the untouched one.
// Root cause: copy-pasted from the immutable `Tuple3Iter` above (where aliasing `&E` is
// harmless) into a `&mut` context without redesigning the cursor for unique-borrow safety.
// Pitfall: a hand-rolled `DoubleEndedIterator` yielding `&mut` references needs independent
// front/back cursors (mirrors `core::slice::IterMut`), never a single shared counter — always
// test a mixed `.next()`/`.next_back()` sequence, not just pure-forward or pure-`.rev()`.
struct Tuple3IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple3IterMut< 'tuple_ref, E >
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
      2 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.2) ) }
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

impl< E > ExactSizeIterator for Tuple3IterMut< '_, E > {}

impl< E > DoubleEndedIterator for Tuple3IterMut< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back {
      return None;
    }

    self.back -= 1;

    match self.back {
      0 => {
        // SAFETY: see `next` — `front`/`back` never cross, so each field is reborrowed
        // at most once across the whole iteration.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 => {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      2 => {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.2) ) }
      },
      _ => unreachable!(),
    }
  }
}

impl< E: Clone > VectorIter< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    Tuple3Iter
    {
      tuple : self,
      front : 0,
      back : 3,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut E >
  where
    E : 'tuple_ref,
  {
    Tuple3IterMut
    {
      tuple : self,
      front : 0,
      back : 3,
    }
  }
}
