use super::*;

impl< E > Collection for [ E ]
{
  type Scalar = E;
}

// Converted implementation using unwrap_or_else with panic! to avoid the Debug requirement
impl< E, const N : usize > IntoArray< E, N > for &[ E ]
where
  [ E ; N ] : for< 'a > TryFrom< &'a [ E ] >
{
  #[ inline ]
  fn into_array( self ) -> [ E ; N ]
  {
    self.try_into().unwrap_or_else
    (
      | _ | panic!( "Slice length does not match array length : {} != {}", self.len(), N )
    )
  }
}

impl< E, const N : usize > ArrayRef< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {} element", N );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ allow( unsafe_code ) ]
    unsafe { &*( self.as_ptr() as *const [ E ; N ] ) }
  }
}

impl< E, const N : usize > ArrayMut< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {} element", N );
    // SAFETY: This is safe if the slice has at least N element.
    #[ allow( unsafe_code ) ]
    unsafe { &mut *( self.as_ptr() as *mut [ E ; N ] ) }
  }
}

impl< E, const N : usize > VectorIter< E, N > for [ E ]
{
  fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
  where
    E : 'a,
  {
    assert!( self.len() >= N, "Slice must have at least {} elements", N );
    <[ E ]>::iter( self ).take( N )
  }
}

impl< E, const N : usize > VectorIterMut< E, N > for [ E ]
{
  fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
  where
    E : 'a,
  {
    assert!( self.len() >= N, "Slice must have at least {} elements", N );
    <[ E ]>::iter_mut( self )
  }
}
