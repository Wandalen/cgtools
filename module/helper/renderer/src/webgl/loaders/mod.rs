mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// glTF material extensions read from raw extension JSON
  layer gltf_extensions;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;

  /// KTX2 / Basis Universal texture loader, for the `KHR_texture_basisu` glTF extension.
  #[ cfg( feature = "ktx2" ) ]
  layer ktx2;
}