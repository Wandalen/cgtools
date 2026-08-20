mod private
{

}

crate::mod_interface!
{
  /// Device, queue and canvas presentation context.
  layer context;

  /// Mesh attribute and index buffers.
  layer geometry;

  /// PBR material description and its GPU binding.
  layer material;

  /// Light list packing for the lights uniform.
  layer light;

  /// Opaque PBR renderer with tone mapping.
  layer renderer;
}
