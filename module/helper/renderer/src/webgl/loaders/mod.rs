mod private
{

}

/// Numbers read from the extension JSON the `gltf` crate leaves untyped
pub( crate ) mod gltf_json;

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

  /// Fuzz ( sheen ) directional-albedo table generation, for the energy
  /// ( albedo ) scaling of the substrate underneath the fuzz layer
  layer sheen_albedo;

  /// USD ( `.usda` / `.usdc` / `.usdz` ) scene ingestion on the pure-Rust
  /// `openusd` stack ( OpenPBR adoption plan §2.3 N3 )
  #[ cfg( feature = "native-formats" ) ]
  layer usd;

  /// meshopt-compressed buffer views ( `EXT_meshopt_compression`, `KHR_meshopt_compression` )
  layer meshopt;

  /// glTF buffer bytes : GLB chunk, fetched uris, meshopt fallbacks, and whether the
  /// CPU-side readers need a copy of them in wasm memory
  layer gltf_buffers;

  /// glTF buffer and image uris resolved against the model's folder
  layer gltf_uri;

  /// KHR_mesh_quantization : integer accessor values and bounds to floats
  layer gltf_quantization;

  /// glTF extension support : which extensions an asset may require ( and so whether
  /// meshopt-compressed and quantized assets are admitted at all ), document validation, the
  /// texture-transform check on quantized texture coordinates, and the material extensions
  /// read from raw extension JSON
  layer gltf_extensions;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}