mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;

  use crate::webgl::
  { 
    Camera, 
    IBL, 
    Object3D,
    Node, 
    ProgramInfo, 
    Scene,
    Primitive,
    AlphaMode,
    program
  };

  pub struct FramebufferContext
  {
    pub texture_width : u32,
    pub texture_height : u32,
    pub multisample_framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    pub resolved_framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    pub depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    pub multisample_main_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    pub multisample_emission_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    pub main_texture : Option< gl::web_sys::WebGlTexture >,
    pub emission_texture : Option< gl::web_sys::WebGlTexture >
  }

  impl FramebufferContext 
  {
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32, samples : i32 ) -> Self
    {
      // Stores multisampled texture
      let multisample_framebuffer = gl.create_framebuffer();
      // Stores resolved texture from multisampled one
      let resolved_framebuffer = gl.create_framebuffer();

      let depth_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, depth_renderbuffer.as_ref() );
      //gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as i32, height as i32 );
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER, 
        samples, 
        gl::DEPTH24_STENCIL8, 
        width as i32, 
        height as i32
      );

      let multisample_main_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_main_renderbuffer.as_ref() );
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER, 
        samples, 
        gl::RGBA16F, 
        width as i32, 
        height as i32
      );

      let multisample_emission_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_emission_renderbuffer.as_ref() );
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER, 
        samples, 
        gl::RGBA16F, 
        width as i32, 
        height as i32
      );
      

      // Create textures
      let main_texture = gl.create_texture();
      let emission_texture = gl.create_texture();

      gl.bind_texture( gl::TEXTURE_2D, main_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width as i32, height  as i32 );
      gl::texture::d2::filter_linear( &gl );
      
      gl.bind_texture( gl::TEXTURE_2D, emission_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width  as i32, height  as i32 );
      gl::texture::d2::filter_linear( &gl );

      gl.bind_framebuffer( gl::FRAMEBUFFER, multisample_framebuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, multisample_main_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, multisample_emission_renderbuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, resolved_framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, main_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, emission_texture.as_ref(), 0 );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1 ] );

      gl.bind_texture( gl::TEXTURE_2D, None );
      gl.bind_renderbuffer( gl::RENDERBUFFER, None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let texture_width = width;
      let texture_height = height;

      Self
      {
        texture_height,
        texture_width,
        resolved_framebuffer,
        multisample_framebuffer,
        depth_renderbuffer,
        multisample_emission_renderbuffer,
        multisample_main_renderbuffer,
        main_texture,
        emission_texture
      }
    }

    pub fn enable_emission_texture( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    pub fn disable_emission_texture( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ gl::COLOR_ATTACHMENT0 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    pub fn resolve( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind_multisample( gl );
      self.bind_resolved( gl );
      gl.bind_framebuffer( gl::READ_FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl.bind_framebuffer( gl::DRAW_FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.blit_framebuffer
      (
        0, 0, self.texture_width as i32, self.texture_height as i32, 
        0, 0, self.texture_width as i32, self.texture_height as i32, 
        gl::COLOR_BUFFER_BIT, gl::LINEAR
      );
      gl.bind_framebuffer( gl::READ_FRAMEBUFFER, None );
      gl.bind_framebuffer( gl::DRAW_FRAMEBUFFER, None );
      self.unbind_multisample( gl );
      self.unbind_resolved( gl );
    }

    pub fn bind_multisample( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, self.multisample_main_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, self.multisample_emission_renderbuffer.as_ref() );
    }

    pub fn bind_resolved( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.main_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, self.emission_texture.as_ref(), 0 );
    }

    pub fn unbind_multisample( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, None );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    pub fn unbind_resolved( &self, gl : &gl::WebGl2RenderingContext )
    {
       gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, None, 0 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }
  }

  /// The source code for the main vertex shader.
  const MAIN_VERTEX_SHADER : &'static str = include_str!( "shaders/main.vert" );
  /// The source code for the main fragment shader.
  const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "shaders/main.frag" );

  /// Manages the rendering process, including program management, IBL setup, and drawing objects in the scene.
  pub struct Renderer
  {
    /// A map of compiled WebGL programs, keyed by a combination of the material ID and vertex shader defines.
    programs : HashMap< String, ProgramInfo< program::PBRShader > >,
    /// Holds the precomputed textures used for Image-Based Lighting.
    ibl : Option< IBL >,
    /// A list of nodes with transparent primitives, sorted by distance to the camera for correct rendering order.
    transparent_nodes : Vec< ( Rc< RefCell< Node > >, Rc< RefCell< Primitive > > ) >,
    /// If set to true, the renderer will add blur to the original image
    use_emission : bool,
    framebuffer_ctx : FramebufferContext
  }

  impl Renderer 
  {
    /// Creates a new `Renderer` instance with default settings.
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32, samples : i32 ) -> Self
    {
      let framebuffer_ctx = FramebufferContext::new( gl, width, height, samples );
      let use_emission = false;
      let programs = HashMap::new();
      let ibl = None;
      let transparent_nodes = Vec::new();
      
      Self
      {
        programs,
        ibl,
        transparent_nodes,
        use_emission,
        framebuffer_ctx
      }
    } 

    /// Sets the Image-Based Lighting (IBL) textures to be used for rendering.
    ///
    /// * `ibl`: The `IBL` struct containing the diffuse and specular environment maps and the BRDF integration texture.
    pub fn set_ibl( &mut self, ibl : IBL )
    {
      self.ibl = Some( ibl );
    }

    pub fn set_use_emission( &mut self, use_emission : bool )
    {
      self.use_emission = use_emission;
    }

    pub fn get_main_texture( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.framebuffer_ctx.main_texture.clone()
    }

    /// Renders the scene using the provided camera.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for rendering.
    /// * `scene`: A mutable reference to the `Scene` to be rendered.
    /// * `camera`: A reference to the `Camera` defining the viewpoint.
    pub fn render
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene, 
      camera : &Camera 
    ) -> Result< (), gl::WebglError >
    {
      //scene.update_world_matrix();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
      gl.clear_depth( 1.0 );

      if self.use_emission 
      {
        self.framebuffer_ctx.enable_emission_texture( gl );    
      }
      else 
      {
        self.framebuffer_ctx.disable_emission_texture( gl );    
      }

      self.framebuffer_ctx.bind_multisample( gl );
      gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

      // Clear the list of transparent nodes before each render.
      self.transparent_nodes.clear();

      for program in self.programs.values()
      {
        let locations = program.get_locations();
        program.bind( gl );
        camera.upload( gl, locations );
      }

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
            let vs_defines = geometry.get_defines();
            // Generate a unique ID for the program based on the material ID and vertex shader defines.
            let program_id = format!( "{}{}", material.id, vs_defines );

            // Retrieve the program info if it already exists, otherwise compile and link a new program.
            let program_info = 
            if let Some( ref program_info ) = self.programs.get( &program_id )
            {
             program_info 
            }
            else
            {
              let ibl_define = if self.ibl.is_some()
              {
                "#define USE_IBL\n"
              }
              else
              {
                ""
              };


              // Compile and link a new WebGL program from the vertex and fragment shaders with the appropriate defines.
              let program = gl::ProgramFromSources::new
              ( 
                &format!( "#version 300 es\n{}\n{}", vs_defines, MAIN_VERTEX_SHADER ), 
                &format!
                ( 
                  "#version 300 es\n{}\n{}\n{}\n{}\n{}", 
                  vs_defines, 
                  ibl_define,
                  "",
                 // "#define USE_EMISSION",
                  material.get_defines(),
                  MAIN_FRAGMENT_SHADER ) 
              ).compile_and_link( gl )?;
              let program_info = ProgramInfo::< program::PBRShader >::new( gl , program );

              // Configure and upload material properties and IBL textures for the new program.
              let locations = program_info.get_locations();
              program_info.bind( gl );
              const IBL_BASE_ACTIVE_TEXTURE : u32 = 10;
              material.configure( gl, locations, IBL_BASE_ACTIVE_TEXTURE );
              material.upload( gl, locations )?;
              camera.upload( gl, locations );
              if let Some( ref ibl ) = self.ibl 
              {
                ibl.bind( gl, IBL_BASE_ACTIVE_TEXTURE );
              }

              // Store the new program info in the cache.
              self.programs.insert( program_id.clone(), program_info );
              self.programs.get( &program_id ).unwrap()
            };

            // Handle transparent objects by adding them to a separate list for later rendering.
            match material.alpha_mode
            {
              AlphaMode::Blend =>
              {
                self.transparent_nodes.push( ( node.clone(), primitive_rc.clone() ) );
                continue; // Skip the immediate drawing of transparent objects.
              },
              _ => {}
            }

            // Get the uniform locations for the current program.
            let locations = program_info.get_locations();

            // Bind the program, upload camera and node matrices, bind the primitive, and draw it.
            program_info.bind( gl );
            node.borrow().upload( gl, locations );
            primitive.bind( gl );
            primitive.draw( gl );
          }
        } 

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      // Sort transparent nodes based on their distance to the camera (furthest to nearest).
      self.transparent_nodes.sort_by( | a, b | 
      {
        let dist1 = camera.get_eye().distance_squared( &a.1.borrow().center() );
        let dist2 = camera.get_eye().distance_squared( &b.1.borrow().center() );

        dist1.partial_cmp( &dist2 ).unwrap()
      });

      gl.enable( gl::BLEND );
      gl.blend_equation( gl::FUNC_ADD );
      gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

      // Render the transparent nodes.
      for ( node, primitive ) in self.transparent_nodes.iter()
      {
        let primitive = primitive.borrow();
        let material = primitive.material.borrow();
        let geometry = primitive.geometry.borrow();
        let vs_defines = geometry.get_defines();
        let program_info = self.programs.get( &format!( "{}{}",  material.id, vs_defines ) ).unwrap();

        let locations = program_info.get_locations();

        program_info.bind( gl );
        node.borrow().upload( gl, locations );
        primitive.bind( gl );
        primitive.draw( gl );
      }

      //gl.bind_framebuffer( gl::FRAMEBUFFER, None );
      self.framebuffer_ctx.unbind_multisample( gl );
      self.framebuffer_ctx.resolve( gl );

      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Renderer
  };
}