mod private
{
  use std::{ cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc };
  use minwebgl as gl;
  use web_sys::{ WebGlTexture, WebGlBuffer, WebGlFramebuffer, WebGlUniformLocation, WebGlVertexArrayObject };
  use gl::{ F32x4, GL, VectorDataType, drawbuffers::drawbuffers };
  use crate::webgl::
  { 
    AttributeInfo, program, Camera, Node, Object3D, ProgramInfo, Scene
  };

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
    GBufferAttachment::ObjectColor
  ];

  #[ derive( Debug, Copy, Clone, Eq, PartialEq, Hash ) ]
  pub enum GBufferAttachment
  {
    Position,
    Color,
    Albedo,
    Normal,
    PbrInfo,
    ObjectColor
  }

  impl GBufferAttachment
  {
    fn attribute_info( &self, buffers : &[ web_sys::WebGlBuffer ] ) -> Vec< AttributeInfo >
    {
      if buffers.is_empty() 
      {
        return vec![];
      }

      let mut descriptors = match self 
      {
        GBufferAttachment::Position => 
        {
          let d0 = gl::BufferDescriptor::new::< [ f32; 3 ] >()
          .normalized( false )
          .vector( VectorDataType::new( mingl::DataType::F32, 3, 1 ) );
          vec![ ( 0, d0 ) ]
        },
        GBufferAttachment::Color => 
        {
          let d1 = gl::BufferDescriptor::new::< [ f32; 4 ] >()
          .normalized( true )
          .vector( VectorDataType::new( mingl::DataType::F32, 4, 1 ) );
          vec![ ( 1, d1 ) ]
        },
        GBufferAttachment::Normal => 
        {
          let d2 = gl::BufferDescriptor::new::< [ f32; 3 ] >()
          .normalized( true )
          .vector( VectorDataType::new( mingl::DataType::F32, 3, 1 ) );
          vec![ ( 2, d2 ) ]
        },
        GBufferAttachment::PbrInfo => 
        {
          let d3 = gl::BufferDescriptor::new::< [ f32; 2 ] >()
          .normalized( true )
          .vector( VectorDataType::new( mingl::DataType::F32, 2, 1 ) );
          vec![ ( 3, d3 ) ]
        },
        _ => vec![]
      };

      for ( _, d ) in descriptors.iter_mut()
      {
        *d = d
        .offset( 0 )
        .stride( 0 );
      }

      let mut attribute_infos = vec![];

      for ( i, ( slot, descriptor ) ) in descriptors.into_iter().enumerate()
      {
        let a = AttributeInfo
        {
          slot,
          buffer : buffers.get( i ).expect( "Some GbufferAttachment hasn't enough buffers" ).clone(),
          descriptor,
          bounding_box : Default::default()
        };

        attribute_infos.push( a );
      }

      attribute_infos
    }

    fn define_const( &self ) -> String
    {
      match self 
      {
        GBufferAttachment::Position => "POSITION",
        GBufferAttachment::Color => "COLOR",
        GBufferAttachment::Albedo => "ALBEDO",
        GBufferAttachment::Normal => "NORMAL",
        GBufferAttachment::PbrInfo => "PBR_INFO",
        GBufferAttachment::ObjectColor => "OBJECT_COLOR",
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

      for ( attachment, buffers ) in &attachment_buffers
      {
        for attribute_info in attachment.attribute_info( buffers )
        {
          attribute_info.upload( gl )?;
        }
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
          GBufferAttachment::ObjectColor => setup_texture( attachment, GL::COLOR_ATTACHMENT4, gl::RGBA16F, GL::NEAREST, GL::CLAMP_TO_EDGE )?,
          _ => ()
        }
      }

      let depthbuffer = gl.create_renderbuffer().ok_or( gl::WebglError::FailedToAllocateResource( "Renderbuffer" ) )?;
      gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
      gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );
      gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

      gl.bind_vertex_array( None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      
      let gbuffer = Self
      {
        program_info,
        attachment_buffers,
        vao,
        width,
        height,
        framebuffer,
        textures,
        color_attachments
      };

      Ok( gbuffer )
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
      object_colors: Option< &[ F32x4 ] >,
      camera : &Camera 
    ) -> Result< (), gl::WebglError >
    {
      self.bind( gl );

      let locations = self.program_info.get_locations();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.depth_mask( true );
      gl.front_face( gl::CCW );
      gl.cull_face( gl::BACK );
      gl.depth_func( gl::LESS );
      gl.clear_depth( 1.0 );
      gl.clear( GL::DEPTH_BUFFER_BIT );

      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 2, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 3, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 4, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );

      upload_camera( gl, &camera, locations );

      let albedo_texture_loc = &self.program_info.get_locations()
      .get( "albedoTexture" ).unwrap().clone().unwrap();

      let object_id_loc = &self.program_info.get_locations()
      .get( "objectId" ).unwrap().clone();

      let material_id_loc = &self.program_info.get_locations()
      .get( "materialId" ).unwrap().clone();

      let object_color_loc = &self.program_info.get_locations()
      .get( "objectColor" ).unwrap().clone();

      let object_id = Rc::new( RefCell::new( 1_u32 ) );

      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node = 
      | 
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          if self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
          {
            gl::uniform::upload( &gl, object_id_loc.as_ref().cloned(), &*object_id.borrow() ).unwrap();
          }

          if self.attachment_buffers.contains_key( &GBufferAttachment::ObjectColor )
          {
            let object_color = if let Some( oc ) = object_colors
            { 
              ( oc.get( ( *object_id.borrow() - 1 ) as usize ) ).cloned().unwrap_or( F32x4::default() )
            }
            else
            {
              F32x4::default()
            };
            gl::uniform::upload( &gl, object_color_loc.as_ref().cloned(), object_color.as_slice() ).unwrap();
          }

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
                upload_texture( gl, &albedo_texture, &albedo_texture_loc, GL::TEXTURE0 );
              }
            }

            if self.attachment_buffers.contains_key( &GBufferAttachment::PbrInfo )
            {
              let material_id = &primitive.material.borrow().id.to_fields_le().0;
              gl::uniform::upload( &gl, material_id_loc.as_ref().cloned(), material_id ).unwrap();
            }

            upload_camera( gl, &camera, locations );
            node.borrow().upload( gl, locations );
            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }

          *object_id.borrow_mut() += 1;
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
    ALL
  };
}