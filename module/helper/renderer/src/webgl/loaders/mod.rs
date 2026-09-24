mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// meshopt-compressed buffer views ( EXT_meshopt_compression )
  layer meshopt;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}