mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// OpenPBR Surface loader from MaterialX (`.mtlx`) documents
  layer openpbr_mtlx;

  /// OpenPBR `.usda` → `.mtlx` asset-reference resolver
  layer openpbr_usda;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}