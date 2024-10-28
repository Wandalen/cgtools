use super::*;

// = 4

impl< E > Collection for ( E, E, E, E )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, E, E, E )
{
  const LEN : usize = 4;
}

impl< E > VectorRef< E, 4 > for ( E, E, E, E )
{
  #[ inline( always ) ]
  fn vector_ref( &self ) -> &[ E ; 4 ]
  {
    use std::mem::{ align_of_val, size_of_val, transmute };

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E, E, E, E)`
    // into a reference to an array `[E; 4]`. This is safe because:
    // 1. The tuple `(E, E, E, E)` and the array `[E; 4]` have the same memory layout.
    //    - Both contain 4 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.

    #[ allow( unsafe_code ) ]
    let result : &[ E; 4 ] = unsafe { transmute( self ) };

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

impl< E > VectorMut< E, 4 > for ( E, E, E, E )
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 4 ]
  {
    use std::mem::{ align_of_val, size_of_val, transmute };

    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );
    #[ cfg( debug_assertions ) ]
    let size_component = size_of_val( &self.1 );
    #[ cfg( debug_assertions ) ]
    let align_component = align_of_val( &self.1 );

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E, E, E, E)`
    // into a reference to an array `[E; 4]`. This is safe because:
    // 1. The tuple `(E, E, E, E)` and the array `[E; 4]` have the same memory layout.
    //    - Both contain 4 elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ E; 4 ] = unsafe { transmute( self ) };

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
struct Tuple4Iter< 'a, E >
{
  tuple : &'a ( E, E, E, E ),
  index : usize,
}

impl< 'a, E > Iterator for Tuple4Iter< 'a, E >
{
  type Item = &'a E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
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
      3 => {
        self.index += 1;
        Some( &self.tuple.3 )
      },
      _ => None,
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = 4 - self.index;
    ( remaining, Some( remaining ) )
  }
}

impl< 'a, E > ExactSizeIterator for Tuple4Iter< 'a, E > {}

impl< 'a, E > DoubleEndedIterator for Tuple4Iter< 'a, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
      0 => {
        self.index += 1;
        Some( &self.tuple.3 )
      },
      1 => {
        self.index += 1;
        Some( &self.tuple.2 )
      },
      2 => {
        self.index += 1;
        Some( &self.tuple.1 )
      },
      3 => {
        self.index += 1;
        Some( &self.tuple.0 )
      },
      _ => None,
    }
  }
}

struct Tuple4IterMut< 'a, E >
{
  tuple : &'a mut ( E, E, E, E ),
  index : usize,
}

impl< 'a, E > Iterator for Tuple4IterMut< 'a, E >
{
  type Item = &'a mut E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
      0 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the first element,
        // and we won't return it again in subsequent calls.
        // qqq : not sure it's sound, either prove it or find a sound solution
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.0 as *mut E ) ) }
      },
      1 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the second element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.1 as *mut E ) ) }
      },
      2 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the third element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.2 as *mut E ) ) }
      },
      3 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the fourth element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.3 as *mut E ) ) }
      },
      _ => None,
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = 4 - self.index;
    ( remaining, Some( remaining ) )
  }
}

impl< 'a, E > ExactSizeIterator for Tuple4IterMut< 'a, E > {}

impl< 'a, E > DoubleEndedIterator for Tuple4IterMut< 'a, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
      0 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the fourth element,
        // and we won't return it again in subsequent calls.
        // qqq : not sure it's sound, either prove it or find a sound solution
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.3 as *mut E ) ) }
      },
      1 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the third element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.2 as *mut E ) ) }
      },
      2 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the second element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.1 as *mut E ) ) }
      },
      3 =>
      {
        self.index += 1;
        // SAFETY: This is safe because we are returning a mutable reference to the first element,
        // and we won't return it again in subsequent calls.
        #[ allow( unsafe_code ) ]
        unsafe { Some( &mut *( &mut self.tuple.0 as *mut E ) ) }
      },
      _ => None,
    }
  }
}

impl< E: Clone > VectorIter< E, 4 > for ( E, E, E, E )
{
  fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
  where
    E : 'a,
  {
    Tuple4Iter
    {
      tuple : self,
      index : 0,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 4 > for ( E, E, E, E )
{
  fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
  where
    E : 'a,
  {
    Tuple4IterMut
    {
      tuple : self,
      index : 0,
    }
  }
}
