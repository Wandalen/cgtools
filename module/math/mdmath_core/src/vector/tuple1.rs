#[ cfg( debug_assertions ) ]
use std::mem::{ align_of_val, size_of_val };

use super::*;

// = 1

impl< E > Collection for ( E, )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, )
{
  const LEN : usize = 1;
}

impl< E > IntoArray< E, 1 > for ( E, )
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 1 ]
  {
    [ self.0 ]
  }
}

impl< E > ArrayRef< E, 1 > for ( E, )
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; 1 ]
  {
    use std::mem::transmute;

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E,)`
    // into a reference to an array `[E; 1]`. This is safe because:
    // 1. The tuple `(E,)` and the array `[E; 1]` have the same memory layout.
    //    - Both contain a single element of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.

    #[ allow( unsafe_code ) ]
    let result : &[ E; 1 ] = unsafe { transmute( self ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // Check size and alignment of the first component
    debug_assert_eq!( size_of_val( &self.0 ), size_of_val( &result[ 0 ] ), "Component size should be the same" );
    debug_assert_eq!( align_of_val( &self.0 ), align_of_val( &result[ 0 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl< E > ArrayMut< E, 1 > for ( E, )
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 1 ]
  {
    use std::mem::transmute;

    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );
    #[ cfg( debug_assertions ) ]
    let size_component = size_of_val( &self.0 );
    #[ cfg( debug_assertions ) ]
    let align_component = align_of_val( &self.0 );

    // SAFETY: We are using `transmute` to convert a reference to a tuple `(E,)`
    // into a reference to an array `[E; 1]`. This is safe because:
    // 1. The tuple `(E,)` and the array `[E; 1]` have the same memory layout.
    //    - Both contain a single element of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ E; 1 ] = unsafe { transmute( self ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_component, size_of_val( &result[ 0 ] ), "Component size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_component, align_of_val( &result[ 0 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl< E: Clone > VectorIter< E, 1 > for ( E, )
{
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    std::iter::once( &self.0 )
  }
}

impl< E: Clone > VectorIterMut< E, 1 > for ( E, )
{
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut E >
  where
    E : 'tuple_ref,
  {
    std::iter::once( &mut self.0 )
  }
}
