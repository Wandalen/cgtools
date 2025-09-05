mod private
{

}

crate::mod_interface!
{
  /// Layer for solid only line functionalities.
  #[ cfg( feature = "solid" ) ]
  layer solid;
  
  /// Layer for line that supports uv coordinates
  #[ cfg( feature = "uv" ) ]
  layer uv;

  /// Layer for line that can be used in blending
  #[ cfg( feature = "transparent" ) ]
  layer transparent;
}
