mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::
  {
    JsCast,
    geometry::BoundingBox,
  };
  use crate::webgl::
  {
    ToFromGlEnum,
    AlphaMode,
    AttributeInfo,
    Geometry,
    IndexInfo,
    MagFilterMode,
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
    WrappingMode
  };
  use web_sys::wasm_bindgen::prelude::Closure;

  use
  {
    std::collections::HashMap,
    crate::webgl::Skeleton,
    gl::
    {
      GL,
      F32x4x4
    }
  };

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
    /// A collection of `Material` objects, defining how the surfaces of the meshes should be shaded.
    pub materials : Vec< Rc< RefCell< Material > > >,
    /// A list of `Mesh` objects, which represent the geometry of the scene.
    pub meshes : Vec< Rc< RefCell< Mesh > > >,
    /// A list of `Animation` objects, which store `Node`'s tranform change in every time moment.
    #[ cfg( feature = "animation" ) ]
    pub animations : Vec< Animation >
  }

  /// Asynchronously loads [`Skeleton`] for one [`Mesh`]
  async fn load_skeleton< 'a >
  (
    gl : &GL,
    skin : gltf::Skin< '_ >,
    nodes : &HashMap< Box< str >, Rc< RefCell< Node > > >,
    buffers : &'a [ Vec< u8 > ],
  )
  -> Option< Rc< RefCell< Skeleton > > >
  {
    let reader = skin.reader
    (
      | buffer | Some( buffers[ buffer.index() ].as_slice() )
    );

    let Some( inverse_bind_matrices_iter ) = reader.read_inverse_bind_matrices()
    else
    {
      return None;
    };

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
          .into_iter()
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

    // gl::info!( "Joints: {:?}", joints );

    let skeleton = Skeleton::new( gl, joints )
    .map( | s | Rc::new( RefCell::new( s ) ) );

    skeleton
  }

  /// Asynchronously loads a glTF (GL Transmission Format) file and its associated resources.
  pub async fn load
  (
    document : &gl::web_sys::Document,
    gltf_path : &str,
    gl : &gl::WebGl2RenderingContext
  ) -> Result< GLTF, gl::WebglError >
  {
    let path = std::path::Path::new( gltf_path );
    let folder_path = path.parent().map_or( "", | p | p.to_str().expect( "Path is not UTF-8 encoded" ) );
    gl::info!( "Folder: {}\nFile: {}", folder_path, gltf_path );

    // let gltf_slice= gl::file::load( &format!( "{}/scene.gltf", gltf_path ) )
    // .await.expect( "Failed to load gltf file" );
    let gltf_slice = gl::file::load( gltf_path ).await.expect( "Failed to load gltf file" );
    let mut gltf_file = gltf::Gltf::from_slice( &gltf_slice ).unwrap();

    let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();

    // Move the GLB bin into buffers
    if let Some( blob ) = gltf_file.blob.as_mut()
    {
      let blob = std::mem::take( blob );
      gl::log::info!( "The gltf binary payload is present: {}", blob.len() );
      buffers.push( blob.as_slice().into() );
    }

    for gltf_buffer in gltf_file.buffers()
    {
      match gltf_buffer.source()
      {
        gltf::buffer::Source::Uri( uri ) =>
        {
          let path = format!( "{}/{}", folder_path, uri );
          let buffer = gl::file::load( &path ).await
          .expect( "Failed to load a buffer" );

          gl::log::info!
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

    gl::info!( "Bufffers: {}", buffers.len() );

    // Upload images
    let images = Rc::new( RefCell::new( Vec::new() ) );

    // Creates an <img> html elements, and sets its src property to 'src' parameter
    // When the image is loaded, creates a texture and adds it to the 'images' array
    let upload_texture = | src : Rc< String > | {
      let texture = gl.create_texture().expect( "Failed to create a texture" );
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

            //match
            gl::web_sys::Url::revoke_object_url( &src ).unwrap();
            // {
            //   Ok( _ ) => { gl::info!( "Remove object url: {}", &src ) },
            //   Err( _ ) => { gl::info!( "Not an object url: {}", &src ) }
            // }

            img.remove();
          }
        }
      );

      img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
      img_element.set_src( &src );
      load_texture.forget();
    };

    // If a source of an image is Uri - load the file
    // If a source of an image is View - create a blob from buffer, then turn it into an Object Url,
    // then load an image from the url
    for gltf_image in gltf_file.images()
    {
      match  gltf_image.source()
      {
        gltf::image::Source::Uri { uri, mime_type: _ } =>
        {
          upload_texture( Rc::new( format!( "static/{}/{}", folder_path, uri ) ) );
        },
        gltf::image::Source::View { view, mime_type } =>
        {
          let buffer = buffers[ view.buffer().index() ].clone();
          let buffer = gl::js_sys::Uint8Array::new_with_byte_offset_and_length( &buffer.buffer(), view.offset() as u32, view.length() as u32 );
          let blob = {
            let options = gl::web_sys::BlobPropertyBag::new();
            options.set_type( mime_type );

            let mut blob_parts = Vec::new();
            blob_parts.push( buffer );

            gl::web_sys::Blob::new_with_u8_slice_sequence_and_options( &( blob_parts.into() ), &options )
          }.expect( "Failed to create a Blob" );

          let url = gl::web_sys::Url::create_object_url_with_blob( &blob ).expect( "Failed to create object url" );
          upload_texture( Rc::new( url ) );
        }
      }
    }

    gl::info!( "Images: {}", images.borrow().len() );

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

    gl::info!( "GL Buffers: {}", gl_buffers.len() );

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

      let texture = Texture::former()
      .target( gl::TEXTURE_2D )
      .source( images.borrow()[ gltf_t.source().index() ].clone() )
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

    let mut materials = Vec::new();
    for gltf_m in gltf_file.materials()
    {
      let pbr = gltf_m.pbr_metallic_roughness();

      let mut material = Material::default();
      material.alpha_mode = match gltf_m.alpha_mode()
      {
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask,
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque
      };
      if let Some( value ) = gltf_m.alpha_cutoff() { material.alpha_cutoff = value; }
      material.base_color_factor = gl::F32x4::from( pbr.base_color_factor() );
      material.roughness_factor =  pbr.roughness_factor();
      material.metallic_factor = pbr.metallic_factor();
      material.base_color_texture = make_texture_info( pbr.base_color_texture() );
      material.metallic_roughness_texture = make_texture_info( pbr.metallic_roughness_texture() );
      material.emissive_texture = make_texture_info( gltf_m.emissive_texture() );
      material.emissive_factor = gl::F32x3::from( gltf_m.emissive_factor() );

      // KHR_materials_specular
      if let Some( s ) = gltf_m.specular()
      {
        material.specular_factor = Some( s.specular_factor() );
        material.specular_color_factor = Some( gl::F32x3::from( s.specular_color_factor() ) );
        // Specular texture
        material.specular_texture = make_texture_info( s.specular_texture() );
        // Specular color texture
        material.specular_color_texture = make_texture_info( s.specular_color_texture() );
      }

      if let Some( n ) = gltf_m.normal_texture()
      {
        material.normal_scale = n.scale();
        material.normal_texture = Some( TextureInfo
        {
          uv_position : n.tex_coord(),
          texture : textures[ n.texture().index() ].clone()
        });
      }

      if let Some( o ) = gltf_m.occlusion_texture()
      {
        material.occlusion_strength = o.strength();
        material.occlusion_texture = Some( TextureInfo
        {
          uv_position : o.tex_coord(),
          texture : textures[ o.texture().index() ].clone()
        });
      }

      materials.push( Rc::new( RefCell::new( material ) ) );
    }

    materials.push( Rc::new( RefCell::new( Material::default() ) ) );

    gl::log::info!( "Materials: {}",materials.len() );
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
          gl::log::info!( "Sparce accessors are not supported yet" );
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
            geometry.add_attribute( gl, "positions", attr_info, false )?;
          },
          gltf::Semantic::Normals =>
          {
            geometry.add_attribute( gl, "normals", make_attibute_info( &acc, 1 ), false )?;
          },
          gltf::Semantic::TexCoords( i ) =>
          {
            assert!( i < 5, "Only 5 types of texture coordinates are supported" );
            geometry.add_attribute
            (
              gl,
              format!( "texture_coordinates_{}", 2 + i ),
              make_attibute_info( &acc, 2 + i ),
              false
            )?;
          },
          gltf::Semantic::Colors( i ) =>
          {
            assert!( i < 2, "Only 2 types of color coordinates are supported" );
            geometry.add_attribute
            (
              gl,
              format!( "colors_{}", 7 + i ),
              make_attibute_info( &acc, 7 + i ),
              false
            )?;
          },
          gltf::Semantic::Tangents =>
          {
            geometry.add_attribute
            (
              gl,
              "tangents",
              make_attibute_info( &acc, 9 ),
              true
            )?;
          },
          gltf::Semantic::Joints( i ) =>
          {
            geometry.add_attribute
            (
              gl,
              format!( "joints_{}", i ),
              make_attibute_info( &acc, 10 + i ),
              true
            )?;
          },
          gltf::Semantic::Weights( i ) =>
          {
            geometry.add_attribute
            (
              gl,
              format!( "weights_{}", i ),
              make_attibute_info( &acc, 13 + i ),
              true
            )?;
          },
          //a => { gl::warn!( "Unsupported attribute: {:?}", a ); continue; }
        };
      }

      let material_id = gltf_primitive.material().index().unwrap_or( materials.len() - 1 );
      let primitive = Primitive
      {
        geometry : Rc::new( RefCell::new( geometry ) ),
        material : materials[ material_id ].clone()
      };

      mesh.add_primitive( Rc::new( RefCell::new( primitive ) ) );
    }

    meshes.push( Rc::new( RefCell::new( mesh ) ) );
    }

    gl::log::info!( "Meshes: {}",meshes.len() );

    let mut nodes = Vec::new();
    let mut skinned_nodes = Vec::new();

    for gltf_node in gltf_file.nodes()
    {
      let mut node = Node::default();

      node.object = if let Some( mesh ) = gltf_node.mesh()
      {
        Object3D::Mesh( meshes[ mesh.index() ].clone() )
      }
      else
      {
        Object3D::Other
      };

      let ( translation, rotation, scale ) = gltf_node.transform().decomposed();
      node.set_scale( scale );
      node.set_translation( translation );
      node.set_rotation( gl::QuatF32::from( rotation ) );
      if let Some( name ) = gltf_node.name() { node.set_name( name ); }

      let node = Rc::new( RefCell::new( node ) );

      if let Some( skin ) = gltf_node.skin()
      {
        skinned_nodes.push( ( node.clone(), skin ) );
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

    gl::log::info!( "Nodes: {}", nodes.len() );

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
    .collect::< HashMap< _, _ > >();

    for ( node, skin ) in skinned_nodes
    {
      if let Object3D::Mesh( mesh ) = &node.borrow().object
      {
        if let Some( skeleton ) = load_skeleton
        (
          gl,
          skin,
          &nodes_map,
          bin_buffers.as_slice()
        )
        .await
        {
          mesh.borrow_mut().skeleton = Some( skeleton );
          for primitive in &mesh.borrow().primitives
          {
            primitive.borrow()
            .geometry.borrow_mut()
            .defines += "#define USE_SKINNING\n";
          }
        }
      }
    }

    #[ cfg( feature = "animation" ) ]
    let animations = crate::webgl::animation::load( &gltf_file, bin_buffers.as_slice(), nodes.as_slice() ).await;

    #[ cfg( feature = "animation" ) ]
    gl::log::info!( "Animations: {}", animations.len() );

    let mut scenes = Vec::new();

    for gltf_scene in gltf_file.scenes()
    {
      let mut scene = Scene::default();
      for gltf_node in gltf_scene.nodes()
      {
        scene.add( nodes[ gltf_node.index() ].clone() );
      }
      scenes.push(  Rc::new( RefCell::new( scene ) ) );
    }

    Ok
    (
      GLTF
      {
        scenes,
        nodes,
        gl_buffers,
        images,
        textures,
        materials,
        meshes,
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
