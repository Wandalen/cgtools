//! Implementation of JFA outline using WebGL2 and web_sys.
//!
//! This example demonstrates how to render a 3D object and generate a real-time
//! outline around it using the Jump Flooding Algorithm ( JFA ).
//!
//! The process involves several rendering passes:
//! 1. **Object Pass:** Render the 3D object to a texture ( framebuffer ) to get a silhouette.
//!    Object pixels are marked ( e.g., white ), background is clear.
//! 2. **JFA Initialization Pass:** Initialize a JFA texture. Pixels corresponding
//!    to the object silhouette store their own texture coordinates ( these are the "seeds" ).
//!    Background pixels store a sentinel value ( e.g., ( -1.0, -1.0 ) ).
//! 3. **JFA Step Passes:** Repeatedly apply the JFA step shader. In each pass,
//!    each pixel samples its neighbors at an decreasing jump distance. It updates
//!    its stored coordinate to the one belonging to the *nearest* "seed" found so far.
//!    This propagates the nearest seed coordinate outwards from the object silhouette.
//!    A ping-pong rendering strategy is used between two framebuffers.
//! 4. **Outline Pass:** Render a final screen-filling quad. Sample the original object
//!    silhouette texture and the final JFA texture. For background pixels, calculate
//!    the distance to the nearest seed ( using the coordinate stored in the JFA texture ).
//!    If this distance is within a defined thickness, draw the outline color; otherwise,
//!    draw the background color. Object pixels are drawn with the object color.

mod private
{
  use minwebgl as gl;
  use gl::
  {
    GL,
    web_sys::
    {
      WebGl2RenderingContext,
      WebGlProgram,
      WebGlUniformLocation,
      WebGlBuffer,
      WebGlTexture,
      WebGlVertexArrayObject,
      WebGlFramebuffer
    }
  };
  use crate::webgl::
  { 
    post_processing::{ Pass, VS_TRIANGLE }, 
    program::
    {
      WideOutlineInitShader,
      WideOutlineStepShader,
      WideOutlineShader
    }, 
    ProgramInfo 
  };

  pub const MAX_OBJECT_COUNT : usize = 1024;

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

  /// Creates a WebGL2 framebuffer and a color attachment texture.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `size` - The size of the framebuffer and its attachment ( width, height ).
  /// * `color_attachment` - The index of the color attachment point ( e.g., 0 for `GL::COLOR_ATTACHMENT0` ).
  ///
  /// # Returns
  ///
  /// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
  /// its color attachment texture, or `None` if creation fails.
  fn create_framebuffer
  (
    gl : &gl::WebGl2RenderingContext,
    width : i32, 
    height : i32,
    color_attachment : u32
  ) 
  -> Option< ( WebGlFramebuffer, WebGlTexture ) >
  {
    let color = gl.create_texture()?;
    gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
    // Use tex_storage_2d for immutable texture storage ( WebGL2 )
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
    // Configure texture parameters (filtering, wrapping)
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );

    let framebuffer = gl.create_framebuffer()?;
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
    // Attach the texture to the framebuffer's color attachment point
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &color ), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    Some( ( framebuffer, color ) )
  }

  /// Binds a framebuffer for rendering and sets the viewport to its size.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `framebuffer` - The framebuffer to bind.
  /// * `width`, `height` - The size of the framebuffer.
  fn upload_framebuffer(
    gl : &gl::WebGl2RenderingContext,
    framebuffer : &WebGlFramebuffer,
    width : i32, 
    height : i32
  )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
    gl.viewport( 0, 0, width, height );
  }

  struct ProgramInfos
  {
    jfa_init : ( WebGlProgram, ProgramInfo< WideOutlineInitShader > ),
    jfa_step : ( WebGlProgram, ProgramInfo< WideOutlineStepShader > ),
    outline : ( WebGlProgram, ProgramInfo< WideOutlineShader > )
  }

  impl ProgramInfos
  {
    fn new() -> Self
    {
      // --- Load and Compile Shaders ---

      let jfa_init_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/jfa_init.frag" );
      let jfa_step_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/jfa_step.frag" );
      let outline_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/outline.frag" );

      // Compile and link shader programs and store them
      let jfa_init_program = gl::ProgramFromSources::new( VS_TRIANGLE, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
      let jfa_step_program = gl::ProgramFromSources::new( VS_TRIANGLE, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
      let outline_program = gl::ProgramFromSources::new( VS_TRIANGLE, outline_fs_src ).compile_and_link( gl ).unwrap();

      let jfa_init = ProgramInfo::< WideOutlineInitShader >::new( jfa_init_program.clone() );
      let jfa_step = ProgramInfo::< WideOutlineStepShader >::new( jfa_step_program.clone() );
      let outline = ProgramInfo::< WideOutlineShader >::new( outline_program.clone() );

      Self
      {
        jfa_init : ( jfa_init_program, jfa_init ),
        jfa_step : ( jfa_step_program, jfa_step ),
        outline : ( outline_program, outline )
      }
    }
  }

  pub struct WideOutlinePass
  {
    program_infos : ProgramInfos,
    framebuffers : HashMap< String, WebGlFramebuffer >,
    textures : HashMap< String, WebGlTexture >,
    outline_thickness : f32,
    object_colors : Option< Vec< [ f32; 4 ] > >,
    object_color_buffer : WebGlBuffer,
    width : u32,
    height : u32
  }

  impl WideOutlinePass 
  {
    /// Creates a new `NarrowOutlinePass` instance.
    pub fn new( 
      gl : &gl::WebGl2RenderingContext, 
      mut textures : HashMap< String, WebGlTexture >,
      outline_thickness : f32,
      width : u32, 
      height : u32 
    ) -> Result< Self, gl::WebglError >
    {
      let program_infos = ProgramInfos::new();

      // --- Create Framebuffers and Textures ---

      // Framebuffer for the JFA initialization pass
      let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( gl, width, height, 0 ).unwrap();
      // Framebuffers for the JFA step passes ( ping-pong )
      let ( jfa_step_fb_0, jfa_step_fb_color_0 ) = create_framebuffer( gl, width, height, 0 ).unwrap();
      let ( jfa_step_fb_1, jfa_step_fb_color_1 ) = create_framebuffer( gl, width, height, 0 ).unwrap();

      // Store the color attachment textures
      textures.insert( "object_fb_color".to_string(), object_fb_color );
      textures.insert( "jfa_init_fb_color".to_string(), jfa_init_fb_color );
      textures.insert( "jfa_step_fb_color_0".to_string(), jfa_step_fb_color_0 );
      textures.insert( "jfa_step_fb_color_1".to_string(), jfa_step_fb_color_1 );

      let mut framebuffers = HashMap::new();

      // Store the framebuffers
      framebuffers.insert( "jfa_init_fb".to_string(), jfa_init_fb );
      framebuffers.insert( "jfa_step_fb_0".to_string(), jfa_step_fb_0 );
      framebuffers.insert( "jfa_step_fb_1".to_string(), jfa_step_fb_1 );

      //-------------------------------------------------------------------------------------------------

      let object_colors_buffer = gl::buffer::create( &gl )?;
      let object_colors_loc = gl.get_uniform_block_index( &program_infos.outline.0, "ObjectColorBlock" );
      gl.uniform_block_binding( &program, object_colors_loc, 0 );
      gl.bind_buffer_base( GL::UNIFORM_BUFFER, 0, Some( &object_colors_buffer ) );
      gl.bind_buffer( GL::UNIFORM_BUFFER, Some( &object_colors_buffer ) );
      gl.buffer_data_with_i32( GL::UNIFORM_BUFFER, MAX_OBJECT_COUNT as i32 * 16, GL::DYNAMIC_DRAW );

      let mut pass = Self
      {
        program_infos,
        framebuffers,
        textures,
        outline_thickness,
        object_colors : None,
        object_colors_buffer,
        width,
        height
      };

      pass.set_object_colors( gl, vec![ [ 0.0; 4 ]; MAX_OBJECT_COUNT ] );

      Ok( pass )
    }    

    fn set_io_textures
    ( 
      &self,   
      gl : &minwebgl::WebGl2RenderingContext,
      input_texture : Option< minwebgl::web_sys::WebGlTexture >,
      output_texture : Option< minwebgl::web_sys::WebGlTexture > 
    )
    {

    }

    pub fn set_outline_thickness( &mut self, new_value : f32 )
    {
      self.outline_thickness = new_value;
    }

    pub fn set_object_colors( &mut self, gl : &gl::WebGl2RenderingContext, object_colors: Vec< [ f32; 4 ] > )
    {
      let object_colors = object_colors.into_iter().flatten().collect::< Vec< _ > >();
      gl::ubo::upload( &gl, &self.object_colors_buffer, 0, &object_colors[ .. ], GL::DYNAMIC_DRAW );
    }

    /// Performs the JFA initialization pass.
    ///
    /// Reads the object silhouette texture and writes texture coordinates for
    /// object pixels and a sentinel value for background pixels to the
    /// `jfa_init_fb`.
    fn jfa_init_pass( &self, gl : &gl::WebGl2RenderingContext )
    {
      let jfa_init_program = &self.program_infos.jfa_init.0;
      let jfa_init_fb = self.framebuffers.get( "jfa_init_fb" ).unwrap();
      let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();

      let object_color_id_texture = self.program_infos.jfa_init.1.get( "objectColorIdTexture" ).unwrap().clone().as_ref();

      gl.use_program( Some( jfa_init_program ) );

      upload_framebuffer( gl, jfa_init_fb, self.width, self.height );

      upload_texture( gl, object_fb_color, object_color_id_texture, GL::TEXTURE0 );

      gl.draw_arrays( GL::TRIANGLES, 0, 3 );
    }

    /// Performs one step of the Jump Flooding Algorithm.
    ///
    /// Reads from the JFA texture of the previous step and writes to one of the
    /// ping-pong JFA framebuffers ( `jfa_step_fb_0` or `jfa_step_fb_1` ).
    ///
    /// # Arguments
    ///
    /// * `i` - The current JFA step index ( 0, 1, 2, ... ).
    /// * `last` - A boolean flag. If true, the result of this step is rendered
    ///            directly to the default framebuffer ( screen ) for debugging.
    fn jfa_step_pass( &self, gl : &gl::WebGl2RenderingContext, i : i32 )
    {
      let jfa_step_program = &self.program_infos.jfa_step.0;
      let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
      let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
      let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap(); // Initial JFA texture
      let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // Color texture for FB 0
      let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // Color texture for FB 1

      let resolution = self.program_infos.jfa_step.1.get( "resolution" ).unwrap().clone().as_ref();
      let u_step_size = self.program_infos.jfa_step.1.get( "stepSize" ).unwrap().clone().as_ref();
      let jfa_init_texture = self.program_infos.jfa_step.1.get( "jfaTexture" ).unwrap().clone().as_ref();

      gl.use_program( Some( jfa_step_program ) );

      // Ping-pong rendering: Determine input texture and output framebuffer based on step index `i`
      if i == 0 // First step uses the initialization result
      {
        upload_framebuffer( gl, jfa_step_fb_0, self.width, self.height ); // Render to FB 0
        upload_texture( gl, jfa_init_fb_color, &jfa_init_texture, GL::TEXTURE0 ); // Input is JFA init texture
      }
      else if i % 2 == 0 // Even steps ( 2, 4, ... ) read from FB 1, render to FB 0
      {
        upload_framebuffer( gl, jfa_step_fb_0, self.width, self.height ); // Render to FB 0
        upload_texture( gl, &jfa_step_fb_color_1, &jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 1
      }
      else // Odd steps ( 1, 3, ... ) read from FB 0, render to FB 1
      {
        upload_framebuffer( gl, jfa_step_fb_1, self.width, self.height ); // Render to FB 1
        upload_texture( gl, jfa_step_fb_color_0, &jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 0
      }

      // Upload resolution uniform ( needed for distance calculations in the shader )
      gl::uniform::upload( gl, Some( resolution.clone() ), &[ self.width as f32, self.height as f32 ] ).unwrap();

      let aspect_ratio = self.width as f32 / self.height as f32;
      let step_size =  self.outline_thickness / ( 2.0_f32 ).powf( i as f32 );
      let step_size = [ step_size * aspect_ratio, step_size ];

      gl::uniform::upload( gl, Some( u_step_size.clone() ), &step_size ).unwrap();

      gl.draw_arrays( GL::TRIANGLES, 0, 3 );
    }

    /// Performs the final outline pass.
    ///
    /// Reads the original object silhouette texture and the final JFA result texture
    /// to draw the final scene with object color, outline color, or background color.
    /// Renders to the default framebuffer ( screen ).
    ///
    /// # Arguments
    ///
    /// * `t` - The current time in milliseconds ( used for animating outline thickness ).
    /// * `num_passes` - The total number of JFA step passes performed. Used to determine
    ///                which of the ping-pong textures ( `jfa_step_fb_color_0` or `jfa_step_fb_color_1` )
    ///                holds the final JFA result.
    fn outline_pass( &self, gl : &gl::WebGl2RenderingContext, num_passes : i32 )
    {
      let outline_program = &self.program_infos.outline.0;
      let object_fb_color = self.textures.get( "object_fb_color" ).unwrap(); // Original silhouette
      let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // JFA ping-pong texture 0
      let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // JFA ping-pong texture 1

      let outline_object_texture = self.program_infos.outline.1.get( "objectColorIdTexture" ).unwrap().clone().as_ref();
      let jfa_step_texture = self.program_infos.outline.1.get( "jfaTexture" ).unwrap().clone().as_ref();
      let resolution = self.program_infos.outline.1.get( "resolution" ).unwrap().clone().as_ref();

      gl.use_program( Some( outline_program ) );

      // Bind the default framebuffer ( render to canvas )
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );

      gl::uniform::upload( gl, Some( resolution.clone() ), &[ width as f32, height as f32 ] ).unwrap();

      upload_texture( gl, object_fb_color, &outline_object_texture, GL::TEXTURE0 );
      // The final JFA result is in jfa_step_fb_color_0 if num_passes is even, otherwise in jfa_step_fb_color_1
      if num_passes % 2 == 0
      {
        upload_texture( gl, jfa_step_fb_color_0, &jfa_step_texture, GL::TEXTURE1 );
      }
      else
      {
        upload_texture( gl, jfa_step_fb_color_1, &jfa_step_texture, GL::TEXTURE1 );
      }

      gl.draw_arrays( GL::TRIANGLES, 0, 3 );
    }
  }

  impl Pass for WideOutlinePass
  {
    fn renders_to_input( &self ) -> bool 
    {
      false
    }
    
    fn render
    (
      &self,
      gl : &minwebgl::WebGl2RenderingContext,
      input_texture : Option< minwebgl::web_sys::WebGlTexture >,
      output_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< minwebgl::web_sys::WebGlTexture >, minwebgl::WebglError > 
    {
      self.set_io_textures( gl, input_texture, output_texture );

      // 2. JFA Initialization Pass: Initialize JFA texture from the silhouette
      self.jfa_init_pass( gl );

      // 3. JFA Step Passes: Perform Jump Flooding Algorithm steps
      // The number of passes required is log2( max( width, height ) ).
      let num_passes = 4;//( self.viewport.0.max( self.viewport.1 ) as f32 ).log2().ceil() as i32;
      for i in 0..num_passes
      {
        self.jfa_step_pass( gl, i );
      }

      // 4. Outline Pass: Generate and render the final scene with the outline
      self.outline_pass( gl, num_passes );

      Ok
      (
        output_texture
      )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    WideOutlinePass,
    MAX_OBJECT_COUNT
  };
}