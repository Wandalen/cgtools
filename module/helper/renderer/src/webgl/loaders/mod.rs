mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// meshopt-compressed buffer views ( EXT_meshopt_compression )
  layer meshopt;

  /// glTF material extensions read from raw extension JSON
  layer gltf_extensions;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}