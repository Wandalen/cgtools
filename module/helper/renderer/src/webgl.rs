mod private
{

}

crate::mod_interface!
{
  own use ::uuid;

  layer post_processing;

  layer mesh;

  layer material;

  layer scene;

  layer texture;

  layer sampler;

  layer geometry;

  layer primitive;

  layer node;

  layer renderer;

  layer camera;

  layer program;

  layer ibl;

  layer loaders;
}

