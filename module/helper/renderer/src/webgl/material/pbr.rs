mod private
{
  use crate::webgl::{ Object3D, material::{Material, TextureInfo, AlphaMode, CullMode} };
  use minwebgl as gl;
  use gl::{ GL, WebGlProgram };
  use mingl::Former;
  use rustc_hash::FxHashMap;
  use crate::webgl::{ MaterialUploadContext, program::{ ShaderProgram, ProgramInfo } };
  use crate::webgl::program::impl_locations;
  use std::cell::Cell;
  use std::fmt::Write as _;

  /// The source code for the main vertex shader.
  const MAIN_VERTEX_SHADER : &str = include_str!( "../shaders/main.vert" );
  /// The source code for the main fragment shader.
  const MAIN_FRAGMENT_SHADER : &str = include_str!( "../shaders/main.frag" );

  /// Max point light sources count
  pub const MAX_POINT_LIGHTS : usize = 8;
  /// Max direct light sources count
  pub const MAX_DIRECT_LIGHTS : usize = 8;
  /// Max spot light sources count
  pub const MAX_SPOT_LIGHTS : usize = 8;

  // Texture units. Every PbrMaterial sampler has a fixed unit; skinning / morph data use the
  // vertex-stage slots in `skeleton.rs` (`GLOBAL_MATRICES_SLOT`..`DISPLACEMENTS_SLOT`), and IBL
  // takes `PBR_IBL_UNIT_COUNT` units starting right after them. `tests/pbr_texture_units_test.rs`
  // checks the whole layout is disjoint and fits WebGL2's guaranteed 16 fragment samplers.
  /// Texture unit of `PbrMaterial`'s `metallicRoughnessTexture` sampler.
  pub const PBR_METALLIC_ROUGHNESS_UNIT : u32 = 0;
  /// Texture unit of `PbrMaterial`'s `baseColorTexture` sampler.
  pub const PBR_BASE_COLOR_UNIT : u32 = 1;
  /// Texture unit of `PbrMaterial`'s `normalTexture` sampler.
  pub const PBR_NORMAL_UNIT : u32 = 2;
  /// Texture unit of `PbrMaterial`'s `occlusionTexture` sampler.
  pub const PBR_OCCLUSION_UNIT : u32 = 3;
  /// Texture unit of `PbrMaterial`'s `emissiveTexture` sampler.
  pub const PBR_EMISSIVE_UNIT : u32 = 4;
  /// Texture unit of `PbrMaterial`'s `specularTexture` sampler.
  pub const PBR_SPECULAR_UNIT : u32 = 5;
  /// Texture unit of `PbrMaterial`'s `specularColorTexture` sampler.
  pub const PBR_SPECULAR_COLOR_UNIT : u32 = 6;
  /// Texture unit of `PbrMaterial`'s `lightMap` sampler.
  pub const PBR_LIGHT_MAP_UNIT : u32 = 7;
  /// Texture unit of `PbrMaterial`'s `clearcoatTexture` sampler.
  pub const PBR_CLEARCOAT_UNIT : u32 = 8;
  /// Texture unit of `PbrMaterial`'s `clearcoatRoughnessTexture` sampler.
  pub const PBR_CLEARCOAT_ROUGHNESS_UNIT : u32 = 9;
  /// Texture unit of `PbrMaterial`'s `clearcoatNormalTexture` sampler.
  pub const PBR_CLEARCOAT_NORMAL_UNIT : u32 = 10;
  /// Texture unit of `PbrMaterial`'s `anisotropyTexture` sampler.
  pub const PBR_ANISOTROPY_UNIT : u32 = 11;
  /// Every `PbrMaterial` sampler uniform with its texture unit, in unit order.
  pub const PBR_TEXTURE_UNITS : [ ( &str, u32 ); 12 ] =
  [
    ( "metallicRoughnessTexture", PBR_METALLIC_ROUGHNESS_UNIT ),
    ( "baseColorTexture", PBR_BASE_COLOR_UNIT ),
    ( "normalTexture", PBR_NORMAL_UNIT ),
    ( "occlusionTexture", PBR_OCCLUSION_UNIT ),
    ( "emissiveTexture", PBR_EMISSIVE_UNIT ),
    ( "specularTexture", PBR_SPECULAR_UNIT ),
    ( "specularColorTexture", PBR_SPECULAR_COLOR_UNIT ),
    ( "lightMap", PBR_LIGHT_MAP_UNIT ),
    ( "clearcoatTexture", PBR_CLEARCOAT_UNIT ),
    ( "clearcoatRoughnessTexture", PBR_CLEARCOAT_ROUGHNESS_UNIT ),
    ( "clearcoatNormalTexture", PBR_CLEARCOAT_NORMAL_UNIT ),
    ( "anisotropyTexture", PBR_ANISOTROPY_UNIT ),
  ];
  /// First of the IBL units (irradiance, prefiltered environment, BRDF LUT): the unit after the
  /// last skinning / morph slot.
  pub const PBR_IBL_BASE_UNIT : u32 = crate::webgl::DISPLACEMENTS_SLOT + 1;
  /// Number of consecutive units IBL uses from `PBR_IBL_BASE_UNIT`.
  pub const PBR_IBL_UNIT_COUNT : u32 = 3;

  // A Physically Based Rendering (PBR) shader.
  impl_locations!
  (
    PBRShader,
    "cameraPosition",
    "viewMatrix",
    "projectionMatrix",

    // Node uniform locations
    "worldMatrix",
    "normalMatrix",

    // Skeleton uniform locations
    "inverseBindMatricesTexture",
    "globalJointTransformMatricesTexture",
    "skinMatricesTextureSize",
    "primitiveOffset",
    "morphWeights",
    "morphTargetsDisplacementsTexture",
    "displacementsTextureSize",
    "morphTargetsCount",
    "morphTargetsDisplacementsOffsets",

    // Light uniform locations
    "pointLights",
    "pointLightsCount",
    "directLights",
    "directLightsCount",
    "spotLights",
    "spotLightsCount",

    // Material uniform  locations
    //// Textures uniform locations
    "metallicRoughnessTexture",
    "baseColorTexture",
    "normalTexture",
    "occlusionTexture",
    "emissiveTexture",
    "specularTexture",
    "specularColorTexture",
    "lightMap",
    "clearcoatTexture",
    "clearcoatRoughnessTexture",
    "clearcoatNormalTexture",
    "anisotropyTexture",
    //// IBL uniform locations
    "irradianceTexture",
    "prefilterEnvMap",
    "integrateBRDF",
    "u_max_lod",
    "mipmapDistanceRange",
    //// Scalers uniform locations
    "baseColorFactor",
    "metallicFactor",
    "roughnessFactor",
    "normalScale",
    "occlusionStrength",
    "specularFactor",
    "specularColorFactor",
    "emissiveFactor",
    "clearcoatFactor",
    "clearcoatRoughnessFactor",
    "clearcoatNormalScale",
    "anisotropyStrength",
    "anisotropyRotation",
    // Luminosity
    "alphaCutoff",
    "exposure"
  );

  /// Represents the visual properties of a surface.
  #[ derive( Former, Debug ) ]
  pub struct PbrMaterial
  {
    /// A unique identifier for the material.
    pub id : uuid::Uuid,
    /// The base color factor, multiplied with the base color texture. Defaults to white (1, 1, 1, 1).
    pub base_color_factor : gl::F32x4,
    /// Optional texture providing the base color.
    base_color_texture : Option< TextureInfo >,
    /// Scaling factor for the metallic component.
    pub metallic_factor : f32,
    /// Scaling factor for the roughness component.
    pub roughness_factor : f32,
    /// Optional texture providing the metallic and roughness values. Metalness is sampled from the B channel and roughness from the G channel.
    metallic_roughness_texture : Option< TextureInfo >,

    /// Scaling factor applied to each normal vector of the normal texture.
    pub normal_scale : f32,
    /// Optional texture containing normal vectors.
    normal_texture : Option< TextureInfo >,

    /// Scalar multiplier applied to the AO values sampled from the occlusion texture.
    pub occlusion_strength : f32,
    /// Optional texture providing ambient occlusion values.
    occlusion_texture : Option< TextureInfo >,

    /// Optional texture providing the emission color of the material.
    emissive_texture : Option< TextureInfo >,
    /// Scaling factor for the emission intensity
    pub emissive_factor : gl::F32x3,

    /// Optional scaling factor for the specular intensity. (KHR_materials_specular extension)
    specular_factor : Option< f32 >,
    /// Optional texture providing the specular intensity. (KHR_materials_specular extension)
    specular_texture : Option< TextureInfo >,
    /// Optional color factor for the specular highlight. (KHR_materials_specular extension)
    specular_color_factor : Option< gl::F32x3 >,
    /// Optional texture providing the specular color. (KHR_materials_specular extension)
    specular_color_texture : Option< TextureInfo >,
    /// Optional lightmap texture containing pre-baked lighting (shadows)
    light_map : Option< TextureInfo >,

    /// Optional scaling factor for the clearcoat layer intensity. (KHR_materials_clearcoat extension)
    clearcoat_factor : Option< f32 >,
    /// Optional texture providing the clearcoat intensity in the R channel. (KHR_materials_clearcoat extension)
    clearcoat_texture : Option< TextureInfo >,
    /// Optional roughness factor for the clearcoat layer. (KHR_materials_clearcoat extension)
    clearcoat_roughness_factor : Option< f32 >,
    /// Optional texture providing the clearcoat roughness in the G channel. (KHR_materials_clearcoat extension)
    clearcoat_roughness_texture : Option< TextureInfo >,
    /// Scaling factor applied to each normal vector of the clearcoat normal texture. (KHR_materials_clearcoat extension)
    pub clearcoat_normal_scale : f32,
    /// Optional texture containing normal vectors for the clearcoat layer. (KHR_materials_clearcoat extension)
    clearcoat_normal_texture : Option< TextureInfo >,

    /// Optional strength of the anisotropy effect. (KHR_materials_anisotropy extension)
    anisotropy_strength : Option< f32 >,
    /// Rotation of the anisotropy direction, in radians. (KHR_materials_anisotropy extension)
    pub anisotropy_rotation : f32,
    /// Optional texture providing the anisotropy direction (RG) and strength (B). (KHR_materials_anisotropy extension)
    anisotropy_texture : Option< TextureInfo >,
    /// Alpha cutoff value for mask mode. Fragments with alpha below this value are discarded.
    pub alpha_cutoff : f32,
    /// The alpha blending mode for the material. Defaults to `Opaque`.
    alpha_mode : AlphaMode,
    /// Determines wheter to draw both or one side of the primitive
    pub double_sided : bool,
    /// Face culling mode. `None` means culling is disabled.
    pub cull_mode : Option< CullMode >,

    /// Range of distances in which environment map's mipmap switching is applied
    pub mipmap_distance_range : std::ops::Range< f32 >,

    /// Hash map of defines in (value, name) format
    vertex_defines : FxHashMap< Box< str >, String >,
    /// Hash map of defines in (value, name) format
    fragment_defines : FxHashMap< Box< str >, String >,

    /// Returns answer need use IBL for current material instance or not
    need_use_ibl : bool,
    /// Signal for updating material uniforms.
    /// Use `needs_update_set(true)` after changing material properties.
    needs_update : Cell< bool >,
    /// Signal that shader defines have changed and program needs recompilation.
    needs_recompile : Cell< bool >,
    /// Cached combined defines string
    cached_defines_str : String,
    /// Cached vertex defines string
    cached_vertex_defines_str : String,
    /// Cached fragment defines string
    cached_fragment_defines_str : String,
  }

  /// Pushes `#define <name>` for a texture the material samples, plus the macro that maps its
  /// per-texture UV varying (`uv_name`) onto the UV set the texture reads (`vUv_<n>`).
  fn texture_define_push( defines : &mut String, name : &str, uv_name : &str, info : Option< &TextureInfo > )
  {
    let _ = writeln!( defines, "#define {name}" );
    let uv_position = info.unwrap().uv_position;
    let _ = writeln!( defines, "#define {uv_name} vUv_{uv_position}" );
  }

  impl PbrMaterial
  {
    /// Creates new [`PbrMaterial`] with predefined optimal parameters
    #[ must_use ]
    pub fn new( _ : &GL ) -> Self
    {
      let id = uuid::Uuid::new_v4();
      let base_color_factor = gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] );

      let base_color_texture = None;
      let metallic_factor = 1.0;
      let roughness_factor = 1.0;
      let metallic_roughness_texture = None;

      let normal_scale = 1.0;
      let normal_texture = None;

      let occlusion_strength = 1.0;
      let occlusion_texture = None;

      let emissive_texture = None;
      let emissive_factor = gl::F32x3::from( [ 0.0, 0.0, 0.0 ] );

      let specular_factor = None;
      let specular_texture = None;
      let specular_color_factor = None;
      let specular_color_texture = None;

      let light_map = None;

      let clearcoat_factor = None;
      let clearcoat_texture = None;
      let clearcoat_roughness_factor = None;
      let clearcoat_roughness_texture = None;
      let clearcoat_normal_scale = 1.0;
      let clearcoat_normal_texture = None;

      let anisotropy_strength = None;
      let anisotropy_rotation = 0.0;
      let anisotropy_texture = None;

      let alpha_mode = AlphaMode::default();
      let alpha_cutoff = 0.5;
      let double_sided = false;
      let cull_mode = None;

      let mipmap_distance_range = 0.0..200.0;

      let vertex_defines = FxHashMap::default();
      let fragment_defines = FxHashMap::default();

      let need_use_ibl = true;

      let mut mat = Self
      {
        id,
        base_color_factor,
        base_color_texture,
        metallic_factor,
        roughness_factor,
        metallic_roughness_texture,
        normal_scale,
        normal_texture,
        occlusion_strength,
        occlusion_texture,
        emissive_texture,
        emissive_factor,
        specular_factor,
        specular_texture,
        specular_color_factor,
        specular_color_texture,
        alpha_mode,
        alpha_cutoff,
        double_sided,
        cull_mode,
        mipmap_distance_range,
        light_map,
        clearcoat_factor,
        clearcoat_texture,
        clearcoat_roughness_factor,
        clearcoat_roughness_texture,
        clearcoat_normal_scale,
        clearcoat_normal_texture,
        anisotropy_strength,
        anisotropy_rotation,
        anisotropy_texture,
        vertex_defines,
        fragment_defines,
        need_use_ibl,
        needs_update : Cell::new( true ),
        needs_recompile : Cell::new( true ),
        cached_defines_str : String::new(),
        cached_vertex_defines_str : String::new(),
        cached_fragment_defines_str : String::new(),
      };
      mat.defines_cache_rebuild();
      mat
    }

    /// Enables or disables Image-Based Lighting (IBL) for this material.
    /// If the value changes, the shader program will be marked for recompilation.
    pub fn need_use_ibl_set( &mut self, value : bool )
    {
      if value != self.need_use_ibl
      {
        self.needs_recompile.set( true );
      }
      self.need_use_ibl = value;
    }

    /// Returns whether Image-Based Lighting (IBL) is enabled for this material.
    pub fn need_use_ibl( &self ) -> bool
    {
      self.need_use_ibl
    }

    /// Sets the base color texture.
    pub fn base_color_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.base_color_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the base color texture.
    pub fn base_color_texture( &self ) -> Option< &TextureInfo >
    {
      self.base_color_texture.as_ref()
    }

    /// Sets the metallic roughness texture.
    pub fn metallic_roughness_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.metallic_roughness_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the metallic roughness texture.
    pub fn metallic_roughness_texture( &self ) -> Option< &TextureInfo >
    {
      self.metallic_roughness_texture.as_ref()
    }

    /// Sets the normal texture.
    pub fn normal_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.normal_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the normal texture.
    pub fn normal_texture( &self ) -> Option< &TextureInfo >
    {
      self.normal_texture.as_ref()
    }

    /// Sets the occlusion texture.
    pub fn occlusion_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.occlusion_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the occlusion texture.
    pub fn occlusion_texture( &self ) -> Option< &TextureInfo >
    {
      self.occlusion_texture.as_ref()
    }

    /// Sets the emissive texture.
    pub fn emissive_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.emissive_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the emissive texture.
    pub fn emissive_texture( &self ) -> Option< &TextureInfo >
    {
      self.emissive_texture.as_ref()
    }

    /// Sets the specular texture.
    pub fn specular_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.specular_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the specular texture.
    pub fn specular_texture( &self ) -> Option< &TextureInfo >
    {
      self.specular_texture.as_ref()
    }

    /// Sets the specular color texture.
    pub fn specular_color_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.specular_color_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the specular color texture.
    pub fn specular_color_texture( &self ) -> Option< &TextureInfo >
    {
      self.specular_color_texture.as_ref()
    }

    /// Sets the light map texture.
    pub fn light_map_set( &mut self, value : Option< TextureInfo > )
    {
      self.light_map = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the light map texture.
    pub fn light_map( &self ) -> Option< &TextureInfo >
    {
      self.light_map.as_ref()
    }

    /// Sets the clearcoat factor.
    pub fn clearcoat_factor_set( &mut self, value : Option< f32 > )
    {
      self.clearcoat_factor = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the clearcoat factor.
    pub fn clearcoat_factor( &self ) -> Option< f32 >
    {
      self.clearcoat_factor
    }

    /// Sets the clearcoat texture.
    pub fn clearcoat_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the clearcoat texture.
    pub fn clearcoat_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_texture.as_ref()
    }

    /// Sets the clearcoat roughness factor.
    pub fn clearcoat_roughness_factor_set( &mut self, value : Option< f32 > )
    {
      self.clearcoat_roughness_factor = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the clearcoat roughness factor.
    pub fn clearcoat_roughness_factor( &self ) -> Option< f32 >
    {
      self.clearcoat_roughness_factor
    }

    /// Sets the clearcoat roughness texture.
    pub fn clearcoat_roughness_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_roughness_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the clearcoat roughness texture.
    pub fn clearcoat_roughness_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_roughness_texture.as_ref()
    }

    /// Sets the clearcoat normal texture.
    pub fn clearcoat_normal_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.clearcoat_normal_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the clearcoat normal texture.
    pub fn clearcoat_normal_texture( &self ) -> Option< &TextureInfo >
    {
      self.clearcoat_normal_texture.as_ref()
    }

    /// Sets the anisotropy strength.
    pub fn anisotropy_strength_set( &mut self, value : Option< f32 > )
    {
      self.anisotropy_strength = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the anisotropy strength.
    pub fn anisotropy_strength( &self ) -> Option< f32 >
    {
      self.anisotropy_strength
    }

    /// Sets the anisotropy texture.
    pub fn anisotropy_texture_set( &mut self, value : Option< TextureInfo > )
    {
      self.anisotropy_texture = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the anisotropy texture.
    pub fn anisotropy_texture( &self ) -> Option< &TextureInfo >
    {
      self.anisotropy_texture.as_ref()
    }

    /// Sets the alpha mode.
    pub fn alpha_mode_set( &mut self, value : AlphaMode )
    {
      self.alpha_mode = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the alpha mode.
    pub fn alpha_mode( &self ) -> AlphaMode
    {
      self.alpha_mode
    }

    /// Sets the specular factor.
    pub fn specular_factor_set( &mut self, value : Option< f32 > )
    {
      self.specular_factor = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the specular factor.
    pub fn specular_factor( &self ) -> Option< f32 >
    {
      self.specular_factor
    }

    /// Sets the specular color factor.
    pub fn specular_color_factor_set( &mut self, value : Option< gl::F32x3 > )
    {
      self.specular_color_factor = value;
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Returns the specular color factor.
    pub fn specular_color_factor( &self ) -> Option< gl::F32x3 >
    {
      self.specular_color_factor
    }

    /// Rebuilds all cached defines strings from current state.
    fn defines_cache_rebuild( &mut self )
    {
      let local_defines = self.local_defines();

      // Build vertex and fragment defines once.
      // Sort entries to ensure deterministic output regardless of FxHashMap iteration order.
      let mut vertex_defines = local_defines.clone();
      let mut vertex_entries : Vec< _ > = self.vertex_defines.iter().collect();
      vertex_entries.sort_by_key( |( k, _ )| *k );
      for ( name, value ) in vertex_entries
      {
        let _ = writeln!( vertex_defines, "#define {name} {value}" );
      }

      let mut fragment_defines = local_defines;
      let mut fragment_entries : Vec< _ > = self.fragment_defines.iter().collect();
      fragment_entries.sort_by_key( |( k, _ )| *k );
      for ( name, value ) in fragment_entries
      {
        let _ = writeln!( fragment_defines, "#define {name} {value}" );
      }

      // Combined = vertex + fragment defines
      let mut combined = vertex_defines.clone();
      combined.push_str( &fragment_defines );

      self.cached_defines_str = combined;
      self.cached_vertex_defines_str = vertex_defines;
      self.cached_fragment_defines_str = fragment_defines;
    }

    /// Added the specified name and value is #define directive to the material
    pub fn vertex_define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.vertex_defines.insert( name.into(), value.into() );
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Added the specified name and value is #define directive to the material
    pub fn fragment_define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      self.fragment_defines.insert( name.into(), value.into() );
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Added the specified name and value is #define directive to the material
    pub fn define_add< A : Into< Box< str > >, B : Into< String > >( &mut self, name : A, value : B )
    {
      let name = name.into();
      let value = value.into();
      self.vertex_defines.insert( name.clone(), value.clone() );
      self.fragment_defines.insert( name, value );
      self.defines_cache_rebuild();
      self.needs_recompile.set( true );
    }

    /// Generates `#define` directives to be inserted into the fragment shader based on the material's properties.
    fn local_defines( &self ) -> String
    {
      let use_base_color_texture = self.base_color_texture.is_some();
      let use_metallic_roughness_texture = self.metallic_roughness_texture.is_some();

      let use_emissive_texture = self.emissive_texture.is_some();

      let use_specular_texture = self.specular_texture.is_some();
      let use_specular_color_texture = self.specular_color_texture.is_some();

      let use_khr_materials_specular = self.specular_factor.is_some()
      || self.specular_color_factor.is_some()
      || use_specular_texture
      || use_specular_color_texture;

      let use_light_map = self.light_map.is_some();

      let use_normal_texture = self.normal_texture.is_some();
      let use_occlusion_texture = self.occlusion_texture.is_some();
      let use_alpha_cutoff = self.alpha_mode == AlphaMode::Mask;

      let mut defines = String::new();

      defines.push_str( format!( "#define MAX_POINT_LIGHTS {MAX_POINT_LIGHTS}\n" ).as_str() );
      defines.push_str( format!( "#define MAX_DIRECT_LIGHTS {MAX_DIRECT_LIGHTS}\n" ).as_str() );
      defines.push_str( format!( "#define MAX_SPOT_LIGHTS {MAX_SPOT_LIGHTS}\n" ).as_str() );

      // Base color texture related
      if use_base_color_texture
      {
        texture_define_push( &mut defines, "USE_BASE_COLOR_TEXTURE", "vBaseColorUv", self.base_color_texture.as_ref() );
      }

      // Metallic roughness texture related
      if use_metallic_roughness_texture
      {
        texture_define_push( &mut defines, "USE_MR_TEXTURE", "vMRUv", self.metallic_roughness_texture.as_ref() );
      }

      // Emission texture related
      if use_emissive_texture
      {
        texture_define_push( &mut defines, "USE_EMISSION_TEXTURE", "vEmissionUv", self.emissive_texture.as_ref() );
      }

      // KHR_Materials_Specular extension related
      if use_khr_materials_specular
      {
        defines.push_str( "#define USE_KHR_materials_specular\n" );
        if use_specular_texture
        {
          texture_define_push( &mut defines, "USE_SPECULAR_TEXTURE", "vSpecularUv", self.specular_texture.as_ref() );
        }

        if use_specular_color_texture
        {
          texture_define_push( &mut defines, "USE_SPECULAR_COLOR_TEXTURE", "vSpecularColorUv", self.specular_color_texture.as_ref() );
        }
      }

      // Normal texture related
      if use_normal_texture
      {
        texture_define_push( &mut defines, "USE_NORMAL_TEXTURE", "vNormalUv", self.normal_texture.as_ref() );
      }

      // Occlusion texture related
      if use_occlusion_texture
      {
        texture_define_push( &mut defines, "USE_OCCLUSION_TEXTURE", "vOcclusionUv", self.occlusion_texture.as_ref() );
      }

      if use_alpha_cutoff
      {
        defines.push_str( "#define USE_ALPHA_CUTOFF\n" );
      }

      if use_light_map
      {
        texture_define_push( &mut defines, "USE_LIGHT_MAP", "vLightMapUv", self.light_map.as_ref() );
      }

      // KHR_materials_clearcoat / KHR_materials_anisotropy
      let layer_extensions_need_tbn = self.layer_extension_defines_push( &mut defines );

      // Shared tangent/bitangent/normal matrix, needed by normal mapping, clearcoat normal
      // mapping and anisotropy alike.
      let use_tbn = use_normal_texture || layer_extensions_need_tbn;
      if use_tbn
      {
        defines.push_str( "#define USE_TBN\n" );
      }

      defines
    }

    /// Pushes the `KHR_materials_clearcoat` / `KHR_materials_anisotropy` defines onto `defines`
    /// and returns whether either extension needs the shared tangent frame (`USE_TBN`).
    fn layer_extension_defines_push( &self, defines : &mut String ) -> bool
    {
      let use_clearcoat_texture = self.clearcoat_texture.is_some();
      let use_clearcoat_roughness_texture = self.clearcoat_roughness_texture.is_some();
      let use_clearcoat_normal_texture = self.clearcoat_normal_texture.is_some();
      let use_khr_materials_clearcoat = self.clearcoat_factor.is_some()
      || self.clearcoat_roughness_factor.is_some()
      || use_clearcoat_texture
      || use_clearcoat_roughness_texture
      || use_clearcoat_normal_texture;

      let use_anisotropy_texture = self.anisotropy_texture.is_some();
      let use_khr_materials_anisotropy = self.anisotropy_strength.is_some() || use_anisotropy_texture;

      // KHR_materials_clearcoat extension related
      if use_khr_materials_clearcoat
      {
        defines.push_str( "#define USE_KHR_materials_clearcoat\n" );
        if use_clearcoat_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_TEXTURE", "vClearcoatUv", self.clearcoat_texture.as_ref() );
        }

        if use_clearcoat_roughness_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_ROUGHNESS_TEXTURE", "vClearcoatRoughnessUv", self.clearcoat_roughness_texture.as_ref() );
        }

        if use_clearcoat_normal_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_NORMAL_TEXTURE", "vClearcoatNormalUv", self.clearcoat_normal_texture.as_ref() );
        }
      }

      // KHR_materials_anisotropy extension related
      if use_khr_materials_anisotropy
      {
        defines.push_str( "#define USE_KHR_materials_anisotropy\n" );
        if use_anisotropy_texture
        {
          texture_define_push( defines, "USE_ANISOTROPY_TEXTURE", "vAnisotropyUv", self.anisotropy_texture.as_ref() );
        }
      }

      use_clearcoat_normal_texture || use_khr_materials_anisotropy
    }

    /// Returns an immutable reference to the local vertex defines map
    pub fn vertex_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.vertex_defines
    }

    /// Returns an immutable reference to the local fragment defines map
    pub fn fragment_defines( &self ) -> &FxHashMap< Box< str >, String >
    {
      &self.fragment_defines
    }
  }

  impl Material for PbrMaterial
  {
    fn id( &self ) -> uuid::Uuid
    {
      self.id
    }

    fn needs_update( &self ) -> bool
    {
      self.needs_update.get()
    }

    fn needs_update_set( &self, value : bool )
    {
      self.needs_update.set( value );
    }

    fn ibl_base_texture_unit( &self ) -> Option< u32 >
    {
      if self.need_use_ibl
      {
        // See the texture-unit constants at the top of this file.
        Some( PBR_IBL_BASE_UNIT )
      }
      else
      {
        None
      }
    }

    fn shader_program_make( &self, minwebgl : &minwebgl::WebGl2RenderingContext, program : &minwebgl::WebGlProgram ) -> Box< dyn ShaderProgram >
    {
      PBRShader::new( minwebgl, program ).dyn_clone()
    }

    fn configure
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      ctx : &MaterialUploadContext< '_ >
    )
    {
      let locations = ctx.locations;

      // Assign each sampler its texture unit (`PBR_TEXTURE_UNITS`).
      //
      // Panicking with the uniform's name (rather than `.unwrap()`) names the missing key: these
      // lookups are only unreachable while `PBRShader`'s `impl_locations!` list above keeps
      // every one of these literals -- an `.unwrap()` panic on drift would instead read as a
      // bare "called `Option::unwrap()` on a `None` value" with no indication of which uniform
      // name fell out of sync.
      for ( name, unit ) in PBR_TEXTURE_UNITS
      {
        let location = locations.get( name )
        .unwrap_or_else( || panic!( "PBRShader::impl_locations! missing \"{name}\"" ) );
        gl.uniform1i( location.as_ref(), unit as i32 );
      }
    }

    fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      ctx : &MaterialUploadContext< '_ >
    )
    -> Result< (), gl::WebglError >
    {
      if let Some( current_primitive_id ) = ctx.primitive_id
      {
        if let Object3D::Mesh( mesh ) = &ctx.node.object
        {
          let mut primitive_offset : u32 = 0;
          for ( i, primitive ) in mesh.borrow().primitives.iter().enumerate()
          {
            if i >= current_primitive_id { break; }
            primitive_offset += primitive.borrow().geometry.borrow().vertex_count;
          }

          let locations = ctx.locations;

          if let Some( primitive_offset_loc ) = locations.get( "primitiveOffset" )
          {
            gl::uniform::upload( gl, primitive_offset_loc.clone(), &primitive_offset )?;
          }
        }
      }

      Ok( () )
    }

    fn upload_on_state_change
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      ctx : &MaterialUploadContext< '_ >
    )
    -> Result< (), gl::WebglError >
    {
      let locations = ctx.locations;

      // `unwrap_or_else( || panic!( .. ) )` rather than `.unwrap()`: `loc` is only known at the
      // call site below, so the panic message can still name the exact missing uniform instead
      // of a bare "called `Option::unwrap()` on a `None` value".
      // Extension factors are optional on the material but always uploaded, falling back to the
      // extension's default: programs are shared between materials with the same defines, and a
      // material can switch an extension on through a texture alone, so skipping the upload for
      // `None` would draw with whatever factor the previous material left in the program.
      let upload = | loc : &str, value : f32 | -> Result< (), gl::WebglError >
      {
        let location = locations.get( loc )
        .unwrap_or_else( || panic!( "PBRShader::impl_locations! missing \"{loc}\"" ) )
        .clone();
        gl::uniform::upload( gl, location, &value )
      };

      let upload_array = | loc : &str, value : &[ f32 ] | -> Result< (), gl::WebglError >
      {
        let location = locations.get( loc )
        .unwrap_or_else( || panic!( "PBRShader::impl_locations! missing \"{loc}\"" ) )
        .clone();
        gl::uniform::upload( gl, location, value )
      };

      // KHR_materials_specular defaults: specularFactor 1, specularColorFactor [ 1, 1, 1 ].
      upload( "specularFactor", self.specular_factor.unwrap_or( 1.0 ) )?;

      gl::uniform::upload( gl, locations.get( "baseColorFactor" ).expect( "PBRShader::impl_locations! missing \"baseColorFactor\"" ).clone(), self.base_color_factor.as_slice() )?;
      gl::uniform::upload( gl, locations.get( "metallicFactor" ).expect( "PBRShader::impl_locations! missing \"metallicFactor\"" ).clone(), &self.metallic_factor )?;
      gl::uniform::upload( gl, locations.get( "roughnessFactor" ).expect( "PBRShader::impl_locations! missing \"roughnessFactor\"" ).clone(), &self.roughness_factor )?;
      gl::uniform::upload( gl, locations.get( "normalScale" ).expect( "PBRShader::impl_locations! missing \"normalScale\"" ).clone(), &self.normal_scale )?;
      gl::uniform::upload( gl, locations.get( "occlusionStrength" ).expect( "PBRShader::impl_locations! missing \"occlusionStrength\"" ).clone(), &self.occlusion_strength )?;
      gl::uniform::upload( gl, locations.get( "alphaCutoff" ).expect( "PBRShader::impl_locations! missing \"alphaCutoff\"" ).clone(), &self.alpha_cutoff )?;
      gl::uniform::upload( gl, locations.get( "emissiveFactor" ).expect( "PBRShader::impl_locations! missing \"emissiveFactor\"" ).clone(), self.emissive_factor.as_slice() )?;
      if let Some( mipmap_distance_range_loc ) = locations.get( "mipmapDistanceRange" )
      {
        let r = &self.mipmap_distance_range;
        gl::uniform::upload( gl, mipmap_distance_range_loc.clone(), &[ r.start, r.end ] )?;
      }

      upload_array( "specularColorFactor", self.specular_color_factor.as_ref().map_or( &[ 1.0, 1.0, 1.0 ], | v | v.as_slice() ) )?;

      // KHR_materials_clearcoat / KHR_materials_anisotropy defaults: all three factors are 0.
      upload( "clearcoatFactor", self.clearcoat_factor.unwrap_or( 0.0 ) )?;
      upload( "clearcoatRoughnessFactor", self.clearcoat_roughness_factor.unwrap_or( 0.0 ) )?;
      gl::uniform::upload( gl, locations.get( "clearcoatNormalScale" ).expect( "PBRShader::impl_locations! missing \"clearcoatNormalScale\"" ).clone(), &self.clearcoat_normal_scale )?;
      upload( "anisotropyStrength", self.anisotropy_strength.unwrap_or( 0.0 ) )?;
      gl::uniform::upload( gl, locations.get( "anisotropyRotation" ).expect( "PBRShader::impl_locations! missing \"anisotropyRotation\"" ).clone(), &self.anisotropy_rotation )?;

      Ok( () )
    }

    fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      let bind = | texture : &Option< TextureInfo >, unit : u32 |
      {
        if let Some( ref t ) = texture
        {
          gl.active_texture( gl::TEXTURE0 + unit );
          t.upload( gl );
        }
      };

      bind( &self.metallic_roughness_texture, PBR_METALLIC_ROUGHNESS_UNIT );
      bind( &self.base_color_texture, PBR_BASE_COLOR_UNIT );
      bind( &self.normal_texture, PBR_NORMAL_UNIT );
      bind( &self.occlusion_texture, PBR_OCCLUSION_UNIT );
      bind( &self.emissive_texture, PBR_EMISSIVE_UNIT );
      bind( &self.specular_texture, PBR_SPECULAR_UNIT );
      bind( &self.specular_color_texture, PBR_SPECULAR_COLOR_UNIT );
      bind( &self.light_map, PBR_LIGHT_MAP_UNIT );
      bind( &self.clearcoat_texture, PBR_CLEARCOAT_UNIT );
      bind( &self.clearcoat_roughness_texture, PBR_CLEARCOAT_ROUGHNESS_UNIT );
      bind( &self.clearcoat_normal_texture, PBR_CLEARCOAT_NORMAL_UNIT );
      bind( &self.anisotropy_texture, PBR_ANISOTROPY_UNIT );
    }

    fn defines_str( &self ) -> &str
    {
      &self.cached_defines_str
    }

    fn vertex_defines_str( &self ) -> &str
    {
      &self.cached_vertex_defines_str
    }

    fn fragment_defines_str( &self ) -> &str
    {
      &self.cached_fragment_defines_str
    }

    fn needs_recompile( &self ) -> bool
    {
      self.needs_recompile.get()
    }

    fn recompile_flag_clear( &self )
    {
      self.needs_recompile.set( false );
    }

    fn fragment_shader( &self ) -> String
    {
      MAIN_FRAGMENT_SHADER.into()
    }

    fn vertex_shader( &self ) -> String
    {
      MAIN_VERTEX_SHADER.into()
    }

    fn dyn_clone( &self ) -> Box< dyn Material >
    {
      Box::new( self.clone() )
    }

    fn alpha_mode( &self ) -> AlphaMode
    {
      self.alpha_mode
    }

    fn cull_mode( &self ) -> Option< CullMode >
    {
      self.cull_mode
    }

    fn type_name( &self ) -> &'static str
    {
      stringify!( PbrMaterial )
    }

    fn has_emission( &self ) -> bool
    {
      self.emissive_texture.is_some()
      || self.emissive_factor.as_slice() != [ 0.0, 0.0, 0.0 ]
    }
  }

  impl Clone for PbrMaterial
  {
    fn clone( &self ) -> Self
    {
      PbrMaterial
      {
        id : uuid::Uuid::new_v4(),
        base_color_factor : self.base_color_factor,
        base_color_texture : self.base_color_texture.clone(),
        metallic_factor : self.metallic_factor,
        roughness_factor : self.roughness_factor,
        metallic_roughness_texture : self.metallic_roughness_texture.clone(),
        normal_scale : self.normal_scale,
        normal_texture : self.normal_texture.clone(),
        occlusion_strength : self.occlusion_strength,
        occlusion_texture : self.occlusion_texture.clone(),
        emissive_texture : self.emissive_texture.clone(),
        emissive_factor : self.emissive_factor,
        specular_factor : self.specular_factor,
        specular_texture : self.specular_texture.clone(),
        specular_color_factor : self.specular_color_factor,
        specular_color_texture : self.specular_color_texture.clone(),
        alpha_cutoff : self.alpha_cutoff,
        alpha_mode : self.alpha_mode,
        double_sided : self.double_sided,
        cull_mode : self.cull_mode,
        mipmap_distance_range : self.mipmap_distance_range.clone(),
        light_map : self.light_map.clone(),
        clearcoat_factor : self.clearcoat_factor,
        clearcoat_texture : self.clearcoat_texture.clone(),
        clearcoat_roughness_factor : self.clearcoat_roughness_factor,
        clearcoat_roughness_texture : self.clearcoat_roughness_texture.clone(),
        clearcoat_normal_scale : self.clearcoat_normal_scale,
        clearcoat_normal_texture : self.clearcoat_normal_texture.clone(),
        anisotropy_strength : self.anisotropy_strength,
        anisotropy_rotation : self.anisotropy_rotation,
        anisotropy_texture : self.anisotropy_texture.clone(),
        vertex_defines : self.vertex_defines.clone(),
        fragment_defines : self.fragment_defines.clone(),
        need_use_ibl : self.need_use_ibl,
        needs_update : Cell::new( true ),
        needs_recompile : Cell::new( true ),
        cached_defines_str : self.cached_defines_str.clone(),
        cached_vertex_defines_str : self.cached_vertex_defines_str.clone(),
        cached_fragment_defines_str : self.cached_fragment_defines_str.clone(),
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    MAX_POINT_LIGHTS,
    MAX_DIRECT_LIGHTS,
    MAX_SPOT_LIGHTS,
    PBR_METALLIC_ROUGHNESS_UNIT,
    PBR_BASE_COLOR_UNIT,
    PBR_NORMAL_UNIT,
    PBR_OCCLUSION_UNIT,
    PBR_EMISSIVE_UNIT,
    PBR_SPECULAR_UNIT,
    PBR_SPECULAR_COLOR_UNIT,
    PBR_LIGHT_MAP_UNIT,
    PBR_CLEARCOAT_UNIT,
    PBR_CLEARCOAT_ROUGHNESS_UNIT,
    PBR_CLEARCOAT_NORMAL_UNIT,
    PBR_ANISOTROPY_UNIT,
    PBR_TEXTURE_UNITS,
    PBR_IBL_BASE_UNIT,
    PBR_IBL_UNIT_COUNT,
    PBRShader,
    PbrMaterial
  };
}
