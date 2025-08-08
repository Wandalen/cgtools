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
      WebGlProgram,
      WebGlUniformLocation,
      WebGlTexture,
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
  use std::collections::HashMap;

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
    color_texture : Option< WebGlTexture >
  ) 
  -> Option< ( WebGlFramebuffer, WebGlTexture ) >
  {
    let color = if let Some( color ) = color_texture
    {
      color
    }
    else
    {
      let color = gl.create_texture()?;
      gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
      // Use tex_storage_2d for immutable texture storage ( WebGL2 )
      gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
      // Configure texture parameters (filtering, wrapping)
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
      color
    };

    let framebuffer = gl.create_framebuffer()?;
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
    // Attach the texture to the framebuffer's color attachment point
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    Some( ( framebuffer, color ) )
  }

  fn set_framebuffer_color
  (
    gl : &gl::WebGl2RenderingContext,
    framebuffer : &WebGlFramebuffer,
    color_texture : Option< WebGlTexture >
  ) 
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
    // Attach the texture to the framebuffer's color attachment point
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, color_texture.as_ref(), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
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
    fn new( gl : &gl::WebGl2RenderingContext ) -> Self
    {
      // --- Load and Compile Shaders ---

      let jfa_init_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/jfa_init.frag" );
      let jfa_step_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/jfa_step.frag" );
      let outline_fs_src = include_str!( "../../shaders/post_processing/outline/wide_outline/outline.frag" );

      // Compile and link shader programs and store them
      let jfa_init_program = gl::ProgramFromSources::new( VS_TRIANGLE, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
      let jfa_step_program = gl::ProgramFromSources::new( VS_TRIANGLE, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
      let outline_program = gl::ProgramFromSources::new( VS_TRIANGLE, outline_fs_src ).compile_and_link( gl ).unwrap();

      let jfa_init = ProgramInfo::< WideOutlineInitShader >::new( gl, jfa_init_program.clone() );
      let jfa_step = ProgramInfo::< WideOutlineStepShader >::new( gl, jfa_step_program.clone() );
      let outline = ProgramInfo::< WideOutlineShader >::new( gl, outline_program.clone() );

      Self
      {
        jfa_init : ( jfa_init_program, jfa_init ),
        jfa_step : ( jfa_step_program, jfa_step ),
        outline : ( outline_program, outline )
      }
    }
  }

  /// A struct representing a multi-pass rendering technique for creating wide outlines.
  /// 
  /// This pass uses an algorithm like the Jump Flood Algorithm (JFA) to generate
  /// a distance field, which allows for rendering thick, smooth outlines. Because
  /// this is a multi-pass process, it requires multiple framebuffers and textures
  /// to store intermediate results.
  pub struct WideOutlinePass
  {
    /// A collection of WebGL program information structs, one for each shader used
    /// in the multi-pass process (e.g., initialization, stepping, and final rendering).
    program_infos : ProgramInfos,
    /// A hash map to manage multiple WebGL framebuffers. These are used to render
    /// to different textures in each pass of the algorithm.
    framebuffers : HashMap< String, WebGlFramebuffer >,
    /// A hash map to store the textures used by the different passes. This includes
    /// the initial data texture and the textures used to store intermediate distance
    /// field results.
    textures : HashMap< String, WebGlTexture >,
    /// The desired thickness of the final outline. This value influences the number
    /// of passes and the final rendering stage.
    outline_thickness : f32,
    /// The width of the rendering surface. This is used to size the framebuffers and
    /// textures correctly.
    width : u32,
    /// The height of the rendering surface.
    height : u32,
    /// The number of rendering passes required for the algorithm. This is typically
    /// related to the outline thickness and the power-of-two size of the textures.
    num_passes : u32
  }

  impl WideOutlinePass 
  {
    /// Creates a new `NarrowOutlinePass` instance.
    pub fn new( 
      gl : &gl::WebGl2RenderingContext, 
      object_color_texture : WebGlTexture,
      outline_thickness : f32,
      width : u32, 
      height : u32 
    ) -> Result< Self, gl::WebglError >
    {
      let program_infos = ProgramInfos::new( gl );

      // --- Create Framebuffers and Textures ---

      // Framebuffer for the JFA initialization pass
      let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( gl, width as i32, height as i32, None ).unwrap();
      // Framebuffers for the JFA step passes ( ping-pong )
      let ( jfa_step_fb_0, jfa_step_fb_color_0 ) = create_framebuffer( gl, width as i32, height as i32, None ).unwrap();
      let ( jfa_step_fb_1, jfa_step_fb_color_1 ) = create_framebuffer( gl, width as i32, height as i32, None ).unwrap();
      let ( outline_fb, output_color ) = create_framebuffer( gl, width as i32, height as i32, None ).unwrap();

      let mut textures = HashMap::new();

      // Store the color attachment textures
      textures.insert( "object_color".to_string(), object_color_texture );
      textures.insert( "jfa_init_fb_color".to_string(), jfa_init_fb_color );
      textures.insert( "jfa_step_fb_color_0".to_string(), jfa_step_fb_color_0 );
      textures.insert( "jfa_step_fb_color_1".to_string(), jfa_step_fb_color_1 );
      textures.insert( "output_color".to_string(), output_color );

      let mut framebuffers = HashMap::new();

      // Store the framebuffers
      framebuffers.insert( "jfa_init_fb".to_string(), jfa_init_fb );
      framebuffers.insert( "jfa_step_fb_0".to_string(), jfa_step_fb_0 );
      framebuffers.insert( "jfa_step_fb_1".to_string(), jfa_step_fb_1 );
      framebuffers.insert( "outline_fb".to_string(), outline_fb );

      let num_passes = 4;//( width.max( height ) as f32 ).log2().ceil() as u32;

      let pass = Self
      {
        program_infos,
        framebuffers,
        textures,
        outline_thickness,
        width,
        height,
        num_passes
      };

      Ok( pass )
    }    

    /// Sets the thickness of the outline.
    pub fn set_outline_thickness( &mut self, new_value : f32 )
    {
      self.outline_thickness = new_value;
    }

    /// Sets the number of passes for the wide outline algorithm.
    pub fn set_num_passes( &mut self, new_value : u32 )
    {
      self.num_passes = new_value;
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
      let object_color = self.textures.get( "object_color" ).unwrap();

      let jfa_init_locs = self.program_infos.jfa_init.1.get_locations();

      let object_color_loc = jfa_init_locs.get( "objectColorTexture" ).unwrap().clone().unwrap();

      gl.use_program( Some( jfa_init_program ) );

      upload_framebuffer( gl, jfa_init_fb, self.width as i32, self.height as i32 );

      upload_texture( gl, object_color, &object_color_loc, GL::TEXTURE0 );

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
    fn jfa_step_pass( &self, gl : &gl::WebGl2RenderingContext, i : u32 )
    {
      let jfa_step_program = &self.program_infos.jfa_step.0;
      let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
      let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
      let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap(); // Initial JFA texture
      let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // Color texture for FB 0
      let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // Color texture for FB 1

      let jfa_step_locs = self.program_infos.jfa_step.1.get_locations();

      let resolution = jfa_step_locs.get( "resolution" ).unwrap().clone().unwrap();
      let u_step_size = jfa_step_locs.get( "stepSize" ).unwrap().clone().unwrap();
      let jfa_init_loc = jfa_step_locs.get( "jfaTexture" ).unwrap().clone().unwrap();

      gl.use_program( Some( jfa_step_program ) );

      // Ping-pong rendering: Determine input texture and output framebuffer based on step index `i`
      if i == 0 // First step uses the initialization result
      {
        upload_framebuffer( gl, jfa_step_fb_0, self.width as i32, self.height as i32 ); // Render to FB 0
        upload_texture( gl, jfa_init_fb_color, &jfa_init_loc, GL::TEXTURE0 ); // Input is JFA init texture
      }
      else if i % 2 == 0 // Even steps ( 2, 4, ... ) read from FB 1, render to FB 0
      {
        upload_framebuffer( gl, jfa_step_fb_0, self.width as i32, self.height as i32 ); // Render to FB 0
        upload_texture( gl, &jfa_step_fb_color_1, &jfa_init_loc, GL::TEXTURE0 ); // Input is texture from FB 1
      }
      else // Odd steps ( 1, 3, ... ) read from FB 0, render to FB 1
      {
        upload_framebuffer( gl, jfa_step_fb_1, self.width as i32, self.height as i32 ); // Render to FB 1
        upload_texture( gl, jfa_step_fb_color_0, &jfa_init_loc, GL::TEXTURE0 ); // Input is texture from FB 0
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
    fn outline_pass
    ( 
      &self, 
      gl : &gl::WebGl2RenderingContext, 
      input_texture : Option< minwebgl::web_sys::WebGlTexture >,
      output_texture : Option< minwebgl::web_sys::WebGlTexture > 
    )
    {
      let outline_program = &self.program_infos.outline.0;
      let outline_fb = self.framebuffers.get( "outline_fb" ).unwrap();
      let source = input_texture.unwrap();
      let object_color = self.textures.get( "object_color" ).unwrap();
      let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // JFA ping-pong texture 0
      let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // JFA ping-pong texture 1

      let outline_locs = self.program_infos.outline.1.get_locations();

      let source_loc = outline_locs.get( "sourceTexture" ).unwrap().clone().unwrap();
      let object_color_loc = outline_locs.get( "objectColorTexture" ).unwrap().clone().unwrap();
      let jfa_step_loc = outline_locs.get( "jfaTexture" ).unwrap().clone().unwrap();
      let resolution = outline_locs.get( "resolution" ).unwrap().clone().unwrap();

      set_framebuffer_color( gl, &outline_fb, Some( output_texture.unwrap() ) );

      gl.use_program( Some( outline_program ) );

      // Bind the default framebuffer ( render to canvas )
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( outline_fb ) );

      gl::uniform::upload( gl, Some( resolution.clone() ), &[ self.width as f32, self.height as f32 ] ).unwrap();

      upload_texture( gl, &source, &source_loc, GL::TEXTURE0 );
      upload_texture( gl, object_color, &object_color_loc, GL::TEXTURE1 );
      // The final JFA result is in jfa_step_fb_color_0 if num_passes is even, otherwise in jfa_step_fb_color_1
      if self.num_passes % 2 == 0
      {
        upload_texture( gl, jfa_step_fb_color_0, &jfa_step_loc, GL::TEXTURE2 );
      }
      else
      {
        upload_texture( gl, jfa_step_fb_color_1, &jfa_step_loc, GL::TEXTURE2 );
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
      // 2. JFA Initialization Pass: Initialize JFA texture from the silhouette
      self.jfa_init_pass( gl );

      // 3. JFA Step Passes: Perform Jump Flooding Algorithm steps
      // The number of passes required is log2( max( width, height ) ).
      for i in 0..self.num_passes
      {
        self.jfa_step_pass( gl, i );
      }

      // 4. Outline Pass: Generate and render the final scene with the outline
      self.outline_pass( gl, input_texture, output_texture.clone() );

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
    WideOutlinePass
  };
}