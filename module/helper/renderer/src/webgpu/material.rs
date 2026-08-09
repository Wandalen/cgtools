mod private
{
  use gpu_hal::{ Buffer, BindGroup, TextureView };

  /// Material flag bit: sample the base color texture.
  ///
  /// Mirrored by `FLAG_USE_BASE_COLOR_TEXTURE` in `shaders/main.wgsl`.
  pub const FLAG_USE_BASE_COLOR_TEXTURE : u32 = 1;
  /// Material flag bit: sample the metallic-roughness texture.
  pub const FLAG_USE_MR_TEXTURE : u32 = 2;
  /// Material flag bit: discard fragments below the alpha cutoff.
  pub const FLAG_USE_ALPHA_CUTOFF : u32 = 4;

  /// CPU-side description of an opaque PBR ( metallic-roughness ) material.
  #[ derive( Debug, Clone ) ]
  pub struct PbrMaterial
  {
    /// Base color multiplier ( linear rgba ).
    pub base_color_factor : [ f32; 4 ],
    /// Metalness multiplier.
    pub metallic_factor : f32,
    /// Roughness multiplier.
    pub roughness_factor : f32,
    /// Alpha-cutoff threshold; `None` disables cutoff discard.
    pub alpha_cutoff : Option< f32 >,
    /// Base color texture ( sRGB-encoded content ), if any.
    pub base_color_texture : Option< TextureView >,
    /// Metallic-roughness texture ( G — roughness, B — metalness ), if any.
    pub metallic_roughness_texture : Option< TextureView >
  }

  impl Default for PbrMaterial
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl PbrMaterial
  {
    /// A material with the glTF defaults: white base color, fully metallic,
    /// fully rough, no textures, no cutoff.
    pub fn new() -> Self
    {
      Self
      {
        base_color_factor : [ 1.0, 1.0, 1.0, 1.0 ],
        metallic_factor : 1.0,
        roughness_factor : 1.0,
        alpha_cutoff : None,
        base_color_texture : None,
        metallic_roughness_texture : None
      }
    }

    /// Packs the factors and derived flag bits into the uniform layout.
    pub fn as_raw( &self ) -> MaterialRaw
    {
      let mut flags = 0;
      if self.base_color_texture.is_some() { flags |= FLAG_USE_BASE_COLOR_TEXTURE; }
      if self.metallic_roughness_texture.is_some() { flags |= FLAG_USE_MR_TEXTURE; }
      if self.alpha_cutoff.is_some() { flags |= FLAG_USE_ALPHA_CUTOFF; }

      MaterialRaw
      {
        base_color_factor : self.base_color_factor,
        metallic_factor : self.metallic_factor,
        roughness_factor : self.roughness_factor,
        alpha_cutoff : self.alpha_cutoff.unwrap_or( 0.0 ),
        flags
      }
    }
  }

  /// GPU layout of `MaterialUniform` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct MaterialRaw
  {
    /// Base color multiplier ( linear rgba ).
    pub base_color_factor : [ f32; 4 ],
    /// Metalness multiplier.
    pub metallic_factor : f32,
    /// Roughness multiplier.
    pub roughness_factor : f32,
    /// Alpha-cutoff threshold; meaningful only with `FLAG_USE_ALPHA_CUTOFF`.
    pub alpha_cutoff : f32,
    /// `FLAG_*` bits.
    pub flags : u32
  }

  /// GPU-resident binding of one material: its uniform buffer + bind group
  /// for group 1 of the opaque pipeline.
  pub struct MaterialBinding
  {
    /// Uniform buffer holding a `MaterialRaw`.
    pub buffer : Buffer,
    /// Bind group for group 1 of the opaque pipeline.
    pub bind_group : BindGroup
  }
}

crate::mod_interface!
{
  orphan use
  {
    PbrMaterial,
    MaterialRaw,
    MaterialBinding,
    FLAG_USE_BASE_COLOR_TEXTURE,
    FLAG_USE_MR_TEXTURE,
    FLAG_USE_ALPHA_CUTOFF
  };
}
