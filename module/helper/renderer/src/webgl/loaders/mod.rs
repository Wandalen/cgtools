mod private
{

}

/// Numbers read from the extension JSON the `gltf` crate leaves untyped
pub( crate ) mod gltf_json;

crate::mod_interface!
{
  /// Gltf loader
  layer gltf;

  /// meshopt-compressed buffer views ( `EXT_meshopt_compression`, `KHR_meshopt_compression` ).
  /// Named for the codec rather than `gltf_meshopt` like its siblings : downstream code and
  /// docs already use the `loaders::meshopt` path, and the vendored `meshopt_decoder.js` keeps
  /// meshoptimizer's own name, which is also the snippet path deployments serve
  /// ( `docs/pitfall/004` ).
  layer meshopt;

  /// glTF buffer bytes : GLB chunk, fetched uris, meshopt fallbacks, and whether the
  /// CPU-side readers need a copy of them in wasm memory
  layer gltf_buffers;

  /// glTF buffer and image uris resolved against the model's folder
  layer gltf_uri;

  /// glTF skins and morph targets read on the CPU into a mesh's skeleton
  layer gltf_skeleton;

  /// glTF `KHR_lights_punctual` lights
  layer gltf_lights;

  /// glTF nodes, skeleton attachment and scenes : the loaded scene graph
  layer gltf_scene;

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