#[ cfg( debug_assertions ) ]
use std::mem::{ align_of_val, size_of_val };

use super::*;

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
    use std::mem::transmute;

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E, E, E)`
    // into a reference to an array `[E; 3]`. This is safe because:
    // 1. The tuple `(E, E, E)` and the array `[E; 3]` have the same memory layout.
    //    - Both contain 3 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.

    #[ allow( unsafe_code ) ]
    let result : &[ E; 3 ] = unsafe { transmute( self ) };

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
    use std::mem::transmute;

    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );
    #[ cfg( debug_assertions ) ]
    let size_component = size_of_val( &self.1 );
    #[ cfg( debug_assertions ) ]
    let align_component = align_of_val( &self.1 );

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E, E, E)`
    // into a reference to an array `[E; 3]`. This is safe because:
    // 1. The tuple `(E, E, E)` and the array `[E; 3]` have the same memory layout.
    //    - Both contain 3 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ E; 3 ] = unsafe { transmute( self ) };

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

#[ derive( Clone ) ]
struct Tuple3Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E, E ),
  index : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple3Iter< 'tuple_ref, E >
{
  type Item = &'tuple_ref E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    match self.index {
      0 => {
        self.index += 1;
        Some( &self.tuple.0 )
      },
      1 => {
        self.index += 1;
        Some( &self.tuple.1 )
      },
      2 => {
        self.index += 1;
        Some( &self.tuple.2 )
      },
      _ => None,
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = 3 - self.index;
    ( remaining, Some( remaining ) )
  }
}

impl< 'tuple_ref, E > ExactSizeIterator for Tuple3Iter< 'tuple_ref, E > {}

impl< 'tuple_ref, E > DoubleEndedIterator for Tuple3Iter< 'tuple_ref, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    match self.index {
      0 => {
        self.index += 1;
        Some( &self.tuple.2 )
      },
      1 => {
        self.index += 1;
        Some( &self.tuple.1 )
      },
      2 => {
        self.index += 1;
        Some( &self.tuple.0 )
      },
      _ => None,
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
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.0 as *mut E ) ) }
      },
      1 =>
      {
        // SAFETY: see the arm above.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.1 as *mut E ) ) }
      },
      2 =>
      {
        // SAFETY: see the arm above.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.2 as *mut E ) ) }
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

impl< 'tuple_ref, E > ExactSizeIterator for Tuple3IterMut< 'tuple_ref, E > {}

impl< 'tuple_ref, E > DoubleEndedIterator for Tuple3IterMut< 'tuple_ref, E >
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
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.0 as *mut E ) ) }
      },
      1 => {
        // SAFETY: see the arm above.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.1 as *mut E ) ) }
      },
      2 => {
        // SAFETY: see the arm above.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.2 as *mut E ) ) }
      },
      _ => unreachable!(),
    }
  }
}

impl< E: Clone > VectorIter< E, 3 > for ( E, E, E )
{
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    Tuple3Iter
    {
      tuple : self,
      index : 0,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 3 > for ( E, E, E )
{
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
