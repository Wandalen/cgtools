/// Internal namespace.
mod private
{
  use crate::*;
  use std::ops::{ Rem, Neg };

  // qqq : xxx : cover by test each operator

  impl< E, const LEN : usize > Neg for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat,
  {
    type Output = Self;

    #[ inline ]
    fn neg( self ) -> Self::Output
    {
      Self( self.0.map( | v | -v ) )
    }
  }

  // xxx : qqq : enable
  //   impl< E, const LEN: usize > Neg for &Vector<E, LEN>
  //   where
  //     E: MatEl + nd::NdFloat,
  //   {
  //     type Output = Vector<E, LEN>;
  //
  //     #[inline]
  //     fn neg(self) -> Self::Output {
  //       Self::Output(self.0.map(|v| -v))
  //     }
  //   }

  #[ inline ]
  fn rem_vector< E, const LEN : usize >( a : &Vector< E, LEN >, b : &Vector< E, LEN > ) -> Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    let mut result = *a;
    for ( r, ( x, y ) ) in result.0.iter_mut().zip( a.0.iter().zip( b.0.iter() ) )
    {
      *r = *x % *y;
    }
    result
  }

  #[ inline ]
  fn rem_scalar< E, const LEN : usize >( a : &Vector< E, LEN >, scalar : E ) -> Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    let mut result = *a;
    for r in result.0.iter_mut()
    {
      *r = *r % scalar;
    }
    result
  }

  // Vector % Vector
  impl< E, const LEN : usize > Rem for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    type Output = Self;

    #[ inline ]
    fn rem( self, rhs : Self ) -> Self::Output
    {
      rem_vector( &self, &rhs )
    }
  }

  impl< E, const LEN : usize > Rem for &Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    type Output = Vector< E, LEN >;

    #[ inline ]
    fn rem( self, rhs : Self ) -> Self::Output
    {
      rem_vector( self, rhs )
    }
  }

  // Vector % scalar
  impl< E, const LEN : usize > Rem< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    type Output = Self;

    #[ inline ]
    fn rem( self, scalar : E ) -> Self::Output
    {
      rem_scalar( &self, scalar )
    }
  }

  impl< E, const LEN : usize > Rem< E > for &Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    type Output = Vector< E, LEN >;

    #[ inline ]
    fn rem( self, scalar : E ) -> Self::Output
    {
      rem_scalar( self, scalar )
    }
  }

  // RemAssign for Vector % Vector
  impl< E, const LEN : usize > std::ops::RemAssign for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    #[ inline ]
    fn rem_assign( &mut self, rhs : Self )
    {
      *self = *self % rhs;
    }
  }

  // RemAssign for Vector % scalar
  impl< E, const LEN : usize > std::ops::RemAssign< E > for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + Rem< Output = E >,
  {
    #[ inline ]
    fn rem_assign( &mut self, scalar : E )
    {
      *self = *self % scalar;
    }
  }

  impl< E, const LEN : usize > std::ops::Div for Vector< E, LEN >
  where
    E : MatEl + nd::NdFloat + DivAssign,
  {
    type Output = Self;
  
    fn div( mut self, rhs : Self ) -> Self::Output
    {
      self.iter_mut().zip( rhs.iter() ).for_each
      ( 
        | ( lhs, rhs ) |
        {
          *lhs /= *rhs;
        }
      );
      self
    }
  }

  // --- New implementations for Index, Deref and IntoIterator ---

  use std::ops::{ Index, IndexMut, Deref, DerefMut };

  impl< E, const N : usize > Index< usize > for Vector< E, N >
  where
    E : MatEl,
  {
    type Output = E;
    #[ inline ]
    fn index( &self, index : usize ) -> &Self::Output
    {
      &self.0[ index ]
    }
  }

  impl< E, const N : usize > IndexMut< usize > for Vector< E, N >
  where
    E : MatEl,
  {
    #[ inline ]
    fn index_mut( &mut self, index : usize ) -> &mut Self::Output
    {
      &mut self.0[ index ]
    }
  }

  impl< E, const N : usize > Deref for Vector< E, N >
  where
    E : MatEl,
  {
    type Target = [ E; N ];
    #[ inline ]
    fn deref( &self ) -> &Self::Target
    {
      &self.0
    }
  }

  impl< E, const N : usize > DerefMut for Vector< E, N >
  where
    E : MatEl,
  {
    #[ inline ]
    fn deref_mut( &mut self ) -> &mut Self::Target
    {
      &mut self.0
    }
  }

  impl< E, const N : usize > IntoIterator for Vector< E, N >
  where
    E : MatEl,
  {
    type Item = E;
    type IntoIter = std::array::IntoIter< E, N >;
    #[ inline ]
    fn into_iter( self ) -> Self::IntoIter
    {
      self.0.into_iter()
    }
  }

  impl< 'a, E, const N : usize > IntoIterator for &'a Vector< E, N >
  where
    E : MatEl,
  {
    type Item = &'a E;
    type IntoIter = std::slice::Iter< 'a, E >;
    #[ inline ]
    fn into_iter( self ) -> Self::IntoIter
    {
      self.0.iter()
    }
  }

  impl< 'a, E, const N : usize > IntoIterator for &'a mut Vector< E, N >
  where
    E : MatEl,
  {
    type Item = &'a mut E;
    type IntoIter = std::slice::IterMut< 'a, E >;
    #[ inline ]
    fn into_iter( self ) -> Self::IntoIter
    {
      self.0.iter_mut()
    }
  }

}

crate::mod_interface!
{
  // own use ::mdmath_core::vector::arithmetics;

  /// Mul trait implementations
  layer mul;
  /// Sub trait implementations
  layer sub;
  /// Add trait implementations
  layer add;
  /// Div trait implementations
  layer div;

}