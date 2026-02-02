mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Animation loading
  layer loaders;

  /// Animation and other base strutures
  layer base;

  /// Tools for capturing current skeleton transformation
  layer pose;

  /// Tools for animation blending
  layer blending;

  /// Tools for scaling animations amplitude
  layer scaling;

  /// Tools for making smooth transition between animations
  layer transition;

  /// Tools for making animations state graph
  layer graph;

  /// Tools for mirroring animations
  layer mirror;
}


