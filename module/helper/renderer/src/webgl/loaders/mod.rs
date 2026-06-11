mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}