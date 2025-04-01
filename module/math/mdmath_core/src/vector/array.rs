use super::*;

impl< E, const N : usize > Collection for [ E ; N ]
{
  type Scalar = E;
}

impl< E, const N : usize > ConstLength for [ E ; N ]
{
  const LEN : usize = N;
}

impl< E, const N : usize > IntoArray< E, N > for [ E ; N ]
{
  #[ inline ]
  fn into_array( self ) -> [ E ; N ]
  {
    self
  }
}

impl< E, const N : usize > ArrayRef< E, N > for [ E ; N ]
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; N ]
  {
    self
  }
}

impl< E, const N : usize > ArrayMut< E, N > for [ E ; N ]
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; N ]
  {
    self
  }
}

impl< E, const N : usize > VectorIter< E, N > for [ E ; N ]
{
  fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
  where
    E : 'a,
  {
    <[ E ]>::iter( self )
  }
}

impl< E, const N : usize > VectorIterMut< E, N > for [ E ; N ]
{
  fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
  where
    E : 'a,
  {
    <[ E ]>::iter_mut( self )
  }
}
