mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use gltf::mesh::iter::MorphTargets;
  use mingl::F32x3;
  use minwebgl as gl;
  use gl::
  {
    JsCast,
    geometry::BoundingBox,
  };
  use crate::webgl::
  {
    skeleton,
    AlphaMode,
    AttributeInfo,
    Geometry,
    IndexInfo,
    MagFilterMode,
    material::PbrMaterial,
    Material,
    Mesh,
    MinFilterMode,
    Node,
    Object3D,
    Primitive,
    Sampler,
    Scene,
    Texture,
    TextureInfo,
    ToFromGlEnum,
    WrappingMode,
    Light,
    PointLight,
    DirectLight,
    SpotLight,
    helpers
  };
  use web_sys::wasm_bindgen::prelude::Closure;

  use rustc_hash::FxHashMap;
  use
  {
    crate::webgl::Skeleton,
    gl::F32x4x4
  };

  const DIRECTION_LIGHT_MIN_MAGNITUDE : f32 = 0.01;

  #[ cfg( feature = "animation" ) ]
  use crate::webgl::animation::Animation;

  /// Represents a loaded glTF (GL Transmission Format) scene.
  pub struct GLTF
  {
    /// A collection of top-level scenes defined in the glTF file.
    pub scenes : Vec< Rc< RefCell< Scene > > >,
    /// A flat list of all nodes in the glTF file.
    pub nodes : Vec< Rc< RefCell< Node > > >,
    /// A list of WebGL buffer objects that store vertex data, indices, etc.
    pub gl_buffers : Vec< gl::WebGlBuffer >,
    /// A shared collection of WebGL textures, which are the raw image data on the GPU.
    pub images : Rc< RefCell< Vec< gl::web_sys::WebGlTexture > > >,
    /// A list of `Texture` objects, which wrap the raw WebGL textures and may contain
    /// additional metadata like sampler information.
    pub textures : Vec< Rc< RefCell< Texture > > >,
    /// A collection of `PbrMaterial` objects, defining how the surfaces of the meshes should be shaded.
    pub materials : Vec< Rc< RefCell< Box< dyn Material > > > >,
    /// A list of `Mesh` objects, which represent the geometry of the scene.
    pub meshes : Vec< Rc< RefCell< Mesh > > >,
    /// List of [`Node`]s that represent light sources
    pub lights : Vec< Rc< RefCell< Node > > >,
    /// A list of `Animation` objects, which store `Node`'s tranform change in every time moment.
    #[ cfg( feature = "animation" ) ]
    pub animations : Vec< Animation >,
  }

  impl GLTF
  {
    /// Casts the trait object to a specific `PbrMaterial`
    pub fn material_get( &self, id : usize ) -> std::cell::Ref< '_, PbrMaterial >
    {
      let material = self.materials[ id ].borrow();
      helpers::cast_unchecked_material_to_ref( material )
    }
  }

  fn load_skeleton_transforms_data
  (
    skin : gltf::Skin< '_ >,
    nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< skeleton::TransformsData >
  {
    let reader = skin.reader
    (
      | buffer | Some( buffers[ buffer.index() ].as_slice() )
    );

    let inverse_bind_matrices_iter = reader.read_inverse_bind_matrices()?;

    let matrices = inverse_bind_matrices_iter
    .map
    (
      | m |
      {
        F32x4x4::from_column_major
        (
          m.iter()
          .cloned()
          .flatten()
          .collect::< Vec< f32 > >()
          .as_chunks::< 16 >()
          .0
          .iter()
          .cloned()
          .next()
          .unwrap()
        )
      }
    )
    .collect::< Vec< _ > >();

    let mut joints = vec![];
    for ( joint, matrix ) in skin.joints().zip( matrices )
    {
      if let Some( name ) = joint.name()
      {
        if let Some( node ) = nodes.get( name )
        {
          joints.push( ( node.clone(), matrix ) );
        }
      }
    }

    Some( skeleton::TransformsData::new( joints ) )
  }

  fn load_skeleton_displacements_data
  (
    primitives_morph_targets : &Option< Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< skeleton::DisplacementsData >
  {
    let get_target_array = | acc : gltf::Accessor< '_ > |
    {
      gltf::mesh::util::ReadPositionDisplacements::new
      (
        acc,
        | buffer | buffers.get( buffer.index() ).map( | x | x.as_slice() )
      )
      .map( | iter | iter.collect::< Vec< _ > >() )
    };

    fn pack_targets
    (
      targets_array : Vec< Vec< [ f32; 3 ] > >
    )
    -> Vec< [ f32; 3 ] >
    {
      if targets_array.is_empty()
      {
        return vec![];
      }
      let mut packed_array = Vec::with_capacity( targets_array.first().unwrap().len() * targets_array.len() );
      for i in 0..targets_array.first().unwrap().len()
      {
        let targets_item = targets_array.iter()
        .map( | arr | arr[ i ] )
        .collect::< Vec< _ > >();
        packed_array.extend( targets_item );
      }

      packed_array
    }

    let skin_vertices_count = primitives_vertices_count.iter().sum::< usize >();
    let ( positions, normals, tangents ) = if let Some( primitives_morph_targets ) = primitives_morph_targets
    {
      let mut skin_positions = Vec::with_capacity( skin_vertices_count );
      let mut skin_normals = Vec::with_capacity( skin_vertices_count );
      let mut skin_tangents = Vec::with_capacity( skin_vertices_count );

      for ( i, morph_targets ) in primitives_morph_targets.iter().enumerate()
      {
        let vertices_count = primitives_vertices_count[ i ];
        let mut targets_positions = Vec::with_capacity( vertices_count );
        let mut targets_normals = Vec::with_capacity( vertices_count );
        let mut targets_tangents = Vec::with_capacity( vertices_count );

        for morph_target in morph_targets.clone()
        {
          if let Some( positions ) = morph_target.positions()
          .map( get_target_array )
          .flatten()
          {
            targets_positions.push( positions );
          }
          else
          {
            targets_positions.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }

          if let Some( normals ) = morph_target.normals()
          .map( get_target_array )
          .flatten()
          {
            targets_normals.push( normals );
          }
          else
          {
            targets_normals.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }

          if let Some( tangents ) = morph_target.tangents()
          .map( get_target_array )
          .flatten()
          {
            targets_tangents.push( tangents );
          }
          else
          {
            targets_tangents.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }
        }

        let primitive_positions = pack_targets( targets_positions );
        let primitive_normals = pack_targets( targets_normals );
        let primitive_tangents = pack_targets( targets_tangents );

        skin_positions.extend( primitive_positions );
        skin_normals.extend( primitive_normals );
        skin_tangents.extend( primitive_tangents );
      }

      (
        ( !skin_positions.is_empty() ).then_some( skin_positions ),
        ( !skin_normals.is_empty() ).then_some( skin_normals ),
        ( !skin_tangents.is_empty() ).then_some( skin_tangents )
      )
    }
    else
    {
      return None;
    };

    let mut displacements = skeleton::DisplacementsData::new();

    let _ = displacements.set_displacement( positions, gltf::Semantic::Positions, skin_vertices_count );
    let _ = displacements.set_displacement( normals, gltf::Semantic::Normals, skin_vertices_count );
    let _ = displacements.set_displacement( tangents, gltf::Semantic::Tangents, skin_vertices_count );
    if let Some( weights ) = weights
    {
      let weights_rc = displacements.get_morph_weights();
      *weights_rc.borrow_mut() = weights;
    }

    Some( displacements )
  }

  /// Loads [`Skeleton`] for one [`Mesh`]
  fn load_skeleton
  (
    skin : Option< gltf::Skin< '_ > >,
    nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > >,
    primitives_morph_targets : &Option< Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< Rc< RefCell< Skeleton > > >
  {
    let mut skeleton = Skeleton::new();

    *skeleton.transforms_as_mut() = skin
    .map( | s | load_skeleton_transforms_data( s, nodes, buffers ) ).flatten();
    *skeleton.displacements_as_mut() = load_skeleton_displacements_data
    (
      primitives_morph_targets,
      primitives_vertices_count,
      weights,
      buffers
    );

    if skeleton.has_skin() || skeleton.has_morph_targets()
    {
      Some( Rc::new( RefCell::new( skeleton ) ) )
    }
    else
    {
      None
    }
  }

  fn get_light_list( gltf : &gltf::Gltf ) -> Option< FxHashMap< usize, Light > >
  {
    let mut lights = FxHashMap::default();
    for ( i, gltf_light ) in gltf.lights()?.enumerate()
    {
      let light_type = gltf_light.kind();
      let light =  match light_type
      {
        gltf::khr_lights_punctual::Kind::Point =>
        {
          let Some( range ) = gltf_light.range()
          else
          {
            continue;
          };
          Light::Point
          (
            PointLight
            {
              position : F32x3::default(),
              color : F32x3::from_slice( &gltf_light.color() ),
              strength : gltf_light.intensity(),
              range
            }
          )
        },
        gltf::khr_lights_punctual::Kind::Directional =>
        {
          Light::Direct
          (
            DirectLight
            {
              direction : F32x3::default(),
              color : F32x3::from_slice( &gltf_light.color() ),
              strength : gltf_light.intensity(),
            }
          )
        },
        gltf::khr_lights_punctual::Kind::Spot { inner_cone_angle, outer_cone_angle } =>
        {
          let color = gltf_light.color();
          let strength = gltf_light.intensity();
          let range = gltf_light.range().unwrap_or( 10.0 );

          Light::Spot
          (
            SpotLight
            {
              position : F32x3::default(),
              direction : F32x3::default(),
              color: color.into(),
              strength : strength as f32,
              range : range as f32,
              inner_cone_angle,
              outer_cone_angle,
              use_light_map : false,
            }
          )
        }
      };

      lights.insert( i, light );
    }

    Some( lights )
  }

  fn get_light( gltf_node : &gltf::Node< '_ >, node : &Node, lights : &FxHashMap< usize, Light > ) -> Option< Light >
  {
    let light_id = gltf_node.extensions()?
    .get_key_value( "KHR_lights_punctual" )?.1
    .get( "light" )?
    .as_u64()?;

    lights.get( &( light_id as usize ) ).cloned()
    .map
    (
      | light |
      {
        match light
        {
          Light::Point( mut point_light ) =>
          {
            point_light.position = node.get_translation();
            Light::Point( point_light )
          },
          Light::Direct( mut direct_light ) =>
          {
            direct_light.direction = node.get_translation();
            if direct_light.direction.mag() < DIRECTION_LIGHT_MIN_MAGNITUDE
            {
              let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
              let rot_matrix = gl::math::d2::F32x3x3::from_quat( node.get_rotation() );
              direct_light.direction = rot_matrix * forward;
            }
            direct_light.direction = direct_light.direction.normalize();
            Light::Direct( direct_light )
          },
          Light::Spot( mut spot_light ) =>
          {
            spot_light.position = node.get_translation();
            spot_light.direction = node.get_translation();
            Light::Spot( spot_light )
          }
        }
      }
    )
  }

  /// Resolves a glTF asset `uri` (buffer or image) against the model's `folder_path`.
  ///
  /// URIs that already carry their own location are returned unchanged, because
  /// prefixing `folder_path` would corrupt them:
  /// * absolute / protocol-relative URLs (`http://`, `https://`, `//`),
  /// * self-contained URIs (`blob:`, `data:`),
  /// * origin-absolute paths (leading `/`).
  ///
  /// Everything else is treated as folder-relative and joined with a single `/`.
  ///
  /// When `folder_path` is empty (the glTF was loaded from a bare filename, so it
  /// sits at the origin root) a folder-relative `uri` resolves to `"/{uri}"`. This
  /// is intentional and harmless: `resolve_url` joins both `"/buffer.bin"` and
  /// `"buffer.bin"` against the origin to the same `"{origin}/buffer.bin"`. A glTF
  /// served from a subdirectory must be loaded with that directory in `gltf_path`
  /// (e.g. `"assets/scene.gltf"`), otherwise the glTF fetch itself fails first.
  fn resolve_asset_uri( folder_path : &str, uri : &str ) -> String
  {
    // `gl::file::load` already resolves self-contained URLs and origin-absolute
    // paths against the window origin; only genuinely folder-relative URIs need
    // the model's folder prefix folded in.
    if gl::file::is_self_contained_url( uri ) || uri.starts_with( '/' )
    {
      uri.to_string()
    }
    else
    {
      format!( "{}/{}", folder_path, uri )
    }
  }

  /// The glTF mime type of a KTX2 image. `KHR_texture_basisu` requires it.
  #[ cfg( feature = "ktx2" ) ]
  const KTX2_MIME : &str = "image/ktx2";

  /// Rejects an asset that *requires* an extension this build cannot honour.
  ///
  /// glTF draws a hard line between `extensionsUsed` and `extensionsRequired`: a client that cannot
  /// support a **required** extension must refuse the asset outright, rather than load a degraded
  /// version of it. `KHR_texture_basisu` in `extensionsRequired` means the author is telling us there
  /// is no fallback — every texture is KTX2 and nothing else.
  ///
  /// # Why this is ours to check, when `gltf` would normally do it
  ///
  /// `gltf` *does* enforce this rule, but it enforces it against `ENABLED_EXTENSIONS` — and that list
  /// contains `KHR_texture_basisu` whenever the `allow_empty_texture` feature is on. We enable that
  /// feature **unconditionally** ( see `Cargo.toml`: it is what makes `Texture::source()` return an
  /// `Option` at all, and splitting it on the `ktx2` feature would give the accessor two different
  /// signatures across builds ). The side effect is that we have switched **gltf's own guard off in
  /// both of our builds**: a basisu-required asset now parses cleanly even in a build that cannot
  /// decode a single one of its textures. `gltf-json` is candid about the hand-off — its comment on
  /// that allowlist reads "Processing is delegated to the user."
  ///
  /// So this is that processing. Without it, the failure would still be caught, but only later and
  /// once per texture, after buffers and images had already been fetched and uploaded.
  ///
  /// Only `KHR_texture_basisu` is checked here, not the other two allowlisted texture extensions
  /// ( `EXT_texture_webp`, `MSFT_texture_dds` ). They are equally undecodable by this renderer and
  /// equally allowlisted, so an asset requiring one of them has the same hole — but fixing that is a
  /// pre-existing concern of its own, not part of adding KTX2 support.
  /// Kept **pure** -- it names the problem and does not report it -- so that it can be tested without
  /// a browser. The logging and the error live at the call site.
  fn unsupported_required_extension( gltf_file : &gltf::Gltf ) -> Option< &'static str >
  {
    #[ cfg( not( feature = "ktx2" ) ) ]
    if gltf_file.extensions_required().any( | extension | extension == "KHR_texture_basisu" )
    {
      return Some( "KHR_texture_basisu" );
    }

    // Unused when the `ktx2` feature is on, which is the supported case.
    let _ = gltf_file;

    None
  }

  /// Index of the image a texture actually uses.
  ///
  /// A texture may name its image in two places, and which one wins depends on what this build can
  /// decode:
  ///
  /// * `extensions.KHR_texture_basisu.source` — a KTX2 image.
  /// * `source` — the texture's own field. `None` for a basisu-only texture.
  ///
  /// # Precedence, and why it is this way round
  ///
  /// When the `ktx2` feature is on, **the extension wins**. An asset is allowed to ship both, and
  /// `KHR_texture_basisu` is explicit that `source` is a *fallback for clients that cannot read the
  /// extension*. So a client that can read it must not take the fallback — otherwise an asset with
  /// both would quietly load the large uncompressed PNG, the KTX2 would go untouched, and the entire
  /// feature would silently do nothing while appearing to work. Preferring `source` here is the one
  /// bug in this function that would leave no visible trace.
  ///
  /// When the feature is off, the extension is **not consulted at all**, and a basisu-only texture
  /// resolves to `None`. That is deliberate too: returning its index would hand back an image that
  /// was never decoded — a 1×1 white placeholder — and the model would render as a blank, *silently
  /// wrong* surface. `None` keeps the loud, actionable error at the call site, which is the better
  /// failure. A texture that ships a plain `source` alongside still resolves through it, and loads
  /// exactly as it always did.
  fn effective_image_source( gltf_texture : &gltf::Texture< '_ > ) -> Option< usize >
  {
    #[ cfg( feature = "ktx2" ) ]
    if let Some( index ) = gltf_texture
    .extension_value( "KHR_texture_basisu" )
    .and_then( | extension | extension.get( "source" ) )
    .and_then( | source | source.as_u64() )
    {
      return Some( index as usize );
    }

    gltf_texture.source().map( | image | image.index() )
  }

  /// The raw bytes of an image if it is KTX2, or `None` if it is any other format.
  ///
  /// KTX2 is the one image format a browser cannot decode through an `<img>` element, so it has to
  /// be recognised *before* the normal path and routed to the CPU transcoder instead.
  #[ cfg( feature = "ktx2" ) ]
  async fn ktx2_image_bytes
  (
    gltf_image : &gltf::Image< '_ >,
    folder_path : &str,
    bin_buffers : &[ Vec< u8 > ],
  ) -> Result< Option< Vec< u8 > >, gl::WebglError >
  {
    match gltf_image.source()
    {
      gltf::image::Source::View { view, mime_type } =>
      {
        if mime_type != KTX2_MIME
        {
          return Ok( None );
        }

        let buffer = bin_buffers.get( view.buffer().index() )
        .ok_or( gl::WebglError::Other( "glTF KTX2 image points at a buffer that does not exist" ) )?;

        let start = view.offset();
        let end = start + view.length();

        // A malformed view could name a range outside its buffer. Slicing would panic; say so.
        let bytes = buffer.get( start..end )
        .ok_or( gl::WebglError::Other( "glTF KTX2 image view lies outside its buffer" ) )?;

        Ok( Some( bytes.to_vec() ) )
      },
      gltf::image::Source::Uri { uri, mime_type } =>
      {
        // `mimeType` is optional on a URI image, so the file extension is the fallback signal.
        let is_ktx2 = mime_type == Some( KTX2_MIME )
          || std::path::Path::new( uri ).extension().is_some_and( | e | e.eq_ignore_ascii_case( "ktx2" ) );

        if !is_ktx2
        {
          return Ok( None );
        }

        let path = resolve_asset_uri( folder_path, uri );
        let bytes = gl::file::load( &path ).await
        .map_err( | e |
        {
          gl::browser::error!( "Failed to load KTX2 image '{path}': {e:?}" );
          gl::WebglError::Other( "Failed to load a KTX2 image" )
        } )?;

        Ok( Some( bytes ) )
      },
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::resolve_asset_uri;

    /// A minimal glTF whose one texture gets its image from `KHR_texture_basisu` rather than from
    /// its own `source` field -- which is the entire shape this feature has to cope with.
    ///
    /// Note there is **no `source` on the texture at all**. That is not an omission in the fixture;
    /// it is what the extension mandates, and it is why `Texture::source()` cannot be used here.
    #[ cfg( feature = "ktx2" ) ]
    const BASISU_GLTF : &str = r#"{
      "asset" : { "version" : "2.0" },
      "extensionsUsed" : [ "KHR_texture_basisu" ],
      "images" : [ { "uri" : "colour.ktx2", "mimeType" : "image/ktx2" } ],
      "textures" :
      [
        { "extensions" : { "KHR_texture_basisu" : { "source" : 0 } } }
      ]
    }"#;

    /// A `KHR_texture_basisu` texture carries its image index in the extension, not in `source`, so
    /// that is where it must be read from.
    ///
    /// This also pins the premise the whole design rests on -- that `Texture::source()` really is
    /// `None` for such a texture. If that ever stopped being true, `effective_image_source` would be
    /// dead code and we would want to know.
    #[ cfg( feature = "ktx2" ) ]
    #[ test ]
    fn image_source_is_read_from_the_basisu_extension()
    {
      let document = gltf::Gltf::from_slice( BASISU_GLTF.as_bytes() )
      .expect( "a KHR_texture_basisu glTF must parse" );

      let texture = document.textures().next().expect( "one texture" );

      assert!
      (
        texture.source().is_none(),
        "premise broken : a KHR_texture_basisu texture is supposed to have no `source` of its own"
      );

      assert_eq!
      (
        super::effective_image_source( &texture ),
        Some( 0 ),
        "the image index must be recovered from the extension"
      );
    }

    /// An ordinary texture still resolves through its own `source`, extension or no extension.
    #[ test ]
    fn image_source_is_read_from_the_texture_for_ordinary_images()
    {
      const PLAIN_GLTF : &str = r#"{
        "asset" : { "version" : "2.0" },
        "images" : [ { "uri" : "a.png" }, { "uri" : "b.png" } ],
        "textures" : [ { "source" : 1 } ]
      }"#;

      let document = gltf::Gltf::from_slice( PLAIN_GLTF.as_bytes() ).expect( "plain glTF must parse" );
      let texture = document.textures().next().expect( "one texture" );

      assert_eq!( super::effective_image_source( &texture ), Some( 1 ) );
    }

    /// A texture carrying **both** a KTX2 image and a plain fallback -- the case `KHR_texture_basisu`
    /// exists to make loadable by everyone.
    ///
    /// Image 0 is the KTX2; image 1 is the uncompressed PNG fallback. Which one is correct depends
    /// entirely on what the build can decode, so the two tests below assert *opposite* answers.
    const DUAL_SOURCE_GLTF : &str = r#"{
      "asset" : { "version" : "2.0" },
      "extensionsUsed" : [ "KHR_texture_basisu" ],
      "images" :
      [
        { "uri" : "colour.ktx2", "mimeType" : "image/ktx2" },
        { "uri" : "colour.png", "mimeType" : "image/png" }
      ],
      "textures" :
      [
        {
          "source" : 1,
          "extensions" : { "KHR_texture_basisu" : { "source" : 0 } }
        }
      ]
    }"#;

    /// With the feature **on**, the KTX2 must win over the fallback.
    ///
    /// This is the assertion that protects the point of the whole feature. Taking the fallback here
    /// would be the one bug in `effective_image_source` that leaves *no visible trace*: the model
    /// renders perfectly, from the large uncompressed PNG, while the KTX2 is never touched and every
    /// byte of transcoder work goes unused.
    #[ cfg( feature = "ktx2" ) ]
    #[ test ]
    fn ktx2_build_prefers_the_extension_over_the_fallback_source()
    {
      let document = gltf::Gltf::from_slice( DUAL_SOURCE_GLTF.as_bytes() ).unwrap();
      let texture = document.textures().next().unwrap();

      assert_eq!
      (
        super::effective_image_source( &texture ),
        Some( 0 ),
        "a ktx2-capable build must use the KTX2 image, not the PNG fallback"
      );
    }

    /// With the feature **off**, the same file must resolve to the plain fallback and load normally.
    ///
    /// This is the other half of the contract: an asset that ships a fallback is supposed to work in
    /// a build that cannot decode KTX2, with no error and no placeholder.
    #[ cfg( not( feature = "ktx2" ) ) ]
    #[ test ]
    fn non_ktx2_build_falls_back_to_the_plain_source()
    {
      let document = gltf::Gltf::from_slice( DUAL_SOURCE_GLTF.as_bytes() ).unwrap();
      let texture = document.textures().next().unwrap();

      assert_eq!
      (
        super::effective_image_source( &texture ),
        Some( 1 ),
        "a build without the ktx2 feature must use the uncompressed fallback"
      );
    }

    /// An asset that **requires** `KHR_texture_basisu`: the author is stating there is no fallback.
    const REQUIRED_BASISU_GLTF : &str = r#"{
      "asset" : { "version" : "2.0" },
      "extensionsUsed" : [ "KHR_texture_basisu" ],
      "extensionsRequired" : [ "KHR_texture_basisu" ],
      "images" : [ { "uri" : "colour.ktx2", "mimeType" : "image/ktx2" } ],
      "textures" : [ { "extensions" : { "KHR_texture_basisu" : { "source" : 0 } } } ]
    }"#;

    /// A basisu-required asset must parse — which is exactly the problem.
    ///
    /// `gltf` normally rejects an asset requiring an extension outside `ENABLED_EXTENSIONS`. But
    /// `KHR_texture_basisu` is on that list whenever `allow_empty_texture` is on, and `renderer`
    /// enables that unconditionally — so **we have switched gltf's own guard off in both builds**.
    /// This test pins that fact, because it is the entire reason
    /// `unsupported_required_extension` has to exist. If a future gltf-rs made this parse fail on its
    /// own, this test would tell us the manual check had become redundant.
    #[ test ]
    fn gltf_does_not_reject_a_basisu_required_asset_for_us()
    {
      assert!
      (
        gltf::Gltf::from_slice( REQUIRED_BASISU_GLTF.as_bytes() ).is_ok(),
        "premise broken : gltf now rejects this itself, so the manual guard may be redundant"
      );
    }

    /// Without the feature, a *required* basisu asset is refused outright.
    #[ cfg( not( feature = "ktx2" ) ) ]
    #[ test ]
    fn non_ktx2_build_refuses_an_asset_that_requires_basisu()
    {
      let document = gltf::Gltf::from_slice( REQUIRED_BASISU_GLTF.as_bytes() ).unwrap();

      assert_eq!
      (
        super::unsupported_required_extension( &document ),
        Some( "KHR_texture_basisu" )
      );
    }

    /// With the feature, the same asset is perfectly loadable.
    #[ cfg( feature = "ktx2" ) ]
    #[ test ]
    fn ktx2_build_accepts_an_asset_that_requires_basisu()
    {
      let document = gltf::Gltf::from_slice( REQUIRED_BASISU_GLTF.as_bytes() ).unwrap();

      assert_eq!( super::unsupported_required_extension( &document ), None );
    }

    /// `used` is not `required`, and the difference decides whether the asset loads.
    ///
    /// An asset that merely *uses* basisu — and ships an uncompressed fallback alongside, as
    /// `DUAL_SOURCE_GLTF` does — must still load in a build that cannot decode KTX2. Hard-erroring on
    /// `extensionsUsed` would break exactly the assets the fallback mechanism was designed to save,
    /// so this asserts the guard stays quiet in **both** builds.
    #[ test ]
    fn an_asset_that_only_uses_basisu_is_never_refused()
    {
      let document = gltf::Gltf::from_slice( DUAL_SOURCE_GLTF.as_bytes() ).unwrap();

      assert_eq!
      (
        super::unsupported_required_extension( &document ),
        None,
        "a merely-used extension has a fallback and must not be a hard error"
      );
    }

    /// An asset using no extensions at all is unaffected.
    #[ test ]
    fn a_plain_asset_is_never_refused()
    {
      const PLAIN : &str = r#"{ "asset" : { "version" : "2.0" } }"#;
      let document = gltf::Gltf::from_slice( PLAIN.as_bytes() ).unwrap();

      assert_eq!( super::unsupported_required_extension( &document ), None );
    }

    #[ test ]
    fn joins_relative_uri_with_folder()
    {
      assert_eq!
      (
        resolve_asset_uri( "models", "scene/buffer.bin" ),
        "models/scene/buffer.bin"
      );
    }

    #[ test ]
    fn passes_blob_uri_through()
    {
      assert_eq!
      (
        resolve_asset_uri( "models", "blob:https://app.example.com/uuid-1234" ),
        "blob:https://app.example.com/uuid-1234"
      );
    }

    #[ test ]
    fn passes_data_uri_through()
    {
      assert_eq!
      (
        resolve_asset_uri( "models", "data:application/octet-stream;base64,Z2xURg==" ),
        "data:application/octet-stream;base64,Z2xURg=="
      );
    }

    #[ test ]
    fn passes_absolute_url_through()
    {
      assert_eq!
      (
        resolve_asset_uri( "models", "https://cdn.example.com/textures/t.png" ),
        "https://cdn.example.com/textures/t.png"
      );
    }

    #[ test ]
    fn passes_origin_absolute_path_through()
    {
      assert_eq!
      (
        resolve_asset_uri( "models", "/textures/t.png" ),
        "/textures/t.png"
      );
    }

    #[ test ]
    fn empty_folder_yields_origin_absolute_uri()
    {
      // Documents the benign empty-folder behavior: origin-absolute and
      // origin-relative forms collapse to the same URL once `resolve_url`
      // joins them against the window origin.
      assert_eq!
      (
        resolve_asset_uri( "", "buffer.bin" ),
        "/buffer.bin"
      );
    }
  }

  /// Asynchronously loads a glTF (GL Transmission Format) file and its associated resources.
  pub async fn load
  (
    document : &gl::web_sys::Document,
    gltf_path : &str,
    gl : &gl::WebGl2RenderingContext
  ) -> Result< GLTF, gl::WebglError >
  {
    gl.bind_vertex_array( None );

    let path = std::path::Path::new( gltf_path );
    let folder_path = path.parent().map_or( "", | p | p.to_str().expect( "Path is not UTF-8 encoded" ) );
    gl::debug!( "Folder: {}\nFile: {}", folder_path, gltf_path );

    // let gltf_slice= gl::file::load( &format!( "{}/scene.gltf", gltf_path ) )
    // .await.expect( "Failed to load gltf file" );
    // Propagate fetch / parse failures as errors instead of panicking: an
    // `.unwrap()` here aborts the whole wasm module (e.g. when a dev server
    // returns an HTML 404 page, or the bytes are not a valid glTF/GLB), leaving
    // it unusable for every subsequent call.
    // `WebglError::Other` only carries a `&'static str`, so the underlying
    // `JsValue` / `gltf::Error` (file path, HTTP status, JSON parse location)
    // would otherwise be lost. Log it to the console before mapping so a failed
    // load is diagnosable in production.
    let gltf_slice = gl::file::load( gltf_path ).await
    .map_err( | e |
    {
      gl::browser::error!( "Failed to load gltf file '{gltf_path}': {e:?}" );
      gl::WebglError::Other( "Failed to load gltf file" )
    } )?;
    let mut gltf_file = gltf::Gltf::from_slice( &gltf_slice )
    .map_err( | e |
    {
      gl::browser::error!( "Failed to parse gltf file '{gltf_path}': {e}" );
      gl::WebglError::Other( "Failed to parse gltf file" )
    } )?;

    if let Some( extension ) = unsupported_required_extension( &gltf_file )
    {
      gl::browser::error!
      (
        "'{gltf_path}' lists {extension} in extensionsRequired, but this build of `renderer` cannot \
         decode it. A required extension means the author has provided no fallback -- every texture \
         in this asset is KTX2 and nothing else -- so the asset cannot be rendered at all. Rebuild \
         `renderer` with the `ktx2` feature enabled."
      );
      return Err( gl::WebglError::Other( "glTF requires an extension this build cannot decode" ) );
    }

    let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();

    // Move the GLB bin into buffers
    if let Some( blob ) = gltf_file.blob.as_mut()
    {
      let blob = std::mem::take( blob );
      gl::debug!( "The gltf binary payload is present: {}", blob.len() );
      buffers.push( blob.as_slice().into() );
    }

    for gltf_buffer in gltf_file.buffers()
    {
      match gltf_buffer.source()
      {
        gltf::buffer::Source::Uri( uri ) =>
        {
          let path = resolve_asset_uri( folder_path, uri );
          let buffer = gl::file::load( &path ).await
          .map_err( | e |
          {
            gl::browser::error!( "Failed to load gltf buffer '{path}': {e:?}" );
            gl::WebglError::Other( "Failed to load a buffer" )
          } )?;

          gl::debug!
          (
            "Buffer path: {}\n
            \tBuffer length: {}",
            path,
            buffer.len()
          );

          buffers.push( buffer.as_slice().into() );
        },
        _ => {}
      }
    }

    let bin_buffers = buffers.iter()
    .map( | b | b.to_vec() )
    .collect::< Vec< _ > >();

    gl::debug!( "Buffers: {}", buffers.len() );

    // Upload images
    let images = Rc::new( RefCell::new( Vec::new() ) );

    // Creates an <img> html elements, and sets its src property to 'src' parameter
    // When the image is loaded, creates a texture and adds it to the 'images' array
    let upload_texture = | src : Rc< str > |
    {
      let texture = gl.create_texture().expect( "Failed to create a texture" );
      gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
      (
        gl::TEXTURE_2D,
        0,
        // Both RGBA and RGBA8 are valid internalformat values for texImage2D in WebGL2
        gl::RGBA as i32,
        1,
        1,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        Some( &[ 255, 255, 255, 255 ] )
      ).expect( "Failed to upload data to texture" );
      gl::texture::d2::filter_linear( gl );

      images.borrow_mut().push( texture.clone() );

      let img_element = document.create_element( "img" ).unwrap().dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
      img_element.style().set_property( "display", "none" ).unwrap();

      let load_texture : Closure< dyn Fn() > = Closure::new
      (
        {
          //let images = images.clone();
          let gl = gl.clone();
          let img = img_element.clone();
          let src = src.clone();
          move ||
          {
            gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
            //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
            gl.tex_image_2d_with_u32_and_u32_and_html_image_element
            (
              gl::TEXTURE_2D,
              0,
              gl::RGBA as i32,
              gl::RGBA,
              gl::UNSIGNED_BYTE,
              &img
            ).expect( "Failed to upload data to texture" );
            //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

            gl.generate_mipmap( gl::TEXTURE_2D );
            gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );

            // revoke_object_url is specified only for blob: URLs; for data: URIs or
            // plain file paths it is a no-op, and unwrapping its result is a latent
            // panic hazard in stricter runtimes. Only revoke the urls we created.
            if src.starts_with( "blob:" )
            {
              gl::web_sys::Url::revoke_object_url( &src ).unwrap();
            }

            img.remove();
          }
        }
      );

      // Without an onerror handler a 404 or malformed image URI fails silently:
      // the 1x1 white placeholder stays bound, nothing is logged, and load()
      // still returns Ok. Mirror the error logging added for buffer URI loads so
      // image failures are diagnosable instead of rendering as blank textures.
      let on_error : Closure< dyn Fn() > = Closure::new
      (
        {
          let img = img_element.clone();
          let src = src.clone();
          move ||
          {
            gl::browser::error!( "Failed to load gltf image '{src}'" );
            img.remove();
          }
        }
      );

      img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
      img_element.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
      img_element.set_src( &src );
      load_texture.forget();
      on_error.forget();
    };

    // Which compressed-texture formats this device can actually sample. Queried once: the answer
    // cannot change over a context's lifetime, and querying is what *enables* the extensions, so it
    // must happen before the first compressed upload.
    #[ cfg( feature = "ktx2" ) ]
    let compressed_support = gl::texture::compressed::Support::query( &gl );

    // If a source of an image is Uri - load the file
    // If a source of an image is View - create a blob from buffer, then turn it into an Object Url,
    // then load an image from the url
    for gltf_image in gltf_file.images()
    {
      // KTX2 images ( `KHR_texture_basisu` ) take a wholly different path. No browser can decode
      // KTX2 through an `<img>` element, so there is no URL to hand to the DOM and nothing to wait
      // for: the container is parsed, the UASTC blocks are transcoded on the CPU into whatever
      // format this GPU supports, and the result is uploaded as compressed blocks. That happens
      // *synchronously*, so unlike the `<img>` path below it pushes a finished texture rather than a
      // placeholder to be filled in later.
      #[ cfg( feature = "ktx2" ) ]
      if let Some( bytes ) = ktx2_image_bytes( &gltf_image, folder_path, &bin_buffers ).await?
      {
        let texture = crate::webgl::loaders::ktx2::load_into_texture
        (
          &gl,
          &bytes,
          compressed_support.best(),
          // Linear, deliberately, even when the KTX2 declares itself sRGB: the fragment shader
          // applies `SrgbToLinear` to base-color, specular and emissive samples itself, so letting
          // the sampler linearise as well would decode twice and darken the image.
          gl::texture::compressed::ColorSpace::Linear,
        )
        .map_err( | e |
        {
          gl::browser::error!( "Failed to decode KTX2 image {} : {e}", gltf_image.index() );
          gl::WebglError::Other( "Failed to decode a KTX2 image" )
        } )?;

        images.borrow_mut().push( texture );
        continue;
      }

      match  gltf_image.source()
      {
        gltf::image::Source::Uri { uri, mime_type: _ } =>
        {
          upload_texture( resolve_asset_uri( folder_path, uri ).into() );
        },
        gltf::image::Source::View { view, mime_type } =>
        {
          let buffer = buffers[ view.buffer().index() ].clone();
          let buffer = gl::js_sys::Uint8Array::new_with_byte_offset_and_length( &buffer.buffer(), view.offset() as u32, view.length() as u32 );
          let blob =
          {
            let options = gl::web_sys::BlobPropertyBag::new();
            options.set_type( mime_type );

            let mut blob_parts = Vec::new();
            blob_parts.push( buffer );

            gl::web_sys::Blob::new_with_u8_slice_sequence_and_options( &( blob_parts.into() ), &options )
          }.expect( "Failed to create a Blob" );

          let url = gl::web_sys::Url::create_object_url_with_blob( &blob ).expect( "Failed to create object url" );
          upload_texture( url.into() );
        }
      }
    }

    gl::debug!( "Images: {}", images.borrow().len() );

    // Upload buffer to the GPU
    let mut gl_buffers = Vec::new();
    // The target option may not be set for the attributes/indices buffers
    // This scenario should be checked
    for view in gltf_file.views()
    {
      let buffer = gl::buffer::create( &gl )?;

      let target =  if let Some( target ) = view.target()
      {
        match target
        {
          gltf::buffer::Target::ArrayBuffer => gl::ARRAY_BUFFER ,
          gltf::buffer::Target::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER
        }
      }
      else
      {
        gl::ARRAY_BUFFER
      };

      gl.bind_buffer( target, Some( &buffer ) );
      gl.buffer_data_with_js_u8_array_and_src_offset_and_length
      (
        target,
        &buffers[ view.buffer().index() ],
        gl::STATIC_DRAW,
        view.offset() as u32,
        view.length() as u32
      );

      gl_buffers.push( buffer );
    }

    gl::debug!( "GL Buffers: {}", gl_buffers.len() );

    // Create textures
    let mut textures = Vec::new();
    for gltf_t in gltf_file.textures()
    {
      let gltf_s = gltf_t.sampler();

      let mut sampler_former = Sampler::former();
      if let Some( filter ) = gltf_s.mag_filter()
      {
        sampler_former = sampler_former.mag_filter( MagFilterMode::from_gl( filter.as_gl_enum() ) );
      }
      if let Some( filter ) = gltf_s.min_filter()
      {
        sampler_former = sampler_former.min_filter( MinFilterMode::from_gl( filter.as_gl_enum() ) );
      }
      let sampler = sampler_former
      .wrap_s( WrappingMode::from_gl( gltf_s.wrap_s().as_gl_enum() ) )
      .wrap_t( WrappingMode::from_gl( gltf_s.wrap_t().as_gl_enum() ) )
      .form();

      // A texture whose image is supplied by an extension carries no `source` field of its own, so
      // `Texture::source()` is `None` and the index has to come from the extension instead --
      // which is what `effective_image_source` resolves. It still returns `None` for an extension
      // this build cannot decode ( `EXT_texture_webp`, `MSFT_texture_dds`, or `KHR_texture_basisu`
      // in a build without the `ktx2` feature ). Without the `allow_empty_texture` gltf feature the
      // accessor would instead panic on `.nth( u32::MAX ).unwrap()`; fail with a diagnosable error.
      let Some( image_index ) = effective_image_source( &gltf_t )
      else
      {
        gl::browser::error!
        (
          "glTF texture {} has no image source this build can decode. It most likely uses an \
           extension that is unsupported or not enabled ( e.g. KHR_texture_basisu ). Rebuild \
           `renderer` with the `ktx2` feature, or re-export the asset with an uncompressed \
           fallback image.",
          gltf_t.index()
        );
        return Err( gl::WebglError::Other( "glTF texture has no decodable image source" ) );
      };

      // The index comes from the file, so it is not to be trusted with a panicking accessor.
      let Some( source ) = images.borrow().get( image_index ).cloned()
      else
      {
        gl::browser::error!
        (
          "glTF texture {} names image {image_index}, but the file only has {} images.",
          gltf_t.index(),
          images.borrow().len()
        );
        return Err( gl::WebglError::Other( "glTF texture names an image that does not exist" ) );
      };

      let texture = Texture::former()
      .target( gl::TEXTURE_2D )
      .source( source )
      .sampler( sampler )
      .form();

      textures.push( Rc::new( RefCell::new( texture ) ) );
    }

    // Create materials
    let make_texture_info = | info : Option< gltf::texture::Info< '_ > > |
    {
      info.map( | v |
      {
        TextureInfo
        {
          uv_position : v.tex_coord(),
          texture : textures[ v.texture().index() ].clone()
        }
      })
    };

    let mut materials : Vec< Rc< RefCell< Box< dyn Material > > > > = Vec::new();
    let mut material_variation_map : FxHashMap< uuid::Uuid, Vec< Rc< RefCell< Box< dyn Material > > > > > = FxHashMap::default();
    let mut used_materials : Vec< Rc< RefCell< Box< dyn Material > > > > = Vec::new();

    for gltf_m in gltf_file.materials()
    {
      let pbr = gltf_m.pbr_metallic_roughness();

      let mut material = PbrMaterial::new( &gl );
      material.set_alpha_mode( match gltf_m.alpha_mode()
      {
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque
      });
      if let Some( value ) = gltf_m.alpha_cutoff() { material.alpha_cutoff = value; }
      material.base_color_factor = gl::F32x4::from( pbr.base_color_factor() );
      material.roughness_factor =  pbr.roughness_factor();
      material.metallic_factor = pbr.metallic_factor();
      material.set_base_color_texture( make_texture_info( pbr.base_color_texture() ) );
      material.set_metallic_roughness_texture( make_texture_info( pbr.metallic_roughness_texture() ) );
      material.set_emissive_texture( make_texture_info( gltf_m.emissive_texture() ) );
      material.emissive_factor = gl::F32x3::from( gltf_m.emissive_factor() );

      // KHR_materials_specular
      if let Some( s ) = gltf_m.specular()
      {
        material.set_specular_factor( Some( s.specular_factor() ) );
        material.set_specular_color_factor( Some( gl::F32x3::from( s.specular_color_factor() ) ) );
        // Specular texture
        material.set_specular_texture( make_texture_info( s.specular_texture() ) );
        // Specular color texture
        material.set_specular_color_texture( make_texture_info( s.specular_color_texture() ) );
      }

      if let Some( n ) = gltf_m.normal_texture()
      {
        material.normal_scale = n.scale();
        material.set_normal_texture( Some( TextureInfo
        {
          uv_position : n.tex_coord(),
          texture : textures[ n.texture().index() ].clone()
        }));
      }

      if let Some( o ) = gltf_m.occlusion_texture()
      {
        material.occlusion_strength = o.strength();
        material.set_occlusion_texture( Some( TextureInfo
        {
          uv_position : o.tex_coord(),
          texture : textures[ o.texture().index() ].clone()
        }));
      }

      material_variation_map.insert( material.id(), Vec::new() );
      materials.push( Rc::new( RefCell::new( Box::new( material ) ) ) );
    }

    let fallback = PbrMaterial::new( &gl );
    material_variation_map.insert( fallback.id(), Vec::new() );
    materials.push( Rc::new( RefCell::new( Box::new( fallback ) ) ) );

    gl::debug!( "PbrMaterials: {}",materials.len() );
    let make_attibute_info = | acc : &gltf::Accessor< '_ >, slot |
    {
      let data_type = match acc.data_type()
      {
        gltf::accessor::DataType::U8 => gl::DataType::U8,
        gltf::accessor::DataType::I8 => gl::DataType::I8,
        gltf::accessor::DataType::U16 => gl::DataType::U16,
        gltf::accessor::DataType::I16 => gl::DataType::I16,
        gltf::accessor::DataType::U32 => gl::DataType::U32,
        gltf::accessor::DataType::F32 => gl::DataType::F32
      };

      let descriptor = gl::BufferDescriptor::new::< [ f32; 1 ] >()
      .offset( acc.offset() as i32 / data_type.byte_size() )
      .normalized( acc.normalized() )
      .stride( acc.view().unwrap().stride().unwrap_or( 0 ) as i32 / data_type.byte_size() )
      .vector( gl::VectorDataType::new( data_type, acc.dimensions().multiplicity() as i32, 1 ) );

      AttributeInfo
      {
        slot,
        buffer : gl_buffers[ acc.view().unwrap().index() ].clone(),
        descriptor,
        bounding_box : Default::default()
      }
    };
    let mut meshes = Vec::new();
    for gltf_mesh in gltf_file.meshes()
    {
      let mut mesh = Mesh::default();

      for gltf_primitive in gltf_mesh.primitives()
      {
        let mut geometry = Geometry::new( gl )?;
        geometry.draw_mode = gltf_primitive.mode().as_gl_enum();

        let material_id = gltf_primitive.material().index().unwrap_or( materials.len() - 1 );
        let mut dummy_material = PbrMaterial::new( &gl );
        let gltf_material = materials[ material_id ].clone();

        let mut add_define = | name : &str |
        {
          dummy_material.add_define( format!( "USE_{}", name.to_uppercase() ), String::new() );
        };

        // Indices
        if let Some( acc ) = gltf_primitive.indices()
        {
          let info = IndexInfo
          {
            buffer : gl_buffers[ acc.view().unwrap().index() ].clone(),
            count : acc.count() as u32,
            offset : acc.offset() as u32,
            data_type : acc.data_type().as_gl_enum()
          };
          geometry.add_index( gl, info )?;
        }

        // Attributes
        for ( sem, acc ) in gltf_primitive.attributes()
        {
          if acc.sparse().is_some()
          {
            gl::debug!( "Sparce accessors are not supported yet" );
            continue;
          }

          match sem
          {
            gltf::Semantic::Positions =>
            {
              geometry.vertex_count = acc.count() as u32;
              let gltf_box = gltf_primitive.bounding_box();

              let mut attr_info = make_attibute_info( &acc, 0 );
              attr_info.bounding_box = BoundingBox::new( gltf_box.min, gltf_box.max );
              geometry.add_attribute( gl, "positions", attr_info )?;
            },
            gltf::Semantic::Normals =>
            {
              geometry.add_attribute( gl, "normals", make_attibute_info( &acc, 1 ) )?;
            },
            gltf::Semantic::TexCoords( i ) =>
            {
              assert!( i < 5, "Only 5 types of texture coordinates are supported" );
              geometry.add_attribute
              (
                gl,
                format!( "texture_coordinates_{}", 2 + i ),
                make_attibute_info( &acc, 2 + i )
              )?;
            },
            gltf::Semantic::Colors( i ) =>
            {
              assert!( i < 2, "Only 2 types of color coordinates are supported" );
              geometry.add_attribute
              (
                gl,
                format!( "colors_{}", 7 + i ),
                make_attibute_info( &acc, 7 + i )
              )?;
            },
            gltf::Semantic::Tangents =>
            {
              add_define( "tangents" );
              geometry.add_attribute
              (
                gl,
                "tangents",
                make_attibute_info( &acc, 9 )
              )?;
            },
            gltf::Semantic::Joints( i ) =>
            {
              let name = format!( "joints_{}", i );
              add_define( &name );
              geometry.add_attribute
              (
                gl,
                name,
                make_attibute_info( &acc, 10 + i ),
              )?;
            },
            gltf::Semantic::Weights( i ) =>
            {
              let name = format!( "weights_{}", i );
              add_define( &name );
              geometry.add_attribute
              (
                gl,
                name,
                make_attibute_info( &acc, 13 + i )
              )?;
            },
            //a => { gl::warn!( "Unsupported attribute: {:?}", a ); continue; }
          };
        }

        // Amongst different materials with the same uuid, find the one that has the same vertex defines
        let new_material = if let Some( material ) = material_variation_map
        .get( &gltf_material.borrow().id() )
        .map
        (
          | m |
          m.iter()
          .find( | m | m.borrow().vertex_defines_str() == dummy_material.vertex_defines_str() )
        )
        .flatten()
        {
          material.clone()
        }
        else
        {
          let material = Rc::new( RefCell::new( gltf_material.borrow().dyn_clone() ) );
          let mut m = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );

          for ( name, value ) in dummy_material.vertex_defines()
          {
            m.add_vertex_define( name.clone(), value );
          }

          std::mem::drop( m );
          used_materials.push( material.clone() );

          material
        };

        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material : new_material
        };

        mesh.add_primitive( Rc::new( RefCell::new( primitive ) ) );
      }

      meshes.push( Rc::new( RefCell::new( mesh ) ) );
    }

    gl::debug!( "Meshes: {}",meshes.len() );

    let gltf_lights = get_light_list( &gltf_file ).unwrap_or_default();

    let mut nodes = Vec::new();
    let mut rigged_nodes = Vec::new();
    let mut lights = Vec::new();

    for gltf_node in gltf_file.nodes()
    {
      let mut node = Node::default();
      node.set_visibility( true, true );
      let mut is_light = false;

      let ( translation, rotation, scale ) = gltf_node.transform().decomposed();
      node.set_scale( scale );
      node.set_translation( translation );
      node.set_rotation( gl::QuatF32::from( rotation ) );

      node.object = if let Some( mesh ) = gltf_node.mesh()
      {
        Object3D::Mesh( meshes[ mesh.index() ].clone() )
      }
      else if let Some( light ) = get_light( &gltf_node, &node, &gltf_lights )
      {
        is_light = true;
        Object3D::Light( light )
      }
      else
      {
        Object3D::Other
      };


      if let Some( name ) = gltf_node.name() { node.set_name( name ); }

      let node = Rc::new( RefCell::new( node ) );

      let ( primitives_morph_targets, weights ) = if let Some( mesh ) = gltf_node.mesh()
      {
        (
          Some( mesh.primitives().map( | p | p.morph_targets() ).collect::< Vec< _ > >() ),
          mesh.weights().map( | v | v.to_vec() )
        )
      }
      else
      {
        ( None, None )
      };
      rigged_nodes.push( ( node.clone(), gltf_node.skin(), primitives_morph_targets, weights ) );

      if is_light
      {
        lights.push( node.clone() );
      }

      nodes.push( node );
    }

    for gltf_node in gltf_file.nodes()
    {
      let mut node = nodes[ gltf_node.index() ].borrow_mut();
      for child in gltf_node.children()
      {
        node.add_child( nodes[ child.index() ].clone() );
      }
    }

    gl::debug!( "Nodes: {}", nodes.len() );

    let nodes_map = nodes.iter()
    .filter_map
    (
      | n |
      {
        n.borrow()
        .get_name()
        .map
        (
          | name |
          ( name, n.clone() )
        )
      }
    )
    .collect::< FxHashMap< _, _ > >();

    for ( node, skin, primitives_morph_targets, weights ) in rigged_nodes
    {
      if let Object3D::Mesh( mesh ) = &node.borrow().object
      {
        let primitives_vertices_count = mesh.borrow().primitives.iter()
        .map( | p | p.borrow().geometry.borrow().vertex_count as usize )
        .collect::< Vec< _ > >();
        if let Some( skeleton ) = load_skeleton
        (
          skin,
          &nodes_map,
          &primitives_morph_targets,
          primitives_vertices_count.as_slice(),
          weights,
          bin_buffers.as_slice()
        )
        {
          mesh.borrow_mut().skeleton = Some( skeleton.clone() );
          for primitive in &mesh.borrow().primitives
          {
            let p = primitive.borrow();
            let mut mat_mut = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >(  p.material.borrow_mut() );

            if skeleton.borrow().has_skin()
            {
              mat_mut.add_define( "USE_SKINNING", String::new() );
            }

            if skeleton.borrow().has_morph_targets()
            {
              mat_mut.add_define( "USE_MORPH_TARGET", String::new() );
            }
          }
        }
      }
    }

    #[ cfg( feature = "animation" ) ]
    let animations = crate::webgl::animation::loaders::gltf::load( &gl, &gltf_file, bin_buffers.as_slice(), nodes.as_slice() ).await;

    #[ cfg( feature = "animation" ) ]
    gl::debug!( "Animations: {}", animations.len() );

    let mut scenes = Vec::new();

    for gltf_scene in gltf_file.scenes()
    {
      let mut scene = Scene::default();
      for gltf_node in gltf_scene.nodes()
      {
        scene.add( nodes[ gltf_node.index() ].clone() );
      }
      scene.update_world_matrix();
      scenes.push( Rc::new( RefCell::new( scene ) ) );
    }

    gl.bind_vertex_array( None );
    gl.flush();

    Ok
    (
      GLTF
      {
        scenes,
        nodes,
        gl_buffers,
        images,
        textures,
        materials : used_materials,
        meshes,
        lights,
        #[ cfg( feature = "animation" ) ]
        animations
      }
    )
  }
}

crate::mod_interface!
{
  own use
  {
    GLTF,
    load
  };
}
