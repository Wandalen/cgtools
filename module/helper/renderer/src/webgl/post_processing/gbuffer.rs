mod private
{
  use std::{ cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc };
  use minwebgl as gl;
  use web_sys::{ WebGlFramebuffer, WebGlProgram };
  use gl::GL;
  use crate::webgl::
  { 
    post_processing, program, AlphaMode, Camera, Node, Object3D, Primitive, ProgramInfo, Scene
  };

  /// The source code for the gbuffer vertex shader.
  const GBUFFER_VERTEX_SHADER : &'static str = include_str!( "shaders/post_processing/gbuffer.vert" );
  /// The source code for the gbuffer fragment shader.
  const GBUFFER_FRAGMENT_SHADER : &'static str = include_str!( "shaders/post_processing/gbuffer.frag" );

  #[ derive( Debug, Copy, Clone, Hash ) ]
  pub enum GBufferAttachment
  {
    Normal,
    Depth,
    ObjectId,
    ObjectColor
  }

  impl GBufferAttachment
  {
    fn define_const( &self ) -> String
    {
      match self 
      {
        GBufferAttachment::Normal => "NORMAL",
        GBufferAttachment::Depth => "DEPTH",
        GBufferAttachment::ObjectId => "OBJECT_ID",
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
      defines += "#define " + &attachment.define_const() + "\n";
    }

    defines
  }

  pub struct GBuffer
  {
    program : WebGlProgram,
    program_info : ProgramInfo< program::GBufferShader >,
    active_attachments: HashSet< GBufferAttachment >,
    width : u32,
    height : u32,
    framebuffer : WebGlFramebuffer,
    textures: HashMap< String, WebGlTexture >
  }

  impl GBuffer 
  {
    /// Creates a new `GBuffer` instance with default settings.
    pub fn new
    ( 
      gl : &gl::WebGl2RenderingContext, 
      width : u32, 
      height : u32, 
      attachments: HashSet< GBufferAttachment > 
    ) -> Result< Self, gl::WebglError >
    {
      let defines = into_defines( &attachments );
      let program = gl::ProgramFromSources::new
      ( 
        &format!( "#version 300 es\n{}\n{}", &defines, GBUFFER_VERTEX_SHADER ), 
        &format!( "#version 300 es\n{}\n{}", &defines, GBUFFER_FRAGMENT_SHADER ), 
      ).compile_and_link( gl )?;
      let program_info = ProgramInfo::< program::GBufferShader >::new( gl , program );

      let mut textures = HashMap::new();
 
      let framebuffer = gl.create_framebuffer()?;
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );

      let mut color_attachments = vec![];

      let setup_texture = | gb_attachment, attachment, internal_type, filter, wrap |
      {
        let texture = gl.create_texture();
        gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
        gl.tex_storage_2d( GL::TEXTURE_2D, 0, internal_type, width, height );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, filter as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap as i32 );
        gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, wrap as i32 );
        gl.framebuffer_texture_2d( GL::FRAMEBUFFER, attachment, GL::TEXTURE_2D, texture.as_ref(), 0 );
        if attachment != GL::DEPTH_ATTACHMENT
        {
          color_attachments.push( attachment );
        }
        textures.insert( gb_attachment.define_const(), texture );
      };

      for attachment in attachments
      {
        match attachment
        {
          GBufferAttachment::Normal => setup_texture( attachment, GL::COLOR_ATTACHMENT0, gl::RGB8, GL::NEAREST, GL::CLAMP_TO_EDGE ),
          GBufferAttachment::Depth => setup_texture( attachment, GL::DEPTH_ATTACHMENT, gl::DEPTH_COMPONENT24, GL::NEAREST, GL::CLAMP_TO_EDGE ),
          GBufferAttachment::ObjectId => setup_texture( attachment, GL::COLOR_ATTACHMENT2, gl::R32UI, GL::NEAREST, GL::CLAMP_TO_EDGE ),
          GBufferAttachment::ObjectColor => setup_texture( attachment, GL::COLOR_ATTACHMENT3, gl::RGBA8, GL::NEAREST, GL::CLAMP_TO_EDGE ),
        }
      }

      drawbuffers( gl, &attachments );

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      
      Ok(
        Self
        {
          program,
          program_info,
          active_attachments : attachments,
          width,
          height,
          framebuffer,
          textures
        }
      )
    } 

    pub fn get_texture( &self, attachment : GBufferAttachment ) -> Option< WebGlTexture >
    {
      self.get( &attachment.define_const() ).clone()
    }

    fn bind( &self )
    {

    }

    pub fn render
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene, 
      camera : &Camera 
    ) -> Result< (), gl::WebglError >
    {
      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
      gl.clear_depth( 1.0 );
      gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

      let locations = self.program_info.get_locations();
      self.program_info.bind( gl );
      self.framebuffer.bind();
      camera.upload( gl, locations );

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
            let material = primitive.material.borrow();
            let geometry = primitive.geometry.borrow();

            material.configure( gl, locations, IBL_BASE_ACTIVE_TEXTURE );
            material.upload( gl, locations )?;
            camera.upload( gl, locations );

            node.borrow().upload( gl, locations );
            primitive.bind( gl );
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
    GBufferAttachment
  };
}