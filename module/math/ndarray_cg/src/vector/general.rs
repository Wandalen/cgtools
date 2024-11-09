mod private
{
  use crate::*;

  impl< E, const N : usize > Collection for Vector< E, N >
  where 
    E : MatEl
  {
    type Scalar = E;
  }

  impl< E, const N : usize > ConstLength for Vector< E, N >
  where 
    E : MatEl
  {
    const LEN : usize = N;
  }

  impl< E, const N : usize > VectorRef< E, N > for Vector< E, N >
  where 
    E : MatEl
  {
    #[ inline( always ) ]
    fn vector_ref( &self ) -> &[ E ; N ]
    {
      &self.0
    }
  }

  impl< E, const N : usize > VectorMut< E, N > for Vector< E, N >
  where 
    E : MatEl
  {
    #[ inline( always ) ]
    fn vector_mut( &mut self ) -> &mut [ E ; N ]
    {
      &mut self.0
    }
  }

  impl< E, const N : usize > VectorIter< E, N > for Vector< E, N >
  where 
    E : MatEl
  {
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where
      E : 'a,
    {
      <[ E ]>::iter( &self.0 )
    }
  }

  impl< E, const N : usize > VectorIterMut< E, N > for Vector< E, N >
  where 
    E : MatEl
  {
    fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
    where
      E : 'a,
    {
      <[ E ]>::iter_mut( &mut self.0 )
    }
  }
}

crate::mod_interface!
{
 
}
