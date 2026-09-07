mod private
{
  /// Canonical OpenPBR Surface parameter model — the full parameter set of the
  /// ASWF OpenPBR Surface spec ( https://academysoftwarefoundation.github.io/OpenPBR/ ,
  /// Parameter reference ), in native OpenPBR units and names. This is the
  /// *content* side of the native-formats lane ( MaterialX `.mtlx` and USD ) :
  /// the glTF `KHR_materials_*` carriers in
  /// [`OpenPbrParams`](crate::webgl::material::OpenPbrParams) are a lossy
  /// transport subset that maps *into* this model, whereas native content maps
  /// onto it directly. Defaults mirror the spec's own parameter tables and the
  /// ASWF `open_pbr_default.mtlx` reference surface.
  ///
  /// Texture maps ( `geometry_normal`, coat/fuzz map inputs, … ) are not
  /// modelled here yet — texture plumbing is a separate pipeline step ( crate
  /// `docs/openpbr_adoption_plan.md` §3.1 ).
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct OpenPbrSurface
  {
    /// `base_weight` — overall weight of the base substrate ( default `1.0` ).
    pub base_weight : f32,
    /// `base_color` — base albedo / metal tint, linear, ACEScg ( default
    /// `[0.8, 0.8, 0.8]` ).
    pub base_color : [ f32; 3 ],
    /// `base_diffuse_roughness` — roughness of the glossy-diffuse lobe
    /// ( default `0.0` ).
    pub base_diffuse_roughness : f32,
    /// `base_metalness` — dielectric/metal substrate mix ( default `0.0`;
    /// `1.0` = metal ).
    pub base_metalness : f32,

    /// `specular_weight` — weight of the specular lobe ( default `1.0` ).
    pub specular_weight : f32,
    /// `specular_color` — specular tint on the dielectric F0 ( default
    /// `[1.0, 1.0, 1.0]` ).
    pub specular_color : [ f32; 3 ],
    /// `specular_roughness` — specular roughness ( default `0.3` ).
    pub specular_roughness : f32,
    /// `specular_roughness_anisotropy` — anisotropy `a ∈ [0, 1]`
    /// ( default `0.0` ).
    pub specular_roughness_anisotropy : f32,
    /// `specular_ior` — dielectric index of refraction ( default `1.5` ).
    pub specular_ior : f32,

    /// `subsurface_weight` — subsurface scattering medium weight
    /// ( default `0.0` ).
    pub subsurface_weight : f32,
    /// `subsurface_color` — subsurface scattering tint ( default
    /// `[0.8, 0.8, 0.8]` ).
    pub subsurface_color : [ f32; 3 ],
    /// `subsurface_radius` — mean free path scale ( default `1.0` ).
    pub subsurface_radius : f32,
    /// `subsurface_radius_scale` — per-channel mean free path multiplier
    /// ( default `[1.0, 0.5, 0.25]` ).
    pub subsurface_radius_scale : [ f32; 3 ],
    /// `subsurface_scatter_anisotropy` — scattering anisotropy ( default `0.0` ).
    pub subsurface_scatter_anisotropy : f32,

    /// `transmission_weight` — translucent-base slab weight ( default `0.0` ).
    pub transmission_weight : f32,
    /// `transmission_color` — transmitted-light tint ( default
    /// `[1.0, 1.0, 1.0]` ).
    pub transmission_color : [ f32; 3 ],
    /// `transmission_depth` — medium mean free path length ( default `0.0` ).
    pub transmission_depth : f32,
    /// `transmission_scatter` — volumetric scattering color ( default
    /// `[0.0, 0.0, 0.0]` ).
    pub transmission_scatter : [ f32; 3 ],
    /// `transmission_scatter_anisotropy` — scattering anisotropy ( default `0.0` ).
    pub transmission_scatter_anisotropy : f32,
    /// `transmission_dispersion_scale` — dispersion strength, `20 / Abbe`
    /// ( default `0.0` ).
    pub transmission_dispersion_scale : f32,
    /// `transmission_dispersion_abbe_number` — Abbe number ( default `20.0` ).
    pub transmission_dispersion_abbe_number : f32,

    /// `thin_film_weight` — thin-film ( iridescence ) layer weight
    /// ( default `0.0` ).
    pub thin_film_weight : f32,
    /// `thin_film_thickness` — thin-film thickness in micrometres
    /// ( default `0.5` ).
    pub thin_film_thickness : f32,
    /// `thin_film_ior` — thin-film index of refraction ( default `1.4` ).
    pub thin_film_ior : f32,

    /// `coat_weight` — clearcoat layer weight ( default `0.0` ).
    pub coat_weight : f32,
    /// `coat_color` — clearcoat absorption tint ( default `[1.0, 1.0, 1.0]` ).
    pub coat_color : [ f32; 3 ],
    /// `coat_roughness` — clearcoat roughness ( default `0.0` ).
    pub coat_roughness : f32,
    /// `coat_roughness_anisotropy` — clearcoat anisotropy ( default `0.0` ).
    pub coat_roughness_anisotropy : f32,
    /// `coat_ior` — clearcoat index of refraction ( default `1.6` ).
    pub coat_ior : f32,
    /// `coat_darkening` — clearcoat darkening ( default `1.0` ).
    pub coat_darkening : f32,

    /// `fuzz_weight` — fuzz ( microfiber ) layer weight ( default `0.0` ).
    pub fuzz_weight : f32,
    /// `fuzz_color` — fuzz tint ( default `[1.0, 1.0, 1.0]` ).
    pub fuzz_color : [ f32; 3 ],
    /// `fuzz_roughness` — fuzz roughness ( default `0.5` ).
    pub fuzz_roughness : f32,

    /// `emission_luminance` — photometric emission in cd/m² ( default `0.0` ).
    pub emission_luminance : f32,
    /// `emission_color` — emission tint, may be HDR ( default `[1.0, 1.0, 1.0]` ).
    pub emission_color : [ f32; 3 ],

    /// `geometry_opacity` — surface opacity ( default `1.0` ).
    pub geometry_opacity : f32,
    /// `geometry_thin_walled` — thin-walled transmission mode ( default `false` ).
    pub geometry_thin_walled : bool,
  }

  impl OpenPbrSurface
  {
    /// The OpenPBR Surface with every parameter at its spec default — identical
    /// to the ASWF `open_pbr_default.mtlx` reference surface.
    #[ must_use ]
    pub fn spec_default() -> Self
    {
      Self
      {
        base_weight : 1.0,
        base_color : [ 0.8, 0.8, 0.8 ],
        base_diffuse_roughness : 0.0,
        base_metalness : 0.0,

        specular_weight : 1.0,
        specular_color : [ 1.0, 1.0, 1.0 ],
        specular_roughness : 0.3,
        specular_roughness_anisotropy : 0.0,
        specular_ior : 1.5,

        subsurface_weight : 0.0,
        subsurface_color : [ 0.8, 0.8, 0.8 ],
        subsurface_radius : 1.0,
        subsurface_radius_scale : [ 1.0, 0.5, 0.25 ],
        subsurface_scatter_anisotropy : 0.0,

        transmission_weight : 0.0,
        transmission_color : [ 1.0, 1.0, 1.0 ],
        transmission_depth : 0.0,
        transmission_scatter : [ 0.0, 0.0, 0.0 ],
        transmission_scatter_anisotropy : 0.0,
        transmission_dispersion_scale : 0.0,
        transmission_dispersion_abbe_number : 20.0,

        thin_film_weight : 0.0,
        thin_film_thickness : 0.5,
        thin_film_ior : 1.4,

        coat_weight : 0.0,
        coat_color : [ 1.0, 1.0, 1.0 ],
        coat_roughness : 0.0,
        coat_roughness_anisotropy : 0.0,
        coat_ior : 1.6,
        coat_darkening : 1.0,

        fuzz_weight : 0.0,
        fuzz_color : [ 1.0, 1.0, 1.0 ],
        fuzz_roughness : 0.5,

        emission_luminance : 0.0,
        emission_color : [ 1.0, 1.0, 1.0 ],

        geometry_opacity : 1.0,
        geometry_thin_walled : false,
      }
    }
  }

  fn float_set( slot : &mut f32, value : &str ) -> bool
  {
    match value.trim().parse::< f32 >()
    {
      Ok( v ) => { *slot = v; true },
      Err( _ ) => false
    }
  }

  fn color_set( slot : &mut [ f32; 3 ], value : &str ) -> bool
  {
    let parts : Vec< &str > = value.split( ',' ).map( str::trim ).collect();
    if parts.len() != 3
    {
      return false;
    }
    for ( slot, part ) in slot.iter_mut().zip( parts.iter() )
    {
      let Ok( v ) = part.parse::< f32 >() else { return false; };
      *slot = v;
    }
    true
  }

  fn bool_set( slot : &mut bool, value : &str ) -> bool
  {
    match value.trim()
    {
      "true" => { *slot = true; true },
      "false" => { *slot = false; true },
      _ => false
    }
  }

  /// Applies one MaterialX `<input name type value>` to a surface. Returns
  /// `false` when the input is not a scalar OpenPBR parameter this model
  /// carries ( unknown name, wrong/unsupported type token, or an unparsable
  /// value ) — so a `.mtlx` that wires a texture map ( `filename` ) into a
  /// scalar slot is reported rather than silently mis-applied.
  #[ must_use ]
  pub fn openpbr_input_apply( surface : &mut OpenPbrSurface, name : &str, type_name : &str, value : &str ) -> bool
  {
    match ( name, type_name )
    {
      ( "base_weight", "float" ) => float_set( &mut surface.base_weight, value ),
      ( "base_color", "color3" ) => color_set( &mut surface.base_color, value ),
      ( "base_diffuse_roughness", "float" ) => float_set( &mut surface.base_diffuse_roughness, value ),
      ( "base_metalness", "float" ) => float_set( &mut surface.base_metalness, value ),

      ( "specular_weight", "float" ) => float_set( &mut surface.specular_weight, value ),
      ( "specular_color", "color3" ) => color_set( &mut surface.specular_color, value ),
      ( "specular_roughness", "float" ) => float_set( &mut surface.specular_roughness, value ),
      ( "specular_roughness_anisotropy", "float" ) => float_set( &mut surface.specular_roughness_anisotropy, value ),
      ( "specular_ior", "float" ) => float_set( &mut surface.specular_ior, value ),

      ( "subsurface_weight", "float" ) => float_set( &mut surface.subsurface_weight, value ),
      ( "subsurface_color", "color3" ) => color_set( &mut surface.subsurface_color, value ),
      ( "subsurface_radius", "float" ) => float_set( &mut surface.subsurface_radius, value ),
      ( "subsurface_radius_scale", "color3" ) => color_set( &mut surface.subsurface_radius_scale, value ),
      ( "subsurface_scatter_anisotropy", "float" ) => float_set( &mut surface.subsurface_scatter_anisotropy, value ),

      ( "transmission_weight", "float" ) => float_set( &mut surface.transmission_weight, value ),
      ( "transmission_color", "color3" ) => color_set( &mut surface.transmission_color, value ),
      ( "transmission_depth", "float" ) => float_set( &mut surface.transmission_depth, value ),
      ( "transmission_scatter", "color3" ) => color_set( &mut surface.transmission_scatter, value ),
      ( "transmission_scatter_anisotropy", "float" ) => float_set( &mut surface.transmission_scatter_anisotropy, value ),
      ( "transmission_dispersion_scale", "float" ) => float_set( &mut surface.transmission_dispersion_scale, value ),
      ( "transmission_dispersion_abbe_number", "float" ) => float_set( &mut surface.transmission_dispersion_abbe_number, value ),

      ( "thin_film_weight", "float" ) => float_set( &mut surface.thin_film_weight, value ),
      ( "thin_film_thickness", "float" ) => float_set( &mut surface.thin_film_thickness, value ),
      ( "thin_film_ior", "float" ) => float_set( &mut surface.thin_film_ior, value ),

      ( "coat_weight", "float" ) => float_set( &mut surface.coat_weight, value ),
      ( "coat_color", "color3" ) => color_set( &mut surface.coat_color, value ),
      ( "coat_roughness", "float" ) => float_set( &mut surface.coat_roughness, value ),
      ( "coat_roughness_anisotropy", "float" ) => float_set( &mut surface.coat_roughness_anisotropy, value ),
      ( "coat_ior", "float" ) => float_set( &mut surface.coat_ior, value ),
      ( "coat_darkening", "float" ) => float_set( &mut surface.coat_darkening, value ),

      ( "fuzz_weight", "float" ) => float_set( &mut surface.fuzz_weight, value ),
      ( "fuzz_color", "color3" ) => color_set( &mut surface.fuzz_color, value ),
      ( "fuzz_roughness", "float" ) => float_set( &mut surface.fuzz_roughness, value ),

      ( "emission_luminance", "float" ) => float_set( &mut surface.emission_luminance, value ),
      ( "emission_color", "color3" ) => color_set( &mut surface.emission_color, value ),

      ( "geometry_opacity", "float" ) => float_set( &mut surface.geometry_opacity, value ),
      ( "geometry_thin_walled", "boolean" ) => bool_set( &mut surface.geometry_thin_walled, value ),

      _ => false
    }
  }

  /// Scalar factor carriers that the glTF transport ( core `pbrMetallicRoughness`
  /// plus the ratified `KHR_materials_*` set ) can express for an OpenPBR
  /// Surface. This is the authoring/lossy bridge into the canonical model
  /// ( adoption-plan N1 remainder ) : factors with a faithful OpenPBR
  /// equivalent are carried, while lobes that glTF has **no** scalar carrier
  /// for ( subsurface, volume, dispersion, iridescence, emission luminance )
  /// are left to the OpenPBR spec default by [`openpbr_from_gltf`] rather than
  /// guessed. Values follow glTF factor semantics and defaults; the mapping in
  /// [`openpbr_from_gltf`] clamps to the OpenPBR ranges.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct OpenPbrFromGltf
  {
    /// `baseColorFactor` RGBA in linear space ( alpha feeds
    /// `geometry_opacity` ).
    pub base_color : [ f32; 4 ],
    /// `metallicFactor` → `base_metalness` ( glTF default `1.0` ).
    pub base_metalness : f32,
    /// `roughnessFactor` → `specular_roughness` ( glTF default `1.0` ).
    pub specular_roughness : f32,
    /// `KHR_materials_ior.ior` → `specular_ior` ( default `1.5` ).
    pub specular_ior : f32,
    /// `KHR_materials_specular.specularFactor` → `specular_weight`.
    pub specular_weight : Option< f32 >,
    /// `KHR_materials_specular.specularColorFactor` → `specular_color`.
    pub specular_color : Option< [ f32; 3 ] >,
    /// `KHR_materials_anisotropy.anisotropyStrength` →
    /// `specular_roughness_anisotropy`.
    pub specular_roughness_anisotropy : Option< f32 >,
    /// `KHR_materials_clearcoat.clearcoatFactor` → `coat_weight`.
    pub coat_weight : Option< f32 >,
    /// `KHR_materials_clearcoat.clearcoatRoughnessFactor` → `coat_roughness`.
    pub coat_roughness : Option< f32 >,
    /// Presence of `KHR_materials_sheen` → `fuzz_weight` ( KHR sheen carries
    /// no separate weight, so presence maps to weight `1.0` ).
    pub fuzz_present : bool,
    /// `KHR_materials_sheen.sheenColorFactor` → `fuzz_color`.
    pub fuzz_color : Option< [ f32; 3 ] >,
    /// `KHR_materials_sheen.sheenRoughnessFactor` → `fuzz_roughness`.
    pub fuzz_roughness : Option< f32 >,
    /// `KHR_materials_transmission.transmissionFactor` → `transmission_weight`.
    pub transmission_weight : Option< f32 >,
  }

  impl Default for OpenPbrFromGltf
  {
    /// The glTF specification defaults ( a fully rough, white metal with no
    /// extensions ), ready to be overridden by the actual asset values.
    fn default() -> Self
    {
      Self
      {
        base_color : [ 1.0, 1.0, 1.0, 1.0 ],
        base_metalness : 1.0,
        specular_roughness : 1.0,
        specular_ior : 1.5,
        specular_weight : None,
        specular_color : None,
        specular_roughness_anisotropy : None,
        coat_weight : None,
        coat_roughness : None,
        fuzz_present : false,
        fuzz_color : None,
        fuzz_roughness : None,
        transmission_weight : None,
      }
    }
  }

  /// Maps the glTF factor carriers onto the canonical OpenPBR Surface.
  ///
  /// Starts from [`OpenPbrSurface::spec_default`] and overrides only what glTF
  /// can express ( see [`OpenPbrFromGltf`] for the correspondence ). Notable
  /// approximations: glTF clearcoat is a fixed-IOR `1.5` dielectric, so
  /// `coat_ior` is set to `1.5` ( OpenPBR spec default is `1.6` ) whenever a
  /// coat is present; KHR sheen has no separate weight, so `fuzz_weight` is
  /// `1.0` when the extension is present ( its black default colour disables
  /// the layer ).
  #[ must_use ]
  pub fn openpbr_from_gltf( input : &OpenPbrFromGltf ) -> OpenPbrSurface
  {
    let mut surface = OpenPbrSurface::spec_default();

    surface.base_color = [ input.base_color[ 0 ], input.base_color[ 1 ], input.base_color[ 2 ] ];
    surface.base_metalness = input.base_metalness.clamp( 0.0, 1.0 );
    surface.specular_roughness = input.specular_roughness.clamp( 0.0, 1.0 );
    surface.specular_ior = input.specular_ior.max( 1.0 );
    surface.geometry_opacity = input.base_color[ 3 ].clamp( 0.0, 1.0 );

    if let Some( weight ) = input.specular_weight
    {
      surface.specular_weight = weight.clamp( 0.0, 1.0 );
    }
    if let Some( color ) = input.specular_color
    {
      surface.specular_color = color;
    }
    if let Some( anisotropy ) = input.specular_roughness_anisotropy
    {
      surface.specular_roughness_anisotropy = anisotropy.clamp( 0.0, 1.0 );
    }

    if input.coat_weight.is_some()
    {
      surface.coat_weight = input.coat_weight.unwrap_or( 0.0 ).clamp( 0.0, 1.0 );
      surface.coat_roughness = input.coat_roughness.unwrap_or( 0.0 ).clamp( 0.0, 1.0 );
      // glTF models the clearcoat as a fixed-IOR ( 1.5 ) dielectric.
      surface.coat_ior = 1.5;
    }

    if input.fuzz_present
    {
      surface.fuzz_weight = 1.0;
      surface.fuzz_color = input.fuzz_color.unwrap_or( [ 1.0, 1.0, 1.0 ] );
      surface.fuzz_roughness = input.fuzz_roughness.unwrap_or( 0.5 ).clamp( 0.0, 1.0 );
    }

    if let Some( transmission ) = input.transmission_weight
    {
      surface.transmission_weight = transmission.clamp( 0.0, 1.0 );
    }

    surface
  }

  /// Runtime-facing factors that [`openpbr_to_runtime`] derives from a
  /// canonical [`OpenPbrSurface`], shaped so a glTF-material consumer (
  /// [`PbrMaterial`](crate::webgl::material::PbrMaterial) ) can render the
  /// surface through the existing `USE_OPENPBR` path. `None`/default values are
  /// left out so the consumer keeps its own defaults ( e.g. an OpenPBR surface
  /// at IOR 1.5 needs no override — the legacy glTF F0 of 0.04 is the same
  /// Fresnel ).
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct OpenPbrRuntime
  {
    /// `base_color` RGB + `geometry_opacity` as alpha ( linear RGBA ).
    pub base_color_rgba : [ f32; 4 ],
    /// `base_metalness`.
    pub base_metalness : f32,
    /// `specular_roughness`.
    pub specular_roughness : f32,
    /// `specular_ior` when it differs from the glTF/legacy default `1.5`.
    pub specular_ior : Option< f32 >,
    /// `specular_weight` when it differs from `1.0`.
    pub specular_weight : Option< f32 >,
    /// `specular_color` when it differs from white.
    pub specular_color : Option< [ f32; 3 ] >,
    /// `coat_weight` when the coat layer is enabled ( > 0 ).
    pub coat_weight : Option< f32 >,
    /// `coat_roughness` when the coat layer is enabled.
    pub coat_roughness : Option< f32 >,
    /// `fuzz_color` when the fuzz layer is enabled ( `fuzz_weight` > 0 ).
    pub fuzz_color : Option< [ f32; 3 ] >,
    /// `fuzz_roughness` when the fuzz layer is enabled.
    pub fuzz_roughness : Option< f32 >,
  }

  /// Reduces a canonical [`OpenPbrSurface`] to the factors the runtime material
  /// model can express today ( glTF-shaped base + KHR specular/clearcoat/sheen
  /// carriers ). Lobes without a runtime sink ( subsurface, transmission/volume,
  /// thin-film, dispersion, emission luminance, `base_diffuse_roughness`,
  /// `coat_darkening` ) are omitted — they stay unimplemented until the
  /// corresponding §3 pipeline step lands.
  #[ must_use ]
  pub fn openpbr_to_runtime( surface : &OpenPbrSurface ) -> OpenPbrRuntime
  {
    let near = | a : f32, b : f32 | ( a - b ).abs() <= 1e-4;
    let not_white = | c : &[ f32; 3 ] | !( near( c[ 0 ], 1.0 ) && near( c[ 1 ], 1.0 ) && near( c[ 2 ], 1.0 ) );

    OpenPbrRuntime
    {
      base_color_rgba : [ surface.base_color[ 0 ], surface.base_color[ 1 ], surface.base_color[ 2 ], surface.geometry_opacity ],
      base_metalness : surface.base_metalness,
      specular_roughness : surface.specular_roughness,
      specular_ior : ( ( surface.specular_ior - 1.5 ).abs() > 1e-4 ).then_some( surface.specular_ior ),
      specular_weight : ( ( surface.specular_weight - 1.0 ).abs() > 1e-4 ).then_some( surface.specular_weight ),
      specular_color : not_white( &surface.specular_color ).then_some( surface.specular_color ),
      coat_weight : ( surface.coat_weight > 1e-3 ).then_some( surface.coat_weight ),
      coat_roughness : ( surface.coat_weight > 1e-3 ).then_some( surface.coat_roughness ),
      fuzz_color : ( surface.fuzz_weight > 1e-3 ).then_some( surface.fuzz_color ),
      fuzz_roughness : ( surface.fuzz_weight > 1e-3 ).then_some( surface.fuzz_roughness ),
    }
  }

  /// Builds the glTF-shaped extension carriers ( [`OpenPbrParams`] ) that make
  /// the runtime `USE_OPENPBR` path evaluate `surface`: specular IOR, the fuzz
  /// ( sheen ) layer and the thin-film ( iridescence ) layer. OpenPBR
  /// `thin_film_thickness` is in micrometres; the glTF-carrier iridescence
  /// thickness is in nanometres ( `×1000` ).
  #[ must_use ]
  pub fn openpbr_params_from_surface( surface : &OpenPbrSurface ) -> crate::webgl::material::OpenPbrParams
  {
    let mut params = crate::webgl::material::OpenPbrParams::default();

    if ( surface.specular_ior - 1.5 ).abs() > 1e-4
    {
      params.ior = Some( surface.specular_ior );
    }

    if surface.fuzz_weight > 1e-3
    {
      params.sheen_color_factor = Some( surface.fuzz_color );
      params.sheen_roughness_factor = Some( surface.fuzz_roughness );
    }

    if surface.thin_film_weight > 1e-3
    {
      let thickness_nm = surface.thin_film_thickness * 1000.0;
      params.iridescence_factor = Some( surface.thin_film_weight );
      params.iridescence_ior = Some( surface.thin_film_ior );
      params.iridescence_thickness_minimum = Some( thickness_nm );
      params.iridescence_thickness_maximum = Some( thickness_nm );
    }

    params
  }
}

crate::mod_interface!
{
  orphan use
  {
    OpenPbrSurface,
    OpenPbrFromGltf,
    OpenPbrRuntime,
    openpbr_input_apply,
    openpbr_from_gltf,
    openpbr_to_runtime,
    openpbr_params_from_surface
  };
}
