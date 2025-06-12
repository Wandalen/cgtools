mod private
{
  use std::{ cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc };
  use mingl::{ AsBytes, VectorDataType };
  use minwebgl as gl;
  use web_sys::{ WebGlTexture, WebGlBuffer, WebGlFramebuffer, WebGlUniformLocation, WebGlVertexArrayObject };
  use gl::{ GL, drawbuffers::drawbuffers };
  use crate::webgl::
  { 
    loaders::gltf::GLTF, AttributeInfo, program, Camera, Node, Object3D, ProgramInfo, Scene
  };
  use core::any::type_name_of_val;

  /// The source code for the gbuffer vertex shader.
  const GBUFFER_VERTEX_SHADER : &'static str = include_str!( "../shaders/post_processing/gbuffer.vert" );
  /// The source code for the gbuffer fragment shader.
  const GBUFFER_FRAGMENT_SHADER : &'static str = include_str!( "../shaders/post_processing/gbuffer.frag" );

  pub const ALL : [ GBufferAttachment; 6 ] = [
    GBufferAttachment::Position,
    GBufferAttachment::Color,
    GBufferAttachment::Albedo,
    GBufferAttachment::Normal,
    GBufferAttachment::PbrInfo,
    GBufferAttachment::ObjectColorId
  ];

  #[ derive( Debug, Copy, Clone, Eq, PartialEq, Hash ) ]
  pub enum GBufferAttachment
  {
    Position,
    Color,
    Albedo,
    Normal,
    PbrInfo,
    ObjectColorId
  }

  impl GBufferAttachment
  {
    fn define_const( &self ) -> String
    {
      match self 
      {
        GBufferAttachment::Position => "POSITION",
        GBufferAttachment::Color => "COLOR",
        GBufferAttachment::Albedo => "ALBEDO",
        GBufferAttachment::Normal => "NORMAL",
        GBufferAttachment::PbrInfo => "PBR_INFO",
        GBufferAttachment::ObjectColorId => "OBJECT_COLOR_ID",
      }
      .to_string()
    }
  }

  fn into_defines( attachments : &HashSet< GBufferAttachment > ) -> String
  {
    let mut defines = String::new();

    for attachment in attachments
    {
      defines = defines + "#define " + &attachment.define_const() + "\n";
    }

    defines
  }

  /// Binds a texture to a texture unit and uploads its location to a uniform.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `texture` - The texture to bind.
  /// * `location` - The uniform location in the shader for the sampler.
  /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  fn upload_texture
  (
    gl : &gl::WebGl2RenderingContext,
    texture : &WebGlTexture,
    location : &WebGlUniformLocation,
    slot : u32,
  )
  {
    gl.active_texture( slot ); 
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) ); 
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
  }

  pub fn upload_buffer_data
  ( 
    gl : &gl::WebGl2RenderingContext, 
    buffer : &WebGlBuffer, 
    target : u32, 
    offset : u32, 
    data : Vec< u8 > 
  ) 
  {
    let data = data.into_iter()
    .collect::< Vec< _ > >();

    gl.bind_buffer( target, Some( buffer ) );
    gl.buffer_data_with_js_u8_array_and_src_offset_and_length
    ( 
      target, 
      &gl::js_sys::Uint8Array::from( data.as_bytes() ), 
      gl::STATIC_DRAW,
      offset,
      data.len() as u32
    );
  }

  fn make_buffer_attibute_info
  ( 
    buffer : &web_sys::WebGlBuffer, 
    offset : i32, 
    stride : i32, 
    slot : u32,
    normalized : bool,
    vector: gl::VectorDataType
  ) -> Result< AttributeInfo, gl::WebglError >
  {
    let descriptor = match vector.scalar
    {
      gl::DataType::U8 => gl::BufferDescriptor::new::< [ u8; 1 ] >(),
      gl::DataType::I8 => gl::BufferDescriptor::new::< [ i8; 1 ] >(),
      gl::DataType::U16 => gl::BufferDescriptor::new::< [ u16; 1 ] >(),
      gl::DataType::I16 => gl::BufferDescriptor::new::< [ i16; 1 ] >(),
      gl::DataType::U32 => gl::BufferDescriptor::new::< [ u32; 1 ] >(),
      gl::DataType::F32 => gl::BufferDescriptor::new::< [ f32; 1 ] >(),
      _ => return Err( gl::WebglError::NotSupportedForType( type_name_of_val( &vector.scalar ) ) )
    };

    let descriptor = descriptor
    .offset( offset )
    .normalized( normalized )
    .stride( stride )
    .vector( vector );

    Ok(
      AttributeInfo
      {
        slot,
        buffer : buffer.clone(),
        descriptor,
        bounding_box : Default::default()
      }
    )
  }

  /// Simplifies new buffer initialization
  pub fn add_buffer
  ( 
    gl : &gl::WebGl2RenderingContext, 
    gltf : &mut GLTF, 
    buffer_data : Vec< u8 > 
  ) -> Result< WebGlBuffer, gl::WebglError >
  {
    let buffer = gl.create_buffer().ok_or( gl::WebglError::FailedToAllocateResource( "Buffer" ) )?;
    upload_buffer_data( gl, &buffer, GL::ARRAY_BUFFER, 0, buffer_data );
    gltf.gl_buffers.push( buffer.clone() );
    Ok( buffer )
  }

  /// Adds additional attributes and their data into [`GLTF`] and 
  /// returns object_id data for updating data for per object attributes
  pub fn add_attributes
  ( 
    gl : &gl::WebGl2RenderingContext, 
    gltf : &mut GLTF, 
    active_attachments : HashSet< GBufferAttachment > 
  ) -> Result< Vec< u32 >, gl::WebglError >
  {
    let mut object_id_data : Vec< u32 > = vec![];
    let mut material_id_data : Vec< u32 > = vec![];

    let uuid_to_id = gltf.materials.iter()
    .enumerate()
    .map( | ( i, m ) | ( m.borrow().id, i as i32 ) )
    .collect::< HashMap< _, _ > >();

    let mut object_id = 1_u32;
    let mut object_vertex_count = 0;
    for mesh in &gltf.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let material_id = *uuid_to_id.get( &primitive.borrow().material.borrow().id ).unwrap_or( &( 0 ) ) as u32 + 1;
        let primitive = primitive.borrow();
        let geometry = primitive.geometry.borrow();
        let vertex_count = geometry.vertex_count as usize;
        object_vertex_count += vertex_count;

        for attachment in &active_attachments
        {
          match attachment 
          {
            GBufferAttachment::PbrInfo => 
            {
              material_id_data.extend( vec![ material_id; vertex_count ] );
            
            },
            _ => ()
          }
        }
      }
      
      object_id_data.extend( vec![ object_id; object_vertex_count ] );

      object_id += 1;
    }

    if active_attachments.contains( &GBufferAttachment::PbrInfo )
    {
      let object_id_data = object_id_data.iter().map( | i | i.to_be_bytes() ).flatten().collect::< Vec< _ > >();
      let _ = add_buffer( gl, gltf, object_id_data )?;
      let material_id_data = material_id_data.iter().map( | i | i.to_be_bytes() ).flatten().collect::< Vec< _ > >();
      let _ = add_buffer( gl, gltf, material_id_data )?;
    }

    Ok( object_id_data )
  }

  fn upload_camera  
  ( 
    gl : &gl::WebGl2RenderingContext, 
    camera : &Camera,
    locations : &HashMap< String, Option< WebGlUniformLocation > >
  )
  {
    camera.upload( gl, locations );

    let [ near, far ] = camera.get_near_far().0;

    gl::uniform::upload
    (
      gl,
      locations.get( "near" ).unwrap().clone(),
      &[ near ]
    ).unwrap();

    gl::uniform::upload
    (
      gl,
      locations.get( "far" ).unwrap().clone(),
      &[ far ]
    ).unwrap();
  }

  pub struct GBuffer
  {
    program_info : ProgramInfo< program::GBufferShader >,
    attachment_buffers: HashMap< GBufferAttachment, Vec< WebGlBuffer > >,
    vao : WebGlVertexArrayObject,
    width : u32,
    height : u32,
    framebuffer : WebGlFramebuffer,
    textures: HashMap< String, WebGlTexture >,
    color_attachments : Vec< u32 >
  }

  impl GBuffer 
  {
    /// Creates a new `GBuffer` instance.
    pub fn new
    ( 
      gl : &gl::WebGl2RenderingContext, 
      width : u32, 
      height : u32, 
      attachment_buffers: HashMap< GBufferAttachment, Vec< WebGlBuffer > >
    ) -> Result< Self, gl::WebglError >
    {
      let attachments_set = attachment_buffers.iter()
      .map( | ( a, _ ) | *a )
      .collect::< HashSet< _ > >();
      let defines = into_defines( &attachments_set );
      let program = gl::ProgramFromSources::new
      ( 
        &format!( "#version 300 es\n{}\n{}", &defines, GBUFFER_VERTEX_SHADER ), 
        &format!( "#version 300 es\n{}\n{}", &defines, GBUFFER_FRAGMENT_SHADER ), 
      ).compile_and_link( gl )?;
      let program_info = ProgramInfo::< program::GBufferShader >::new( gl , program.clone() );

      let vao = gl.create_vertex_array().ok_or( gl::WebglError::FailedToAllocateResource( "VAO" ) )?;
      gl.bind_vertex_array( Some( &vao ) );

      let mut attribute_info_input = vec![];

      for ( attachment, buffers ) in &attachment_buffers
      {
        if buffers.is_empty() 
        {
          continue;
        }
        let ( s, v, n ) = match attachment
        {
          GBufferAttachment::Position => ( 0, VectorDataType::new( mingl::DataType::F32, 3, 1 ), false ),
          GBufferAttachment::Color => ( 1, VectorDataType::new( mingl::DataType::F32, 4, 1 ), true ),
          GBufferAttachment::Normal => ( 2, VectorDataType::new( mingl::DataType::F32, 3, 1 ), false ),
          GBufferAttachment::PbrInfo => 
          {
            let ( s, v, n ) = ( 3, VectorDataType::new( mingl::DataType::F32, 1, 1 ), false );
            attribute_info_input.push( ( s, buffers[ 0 ].clone(), v, n ) );
            let ( s, v, n ) = ( 4, VectorDataType::new( mingl::DataType::F32, 1, 1 ), false );
            attribute_info_input.push( ( s, buffers[ 1 ].clone(), v, n ) );
            let ( s, v, n ) = ( 5, VectorDataType::new( mingl::DataType::F32, 2, 1 ), true );
            attribute_info_input.push( ( s, buffers[ 2 ].clone(), v, n ) );
            continue;
          },
          GBufferAttachment::ObjectColorId => ( 6, VectorDataType::new( mingl::DataType::F32, 1, 1 ), false ),
          _ => continue
        };
        attribute_info_input.push( ( s, buffers[ 0 ].clone(), v, n ) );
      }

      for ( slot, buffer, vector, normalized ) in attribute_info_input.iter()
      {
        let attribute_info = make_buffer_attibute_info(
          &buffer,
          vector.natoms * vector.nelements,
          0,
          *slot as u32,
          *normalized,
          *vector
        )?;
        attribute_info.upload( gl )?;
        //offset += vector.natoms * vector.nelements;
      }

      let mut textures = HashMap::new();
 
      let framebuffer = gl.create_framebuffer().ok_or( gl::WebglError::FailedToAllocateResource( "Framebuffer" ) )?;
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
      gl.viewport( 0, 0, width as i32, height as i32 );

      let mut color_attachments = vec![];

      let mut setup_texture = | gb_attachment : &GBufferAttachment, attachment, internal_format, filter, wrap |
      {
        let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "Texture" ) )?;
        gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
        gl.tex_storage_2d( GL::TEXTURE_2D, 1, internal_format, width as i32, height as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, filter as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, wrap as i32 );
        gl.framebuffer_texture_2d( GL::FRAMEBUFFER, attachment, GL::TEXTURE_2D, Some( &texture ), 0 );
        if attachment != GL::DEPTH_ATTACHMENT
        {
          color_attachments.push( attachment - GL::COLOR_ATTACHMENT0 );
        }
        textures.insert( gb_attachment.define_const(), texture );
        Ok::< (), gl::WebglError >( () )
      };

      for ( attachment, _) in &attachment_buffers
      {
        match attachment
        {
          GBufferAttachment::Position => setup_texture( attachment, GL::COLOR_ATTACHMENT0, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::Albedo => setup_texture( attachment, GL::COLOR_ATTACHMENT1, gl::RGBA8, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::Normal => setup_texture( attachment, GL::COLOR_ATTACHMENT2, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::PbrInfo => setup_texture( attachment, GL::COLOR_ATTACHMENT3, gl::RGBA8, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          GBufferAttachment::ObjectColorId => setup_texture( attachment, GL::COLOR_ATTACHMENT4, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          _ => ()
        }
      }

      let depthbuffer = gl.create_renderbuffer().ok_or( gl::WebglError::FailedToAllocateResource( "Renderbuffer" ) )?;
      gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
      gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );
      gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

      gl.bind_vertex_array( None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      
      Ok(
        Self
        {
          program_info,
          attachment_buffers,
          vao,
          width,
          height,
          framebuffer,
          textures,
          color_attachments
        }
      )
    } 

    /// Binds the gbuffer's program, VAO, framebuffer and set drawbuffers
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.program_info.bind( gl );
      gl.bind_vertex_array( Some( &self.vao ) );
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport( 0, 0, self.width as i32, self.height as i32 );
      drawbuffers( gl, &self.color_attachments );
    }

    pub fn get_texture( &self, attachment : GBufferAttachment ) -> Option< WebGlTexture >
    {
      self.textures.get( &attachment.define_const() ).clone().cloned()
    }

    pub fn render
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene, 
      camera : &Camera 
    ) -> Result< (), gl::WebglError >
    {
      self.bind( gl );

      let locations = self.program_info.get_locations();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );

      gl.clear_depth( 1.0 );
      gl.clear( gl::DEPTH_BUFFER_BIT );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 2, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 3, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 4, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      //gl.clear_bufferuiv_with_u32_array( gl::COLOR, 4, [ 0, 0, 0, 0 ].as_slice() );

      upload_camera( gl, &camera, locations );

      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node = 
      | 
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          // Iterate over each primitive in the mesh.
          for primitive_rc in mesh.borrow().primitives.iter()
          {
            let primitive = primitive_rc.borrow();

            if self.attachment_buffers.contains_key( &GBufferAttachment::Albedo ) 
            && self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
            {
              let albedo_texture = primitive.material.borrow().base_color_texture.as_ref()
              .map( | t | t.texture.borrow().source.clone() ).flatten();

              if let Some( albedo_texture ) = albedo_texture
              {
                let albedo_texture_loc = self.program_info.get_locations()
                .get( "albedoTexture" ).unwrap().clone().unwrap();
                upload_texture( gl, &albedo_texture, &albedo_texture_loc, GL::TEXTURE0 );
              }
            }

            upload_camera( gl, &camera, locations );
            node.borrow().upload( gl, locations );
            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }
        } 

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    GBuffer,
    GBufferAttachment,
    add_attributes,
    add_buffer,
    upload_buffer_data,
    ALL
  };
}