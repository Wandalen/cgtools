//! Integer-only arithmetic helpers for `Vector`.
//!
//! Methods in this layer provide component-wise `saturating_*`, `wrapping_*`,
//! and `checked_*` arithmetic for vectors whose element type is one of the
//! standard integer primitives.

mod private
{
  use crate::*;
  use ::num_traits::
  {
    Saturating,
    WrappingAdd, WrappingSub, WrappingMul,
    CheckedAdd, CheckedSub, CheckedMul,
  };

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + Saturating,
  {

    /// Component-wise saturating addition. Each component saturates at the
    /// numeric bounds of `E` instead of overflowing.
    #[ inline ]
    pub fn saturating_add( self, rhs : Self ) -> Self
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = ( *o ).saturating_add( *r );
      }
      out
    }

    /// Component-wise saturating subtraction.
    #[ inline ]
    pub fn saturating_sub( self, rhs : Self ) -> Self
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = ( *o ).saturating_sub( *r );
      }
      out
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + WrappingAdd,
  {
    /// Component-wise wrapping addition.
    #[ inline ]
    pub fn wrapping_add( self, rhs : Self ) -> Self
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.wrapping_add( r );
      }
      out
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + WrappingSub,
  {
    /// Component-wise wrapping subtraction.
    #[ inline ]
    pub fn wrapping_sub( self, rhs : Self ) -> Self
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.wrapping_sub( r );
      }
      out
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + WrappingMul,
  {
    /// Component-wise wrapping multiplication.
    #[ inline ]
    pub fn wrapping_mul( self, rhs : Self ) -> Self
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.wrapping_mul( r );
      }
      out
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + CheckedAdd,
  {
    /// Component-wise checked addition. Returns `None` if any component
    /// would overflow.
    #[ inline ]
    pub fn checked_add( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.checked_add( r )?;
      }
      Some( out )
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + CheckedSub,
  {
    /// Component-wise checked subtraction. Returns `None` if any component
    /// would overflow.
    #[ inline ]
    pub fn checked_sub( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.checked_sub( r )?;
      }
      Some( out )
    }
  }

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl + CheckedMul,
  {
    /// Component-wise checked multiplication. Returns `None` if any
    /// component would overflow.
    #[ inline ]
    pub fn checked_mul( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      for ( o, r ) in out.0.iter_mut().zip( rhs.0.iter() )
      {
        *o = o.checked_mul( r )?;
      }
      Some( out )
    }
  }
}

crate::mod_interface!
{
}
