//! Matrix-related functions and trait implementations for 2D ndarray types, including formatting and transposition.

mod private
{
  use crate::*;
  use std::fmt;

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > fmt::Debug
  for Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl,
    E : fmt::Debug,
    Self : IndexingRef< Scalar = E >
  {
    fn fmt( &self, f : &mut fmt::Formatter<'_> ) -> fmt::Result
    {
      let _raw_slice = self.raw_slice();
      write!
      (
        f,
        "Mat {{ order : {} | Coordinate : {} }}\n",
        Descriptor::order_str(),
        Descriptor::coords_str()
      )?;


      for row in 0..ROWS
      {
        write!( f, "  [ " )?;
        for ( i, col ) in self.lane_iter( 0, row ).enumerate()
        {
          if i > 0
          {
            write!( f, ", " )?;
          }
          write!( f, "{:?}", col )?;
        }
        write!( f, " ],\n" )?;
      }
      Ok(())
    }
  }

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl,
  {

    /// Transpose, rarranging data in memory, not just shape descriptor.
    #[ inline( always ) ]
    pub fn transpose( &self ) -> Mat< COLS, ROWS, E, Descriptor >
    where
      E : nd::NdFloat + Default + Copy,
      Self : IndexingRef< Scalar = E >,
      Mat< COLS, ROWS, E, Descriptor > : IndexingMut< Scalar = E >,
    {
      let mut result : Mat< COLS, ROWS, E, Descriptor > = Default::default();
      for ( r, s ) in result.iter_lsfirst_mut().zip( self.iter_msfirst() )
      {
        *r = *s;
      }
      result
    }

  }

}

crate::mod_interface!
{

  exposed use
  {
    // Mat,
  };

}
