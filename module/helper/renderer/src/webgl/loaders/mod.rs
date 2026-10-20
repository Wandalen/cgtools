mod private
{

}

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// meshopt-compressed buffer views ( `EXT_meshopt_compression`, `KHR_meshopt_compression` )
  layer meshopt;

  /// glTF buffer bytes : GLB chunk, fetched uris, meshopt fallbacks
  layer gltf_buffers;

  /// glTF buffer and image uris resolved against the model's folder
  layer gltf_uri;

  /// KHR_mesh_quantization : integer accessor values and bounds to floats
  layer gltf_quantization;

  /// glTF material extensions read from raw extension JSON
  layer gltf_extensions;

  /// IBL textures loader
  layer ibl;

  /// HDR textures loader
  layer hdr_texture;

  /// PMREM IBL generator from equirectangular HDR
  layer pmrem;
}