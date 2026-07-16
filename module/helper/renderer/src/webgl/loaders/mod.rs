mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// KHR_draco_mesh_compression geometry decode for the glTF loader
  #[ cfg( feature = "draco" ) ]
  layer draco;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}