use std::{ cell::RefCell, rc::Rc };

use minwebgl::{ self as gl, JsCast };
use renderer::webgl::
{
  AlphaMode, 
  AttributeInfo, 
  BoundingBox, 
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

pub async fn load
( 
  document : &gl::web_sys::Document,
  gltf_path : &str, 
  gl : &gl::WebGl2RenderingContext 
) -> Result< Vec< Scene >, gl::WebglError >
{
  let gltf_slice= gl::file::load( &format!( "{}/scene.gltf", gltf_path ) )
  .await.expect( "Failed to load gltf file" );
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
        let path = format!( "{}/{}", gltf_path, uri );
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

  gl::info!( "Bufffers: {}", buffers.len() );

  // Upload images
  let images = Rc::new( RefCell::new( Vec::new() ) );

  // Creates an <img> html elements, and sets its src property to 'src' parameter
  // When the image is loaded, createa a texture and adds it to the 'images' array
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
        upload_texture( Rc::new( format!( "static/{}/{}", gltf_path, uri ) ) );
      },
      gltf::image::Source::View { view, mime_type } =>
      {
        let buffer = buffers[ view.buffer().index() ].clone();
        let blob = {
          let options = gl::web_sys::BlobPropertyBag::new();
          options.set_type( mime_type );

          let mut blob_parts = Vec::new();
          blob_parts.push( buffer );

          gl::web_sys::Blob::new_with_u8_array_sequence_and_options( &( blob_parts.into() ), &options )
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
  let make_texture_info = | info : Option< gltf::texture::Info > |
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
    material.alpha_cutoff = gltf_m.alpha_cutoff();
    material.base_color_factor = gl::F32x4::from( pbr.base_color_factor() );
    material.roughness_factor = Some( pbr.roughness_factor() );
    material.metallic_factor = Some( pbr.metallic_factor() );
    material.base_color_texture = make_texture_info( pbr.base_color_texture() );
    material.metallic_roughness_texture = make_texture_info( pbr.metallic_roughness_texture() );
    material.emissive_texture = make_texture_info( gltf_m.emissive_texture() );
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
      material.normal_scale = Some( n.scale() );
      material.normal_texture = Some( TextureInfo
      {
        uv_position : n.tex_coord(),
        texture : textures[ n.texture().index() ].clone()
      });
    }

    if let Some( o ) = gltf_m.occlusion_texture()
    {
      material.occlusion_strength = Some( o.strength() );
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
  let make_attibute_info = | acc : &gltf::Accessor, slot |
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
            &format!( "texture_coordinates_{}", 2 + i ), 
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
            &format!( "colors_{}", 7 + i ), 
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
        a => { gl::warn!( "Unsupported attribute: {:?}", a ); continue; }
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

    match gltf_node.transform()
    {
      gltf::scene::Transform::Matrix { matrix } =>
      {
        node.matrix = gl::F32x4x4::from_column_major( glam::Mat4::from_cols_array_2d( &matrix ).to_cols_array() );
        let mat = glam::Mat4::from_cols_array_2d( &matrix );
        let ( s, r, t ) = mat.to_scale_rotation_translation();

        node.scale = s.to_array().into();
        node.translation = t.to_array().into();
        node.rotation = r;
      },
      gltf::scene::Transform::Decomposed { translation, rotation, scale } =>
      {
        node.scale = scale.into();
        node.translation = translation.into();
        node.rotation = glam::Quat::from_array( rotation );

        let mat = glam::Mat4::from_scale_rotation_translation( scale.into(), glam::Quat::from_array( rotation ), translation.into() );
        node.matrix = gl::F32x4x4::from_column_major( mat.to_cols_array() );
      }
    }
    
    nodes.push( Rc::new( RefCell::new( node ) ) );
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

  let mut scenes = Vec::new();

  for gltf_scene in gltf_file.scenes()
  {
    let mut scene = Scene::default();
    for gltf_node in gltf_scene.nodes()
    {
      scene.add( nodes[ gltf_node.index() ].clone() );
    }
    scenes.push( scene );
  }

  Ok( scenes )
}