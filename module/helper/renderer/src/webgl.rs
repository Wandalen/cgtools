mod private
{

}

crate::mod_interface!
{
  own use ::uuid;

  /// Filters and functonality for post-processing
  layer post_processing;

  /// Mesh related functionality
  layer mesh;

  /// Material related functionality
  layer material;

  /// Scene related functionality
  layer scene;

  /// Animation related functionality
  #[ cfg( feature = "animation" ) ]
  layer animation;

  /// Skeleton related functionality
  layer skeleton;

  /// Texture related functionality
  layer texture;

  /// Sampler related functionlity
  layer sampler;

  /// Geometry related functionality
  layer geometry;

  /// Primitive related functionality
  layer primitive;

  /// Node related functionality
  layer node;

  /// Renderer related functionality
  layer renderer;

  /// Camera related functionality
  layer camera;

  /// Program related functionality
  layer program;

  /// Image based lightning related functionality
  layer ibl;

  /// File loaders
  layer loaders;
}

