#[ cfg( debug_assertions ) ]
use core::mem::{ align_of_val, size_of_val };

use crate::{ Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, Ix };
use ::ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4 };

// = 0

impl Collection for Ix0
{
  type Scalar = usize;
}

impl ConstLength for Ix0
{
  const LEN : usize = 0;
}

impl IntoArray< usize, 0 > for Ix0
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 0 ]
  {
    []
  }
}

impl ArrayRef< usize, 0 > for Ix0
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 0 ]
  {
    &[]
  }
}

impl ArrayMut< usize, 0 > for Ix0
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 0 ]
  {
    &mut []
  }
}

// = 1

impl Collection for Ix1
{
  type Scalar = Ix;
}

impl ConstLength for Ix1
{
  const LEN : usize = 1;
}

impl IntoArray< usize, 1 > for Ix1
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 1 ]
  {
    [ self[ 0 ] ]
  }
}

impl ArrayRef< usize, 1 > for Ix1
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 1 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &[ Ix ; 1 ] = unsafe { &*( ( self as *const Ix1 ).cast::< [ Ix ; 1 ] >() ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // // Check size and alignment of the first component
    // debug_assert_eq!( size_of_val( &self.0 ), size_of_val( &result[ 0 ] ), "Component size should be the same" );
    // debug_assert_eq!( align_of_val( &self.0 ), align_of_val( &result[ 0 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl ArrayMut< usize, 1 > for Ix1
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 1 ]
  {
    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ Ix ; 1 ] = unsafe { &mut *( ( self as *mut Ix1 ).cast::< [ Ix ; 1 ] >() ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );

    // Return the result
    result
  }
}

// = 2

impl Collection for Ix2
{
  type Scalar = Ix;
}

impl ConstLength for Ix2
{
  const LEN : usize = 2;
}

impl IntoArray< usize, 2 > for Ix2
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 2 ]
  {
    [ self[ 0 ], self[ 1 ] ]
  }
}

impl ArrayRef< usize, 2 > for Ix2
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 2 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &[ Ix ; 2 ] = unsafe { &*( ( self as *const Ix2 ).cast::< [ Ix ; 2 ] >() ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // // Check size and alignment of the first component
    // debug_assert_eq!( size_of_val( &self.0 ), size_of_val( &result[ 0 ] ), "Component size should be the same" );
    // debug_assert_eq!( align_of_val( &self.0 ), align_of_val( &result[ 0 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl ArrayMut< usize, 2 > for Ix2
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 2 ]
  {
    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ Ix ; 2 ] = unsafe { &mut *( ( self as *mut Ix2 ).cast::< [ Ix ; 2 ] >() ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );

    // Return the result
    result
  }
}

// = 3

impl Collection for Ix3
{
  type Scalar = Ix;
}

impl ConstLength for Ix3
{
  const LEN : usize = 3;
}

impl IntoArray< usize, 3 > for Ix3
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 3 ]
  {
    [ self[ 0 ], self[ 1 ], self[ 2 ] ]
  }
}

impl ArrayRef< usize, 3 > for Ix3
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 3 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &[ Ix ; 3 ] = unsafe { &*( ( self as *const Ix3 ).cast::< [ Ix ; 3 ] >() ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // // Check size and alignment of the first component
    // debug_assert_eq!( size_of_val( &self.0 ), size_of_val( &result[ 0 ] ), "Component size should be the same" );
    // debug_assert_eq!( align_of_val( &self.0 ), align_of_val( &result[ 0 ] ), "Component alignment should be the same" );

    // Return the result
    result
  }
}

impl ArrayMut< usize, 3 > for Ix3
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 3 ]
  {
    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ Ix ; 3 ] = unsafe { &mut *( ( self as *mut Ix3 ).cast::< [ Ix ; 3 ] >() ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );

    // Return the result
    result
  }
}

// = 4

impl Collection for Ix4
{
  type Scalar = Ix;
}

impl ConstLength for Ix4
{
  const LEN : usize = 4;
}

impl IntoArray< usize, 4 > for Ix4
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 4 ]
  {
    [ self[ 0 ], self[ 1 ], self[ 2 ], self[ 3 ] ]
  }
}

impl ArrayRef< usize, 4 > for Ix4
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 4 ]
  {
    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &[ Ix ; 4 ] = unsafe { &*( ( self as *const Ix4 ).cast::< [ Ix ; 4 ] >() ) };

    // Check size and alignment of the whole collection
    debug_assert_eq!( size_of_val( self ), size_of_val( result ), "Size should be the same" );
    debug_assert_eq!( align_of_val( self ), align_of_val( result ), "Alignment should be the same" );

    // Return the result
    result
  }
}

impl ArrayMut< usize, 4 > for Ix4
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 4 ]
  {
    // Store layout information in temporary variables
    #[ cfg( debug_assertions ) ]
    let size_self = size_of_val( self );
    #[ cfg( debug_assertions ) ]
    let align_self = align_of_val( self );

    // SAFETY: We are using a raw-pointer cast to convert a reference to a tuple `([ usize; N ])`
    // into a reference to an array `[usize; N]`. This is safe because:
    // 1. The tuple `([usize; N])` and the array `[ usize; N ]` have the same memory layout.
    //    - Both contain N elements of type `E`.
    // 2. We ensure that the size and alignment of the tuple and the array are the same
    //    using `debug_assert_eq!`. This guarantees that they are layout-compatible.
    // 3. The lifetime of the resulting reference is tied to the lifetime of `self`,
    //    ensuring that the reference does not outlive the data it points to.
    #[ allow( unsafe_code ) ]
    let result : &mut [ Ix ; 4 ] = unsafe { &mut *( ( self as *mut Ix4 ).cast::< [ Ix ; 4 ] >() ) };

    // Perform checks under debug conditions
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( size_self, size_of_val( result ), "Size should be the same" );
    #[ cfg( debug_assertions ) ]
    debug_assert_eq!( align_self, align_of_val( result ), "Alignment should be the same" );

    // Return the result
    result
  }
}
