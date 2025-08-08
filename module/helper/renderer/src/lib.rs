//! Graphics PBR renderer

mod private
{


}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Webgl implementation of the renderer
  //#[ cfg( feature = "webgl" ) ]
  layer webgl;
}
