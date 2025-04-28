//!
//! # Emboidery errors
//! 

mod private
{
  use thiserror::Error;
  use std::{ borrow::Cow, io };

  /// Represents errors that can be possibly encountered while decoding or encoding emroidery file
  #[ derive( Debug, Error ) ]
  pub enum EmbroideryError
  {
    /// Error occured during IO operations
    #[ error( "IO error occured: `{0}`" ) ]
    IOError( #[ from ] io::Error ),
    /// Error occured if embroidery file's content is incompatible with targeting format
    #[ error( "Compatibility error: `{0}`" ) ]
    CompatibilityError( Cow< 'static, str > ),
    /// Error occured while decoding a file
    #[ error( "Decoding error: `{0}`" ) ]
    DecodingError( Cow< 'static, str > ),
    /// Error for unsupported file formats
    #[ error( "Unsupported format error: `{0}`" ) ]
    UnsupportedFormatError( Cow< 'static, str > ),
  }
}

crate::mod_interface!
{
  own use EmbroideryError;
}
