//!
//! Collection of raster tools.
//!

mod private
{

  /// Parses a string of HEX represented colors
  pub fn color_parser( s : &str ) -> Result< [ u8; 3 ], String >
  {
    let color = u32::from_str_radix( s.trim(), 16 )
    .map_err( | e | format!( "Could not parse color: {}", e ) )?;

    let r = ( color >> 16 ) as u8;
    let g = ( color >> 8 ) as u8;
    let b = ( color >> 0 ) as u8;
    Ok( [ r, g, b ] )
  }
  
}

crate::mod_interface!
{
  own use
  {
    color_parser
  };
}