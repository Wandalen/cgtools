
mod private 
{
  use serde_with::DisplayFromStr;
  use serde;
  use error_tools::typed::Error;
  use derive_tools::AsRefStr;

  /// Collective enum for errors in Raster actions.
  ///
  /// This enum represents various errors that can occur during raster actions,
  /// including API errors, image processing errors, input/output errors, and keying errors.
  ///
  /// # Variants
  /// * `ApiError` - Represents an error returned by the underlying implementation crate's API.
  /// * `ImageError` - Represents an error returned by the `image` crate during image processing.
  /// * `IOError` - Represents a general input/output error.
  /// * `KeyColorError` - Indicates that no unused color could be found in the image for keying.
  #[ serde_with::serde_as ]
  #[ derive( Debug, Error, AsRefStr, serde::Serialize ) ]
  #[ serde( tag = "type", content = "data" ) ]
  pub enum Error
  {
    /// API error from the underlying implementation crate.
    /// Vtrace core doesn't have custom error type
    #[ error( "Raster API returned an error:\n{0}" ) ]
    ApiError
    (
      #[ serde_as( as = "DisplayFromStr" ) ]
      String
    ),
    /// Error occured while IO operations
    #[ error( "Io operation return an error:\n{0}" ) ]
    IOError
    (
      #[ serde_as( as = "DisplayFromStr" ) ]
      std::io::Error
    ),
    /// Error on no colors available for keying
    #[ error( "Unable to find unused color in the image to use as key" ) ]
    KeyColorError
  }

  /// Shorthand for `Result` in Raster actions.
  pub type Result< T > = core::result::Result< T, Error >;
}

crate::mod_interface!
{
  orphan use
  {
    Error,
    Result
  };
}