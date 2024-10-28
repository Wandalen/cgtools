mod private
{
  use crate::*;
  use std::fmt;

  impl< E, const ROWS : usize, const COLS : usize, Descriptor : mat::Descriptor > fmt::Debug
  for Mat< ROWS, COLS, E, Descriptor >
  where
    E : MatEl,
    E : fmt::Debug,
  {
    fn fmt( &self, f : &mut fmt::Formatter<'_> ) -> fmt::Result
    {
      let raw_slice = self.raw_slice();
      write!
      (
        f,
        "Mat {{ order : {} }}\n",
        Descriptor::order_str(),
      )?;
      for row in 0..ROWS
      {
        write!( f, "  [ " )?;
        for col in 0..COLS
        {
          if col > 0
          {
            write!( f, ", " )?;
          }
          write!( f, "{:?}", raw_slice[ row * COLS + col ] )?;
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
