mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use crate::webgl::loaders::gltf_extensions::{ document_validate, material_layer_extensions_apply };
  use crate::webgl::loaders::gltf_buffers::{ buffers_load, cpu_buffers_needed };
  use crate::webgl::loaders::gltf_quantization::position_bounds_dequantize;
  use minwebgl as gl;
  use gl::
  {
    JsCast,
    geometry::BoundingBox,
  };
  use crate::webgl::
  {
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
    Primitive,
    Sampler,
    Scene,
    Texture,
    TextureInfo,
    ToFromGlEnum,
    WrappingMode,
    helpers
  };
  use web_sys::wasm_bindgen::prelude::Closure;

  use rustc_hash::FxHashMap;

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
  pub type SharedMaterial = Rc< RefCell< Box< dyn Material > > >;

  // The skin and morph target readers live in `gltf_skeleton`, the light readers in
  // `gltf_lights`; their public functions are re-exported here, where they have always been.
  pub use crate::webgl::loaders::gltf_skeleton::{ skeleton_transforms_data_load, skeleton_displacements_data_load };
  pub use crate::webgl::loaders::gltf_lights::{ light_list_get, light_get };
  // So does the node and scene assembly, in `gltf_scene`.
  pub use crate::webgl::loaders::gltf_scene::{ RiggedNode, NodesCreated, nodes_create, skeletons_attach, scenes_create };

  // Lives in `gltf_uri` so that `gltf_buffers` can use it without importing this module, which
  // imports `gltf_buffers`; re-exported here, where it has always been.
  pub use crate::webgl::loaders::gltf_uri::asset_uri_resolve;

  std::thread_local!
  {
    static TEXTURE_UPLOAD_COUNT : std::cell::Cell< u64 > = const { std::cell::Cell::new( 0 ) };
  }

  /// Number of glTF images whose decoded pixels have replaced their 1x1 white
  /// placeholder, counted across every [`load`] since the page started.
  ///
  /// Image uploads finish asynchronously, after `load` has returned, and nothing
  /// else signals them. A renderer that skips redrawing unchanged frames should
  /// redraw whenever this value moves : otherwise the frame drawn with the white
  /// placeholders stays on screen until something unrelated changes, and a
  /// textured model shows up blank. Compare against the value seen last frame;
  /// reading it is one thread-local load.
  #[ must_use ]
  pub fn texture_upload_count_get() -> u64
  {
    TEXTURE_UPLOAD_COUNT.with( std::cell::Cell::get )
  }

  /// Creates an `<img>` element for `src` and uploads it into a new WebGL
  /// texture pushed onto `images` : a 1x1 white placeholder immediately, the
  /// decoded image ( with mipmaps ) once the element's onload fires, which bumps
  /// [`texture_upload_count_get`].
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
          TEXTURE_UPLOAD_COUNT.with( | count | count.set( count.get() + 1 ) );

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

      material_layer_extensions_apply( &gltf_m, textures, &mut material );

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

  /// Computes a vertex attribute's [`gl::BufferDescriptor`] from its glTF
  /// accessor : data type, offset, stride, and dimensionality. Split out of
  /// [`attribute_info_make`] -- pure data transform over the accessor's own
  /// metadata, no GL calls -- so it can be tested without a live
  /// `WebGl2RenderingContext`.
  ///
  /// # Panics
  ///
  /// Panics if `acc`'s buffer view is absent -- every accessor this loader
  /// resolves ( via `gltf_primitive.attributes()` ) is backed by a view
  /// pointing at real buffer bytes, never a sparse-only accessor.
  #[ must_use ]
  pub fn attribute_descriptor_make( acc : &gltf::Accessor< '_ > ) -> gl::BufferDescriptor
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

    gl::BufferDescriptor::new::< [ f32; 1 ] >()
    .offset( acc.offset() as i32 / data_type.byte_size() )
    .normalized( acc.normalized() )
    .stride( acc.view().unwrap().stride().unwrap_or( 0 ) as i32 / data_type.byte_size() )
    .vector( gl::VectorDataType::new( data_type, acc.dimensions().multiplicity() as i32, 1 ) )
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
    AttributeInfo
    {
      slot,
      buffer : gl_buffers[ acc.view().unwrap().index() ].clone(),
      descriptor : attribute_descriptor_make( acc ),
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
          let gltf_box = position_bounds_dequantize
          (
            gltf_primitive.bounding_box(),
            acc.data_type(),
            acc.normalized()
          );

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

  /// Looks up an existing material variation under `material_id` whose
  /// `vertex_defines_str()` matches `vertex_defines_str`, returning a shared clone of it;
  /// if none matches, calls `new_material` to build one, records it under `material_id`
  /// for future lookups, and returns it.
  // Fix(BUG-245): this lookup-and-insert pairing used to be split apart inside
  // `primitive_material_resolve`, with the insert half silently missing -- the map was passed by
  // shared reference, so it had no way to be written to at all, and every entry
  // `materials_create` seeded as an empty `Vec::new()` stayed empty forever. Every primitive
  // sharing a glTF material + vertex-defines combination therefore got its own independent clone
  // instead of the intended shared instance. Root cause: a cache that is only ever read, never
  // written, is not a cache. Extracted into its own function, taking `material_variation_map` as
  // `&mut` and inserting immediately after construction, so the lookup and its matching insert
  // can no longer drift apart -- and so this pairing is unit-testable on its own, independent of
  // the glTF-parsing and GPU-material-construction context around it.
  #[ must_use ]
  #[ expect( clippy::implicit_hasher, reason = "this loader uses FxHashMap exclusively, everywhere -- \
    genericizing over BuildHasher here would be speculative, never-exercised generality" ) ]
  pub fn material_variation_resolve
  (
    material_variation_map : &mut FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
    material_id : uuid::Uuid,
    vertex_defines_str : &str,
    new_material : impl FnOnce() -> SharedMaterial
  )
  -> SharedMaterial
  {
    // Amongst different materials with the same uuid, find the one that has the same vertex defines
    let variation = material_variation_map
    .get( &material_id )
    .and_then( | m | m.iter().find( | m | m.borrow().vertex_defines_str() == vertex_defines_str ) )
    .cloned();

    if let Some( existing ) = variation
    {
      existing
    }
    else
    {
      let material = new_material();
      material_variation_map.entry( material_id ).or_default().push( material.clone() );
      material
    }
  }

  /// Picks the material clone for one primitive : reuses a clone whose vertex
  /// defines match `dummy_material`'s, otherwise clones the primitive's glTF
  /// material, applies the defines, and records it via [`material_variation_resolve`].
  fn primitive_material_resolve
  (
    gltf_primitive : &gltf::Primitive< '_ >,
    materials : &[ SharedMaterial ],
    material_variation_map : &mut FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
    used_materials : &mut Vec< SharedMaterial >,
    dummy_material : &PbrMaterial
  )
  -> SharedMaterial
  {
    let material_id = gltf_primitive.material().index().unwrap_or( materials.len() - 1 );
    let gltf_material = materials[ material_id ].clone();
    let gltf_material_id = gltf_material.borrow().id();

    material_variation_resolve
    (
      material_variation_map,
      gltf_material_id,
      dummy_material.vertex_defines_str(),
      ||
      {
        let material = Rc::new( RefCell::new( gltf_material.borrow().dyn_clone() ) );
        let mut m = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );

        for ( name, value ) in dummy_material.vertex_defines()
        {
          m.vertex_define_add( name.clone(), value );
        }

        // Fix(BUG-535): attribute defines are needed in the fragment stage too: `main.frag` builds
        // its tangent frame from `vTangent` only under `USE_TANGENTS`, so every asset that ships
        // TANGENT data was shaded with the derivative frame instead of its authored tangents.
        // Root cause: only `dummy_material.vertex_defines()` was copied. Master never copied the
        // fragment defines, and this loop, once added on the branch, was lost in the merge 7a37bd43.
        // Pitfall: the two stages compile from separate define sets, so a define recorded for both
        // must be copied to both.
        for ( name, value ) in dummy_material.fragment_defines()
        {
          m.fragment_define_add( name.clone(), value );
        }

        std::mem::drop( m );
        used_materials.push( material.clone() );

        material
      }
    )
  }

  /// Assembles every glTF mesh from its primitives' geometry and resolved
  /// material clones.
  fn meshes_create
  (
    gl : &gl::WebGl2RenderingContext,
    gltf_file : &gltf::Gltf,
    gl_buffers : &[ gl::WebGlBuffer ],
    materials : &[ SharedMaterial ],
    material_variation_map : &mut FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
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

  /// Fetches, parses and validates the glTF file at `gltf_path` : the first stage of [`load`].
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other`, with the details on the console, when the fetch or the parse
  /// fails, or the asset fails [`document_validate`].
  async fn document_read( gltf_path : &str ) -> Result< gltf::Gltf, gl::WebglError >
  {
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
    // Parsed without `gltf`'s own validation, which would reject any required extension
    // outside `gltf-json`'s typed list ( see `document_validate` ); `document_validate` runs
    // the same checks with this loader's extension rule right below.
    let gltf_file = gltf::Gltf::from_slice_without_validation( &gltf_slice )
    .map_err( | e |
    {
      gl::browser::error!( "Failed to parse gltf file '{gltf_path}': {e}" );
      gl::WebglError::Other( "Failed to parse gltf file" )
    } )?;

    // Per glTF 2.0's "Specifying Extensions", a conformant client MUST refuse to
    // load an asset whose `extensionsRequired` names an extension it doesn't
    // support. Checked immediately after parsing, before any buffer/image/GL work.
    document_validate( &gltf_file )
    .inspect_err( | _ | gl::browser::error!( "Invalid gltf file '{gltf_path}'" ) )?;
    Ok( gltf_file )
  }

  /// The buffer stage of [`load`] : every buffer's bytes as JS arrays, with the meshopt views
  /// decoded into them, and a copy in wasm memory when a CPU-side reader needs one (
  /// [`cpu_buffers_needed`] ), empty otherwise.
  ///
  /// # Errors
  ///
  /// Returns what `buffers_load` and `meshopt::views_decode` return.
  async fn buffers_prepare
  (
    gltf_file : &mut gltf::Gltf,
    folder_path : &str
  )
  -> Result< ( Vec< gl::js_sys::Uint8Array >, Vec< Vec< u8 > > ), gl::WebglError >
  {
    let buffers = buffers_load( gltf_file, folder_path ).await?;
    crate::webgl::loaders::meshopt::views_decode( gltf_file, &buffers ).await?;

    let bin_buffers = if cpu_buffers_needed( gltf_file )
    {
      buffers.iter().map( gl::js_sys::Uint8Array::to_vec ).collect()
    }
    else
    {
      Vec::new()
    };
    Ok( ( buffers, bin_buffers ) )
  }

  /// Asynchronously loads a glTF (GL Transmission Format) file and its associated resources.
  ///
  /// # Errors
  ///
  /// Returns `WebglError`, with the details on the console, when :
  /// - fetching or parsing the glTF file fails;
  /// - the asset fails validation, including an `extensionsRequired` entry this loader doesn't
  ///   support ( see [`document_validate`] );
  /// - a buffer cannot be loaded : a fetch fails, a buffer other than buffer 0 has no uri,
  ///   buffer 0 has no uri and the file no GLB binary chunk, or the meshopt fallback buffers
  ///   declare more bytes together than the loader allocates for one document;
  /// - an uncompressed buffer view reaches past the end of its buffer;
  /// - meshopt decoding fails, for any reason `meshopt::views_decode` lists : a buffer view or
  ///   buffer breaking a meshopt rule, overlapping views, the decoder failing to initialise, or
  ///   the decoder rejecting the data.
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

    let mut gltf_file = document_read( gltf_path ).await?;
    let ( buffers, bin_buffers ) = buffers_prepare( &mut gltf_file, folder_path ).await?;

    gl::debug!( "Buffers: {}", buffers.len() );

    let images = images_upload( document, gl, &gltf_file, folder_path, &buffers );

    gl::debug!( "Images: {}", images.borrow().len() );

    let gl_buffers = gl_buffers_upload( gl, &gltf_file, &buffers )?;

    gl::debug!( "GL Buffers: {}", gl_buffers.len() );

    let textures = textures_create( &gltf_file, &images );

    let ( materials, mut material_variation_map ) = materials_create( gl, &gltf_file, &textures );
    let mut used_materials : Vec< SharedMaterial > = Vec::new();

    gl::debug!( "PbrMaterials: {}",materials.len() );
    let meshes = meshes_create
    (
      gl, &gltf_file, &gl_buffers, &materials, &mut material_variation_map, &mut used_materials
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
    texture_upload_count_get,
    asset_uri_resolve,
    light_list_get,
    light_get,
    skeleton_transforms_data_load,
    skeleton_displacements_data_load,
    material_variation_resolve,
    SharedMaterial,
    RiggedNode,
    NodesCreated,
    nodes_create,
    skeletons_attach,
    scenes_create,
    attribute_descriptor_make
  };
}
