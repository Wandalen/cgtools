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

  /// Kulla–Conty multi-scatter energy-compensation table generation
  layer kulla_conty;

  /// USD ( `.usda` / `.usdc` / `.usdz` ) scene ingestion on the pure-Rust
  /// `openusd` stack ( OpenPBR adoption plan §2.3 N3 )
  #[ cfg( feature = "native-formats" ) ]
  layer usd;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}