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
  /// Texture unit of `PbrMaterial`'s `engravingTexture` sampler.
  pub const PBR_ENGRAVING_UNIT : u32 = 12;
  /// A `PbrMaterial` texture getter, as stored in [`PBR_TEXTURE_UNITS`].
  pub type PbrTextureGet = fn( &PbrMaterial ) -> Option< &TextureInfo >;
  /// Every `PbrMaterial` sampler uniform with its texture unit and the texture bound to it, in
  /// unit order. `configure()` assigns the units and `bind()` binds the textures from this one
  /// table, so a sampler, its unit and its texture can't be paired differently in two places.
  pub const PBR_TEXTURE_UNITS : [ ( &str, u32, PbrTextureGet ); 13 ] =
  [
    ( "metallicRoughnessTexture", PBR_METALLIC_ROUGHNESS_UNIT, PbrMaterial::metallic_roughness_texture ),
    ( "baseColorTexture", PBR_BASE_COLOR_UNIT, PbrMaterial::base_color_texture ),
    ( "normalTexture", PBR_NORMAL_UNIT, PbrMaterial::normal_texture ),
    ( "occlusionTexture", PBR_OCCLUSION_UNIT, PbrMaterial::occlusion_texture ),
    ( "emissiveTexture", PBR_EMISSIVE_UNIT, PbrMaterial::emissive_texture ),
    ( "specularTexture", PBR_SPECULAR_UNIT, PbrMaterial::specular_texture ),
    ( "specularColorTexture", PBR_SPECULAR_COLOR_UNIT, PbrMaterial::specular_color_texture ),
    ( "lightMap", PBR_LIGHT_MAP_UNIT, PbrMaterial::light_map ),
    ( "clearcoatTexture", PBR_CLEARCOAT_UNIT, PbrMaterial::clearcoat_texture ),
    ( "clearcoatRoughnessTexture", PBR_CLEARCOAT_ROUGHNESS_UNIT, PbrMaterial::clearcoat_roughness_texture ),
    ( "clearcoatNormalTexture", PBR_CLEARCOAT_NORMAL_UNIT, PbrMaterial::clearcoat_normal_texture ),
    ( "anisotropyTexture", PBR_ANISOTROPY_UNIT, PbrMaterial::anisotropy_texture ),
    ( "engravingTexture", PBR_ENGRAVING_UNIT, PbrMaterial::engraving_texture ),
  ];
  /// First of the IBL units (irradiance, prefiltered environment, BRDF LUT): the unit after the
  /// last skinning / morph slot.
  pub const PBR_IBL_BASE_UNIT : u32 = crate::webgl::DISPLACEMENTS_SLOT + 1;
  /// Number of consecutive units IBL uses from `PBR_IBL_BASE_UNIT`.
  pub const PBR_IBL_UNIT_COUNT : u32 = 3;

  // OpenPBR units, after the skinning / morph slots (13-15), IBL (16-18) and the Kulla-Conty
  // LUT (19, bound by the `Renderer`). They sit outside `PBR_TEXTURE_UNITS`, whose count
  // `tests/pbr_texture_units_test.rs` holds to WebGL2's 16 guaranteed fragment samplers.
  /// Texture unit of `PbrMaterial`'s `sheenColorTexture` sampler ( OpenPBR fuzz color ).
  pub const PBR_SHEEN_COLOR_UNIT : u32 = 20;
  /// Texture unit of `PbrMaterial`'s `sheenRoughnessTexture` sampler ( OpenPBR fuzz roughness ).
  pub const PBR_SHEEN_ROUGHNESS_UNIT : u32 = 21;
  /// Texture unit of `PbrMaterial`'s `iridescenceTexture` sampler ( OpenPBR thin-film weight ).
  pub const PBR_IRIDESCENCE_UNIT : u32 = 22;
  /// Fragment texture unit of the transmission target color ( §3.3 ). Placed
  /// after the lobe carriers ( 20-22 ); depth is bound at the next unit. The
  /// fuzz directional-albedo LUT takes 25 ( bound by the `Renderer`, not by
  /// this material ).
  pub const TRANSMISSION_TEXTURE_UNIT : u32 = 23;

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
    "sheenColorTexture",
    "sheenRoughnessTexture",
    "iridescenceTexture",
    "kullaConty",
    "sheenAlbedo",
    //// Transmission uniform locations ( §3.3 )
    "uScreenSize",
    "transmissionSampler",
    "transmissionDepthSampler",
    "transmissionFactor",
    "transmissionThickness",
    "transmissionDepth",
    "transmissionColor",
    "transmissionDispersion",
    "diffuseTransmissionFactor",
    "diffuseTransmissionColor",
    "transmissionIor",
    "engravingTexture",
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
    "engravingStrength",
    "engravingRoughness",
    "engravingDarkening",
    // OpenPBR Surface carriers ( USE_OPENPBR / USE_KHR_materials_emissive_strength )
    "ior",
    "sheenColorFactor",
    "sheenRoughnessFactor",
    "iridescenceFactor",
    "iridescenceIor",
    "iridescenceThickness",
    "emissiveStrength",
    // Interactive highlight (hover / selection feedback)
    "highlightColor",
    "highlightStrength",
    // Luminosity
    "alphaCutoff",
    "exposure"
  );

  /// Scalar/color factors of the ratified `KHR_materials_*` glTF extensions that
  /// carry parameters of the ASWF OpenPBR Surface shading model
  /// ( https://academysoftwarefoundation.github.io/OpenPBR/ ) — the glTF
  /// transport encoding OpenPBR content ships in for real-time pipelines. One
  /// `Option` per parameter : `Some` only while the extension named in the
  /// field's doc comment is present on the glTF material, already defaulted to
  /// the extension's own schema default when the extension omits the key
  /// ( `volume_attenuation_distance` alone keeps `None` for its `+inf` schema
  /// default, which has no finite carrier ).
  ///
  /// This captures the scalar/color factor carriers only. Texture maps that
  /// vary these lobes per-texel ( e.g. `sheenColorTexture` ), and shader
  /// evaluation of the lobes themselves, are deferred to the shading steps
  /// that consume them — glTF texture upload already exists via the loader's
  /// `TextureInfo` slots. Field names follow the KHR JSON keys; the OpenPBR
  /// parameter each one transports is named in the field docs. See the crate
  /// `readme.md` "OpenPBR adoption" section for the full correspondence table.
  #[ derive( Default, Clone, Debug, PartialEq ) ]
  pub struct OpenPbrParams
  {
    /// Dielectric index of refraction — carries OpenPBR `specular_ior`
    /// ( `KHR_materials_ior.ior`, schema default `1.5` ).
    pub ior : Option< f32 >,
    /// Microfiber ( sheen ) layer color — carries OpenPBR `fuzz_color`
    /// ( `KHR_materials_sheen.sheenColorFactor`, schema default `[0, 0, 0]`,
    /// i.e. the sheen layer is disabled ).
    pub sheen_color_factor : Option< [ f32; 3 ] >,
    /// Microfiber ( sheen ) layer roughness — carries OpenPBR `fuzz_roughness`
    /// ( `KHR_materials_sheen.sheenRoughnessFactor`, schema default `0.0` ).
    pub sheen_roughness_factor : Option< f32 >,
    /// Fraction of light transmitted through the surface — carries OpenPBR
    /// `transmission_weight` ( `KHR_materials_transmission.transmissionFactor`,
    /// schema default `0.0` ).
    pub transmission_factor : Option< f32 >,
    /// Geometric thickness of the volume beneath the surface, in world units
    /// — how far a refracted ray travels inside the slab, which is what the
    /// screen-space refraction offsets by ( `KHR_materials_volume.thicknessFactor`,
    /// schema default `0.0`, i.e. thin-walled : an infinitely thin shell with
    /// no interior volume, OpenPBR `geometry_thin_walled` ). This is *not*
    /// OpenPBR `transmission_depth` — that is the absorption length scale
    /// carried by `volume_attenuation_distance` below.
    pub volume_thickness_factor : Option< f32 >,
    /// Distance over which `volume_attenuation_color` is reached — the glTF
    /// carrier of the OpenPBR `transmission_depth` absorption length scale
    /// ( `KHR_materials_volume.attenuationDistance` ). `None` when omitted
    /// ( schema default `+inf` : non-absorbing ).
    pub volume_attenuation_distance : Option< f32 >,
    /// Color white light turns into after one attenuation distance — the glTF
    /// carrier of OpenPBR `transmission_color`
    /// ( `KHR_materials_volume.attenuationColor`, schema default `[1, 1, 1]` ).
    pub volume_attenuation_color : Option< [ f32; 3 ] >,
    /// Thin-film ( iridescence ) intensity — carries OpenPBR `thin_film_weight`
    /// ( `KHR_materials_iridescence.iridescenceFactor`, schema default `0.0` ).
    pub iridescence_factor : Option< f32 >,
    /// Thin-film index of refraction — carries OpenPBR `thin_film_ior`
    /// ( `KHR_materials_iridescence.iridescenceIor`, schema default `1.3` ).
    pub iridescence_ior : Option< f32 >,
    /// Thin-film thickness at texture value `0.0` / minimum, in nanometers
    /// ( `KHR_materials_iridescence.iridescenceThicknessMinimum`, schema
    /// default `100.0` ).
    pub iridescence_thickness_minimum : Option< f32 >,
    /// Thin-film thickness at texture value `1.0` / maximum, in nanometers
    /// ( `KHR_materials_iridescence.iridescenceThicknessMaximum`, schema
    /// default `400.0` ).
    pub iridescence_thickness_maximum : Option< f32 >,
    /// Emissive intensity multiplier on the core `emissiveFactor` — the glTF
    /// carrier of OpenPBR emission luminance scale
    /// ( `KHR_materials_emissive_strength.emissiveStrength`, schema default
    /// `1.0` ).
    pub emissive_strength : Option< f32 >,
    /// Chromatic-dispersion strength, stored as `20 / Abbe_number` — the glTF
    /// carrier of OpenPBR `transmission_dispersion_scale`
    /// ( `KHR_materials_dispersion.dispersion`, schema default `0.0` ).
    pub dispersion : Option< f32 >,
    /// Diffuse-transmission ( translucency ) factor — the glTF carrier of the
    /// OpenPBR translucent-base scattering weight
    /// ( `KHR_materials_diffuse_transmission.diffuseTransmissionFactor`,
    /// schema default `0.0` ).
    pub diffuse_transmission_factor : Option< f32 >,
    /// Color modulating the diffusely transmitted light
    /// ( `KHR_materials_diffuse_transmission.diffuseTransmissionColorFactor`,
    /// schema default `[1, 1, 1]` ).
    pub diffuse_transmission_color_factor : Option< [ f32; 3 ] >,
  }

  impl OpenPbrParams
  {
    /// Returns `true` when no OpenPBR-carrying extension is present, i.e. the
    /// material should keep the legacy glTF metallic-roughness shading path
    /// ( no `USE_OPENPBR` define, no OpenPBR-only uniforms uploaded ).
    #[ must_use ]
    pub fn is_empty( &self ) -> bool
    {
      self.ior.is_none()
      && self.sheen_color_factor.is_none()
      && self.sheen_roughness_factor.is_none()
      && self.transmission_factor.is_none()
      && self.volume_thickness_factor.is_none()
      && self.volume_attenuation_distance.is_none()
      && self.volume_attenuation_color.is_none()
      && self.iridescence_factor.is_none()
      && self.iridescence_ior.is_none()
      && self.iridescence_thickness_minimum.is_none()
      && self.iridescence_thickness_maximum.is_none()
      && self.emissive_strength.is_none()
      && self.dispersion.is_none()
      && self.diffuse_transmission_factor.is_none()
      && self.diffuse_transmission_color_factor.is_none()
    }
  }

  /// Represents the visual properties of a surface.
  #[ derive( Former, Debug ) ]
  pub struct PbrMaterial
  {
    /// A unique identifier for the material.
    pub id : uuid::Uuid,
    /// The base color factor, multiplied with the base color texture. Defaults to white (1, 1, 1, 1).
    pub base_color_factor : gl::F32x4,
    /// Optional texture providing the base color.
    pub( super ) base_color_texture : Option< TextureInfo >,
    /// Scaling factor for the metallic component.
    pub metallic_factor : f32,
    /// Scaling factor for the roughness component.
    pub roughness_factor : f32,
    /// Optional texture providing the metallic and roughness values. Metalness is sampled from the B channel and roughness from the G channel.
    pub( super ) metallic_roughness_texture : Option< TextureInfo >,

    /// Scaling factor applied to each normal vector of the normal texture.
    pub normal_scale : f32,
    /// Optional texture containing normal vectors.
    pub( super ) normal_texture : Option< TextureInfo >,

    /// Scalar multiplier applied to the AO values sampled from the occlusion texture.
    pub occlusion_strength : f32,
    /// Optional texture providing ambient occlusion values.
    pub( super ) occlusion_texture : Option< TextureInfo >,

    /// Optional texture providing the emission color of the material.
    pub( super ) emissive_texture : Option< TextureInfo >,
    /// Scaling factor for the emission intensity
    pub emissive_factor : gl::F32x3,

    /// Optional scaling factor for the specular intensity. (KHR_materials_specular extension)
    pub( super ) specular_factor : Option< f32 >,
    /// Optional texture providing the specular intensity. (KHR_materials_specular extension)
    pub( super ) specular_texture : Option< TextureInfo >,
    /// Optional color factor for the specular highlight. (KHR_materials_specular extension)
    pub( super ) specular_color_factor : Option< gl::F32x3 >,
    /// Optional texture providing the specular color. (KHR_materials_specular extension)
    pub( super ) specular_color_texture : Option< TextureInfo >,
    /// Optional lightmap texture containing pre-baked lighting (shadows)
    pub( super ) light_map : Option< TextureInfo >,

    /// Optional scaling factor for the clearcoat layer intensity. (KHR_materials_clearcoat extension)
    pub( super ) clearcoat_factor : Option< f32 >,
    /// Optional texture providing the clearcoat intensity in the R channel. (KHR_materials_clearcoat extension)
    pub( super ) clearcoat_texture : Option< TextureInfo >,
    /// Optional roughness factor for the clearcoat layer. (KHR_materials_clearcoat extension)
    pub( super ) clearcoat_roughness_factor : Option< f32 >,
    /// Optional texture providing the clearcoat roughness in the G channel. (KHR_materials_clearcoat extension)
    pub( super ) clearcoat_roughness_texture : Option< TextureInfo >,
    /// Scaling factor applied to each normal vector of the clearcoat normal texture. (KHR_materials_clearcoat extension)
    pub clearcoat_normal_scale : f32,
    /// Optional texture containing normal vectors for the clearcoat layer. (KHR_materials_clearcoat extension)
    pub( super ) clearcoat_normal_texture : Option< TextureInfo >,

    /// Optional strength of the anisotropy effect. (KHR_materials_anisotropy extension)
    pub( super ) anisotropy_strength : Option< f32 >,
    /// Rotation of the anisotropy direction, in radians. (KHR_materials_anisotropy extension)
    pub anisotropy_rotation : f32,
    /// Optional texture providing the anisotropy direction (RG) and strength (B). (KHR_materials_anisotropy extension)
    pub( super ) anisotropy_texture : Option< TextureInfo >,

    /// Optional texture providing the sheen (fuzz) layer color in the RGB channels. (KHR_materials_sheen extension, OpenPBR `fuzz_color`)
    pub( super ) sheen_color_texture : Option< TextureInfo >,
    /// Optional texture providing the sheen (fuzz) layer roughness in the A channel. (KHR_materials_sheen extension, OpenPBR `fuzz_roughness`)
    pub( super ) sheen_roughness_texture : Option< TextureInfo >,
    /// Optional texture providing the thin-film (iridescence) weight in the A channel. (KHR_materials_iridescence extension, OpenPBR `thin_film_weight`)
    pub( super ) iridescence_texture : Option< TextureInfo >,

    /// OpenPBR Surface parameters carried by the ratified `KHR_materials_*`
    /// extensions. Filled by the glTF loader via
    /// `loaders::gltf::material_openpbr_params_read` — a wholesale capture of
    /// the scalar/color factor carriers; the lobes' shader consumption is a
    /// separate step ( see the field type's own docs ).
    pub openpbr_params : OpenPbrParams,

    /// Optional engraving mask texture: white text on a black background in the R channel,
    /// sampled at `vUv_{uv_position}` and bounds-checked against `[0, 1]` in the shader. This is
    /// the sole gate for the `USE_ENGRAVING` define — see [`Self::engraving_texture_set`].
    pub( super ) engraving_texture : Option< TextureInfo >,
    /// Intensity of the relief normal perturbation at letter edges (bevel depth).
    pub engraving_strength : f32,
    /// Target roughness inside the carved groove, simulating a matte laser-etched finish.
    pub engraving_roughness : f32,
    /// Fraction by which the base albedo / specular color (f0) is darkened inside the groove,
    /// simulating micro-occlusion without altering the underlying metal hue.
    pub engraving_darkening : f32,

    /// Multiplier the lit colour is blended toward while highlighted — e.g. hover or
    /// selection feedback after object picking. Components above `1.0` brighten. Only the
    /// lit colour is affected, never the emission (bloom) output.
    pub highlight_color : gl::F32x3,
    /// Blend weight toward [`PbrMaterial::highlight_color`]: `0.0` (the default) leaves the
    /// material untouched, `1.0` applies the full multiplier. Call `needs_update_set( true )`
    /// after changing either highlight field.
    pub highlight_strength : f32,

    /// Alpha cutoff value for mask mode. Fragments with alpha below this value are discarded.
    pub alpha_cutoff : f32,
    /// The alpha blending mode for the material. Defaults to `Opaque`.
    pub( super ) alpha_mode : AlphaMode,
    /// Determines wheter to draw both or one side of the primitive
    pub double_sided : bool,
    /// Face culling mode. `None` means culling is disabled.
    pub cull_mode : Option< CullMode >,

    /// Range of distances in which environment map's mipmap switching is applied
    pub mipmap_distance_range : std::ops::Range< f32 >,

    /// Hash map of defines in (value, name) format
    pub( super ) vertex_defines : FxHashMap< Box< str >, String >,
    /// Hash map of defines in (value, name) format
    pub( super ) fragment_defines : FxHashMap< Box< str >, String >,

    /// Returns answer need use IBL for current material instance or not
    pub( super ) need_use_ibl : bool,
    /// Signal for updating material uniforms.
    /// Use `needs_update_set(true)` after changing material properties.
    pub( super ) needs_update : Cell< bool >,
    /// Signal that shader defines have changed and program needs recompilation.
    pub( super ) needs_recompile : Cell< bool >,
    /// Cached combined defines string
    cached_defines_str : String,
    /// Cached vertex defines string
    cached_vertex_defines_str : String,
    /// Cached fragment defines string
    cached_fragment_defines_str : String,
  }

  /// Number of UV sets `main.frag` declares ( `vUv_0` to `vUv_4` ).
  const UV_SET_COUNT : u32 = 5;

  /// Pushes `#define <name>` for a texture the material samples, plus the macro that maps its
  /// per-texture UV varying (`uv_name`) onto the UV set the texture reads (`vUv_<n>`).
  ///
  /// Every texture's UV define is written here, core and extension textures alike, so this is
  /// where the UV set is bounded: one the shader doesn't declare would fail the program's compile,
  /// and a failed program isn't cached, so every frame would compile it again and draw nothing.
  /// It falls back to UV set 0 with a warning.
  // Fix(BUG-534): a texture on UV set 5 or above became `#define v<Name>Uv vUv_5`, a varying
  // `main.frag` doesn't declare, so the program failed to compile and every frame retried and drew
  // nothing.
  // Root cause: the UV set index went from the asset ( `texCoord` ) into the define unchecked;
  // `gltf-json` doesn't bound it, and the extension textures are read from raw JSON.
  // Pitfall: any asset index that names a shader symbol must be bounded by what the shader
  // declares, here `UV_SET_COUNT`, kept in step with `main.vert` / `main.frag`.
  fn texture_define_push( defines : &mut String, name : &str, uv_name : &str, info : &TextureInfo )
  {
    let _ = writeln!( defines, "#define {name}" );
    let mut uv_position = info.uv_position;
    if uv_position >= UV_SET_COUNT
    {
      gl::warn!( "{name}: UV set {uv_position} is not supported ( vUv_0 to vUv_{} ), using UV set 0", UV_SET_COUNT - 1 );
      uv_position = 0;
    }
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

      let sheen_color_texture = None;
      let sheen_roughness_texture = None;
      let iridescence_texture = None;

      let engraving_texture = None;
      let engraving_strength = 1.0;
      let engraving_roughness = 0.7;
      let engraving_darkening = 0.35;
      let highlight_color = gl::F32x3::splat( 1.0 );
      let highlight_strength = 0.0;

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
        sheen_color_texture,
        sheen_roughness_texture,
        iridescence_texture,
        openpbr_params : OpenPbrParams::default(),
        engraving_texture,
        engraving_strength,
        engraving_roughness,
        engraving_darkening,
        highlight_color,
        highlight_strength,
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

    /// Rebuilds the cached defines after a property change. A recompile is requested only when
    /// the define set changed: a new value within the same shader variant just needs its uniform
    /// uploaded again, which `needs_update` requests.
    pub( super ) fn defines_update( &mut self )
    {
      if self.defines_cache_rebuild()
      {
        self.needs_recompile.set( true );
      }
      self.needs_update.set( true );
    }

    /// Rebuilds all cached defines strings from current state, and returns whether they changed.
    fn defines_cache_rebuild( &mut self ) -> bool
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

      let changed = combined != self.cached_defines_str;
      self.cached_defines_str = combined;
      self.cached_vertex_defines_str = vertex_defines;
      self.cached_fragment_defines_str = fragment_defines;
      changed
    }

    /// Generates `#define` directives to be inserted into the fragment shader based on the material's properties.
    fn local_defines( &self ) -> String
    {
      let use_specular_texture = self.specular_texture.is_some();
      let use_specular_color_texture = self.specular_color_texture.is_some();

      let use_khr_materials_specular = self.specular_factor.is_some()
      || self.specular_color_factor.is_some()
      || use_specular_texture
      || use_specular_color_texture;

      let use_normal_texture = self.normal_texture.is_some();
      let use_alpha_cutoff = self.alpha_mode == AlphaMode::Mask;

      let mut defines = String::new();

      defines.push_str( format!( "#define MAX_POINT_LIGHTS {MAX_POINT_LIGHTS}\n" ).as_str() );
      defines.push_str( format!( "#define MAX_DIRECT_LIGHTS {MAX_DIRECT_LIGHTS}\n" ).as_str() );
      defines.push_str( format!( "#define MAX_SPOT_LIGHTS {MAX_SPOT_LIGHTS}\n" ).as_str() );

      // Base color texture related
      if let Some( info ) = &self.base_color_texture
      {
        texture_define_push( &mut defines, "USE_BASE_COLOR_TEXTURE", "vBaseColorUv", info );
      }

      // Metallic roughness texture related
      if let Some( info ) = &self.metallic_roughness_texture
      {
        texture_define_push( &mut defines, "USE_MR_TEXTURE", "vMRUv", info );
      }

      // Emission texture related
      if let Some( info ) = &self.emissive_texture
      {
        texture_define_push( &mut defines, "USE_EMISSION_TEXTURE", "vEmissionUv", info );
      }

      // KHR_Materials_Specular extension related
      if use_khr_materials_specular
      {
        defines.push_str( "#define USE_KHR_materials_specular\n" );
        if let Some( info ) = &self.specular_texture
        {
          texture_define_push( &mut defines, "USE_SPECULAR_TEXTURE", "vSpecularUv", info );
        }

        if let Some( info ) = &self.specular_color_texture
        {
          texture_define_push( &mut defines, "USE_SPECULAR_COLOR_TEXTURE", "vSpecularColorUv", info );
        }
      }

      // Normal texture related
      if let Some( info ) = &self.normal_texture
      {
        texture_define_push( &mut defines, "USE_NORMAL_TEXTURE", "vNormalUv", info );
      }

      // Occlusion texture related
      if let Some( info ) = &self.occlusion_texture
      {
        texture_define_push( &mut defines, "USE_OCCLUSION_TEXTURE", "vOcclusionUv", info );
      }

      if use_alpha_cutoff
      {
        defines.push_str( "#define USE_ALPHA_CUTOFF\n" );
      }

      if let Some( info ) = &self.light_map
      {
        texture_define_push( &mut defines, "USE_LIGHT_MAP", "vLightMapUv", info );
      }

      // KHR_materials_clearcoat / KHR_materials_anisotropy
      let layer_extensions_need_tbn = self.layer_extension_defines_push( &mut defines );

      // Engraving. Unlike the KHR_materials_* extensions, it has no "factor" uniform: the mask
      // texture's presence is the only thing that turns it on. Its relief perturbs the normal
      // along the tangent frame, so it needs `USE_TBN` too.
      let use_engraving = self.engraving_texture.is_some();
      if let Some( info ) = &self.engraving_texture
      {
        texture_define_push( &mut defines, "USE_ENGRAVING", "vEngravingUv", info );
      }

      // OpenPBR Surface defines ( carriers + §3.1 lobe textures + emissive
      // strength ) live in their own helper to keep this function readable.
      self.openpbr_defines( &mut defines );

      // Shared tangent/bitangent/normal matrix, needed by normal mapping, clearcoat normal
      // mapping, anisotropy and engraving alike.
      let use_tbn = use_normal_texture || layer_extensions_need_tbn || use_engraving;
      if use_tbn
      {
        defines.push_str( "#define USE_TBN\n" );
      }

      defines
    }

    /// Pushes the `KHR_materials_clearcoat` / `KHR_materials_anisotropy` defines onto `defines`
    /// and returns whether either extension needs the shared tangent frame (`USE_TBN`).
    ///
    /// Each layer's variant is selected by the value that turns it on, not by which properties
    /// are set: per the extensions, a `clearcoatFactor` of 0 disables the whole coat (its texture
    /// only multiplies the factor), and an `anisotropyStrength` of 0 leaves the roughness
    /// isotropic. An empty extension object, whose factors default to 0, therefore costs nothing.
    fn layer_extension_defines_push( &self, defines : &mut String ) -> bool
    {
      let use_khr_materials_clearcoat = self.clearcoat_factor.is_some_and( | f | f > 0.0 );
      let use_clearcoat_normal_texture = use_khr_materials_clearcoat && self.clearcoat_normal_texture.is_some();

      let use_khr_materials_anisotropy = self.anisotropy_strength.is_some_and( | s | s > 0.0 );

      // KHR_materials_clearcoat extension related
      if use_khr_materials_clearcoat
      {
        defines.push_str( "#define USE_KHR_materials_clearcoat\n" );
        if let Some( info ) = &self.clearcoat_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_TEXTURE", "vClearcoatUv", info );
        }

        if let Some( info ) = &self.clearcoat_roughness_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_ROUGHNESS_TEXTURE", "vClearcoatRoughnessUv", info );
        }

        if let Some( info ) = &self.clearcoat_normal_texture
        {
          texture_define_push( defines, "USE_CLEARCOAT_NORMAL_TEXTURE", "vClearcoatNormalUv", info );
        }
      }

      // KHR_materials_anisotropy extension related
      if use_khr_materials_anisotropy
      {
        defines.push_str( "#define USE_KHR_materials_anisotropy\n" );
        if let Some( info ) = &self.anisotropy_texture
        {
          texture_define_push( defines, "USE_ANISOTROPY_TEXTURE", "vAnisotropyUv", info );
        }
      }

      use_clearcoat_normal_texture || use_khr_materials_anisotropy
    }

    /// Generates the OpenPBR Surface part of the `#define` set.
    ///
    /// `USE_OPENPBR` selects the spec's opaque layered evaluation ( fuzz/sheen
    /// lobe, thin-film iridescence, Kulla-Conty multi-scatter ) for materials
    /// carrying those OpenPBR-only lobes; the legacy glTF metallic-roughness
    /// path is kept otherwise. `specular_ior` is NOT an OpenPBR-only feature —
    /// it is a base-level F0 that also matters on the legacy path — so it keeps
    /// its own `USE_OPENPBR_IOR` define instead of forcing the whole OpenPBR
    /// path. Textured lobe carriers ( adoption plan §3.1 subset ) activate their
    /// layer on texture presence alone, mirroring the glTF rule that a factor
    /// omitted while its texture is present means full strength.
    fn openpbr_defines( &self, defines : &mut String )
    {
      let p = &self.openpbr_params;
      let use_openpbr_ior = p.ior.is_some();
      let use_sheen_color_texture = self.sheen_color_texture.is_some();
      let use_sheen_roughness_texture = self.sheen_roughness_texture.is_some();
      let use_iridescence_texture = self.iridescence_texture.is_some();
      // Iridescence follows §3.2 ( thin-film layer ); the weight texture gates it too.
      let use_openpbr_iridescence = p.iridescence_factor.is_some() || use_iridescence_texture;
      let use_openpbr = p.sheen_color_factor.is_some()
      || p.sheen_roughness_factor.is_some()
      || use_sheen_color_texture
      || use_sheen_roughness_texture
      || use_openpbr_iridescence
      || p.transmission_factor.is_some()
      || p.volume_thickness_factor.is_some()
      || p.volume_attenuation_distance.is_some()
      || p.volume_attenuation_color.is_some()
      || p.dispersion.is_some()
      || p.diffuse_transmission_factor.is_some()
      || p.diffuse_transmission_color_factor.is_some();

      if use_openpbr_ior
      {
        defines.push_str( "#define USE_OPENPBR_IOR\n" );
      }
      if use_openpbr
      {
        defines.push_str( "#define USE_OPENPBR\n" );
        if let Some( info ) = &self.sheen_color_texture
        {
          texture_define_push( defines, "USE_SHEEN_COLOR_TEXTURE", "vSheenColorUv", info );
        }
        if let Some( info ) = &self.sheen_roughness_texture
        {
          texture_define_push( defines, "USE_SHEEN_ROUGHNESS_TEXTURE", "vSheenRoughnessUv", info );
        }
      }
      if use_openpbr_iridescence
      {
        defines.push_str( "#define USE_OPENPBR_IRIDESCENCE\n" );
        if let Some( info ) = &self.iridescence_texture
        {
          texture_define_push( defines, "USE_IRIDESCENCE_TEXTURE", "vIridescenceUv", info );
        }
      }
      // Weight-gated ( not just carrier presence ) : the define also enables the
      // refraction block in `main.frag`, whose samplers are only bound in the
      // transmission pass — a zero-weight material must stay fully in the
      // opaque/transparent passes ( matching `transmission_active` routing ).
      if p.transmission_factor.is_some_and( | w | w > 0.0 )
      {
        defines.push_str( "#define USE_TRANSMISSION\n" );

        // Thin-walled ( OpenPBR `geometry_thin_walled`, glTF `thicknessFactor = 0` ) :
        // an infinitely thin shell has no interior volume, so the refracted ray
        // exits where it entered - no lateral offset, no traversal length, and
        // hence no Beer-Lambert absorption to evaluate. A define rather than a
        // runtime branch keeps the slab math out of the shell variant entirely.
        let thin_walled = p.volume_thickness_factor.is_some_and( | t | t <= 0.0 );
        // Volumetric absorption ( adoption plan §3.4 ) is only defined for a
        // finite, positive `transmission_depth` inside a slab that has an
        // interior at all; the spec's `lambda = 0` case is "the interior medium
        // is absent", where `transmission_color` degenerates into a constant
        // tint - which is what the shader does without this define.
        let absorbing = !thin_walled
        && p.volume_attenuation_distance.is_some_and( | d | d > 0.0 && d.is_finite() );
        if thin_walled
        {
          defines.push_str( "#define USE_TRANSMISSION_THIN_WALLED\n" );
        }
        if absorbing
        {
          defines.push_str( "#define USE_TRANSMISSION_ABSORPTION\n" );
        }
        // Chromatic dispersion ( adoption plan §3.6 ) : three refracted taps at
        // per-channel IORs instead of one. A thin-walled shell has no refraction
        // offset for the spread to act on, so the three taps would coincide -
        // there is nothing to compile in that case.
        if !thin_walled && p.dispersion.is_some_and( | d | d > 0.0 && d.is_finite() )
        {
          defines.push_str( "#define USE_TRANSMISSION_DISPERSION\n" );
        }
      }
      // Diffuse transmission ( KHR_materials_diffuse_transmission ) — OpenPBR
      // `subsurface_*` on thin-walled geometry. A zero weight is the extension’s
      // own default and means "no lobe", so it must not compile the split in.
      if p.diffuse_transmission_factor.is_some_and( | w | w > 0.0 )
      {
        defines.push_str( "#define USE_DIFFUSE_TRANSMISSION\n" );
      }
      if p.emissive_strength.is_some()
      {
        defines.push_str( "#define USE_KHR_materials_emissive_strength\n" );
      }
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

    fn transmission_active( &self ) -> bool
    {
      self.openpbr_params.transmission_factor.is_some_and( | w | w > 0.0 )
    }

    fn transmission_thickness_authored( &self ) -> bool
    {
      // The `KHR_materials_volume` carrier. A `.mtlx` / `.usda` OpenPBR surface
      // leaves this `None` ( the spec has no geometric thickness ) unless it is
      // thin-walled, which sets it to `0.0` and is an authored answer too.
      self.openpbr_params.volume_thickness_factor.is_some()
    }

    fn transmission_texture_unit( &self ) -> Option< u32 >
    {
      if self.transmission_active()
      {
        Some( TRANSMISSION_TEXTURE_UNIT )
      }
      else
      {
        None
      }
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
      for ( name, unit, _ ) in PBR_TEXTURE_UNITS
      {
        let location = locations.get( name )
        .unwrap_or_else( || panic!( "PBRShader::impl_locations! missing \"{name}\"" ) );
        gl.uniform1i( location.as_ref(), unit as i32 );
      }
      // OpenPBR lobe textures ( adoption plan §3.1 subset ).
      for ( name, unit ) in
      [
        ( "sheenColorTexture", PBR_SHEEN_COLOR_UNIT ),
        ( "sheenRoughnessTexture", PBR_SHEEN_ROUGHNESS_UNIT ),
        ( "iridescenceTexture", PBR_IRIDESCENCE_UNIT ),
      ]
      {
        let location = locations.get( name )
        .unwrap_or_else( || panic!( "PBRShader::impl_locations! missing \"{name}\"" ) );
        gl.uniform1i( location.as_ref(), unit as i32 );
      }
      // Transmission target ( §3.3 ) : color at unit 23, depth at 24. The
      // textures themselves are bound once per frame by the Renderer's
      // transmission pass, not by this material.
      gl.uniform1i( locations.get( "transmissionSampler" ).expect( "PBRShader::impl_locations! missing \"transmissionSampler\"" ).clone().as_ref(), TRANSMISSION_TEXTURE_UNIT as i32 );
      gl.uniform1i( locations.get( "transmissionDepthSampler" ).expect( "PBRShader::impl_locations! missing \"transmissionDepthSampler\"" ).clone().as_ref(), TRANSMISSION_TEXTURE_UNIT as i32 + 1 );
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
      upload( "clearcoatNormalScale", self.clearcoat_normal_scale )?;
      upload( "anisotropyStrength", self.anisotropy_strength.unwrap_or( 0.0 ) )?;
      upload( "anisotropyRotation", self.anisotropy_rotation )?;
      // Engraving parameters, uploaded whatever the mask for the same reason as the factors.
      upload( "engravingStrength", self.engraving_strength )?;
      upload( "engravingRoughness", self.engraving_roughness )?;
      upload( "engravingDarkening", self.engraving_darkening )?;
      upload_array( "highlightColor", self.highlight_color.as_slice() )?;
      upload( "highlightStrength", self.highlight_strength )?;

      // OpenPBR Surface carriers — uploaded only while their define is on ( so the shader
      // declares the uniform and the location exists ). `ior` is uploaded for the
      // `USE_OPENPBR_IOR` path; sheen/fuzz for `USE_OPENPBR`; iridescence for
      // `USE_OPENPBR_IRIDESCENCE`.
      if self.openpbr_params.ior.is_some()
      {
        upload( "ior", self.openpbr_params.ior.unwrap_or( 1.5 ) )?;
      }
      if self.openpbr_params.sheen_color_factor.is_some() || self.openpbr_params.sheen_roughness_factor.is_some()
      || self.sheen_color_texture.is_some() || self.sheen_roughness_texture.is_some()
      {
        // glTF TextureInfo semantics: a factor omitted while its texture is present
        // defaults to full strength ( 1 ), not the no-texture schema default.
        let rough_default = if self.sheen_roughness_texture.is_some() { 1.0 } else { 0.0 };
        upload( "sheenRoughnessFactor", self.openpbr_params.sheen_roughness_factor.unwrap_or( rough_default ) )?;
        let color_default = if self.sheen_color_texture.is_some() { [ 1.0, 1.0, 1.0 ] } else { [ 0.0, 0.0, 0.0 ] };
        let sheen_color = self.openpbr_params.sheen_color_factor.unwrap_or( color_default );
        upload_array( "sheenColorFactor", &sheen_color )?;
      }
      if self.openpbr_params.iridescence_factor.is_some() || self.iridescence_texture.is_some()
      {
        upload( "iridescenceFactor", self.openpbr_params.iridescence_factor.unwrap_or( 1.0 ) )?;
        upload( "iridescenceIor", self.openpbr_params.iridescence_ior.unwrap_or( 1.4 ) )?;
        let min = self.openpbr_params.iridescence_thickness_minimum.unwrap_or( 0.0 );
        let max = self.openpbr_params.iridescence_thickness_maximum.unwrap_or( 0.0 );
        upload( "iridescenceThickness", ( min + max ) * 0.5 )?;
      }
      if self.openpbr_params.transmission_factor.is_some_and( | w | w > 0.0 )
      {
        let p = &self.openpbr_params;
        upload( "transmissionFactor", p.transmission_factor.unwrap_or( 0.0 ).clamp( 0.0, 1.0 ) )?;
        // Geometric depth of the slab the refracted ray crosses ( glTF
        // `thicknessFactor` ). A carrier-less material gets a mid-sized default
        // so the refraction stays visible but mild; `0` is thin-walled and
        // raises `USE_TRANSMISSION_THIN_WALLED`, where the shader ignores it.
        upload( "transmissionThickness", p.volume_thickness_factor.unwrap_or( 0.5 ).max( 0.0 ) )?;
        upload( "transmissionIor", p.ior.filter( | i | *i > 1.0 ).unwrap_or( 1.5 ) )?;
        // OpenPBR `transmission_color` : the color white light becomes after
        // travelling `transmission_depth` through the medium ( carried by the
        // glTF attenuation pair ). Without a finite depth the shader falls back
        // to the spec's constant-tint reading of it.
        let tint = p.volume_attenuation_color.unwrap_or( [ 1.0, 1.0, 1.0 ] );
        upload_array( "transmissionColor", &tint )?;
        if let Some( depth ) = p.volume_attenuation_distance.filter( | d | *d > 0.0 && d.is_finite() )
        {
          upload( "transmissionDepth", depth )?;
        }
        if let Some( dispersion ) = p.dispersion.filter( | d | *d > 0.0 && d.is_finite() )
        {
          upload( "transmissionDispersion", dispersion )?;
        }
      }
      if let Some( weight ) = self.openpbr_params.diffuse_transmission_factor.filter( | w | *w > 0.0 )
      {
        upload( "diffuseTransmissionFactor", weight.clamp( 0.0, 1.0 ) )?;
        // The extension defaults the tint to white, i.e. an untinted sheet.
        let tint = self.openpbr_params.diffuse_transmission_color_factor.unwrap_or( [ 1.0, 1.0, 1.0 ] );
        upload_array( "diffuseTransmissionColor", &tint )?;
      }
      if let Some( strength ) = self.openpbr_params.emissive_strength
      {
        upload( "emissiveStrength", strength )?;
      }

      Ok( () )
    }

    fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      // Each texture on the unit its sampler reads ( `PBR_TEXTURE_UNITS` ).
      for ( _, unit, texture ) in PBR_TEXTURE_UNITS
      {
        if let Some( t ) = texture( self )
        {
          gl.active_texture( gl::TEXTURE0 + unit );
          t.upload( gl );
        }
      }
      // OpenPBR lobe textures, on the units `configure` assigns them.
      for ( unit, texture ) in
      [
        ( PBR_SHEEN_COLOR_UNIT, &self.sheen_color_texture ),
        ( PBR_SHEEN_ROUGHNESS_UNIT, &self.sheen_roughness_texture ),
        ( PBR_IRIDESCENCE_UNIT, &self.iridescence_texture ),
      ]
      {
        if let Some( t ) = texture
        {
          gl.active_texture( gl::TEXTURE0 + unit );
          t.upload( gl );
        }
      }
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

    fn as_any( &self ) -> &dyn std::any::Any
    {
      self
    }

    fn as_any_mut( &mut self ) -> &mut dyn std::any::Any
    {
      self
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
        sheen_color_texture : self.sheen_color_texture.clone(),
        sheen_roughness_texture : self.sheen_roughness_texture.clone(),
        iridescence_texture : self.iridescence_texture.clone(),
        openpbr_params : self.openpbr_params.clone(),
        engraving_texture : self.engraving_texture.clone(),
        engraving_strength : self.engraving_strength,
        engraving_roughness : self.engraving_roughness,
        engraving_darkening : self.engraving_darkening,
        highlight_color : self.highlight_color,
        highlight_strength : self.highlight_strength,
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
  /// `PbrMaterial`'s property getters and setters.
  layer accessors;

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
    PbrTextureGet,
    PBR_IBL_BASE_UNIT,
    PBR_IBL_UNIT_COUNT,
    PBRShader,
    PbrMaterial,
    OpenPbrParams
  };
}
