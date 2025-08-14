//!

mod private
{
  use bytemuck::cast_slice;
  use minwebgl as gl;

  use gl::F32x4x4;
  use std::{ cell::RefCell, rc::Rc };
  use renderer::webgl::
  {
    Object3D,
    Node,
    Scene
  };

  pub struct Skeletons
  {
    gl_buffers : Vec< WebGlBuffer >, 
    pub Skeletons : Vec< Skeleton >,
    pub animations : Vec< Sequencer >
  }

  impl Skeletons
  {
    fn new() -> Self
    {
      Self
      {
        gl_buffers : vec![],
        skeletons : vec![]
      }
    }
  }

  // Scene analog
  pub struct Skeleton
  {
    // Contains global matrix, local matrix and joints, weights attributes
    root : Rc< RefCell< Node > >,
    inverse_bind_matrices :  Vec< F32x4x4 >, //HashMap< String, F32x4x4 >
    offset : usize
  }

  impl Skeleton
  {
    fn a( : Rc< RefCell< Node > > )
    {

    }
  }

  pub async fn load
  (
    document : &gl::web_sys::Document,
    gltf_path : &str,
    gl : &gl::WebGl2RenderingContext
  ) 
  -> Result< Skeletons, gl::WebglError >
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

          buffers.push( buffer.as_slice().into() );
        },
        _ => {}
      }
    }

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
            gltf::Semantic::Joints( i ) =>
            {
              geometry.add_attribute
              (
                gl,
                format!( "joints_{}", i ),
                make_attibute_info( &acc, 10 + i ),
                false
              )?;
            },
            gltf::Semantic::Weights( i ) =>
            {
              geometry.add_attribute
              (
                gl,
                format!( "weights_{}", i ),
                make_attibute_info( &acc, 13 + i ),
                false
              )?;
            },
            _ => continue
          };
        }

        let primitive = Primitive
        {
          geometry : Rc::new( RefCell::new( geometry ) ),
          material : materials[ material_id ].clone()
        };

        mesh.add_primitive( Rc::new( RefCell::new( primitive ) ) );
      }

      meshes.push( Rc::new( RefCell::new( mesh ) ) );
    }

    let mut nodes = Vec::new();
    let mut skeletons = Vec::new();
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
        if let Some( acc ) = skin.inverse_bind_matrices()
        {
          let mut matrices = vec![];

          let view = acc.view();
          let gltf_buffer = view.buffer();
          let mut matrix_buffer_slice : Option< &[ [ f32; 16 ] ] > = None;
          match gltf_buffer.source()
          {
            gltf::buffer::Source::Uri( uri ) =>
            {
              let path = format!( "{}/{}", folder_path, uri );
              let buffer = gl::file::load( &path ).await
              .expect( "Failed to load a buffer" );

              let vl = view.length();
              let vo = view.offset();
              let ac = acc.count();
              let ao = acc.offset();

              matrix_buffer_slice : &[ [ f32; 16 ] ] = Some
              ( 
                cast_slice( buffer.as_slice()[ vo..( vo + vl ) ][ ao..( ao + ( 16 * 4 * ac ) ) ] ) 
              );
            },
            _ => {}
          }

          if let Some( slice ) = matrix_buffer_slice
          {
            matrices = slice.into_iter()
            .map( | array | F32x4x4::from_column_major( array ) )
            .collect::< Vec< _ > >();
          }

          // let mut name_to_matrix = HashMap::new();
          // let mut inverse_bind_matrices = HashMap::new();
          // for ( joint, matrix ) in skin.joints().zip( matrices )
          // {
          //   if let Some( name ) = joint.name()
          //   {  
          //     inverse_bind_matrices.insert( name.to_string(), matrix );
          //   }
          // }

          let skeleton = Skeleton
          {
            root : node.clone(),
            inverse_bind_matrices : matrices,
            offset : skin.joints()
          }
        }
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

    let mut skeletons = Skeletons::new();

    for gltf_scene in gltf_file.scenes()
    {
      let mut scene = Scene::default();
      for gltf_node in gltf_scene.nodes()
      {
        scene.add( nodes[ gltf_node.index() ].clone() );
      }
      scenes.push(  Rc::new( RefCell::new( scene ) ) );
    }

    Ok( skeletons )
  }
}

crate::mod_interface!
{
  orphan use
  {
    
  };
}