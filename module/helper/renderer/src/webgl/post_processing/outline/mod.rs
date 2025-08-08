mod private
{

}

crate::mod_interface!
{
  /// A module containing shaders and logic for normal-depth based outlines.
  layer normal_depth_outline;
  /// A module for rendering thin, or "narrow," outlines.
  layer narrow_outline;
  /// A module for rendering thick, or "wide," outlines.
  layer wide_outline;
}