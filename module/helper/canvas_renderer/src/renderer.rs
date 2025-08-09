//! This module contains the implementation for offscreen 
//! rendering to a texture using WebGL2. It includes a utility 
//! function to create a framebuffer and the `CanvasRenderer` 
//! struct for managing the rendering process.

mod private
{
  use minwebgl as gl;
  use gl::
  {
    F32x4,
    drawbuffers::drawbuffers,
    GL,
    web_sys::
    {
      WebGlFramebuffer, 
      WebGlProgram, 
      WebGlTexture
    }
  };
  use renderer::webgl::
  {
    Object3D,
    Node,
    Camera,
    Scene
  };
  use std::collections::HashMap;
  use std::cell::RefCell;
  use std::rc::Rc;

  /// Creates a WebGL2 framebuffer and a color attachment texture.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `width`, `height` - The size of the framebuffer and its attachment.
  ///
  /// # Returns
  ///
  /// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
  /// its color attachment texture, or `None` if creation fails.
  fn create_framebuffer
  (
    gl : &gl::GL,
    width : u32,
    height : u32
  ) 
  -> Option< ( WebGlFramebuffer, WebGlTexture ) >
  {
    let color = gl.create_texture()?;
    gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width as i32, height as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    let depthbuffer = gl.create_renderbuffer().unwrap();
    gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
    gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );

    let framebuffer = gl.create_framebuffer()?;
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
    gl.viewport(0, 0, width as i32, height as i32 );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

    drawbuffers( gl, &[ 0 ] );

    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    Some( ( framebuffer, color ) )
  }

  /// A 2D canvas renderer that renders 3D scenes to a texture using WebGL.
  ///
  /// This renderer creates a framebuffer with a color attachment texture and provides
  /// methods to render scenes with custom colors and camera configurations.
  pub struct CanvasRenderer
  {
    /// The WebGL program used for rendering.
    program : WebGlProgram,
    /// A map storing the locations of uniform variables in the program.
    uniforms : HashMap< String, Option< gl::WebGlUniformLocation > >,
    /// The WebGL framebuffer used for offscreen rendering.
    framebuffer : WebGlFramebuffer,
    /// The texture attached to the framebuffer, where the rendering results are stored.
    output_texture : WebGlTexture,
    /// The width of the framebuffer and its output texture.
    width : u32,
    /// The height of the framebuffer and its output texture.
    height : u32
  } 

  impl CanvasRenderer
  {
    /// Creates a new canvas renderer with the specified dimensions.
    ///
    /// This function compiles and links the canvas shaders, initializes uniform locations,
    /// and creates a framebuffer with color and depth attachments.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `width` - Width of the render target in pixels
    /// * `height` - Height of the render target in pixels
    ///
    /// # Returns
    ///
    /// Returns `Ok(CanvasRenderer)` on success, or `Err(WebglError)` if initialization fails.
    pub fn new( gl : &GL, width : u32, height : u32 ) -> Result< Self, gl::WebglError >
    {
      let vertex_shader_src = include_str!( "../shaders/canvas.vert" );
      let fragment_shader_src = include_str!( "../shaders/canvas.frag" );
      let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
      .compile_and_link( &gl )?;

      let mut uniforms = HashMap::new(); 
      let mut add_location = 
      | name : &str | 
      {
        uniforms.insert
        ( 
          name.to_string(), 
          gl.get_uniform_location( &program, name ) 
        )
      };

      add_location( "color" );
      add_location( "worldMatrix" );
      add_location( "viewMatrix" );
      add_location( "projectionMatrix" );

      let Some( ( framebuffer, output_texture ) ) = create_framebuffer( gl, width, height )
      else
      {
        return Err( gl::WebglError::FailedToAllocateResource( "Framebuffer" ) );
      };

      Ok(
        Self
        {
          program,
          uniforms,
          framebuffer,
          output_texture,
          width,
          height
        }
      )
    }

    /// Uploads the camera's view and projection matrices to the shader uniforms.
    fn upload_camera( &self, gl : &GL, camera : &Camera )
    {
      gl::uniform::matrix_upload
      ( 
        &gl,
        self.uniforms.get( "viewMatrix" ).unwrap().clone(),
        &camera.get_view_matrix().to_array(), 
        true 
      ).unwrap();

      gl::uniform::matrix_upload
      ( 
        &gl,
        self.uniforms.get( "projectionMatrix" ).unwrap().clone(),
        &camera.get_projection_matrix().to_array(), 
        true 
      ).unwrap();
    }

    /// Uploads the world transformation matrix of a node to the GPU.
    ///
    /// This method updates the "worldMatrix" uniform with the node's world transformation matrix.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `node` - The scene node whose world matrix will be uploaded
    pub fn upload_node
    (
      &self,
      gl : &GL,
      node : &Rc< RefCell< Node > >
    )
    {
      gl::uniform::matrix_upload
      (
        &gl,
        self.uniforms.get( "worldMatrix" ).unwrap().clone(),
        node.borrow().get_world_matrix().to_array().as_slice(),
        true
      ).unwrap();
    }

    /// Renders a 3D scene to the internal framebuffer using specified colors.
    ///
    /// This method configures WebGL state, binds the framebuffer, and traverses the scene
    /// to render all mesh nodes with their corresponding colors from the colors array.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `scene` - The 3D scene to render (will update world matrices)
    /// * `camera` - The camera defining view and projection matrices
    /// * `colors` - Array of colors to apply to scene nodes in order
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful rendering, or `Err(WebglError)` if rendering fails.
    pub fn render
    ( 
      &self, 
      gl : &GL, 
      scene : &mut Scene, 
      camera : &Camera,
      colors : &[ F32x4 ]
    ) -> Result< (), gl::WebglError >
    {
      scene.update_world_matrix();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.depth_mask( true );
      gl.clear_depth( 1.0 );
      gl.front_face( gl::CCW );

      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport(0, 0, self.width as i32, self.height as i32 );

      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 0.0 ] );
      gl.clear( gl::DEPTH_BUFFER_BIT );

      gl.use_program( Some( &self.program ) );

      let mut i = 0; 
      let default_color = F32x4::from_array( [ 1.0, 0.0, 1.0, 1.0 ] ); 
      
      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node = 
      | 
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          gl::uniform::upload
          (
            &gl,
            self.uniforms.get( "color" ).unwrap().clone(),
            colors.get( i ).unwrap_or( &default_color ).as_slice()
          ).unwrap();

          // Iterate over each primitive in the mesh.
          for primitive_rc in mesh.borrow().primitives.iter()
          {
            let primitive = primitive_rc.borrow();

            self.upload_camera( gl, camera );
            self.upload_node( gl, &node );

            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }
        } 

        i += 1;

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      Ok( () )
    }

    /// Sets a new output texture as the color attachment for the framebuffer.
    ///
    /// This method replaces the current output texture with the provided one,
    /// effectively changing where the renderer will draw its output.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `output_texture` - The new texture to use as the color attachment
    pub fn set_texture
    ( 
      &mut self, 
      gl : &GL, 
      output_texture : WebGlTexture 
    )
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport(0, 0, self.width as i32, self.height as i32 );
      gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &output_texture ), 0 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      self.output_texture = output_texture;
    }

    /// Returns a clone of the current output texture.
    ///
    /// This method provides access to the texture that contains the rendered output,
    /// which can be used for further processing or display.
    ///
    /// # Returns
    ///
    /// A clone of the WebGlTexture that serves as the color attachment.
    pub fn get_texture( &self ) -> WebGlTexture
    {
      self.output_texture.clone()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    CanvasRenderer
  };
}