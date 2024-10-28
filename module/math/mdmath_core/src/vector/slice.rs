use super::*;

impl< E > Collection for [ E ]
{
  type Scalar = E;
}

impl< E, const N : usize > VectorRef< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn vector_ref( &self ) -> &[ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {} element", N );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ allow( unsafe_code ) ]
    unsafe { &*( self.as_ptr() as *const [ E ; N ] ) }
  }
}

impl< E, const N : usize > VectorMut< E, N > for [ E ]
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
