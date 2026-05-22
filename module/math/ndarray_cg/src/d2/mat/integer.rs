//! Integer-only arithmetic helpers for `Mat`.
//!
//! Methods in this layer provide component-wise `saturating_*`, `wrapping_*`,
//! and `checked_*` arithmetic for matrices whose element type is one of the
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

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + Saturating,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise saturating addition.
    #[ inline ]
    pub fn saturating_add( self, rhs : Self ) -> Self
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = ( *o ).saturating_add( rs[ i ] );
      }
      out
    }

    /// Component-wise saturating subtraction.
    #[ inline ]
    pub fn saturating_sub( self, rhs : Self ) -> Self
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = ( *o ).saturating_sub( rs[ i ] );
      }
      out
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + WrappingAdd,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise wrapping addition.
    #[ inline ]
    pub fn wrapping_add( self, rhs : Self ) -> Self
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.wrapping_add( &rs[ i ] );
      }
      out
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + WrappingSub,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise wrapping subtraction.
    #[ inline ]
    pub fn wrapping_sub( self, rhs : Self ) -> Self
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.wrapping_sub( &rs[ i ] );
      }
      out
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + WrappingMul,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise wrapping multiplication.
    #[ inline ]
    pub fn wrapping_mul( self, rhs : Self ) -> Self
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.wrapping_mul( &rs[ i ] );
      }
      out
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + CheckedAdd,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise checked addition. Returns `None` if any component
    /// would overflow.
    #[ inline ]
    pub fn checked_add( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.checked_add( &rs[ i ] )?;
      }
      Some( out )
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + CheckedSub,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise checked subtraction. Returns `None` if any component
    /// would overflow.
    #[ inline ]
    pub fn checked_sub( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.checked_sub( &rs[ i ] )?;
      }
      Some( out )
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl + CheckedMul,
    Self : RawSlice< Scalar = E > + RawSliceMut< Scalar = E >,
  {
    /// Component-wise checked multiplication. Returns `None` if any
    /// component would overflow.
    #[ inline ]
    pub fn checked_mul( self, rhs : Self ) -> Option< Self >
    {
      let mut out = self;
      let rs = rhs.raw_slice();
      for ( i, o ) in out.raw_slice_mut().iter_mut().enumerate()
      {
        *o = o.checked_mul( &rs[ i ] )?;
      }
      Some( out )
    }
  }
}

crate::mod_interface!
{
}
