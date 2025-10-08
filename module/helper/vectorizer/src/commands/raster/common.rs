//!
//! Collection of raster tools.
//!

mod private
{

  /// Parses a string of HEX represented colors
  ///
  /// # Errors
  /// 
  /// Returns `Err(String)` if the input string cannot be parsed as a valid hexadecimal color value.
  pub fn color_parser( s : &str ) -> Result< [ u8; 3 ], String >
  {
    let color = u32::from_str_radix( s.trim(), 16 )
    .map_err( | e | format!( "Could not parse color: {}", e ) )?;

    let red = ( color >> 16 ) as u8;
    let green = ( color >> 8 ) as u8;
    let blue = color as u8;
    Ok( [ red, green, blue ] )
  }
  
}

crate::mod_interface!
{
  own use
  {
    color_parser
  };
}