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
    #[ must_use ]
    pub fn material_get( &self, id : usize ) -> std::cell::Ref< '_, PbrMaterial >
    {
      let material = self.materials[ id ].borrow();
      helpers::cast_unchecked_material_to_ref( material )
    }
  }

  /// A material shared between primitives, mutable behind `Rc< RefCell< _ > >`.
  type SharedMaterial = Rc< RefCell< Box< dyn Material > > >;

  fn skeleton_transforms_data_load
  (
    skin : &gltf::Skin< '_ >,
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
        let mut matrix = [ 0.0_f32; 16 ];
        for ( dst, src ) in matrix.iter_mut().zip( m.iter().flatten() )
        {
          *dst = *src;
        }
        F32x4x4::from_column_major( matrix )
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

  fn skeleton_displacements_data_load
  (
    primitives_morph_targets : Option< &Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< skeleton::DisplacementsData >
  {
    fn targets_pack
    (
      targets_array : &[ Vec< [ f32; 3 ] > ]
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

    let get_target_array = | acc : gltf::Accessor< '_ > |
    {
      gltf::mesh::util::ReadPositionDisplacements::new
      (
        acc,
        | buffer | buffers.get( buffer.index() ).map( std::vec::Vec::as_slice )
      )
      .map( std::iter::Iterator::collect::< Vec< _ > > )
    };

    let skin_vertices_count = primitives_vertices_count.iter().sum::< usize >();
    let ( positions, normals, tangents ) =
    {
      let primitives_morph_targets = primitives_morph_targets?;

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
          .and_then(get_target_array)
          {
            targets_positions.push( positions );
          }
          else
          {
            targets_positions.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }

          if let Some( normals ) = morph_target.normals()
          .and_then(get_target_array)
          {
            targets_normals.push( normals );
          }
          else
          {
            targets_normals.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }

          if let Some( tangents ) = morph_target.tangents()
          .and_then(get_target_array)
          {
            targets_tangents.push( tangents );
          }
          else
          {
            targets_tangents.push( vec![ [ 0.0; 3 ]; vertices_count ] );
          }
        }

        let primitive_positions = targets_pack( &targets_positions );
        let primitive_normals = targets_pack( &targets_normals );
        let primitive_tangents = targets_pack( &targets_tangents );

        skin_positions.extend( primitive_positions );
        skin_normals.extend( primitive_normals );
        skin_tangents.extend( primitive_tangents );
      }

      (
        ( !skin_positions.is_empty() ).then_some( skin_positions ),
        ( !skin_normals.is_empty() ).then_some( skin_normals ),
        ( !skin_tangents.is_empty() ).then_some( skin_tangents )
      )
    };

    let mut displacements = skeleton::DisplacementsData::new();

    let _ = displacements.displacement_set( positions, &gltf::Semantic::Positions, skin_vertices_count );
    let _ = displacements.displacement_set( normals, &gltf::Semantic::Normals, skin_vertices_count );
    let _ = displacements.displacement_set( tangents, &gltf::Semantic::Tangents, skin_vertices_count );
    if let Some( weights ) = weights
    {
      let weights_rc = displacements.morph_weights_get();
      *weights_rc.borrow_mut() = weights;
    }

    Some( displacements )
  }

  /// Loads [`Skeleton`] for one [`Mesh`]
  fn skeleton_load
  (
    skin : Option< gltf::Skin< '_ > >,
    nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > >,
    primitives_morph_targets : Option< &Vec< MorphTargets< '_ > > >,
    primitives_vertices_count : &[ usize ],
    weights : Option< Vec< f32 > >,
    buffers : &[ Vec< u8 > ]
  )
  -> Option< Rc< RefCell< Skeleton > > >
  {
    let mut skeleton = Skeleton::new();

    *skeleton.transforms_as_mut() = skin
    .and_then(| s | skeleton_transforms_data_load( &s, nodes, buffers ));
    *skeleton.displacements_as_mut() = skeleton_displacements_data_load
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

  fn light_list_get( gltf : &gltf::Gltf ) -> Option< FxHashMap< usize, Light > >
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
              strength,
              range,
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

  fn light_get( gltf_node : &gltf::Node< '_ >, node : &Node, lights : &FxHashMap< usize, Light > ) -> Option< Light >
  {
    let light_id = gltf_node.extensions()?
    .get_key_value( "KHR_lights_punctual" )?.1
    .get( "light" )?
    .as_u64()?;

    lights.get( &( light_id as usize ) ).copied()
    .map
    (
      | light |
      {
        match light
        {
          Light::Point( mut point_light ) =>
          {
            point_light.position = node.translation_get();
            Light::Point( point_light )
          },
          Light::Direct( mut direct_light ) =>
          {
            direct_light.direction = node.translation_get();
            if direct_light.direction.mag() < DIRECTION_LIGHT_MIN_MAGNITUDE
            {
              let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
              let rot_matrix = gl::math::d2::F32x3x3::from( node.rotation_get() );
              direct_light.direction = rot_matrix * forward;
            }
            direct_light.direction = direct_light.direction.normalize();
            Light::Direct( direct_light )
          },
          Light::Spot( mut spot_light ) =>
          {
            spot_light.position = node.translation_get();
            spot_light.direction = node.translation_get();
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
  /// is intentional and harmless: `url_resolve` joins both `"/buffer.bin"` and
  /// `"buffer.bin"` against the origin to the same `"{origin}/buffer.bin"`. A glTF
  /// served from a subdirectory must be loaded with that directory in `gltf_path`
  /// (e.g. `"assets/scene.gltf"`), otherwise the glTF fetch itself fails first.
  #[ must_use ]
  pub fn asset_uri_resolve( folder_path : &str, uri : &str ) -> String
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
      format!( "{folder_path}/{uri}" )
    }
  }

  /// Collects the raw byte payload of every glTF buffer : the embedded GLB
  /// binary chunk first ( when present ), then each URI-addressed buffer
  /// fetched relative to `folder_path`.
  async fn buffers_load
  (
    gltf_file : &mut gltf::Gltf,
    folder_path : &str
  )
  -> Result< Vec< gl::js_sys::Uint8Array >, gl::WebglError >
  {
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
      if let gltf::buffer::Source::Uri( uri ) = gltf_buffer.source()
      {
        let path = asset_uri_resolve( folder_path, uri );
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
      }
    }

    Ok( buffers )
  }

  /// Creates an `<img>` element for `src` and uploads it into a new WebGL
  /// texture pushed onto `images` : a 1x1 white placeholder immediately, the
  /// decoded image ( with mipmaps ) once the element's onload fires.
  fn texture_upload
  (
    document : &gl::web_sys::Document,
    gl : &gl::WebGl2RenderingContext,
    images : &Rc< RefCell< Vec< gl::web_sys::WebGlTexture > > >,
    src : &Rc< str >
  )
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
        let gl = gl.clone();
        let img = img_element.clone();
        let src = src.clone();
        move ||
        {
          gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
          gl.tex_image_2d_with_u32_and_u32_and_html_image_element
          (
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img
          ).expect( "Failed to upload data to texture" );

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
    img_element.set_src( src );
    load_texture.forget();
    on_error.forget();
  }

  /// Starts an asynchronous texture upload for every glTF image ( URI-sourced
  /// or embedded buffer view ) and returns the shared texture list the
  /// uploads fill in.
  fn images_upload
  (
    document : &gl::web_sys::Document,
    gl : &gl::WebGl2RenderingContext,
    gltf_file : &gltf::Gltf,
    folder_path : &str,
    buffers : &[ gl::js_sys::Uint8Array ]
  )
  -> Rc< RefCell< Vec< gl::web_sys::WebGlTexture > > >
  {
    let images = Rc::new( RefCell::new( Vec::new() ) );

    // If a source of an image is Uri - load the file
    // If a source of an image is View - create a blob from buffer, then turn it into an Object Url,
    // then load an image from the url
    for gltf_image in gltf_file.images()
    {
      match gltf_image.source()
      {
        gltf::image::Source::Uri { uri, mime_type: _ } =>
        {
          texture_upload( document, gl, &images, &asset_uri_resolve( folder_path, uri ).into() );
        },
        gltf::image::Source::View { view, mime_type } =>
        {
          let buffer = buffers[ view.buffer().index() ].clone();
          let buffer = gl::js_sys::Uint8Array::new_with_byte_offset_and_length( &buffer.buffer(), view.offset() as u32, view.length() as u32 );
          let blob =
          {
            let options = gl::web_sys::BlobPropertyBag::new();
            options.set_type( mime_type );

            let blob_parts = vec![ buffer ];

            gl::web_sys::Blob::new_with_u8_slice_sequence_and_options( &( blob_parts.into() ), &options )
          }.expect( "Failed to create a Blob" );

          let url = gl::web_sys::Url::create_object_url_with_blob( &blob ).expect( "Failed to create object url" );
          texture_upload( document, gl, &images, &url.into() );
        }
      }
    }

    images
  }

  /// Uploads every glTF buffer view into its own GPU buffer bound to the
  /// view's declared target ( `ARRAY_BUFFER` when absent ).
  fn gl_buffers_upload
  (
    gl : &gl::WebGl2RenderingContext,
    gltf_file : &gltf::Gltf,
    buffers : &[ gl::js_sys::Uint8Array ]
  )
  -> Result< Vec< gl::WebGlBuffer >, gl::WebglError >
  {
    let mut gl_buffers = Vec::new();
    // The target option may not be set for the attributes/indices buffers
    // This scenario should be checked
    for view in gltf_file.views()
    {
      let buffer = gl::buffer::create( gl )?;

      let target = if let Some( target ) = view.target()
      {
        match target
        {
          gltf::buffer::Target::ArrayBuffer => gl::ARRAY_BUFFER,
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

    Ok( gl_buffers )
  }

  /// Wraps every raw uploaded image in a [`Texture`] carrying its glTF
  /// sampler's filtering and wrapping modes.
  fn textures_create
  (
    gltf_file : &gltf::Gltf,
    images : &Rc< RefCell< Vec< gl::web_sys::WebGlTexture > > >
  )
  -> Vec< Rc< RefCell< Texture > > >
  {
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

      let texture = Texture::former()
      .target( gl::TEXTURE_2D )
      .source( images.borrow()[ gltf_t.source().index() ].clone() )
      .sampler( sampler )
      .form();

      textures.push( Rc::new( RefCell::new( texture ) ) );
    }

    textures
  }

  /// Builds a [`PbrMaterial`] per glTF material plus a trailing fallback for
  /// primitives without one, and seeds the per-material variation map used to
  /// share shader-define clones between primitives.
  fn materials_create
  (
    gl : &gl::WebGl2RenderingContext,
    gltf_file : &gltf::Gltf,
    textures : &[ Rc< RefCell< Texture > > ]
  )
  -> ( Vec< SharedMaterial >, FxHashMap< uuid::Uuid, Vec< SharedMaterial > > )
  {
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

    let mut materials : Vec< SharedMaterial > = Vec::new();
    let mut material_variation_map : FxHashMap< uuid::Uuid, Vec< SharedMaterial > > = FxHashMap::default();

    for gltf_m in gltf_file.materials()
    {
      let pbr = gltf_m.pbr_metallic_roughness();

      let mut material = PbrMaterial::new( gl );
      material.alpha_mode_set( match gltf_m.alpha_mode()
      {
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque
      });
      if let Some( value ) = gltf_m.alpha_cutoff() { material.alpha_cutoff = value; }
      material.base_color_factor = gl::F32x4::from( pbr.base_color_factor() );
      material.roughness_factor =  pbr.roughness_factor();
      material.metallic_factor = pbr.metallic_factor();
      material.base_color_texture_set( make_texture_info( pbr.base_color_texture() ) );
      material.metallic_roughness_texture_set( make_texture_info( pbr.metallic_roughness_texture() ) );
      material.emissive_texture_set( make_texture_info( gltf_m.emissive_texture() ) );
      material.emissive_factor = gl::F32x3::from( gltf_m.emissive_factor() );

      // KHR_materials_specular
      if let Some( s ) = gltf_m.specular()
      {
        material.specular_factor_set( Some( s.specular_factor() ) );
        material.specular_color_factor_set( Some( gl::F32x3::from( s.specular_color_factor() ) ) );
        // Specular texture
        material.specular_texture_set( make_texture_info( s.specular_texture() ) );
        // Specular color texture
        material.specular_color_texture_set( make_texture_info( s.specular_color_texture() ) );
      }

      if let Some( n ) = gltf_m.normal_texture()
      {
        material.normal_scale = n.scale();
        material.normal_texture_set( Some( TextureInfo
        {
          uv_position : n.tex_coord(),
          texture : textures[ n.texture().index() ].clone()
        }));
      }

      if let Some( o ) = gltf_m.occlusion_texture()
      {
        material.occlusion_strength = o.strength();
        material.occlusion_texture_set( Some( TextureInfo
        {
          uv_position : o.tex_coord(),
          texture : textures[ o.texture().index() ].clone()
        }));
      }

      material_variation_map.insert( material.id(), Vec::new() );
      materials.push( Rc::new( RefCell::new( Box::new( material ) ) ) );
    }

    let fallback = PbrMaterial::new( gl );
    material_variation_map.insert( fallback.id(), Vec::new() );
    materials.push( Rc::new( RefCell::new( Box::new( fallback ) ) ) );

    ( materials, material_variation_map )
  }

  /// Describes one vertex attribute over the uploaded GPU buffers from its
  /// glTF accessor : data type, offset, stride, and dimensionality.
  fn attribute_info_make
  (
    gl_buffers : &[ gl::WebGlBuffer ],
    acc : &gltf::Accessor< '_ >,
    slot : u32
  )
  -> AttributeInfo
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
      bounding_box : gl::geometry::BoundingBox::default()
    }
  }

  /// Uploads every supported vertex attribute of one glTF primitive into its
  /// [`Geometry`], registering skinning / morph shader defines on
  /// `dummy_material` as they are encountered.
  fn geometry_attributes_add
  (
    gl : &gl::WebGl2RenderingContext,
    geometry : &mut Geometry,
    gltf_primitive : &gltf::Primitive< '_ >,
    gl_buffers : &[ gl::WebGlBuffer ],
    dummy_material : &mut PbrMaterial
  )
  -> Result< (), gl::WebglError >
  {
    let mut add_define = | name : &str |
    {
      dummy_material.define_add( format!( "USE_{}", name.to_uppercase() ), String::new() );
    };

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

          let mut attr_info = attribute_info_make( gl_buffers, &acc, 0 );
          attr_info.bounding_box = BoundingBox::new( gltf_box.min, gltf_box.max );
          geometry.attribute_add( gl, "positions", attr_info )?;
        },
        gltf::Semantic::Normals =>
        {
          geometry.attribute_add( gl, "normals", attribute_info_make( gl_buffers, &acc, 1 ) )?;
        },
        gltf::Semantic::TexCoords( i ) =>
        {
          assert!( i < 5, "Only 5 types of texture coordinates are supported" );
          geometry.attribute_add
          (
            gl,
            format!( "texture_coordinates_{}", 2 + i ),
            attribute_info_make( gl_buffers, &acc, 2 + i )
          )?;
        },
        gltf::Semantic::Colors( i ) =>
        {
          assert!( i < 2, "Only 2 types of color coordinates are supported" );
          geometry.attribute_add
          (
            gl,
            format!( "colors_{}", 7 + i ),
            attribute_info_make( gl_buffers, &acc, 7 + i )
          )?;
        },
        gltf::Semantic::Tangents =>
        {
          add_define( "tangents" );
          geometry.attribute_add
          (
            gl,
            "tangents",
            attribute_info_make( gl_buffers, &acc, 9 )
          )?;
        },
        gltf::Semantic::Joints( i ) =>
        {
          let name = format!( "joints_{i}" );
          add_define( &name );
          geometry.attribute_add
          (
            gl,
            name,
            attribute_info_make( gl_buffers, &acc, 10 + i ),
          )?;
        },
        gltf::Semantic::Weights( i ) =>
        {
          let name = format!( "weights_{i}" );
          add_define( &name );
          geometry.attribute_add
          (
            gl,
            name,
            attribute_info_make( gl_buffers, &acc, 13 + i )
          )?;
        }
      }
    }

    Ok( () )
  }

  /// Builds one glTF primitive's [`Geometry`] : draw mode, indices, and
  /// vertex attributes.
  fn primitive_geometry_create
  (
    gl : &gl::WebGl2RenderingContext,
    gltf_primitive : &gltf::Primitive< '_ >,
    gl_buffers : &[ gl::WebGlBuffer ],
    dummy_material : &mut PbrMaterial
  )
  -> Result< Geometry, gl::WebglError >
  {
    let mut geometry = Geometry::new( gl )?;
    geometry.draw_mode = gltf_primitive.mode().as_gl_enum();

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
      geometry.index_add( gl, info )?;
    }

    geometry_attributes_add( gl, &mut geometry, gltf_primitive, gl_buffers, dummy_material )?;

    Ok( geometry )
  }

  /// Picks the material clone for one primitive : reuses a clone whose vertex
  /// defines match `dummy_material`'s, otherwise clones the primitive's glTF
  /// material, applies the defines, and records it in `used_materials`.
  fn primitive_material_resolve
  (
    gltf_primitive : &gltf::Primitive< '_ >,
    materials : &[ SharedMaterial ],
    material_variation_map : &FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
    used_materials : &mut Vec< SharedMaterial >,
    dummy_material : &PbrMaterial
  )
  -> SharedMaterial
  {
    let material_id = gltf_primitive.material().index().unwrap_or( materials.len() - 1 );
    let gltf_material = materials[ material_id ].clone();

    // Amongst different materials with the same uuid, find the one that has the same vertex defines
    let variation = material_variation_map
    .get( &gltf_material.borrow().id() )
    .and_then(| m |
      m.iter()
      .find( | m | m.borrow().vertex_defines_str() == dummy_material.vertex_defines_str() ))
    .cloned();

    if let Some( material ) = variation
    {
      material
    }
    else
    {
      let material = Rc::new( RefCell::new( gltf_material.borrow().dyn_clone() ) );
      let mut m = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );

      for ( name, value ) in dummy_material.vertex_defines()
      {
        m.vertex_define_add( name.clone(), value );
      }

      std::mem::drop( m );
      used_materials.push( material.clone() );

      material
    }
  }

  /// Assembles every glTF mesh from its primitives' geometry and resolved
  /// material clones.
  fn meshes_create
  (
    gl : &gl::WebGl2RenderingContext,
    gltf_file : &gltf::Gltf,
    gl_buffers : &[ gl::WebGlBuffer ],
    materials : &[ SharedMaterial ],
    material_variation_map : &FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
    used_materials : &mut Vec< SharedMaterial >
  )
  -> Result< Vec< Rc< RefCell< Mesh > > >, gl::WebglError >
  {
    let mut meshes = Vec::new();
    for gltf_mesh in gltf_file.meshes()
    {
      let mut mesh = Mesh::default();

      for gltf_primitive in gltf_mesh.primitives()
      {
        let mut dummy_material = PbrMaterial::new( gl );
        let geometry = primitive_geometry_create( gl, &gltf_primitive, gl_buffers, &mut dummy_material )?;
        let new_material = primitive_material_resolve
        (
          &gltf_primitive, materials, material_variation_map, used_materials, &dummy_material
        );

        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material : new_material
        };

        mesh.primitive_add( Rc::new( RefCell::new( primitive ) ) );
      }

      meshes.push( Rc::new( RefCell::new( mesh ) ) );
    }

    Ok( meshes )
  }

  /// A node prepared for skeleton attachment : the node, its glTF skin, its
  /// primitives' morph targets, and its mesh's morph weights.
  type RiggedNode< 'a > =
  (
    Rc< RefCell< Node > >,
    Option< gltf::Skin< 'a > >,
    Option< Vec< MorphTargets< 'a > > >,
    Option< Vec< f32 > >
  );

  /// Product of [`nodes_create`] : the flat node list, per-node
  /// skeleton-attachment data, and the nodes carrying lights.
  struct NodesCreated< 'a >
  {
    nodes : Vec< Rc< RefCell< Node > > >,
    rigged_nodes : Vec< RiggedNode< 'a > >,
    lights : Vec< Rc< RefCell< Node > > >
  }

  /// Instantiates every glTF node with its transform and object ( mesh,
  /// light, or plain ), wires the child hierarchy, and returns the flat node
  /// list, skeleton-attachment data, and the light nodes.
  fn nodes_create< 'a >
  (
    gltf_file : &'a gltf::Gltf,
    meshes : &[ Rc< RefCell< Mesh > > ]
  )
  -> NodesCreated< 'a >
  {
    let gltf_lights = light_list_get( gltf_file ).unwrap_or_default();

    let mut nodes = Vec::new();
    let mut rigged_nodes = Vec::new();
    let mut lights = Vec::new();

    for gltf_node in gltf_file.nodes()
    {
      let mut node = Node::default();
      node.visibility_set( true, true );
      let mut is_light = false;

      let ( translation, rotation, scale ) = gltf_node.transform().decomposed();
      node.scale_set( scale );
      node.translation_set( translation );
      node.rotation_set( gl::QuatF32::from( rotation ) );

      node.object = if let Some( mesh ) = gltf_node.mesh()
      {
        Object3D::Mesh( meshes[ mesh.index() ].clone() )
      }
      else if let Some( light ) = light_get( &gltf_node, &node, &gltf_lights )
      {
        is_light = true;
        Object3D::Light( light )
      }
      else
      {
        Object3D::Other
      };

      if let Some( name ) = gltf_node.name() { node.name_set( name ); }

      let node = Rc::new( RefCell::new( node ) );

      let ( primitives_morph_targets, weights ) = if let Some( mesh ) = gltf_node.mesh()
      {
        (
          Some( mesh.primitives().map( | p | p.morph_targets() ).collect::< Vec< _ > >() ),
          mesh.weights().map( <[f32]>::to_vec )
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
        node.child_add( nodes[ child.index() ].clone() );
      }
    }

    NodesCreated { nodes, rigged_nodes, lights }
  }

  /// Builds the name-to-node map and attaches a [`Skeleton`] to every rigged
  /// mesh, switching its materials onto the skinning / morph-target shader
  /// paths.
  fn skeletons_attach
  (
    nodes : &[ Rc< RefCell< Node > > ],
    rigged_nodes : Vec< RiggedNode< '_ > >,
    bin_buffers : &[ Vec< u8 > ]
  )
  {
    let nodes_map = nodes.iter()
    .filter_map
    (
      | n |
      {
        n.borrow()
        .name_get()
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
        if let Some( skeleton ) = skeleton_load
        (
          skin,
          &nodes_map,
          primitives_morph_targets.as_ref(),
          primitives_vertices_count.as_slice(),
          weights,
          bin_buffers
        )
        {
          mesh.borrow_mut().skeleton = Some( skeleton.clone() );
          for primitive in &mesh.borrow().primitives
          {
            let p = primitive.borrow();
            let mut mat_mut = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >(  p.material.borrow_mut() );

            if skeleton.borrow().has_skin()
            {
              mat_mut.define_add( "USE_SKINNING", String::new() );
            }

            if skeleton.borrow().has_morph_targets()
            {
              mat_mut.define_add( "USE_MORPH_TARGET", String::new() );
            }
          }
        }
      }
    }
  }

  /// Builds every glTF scene from the instantiated nodes and computes the
  /// initial world matrices.
  fn scenes_create
  (
    gltf_file : &gltf::Gltf,
    nodes : &[ Rc< RefCell< Node > > ]
  )
  -> Vec< Rc< RefCell< Scene > > >
  {
    let mut scenes = Vec::new();

    for gltf_scene in gltf_file.scenes()
    {
      let mut scene = Scene::default();
      for gltf_node in gltf_scene.nodes()
      {
        scene.add( nodes[ gltf_node.index() ].clone() );
      }
      scene.world_matrix_update();
      scenes.push( Rc::new( RefCell::new( scene ) ) );
    }

    scenes
  }

  /// Asynchronously loads a glTF (GL Transmission Format) file and its associated resources.
  ///
  /// # Errors
  ///
  /// Returns `WebglError` if fetching or parsing the glTF file or its buffers fails.
  ///
  /// # Panics
  ///
  /// Panics if the path is not UTF-8, or if texture creation/upload fails.
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
    gl::debug!( "Folder: {folder_path}\nFile: {gltf_path}" );

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

    let buffers = buffers_load( &mut gltf_file, folder_path ).await?;

    let bin_buffers = buffers.iter()
    .map( minwebgl::js_sys::Uint8Array::to_vec )
    .collect::< Vec< _ > >();

    gl::debug!( "Buffers: {}", buffers.len() );

    let images = images_upload( document, gl, &gltf_file, folder_path, &buffers );

    gl::debug!( "Images: {}", images.borrow().len() );

    let gl_buffers = gl_buffers_upload( gl, &gltf_file, &buffers )?;

    gl::debug!( "GL Buffers: {}", gl_buffers.len() );

    let textures = textures_create( &gltf_file, &images );

    let ( materials, material_variation_map ) = materials_create( gl, &gltf_file, &textures );
    let mut used_materials : Vec< SharedMaterial > = Vec::new();

    gl::debug!( "PbrMaterials: {}",materials.len() );
    let meshes = meshes_create
    (
      gl, &gltf_file, &gl_buffers, &materials, &material_variation_map, &mut used_materials
    )?;

    gl::debug!( "Meshes: {}",meshes.len() );

    let NodesCreated { nodes, rigged_nodes, lights } = nodes_create( &gltf_file, &meshes );

    gl::debug!( "Nodes: {}", nodes.len() );

    skeletons_attach( &nodes, rigged_nodes, &bin_buffers );

    #[ cfg( feature = "animation" ) ]
    let animations = crate::webgl::animation::loaders::gltf::load( gl, &gltf_file, bin_buffers.as_slice(), nodes.as_slice() ).await;

    #[ cfg( feature = "animation" ) ]
    gl::debug!( "Animations: {}", animations.len() );

    let scenes = scenes_create( &gltf_file, &nodes );

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
    load,
    asset_uri_resolve
  };
}
