
mod private
{
  use minwebgl as gl;
  use gl::GL;
  use crate::webgl::{ ProgramInfo, ShaderProgram, post_processing::{ Pass, VS_TRIANGLE }, program::NarrowOutlineShader };

  /// A struct representing a rendering pass for drawing narrow outlines.
  pub struct NarrowOutlinePass
  {
    /// `ProgramInfo` holds the WebGL program and its uniform/attribute locations.
    program_info : ProgramInfo,
    /// The texture containing position data from the G-Buffer.
    position_texture : Option< gl::web_sys::WebGlTexture >,
    /// The texture containing object color or ID data from the G-Buffer.
    object_color_texture : Option< gl::web_sys::WebGlTexture >,
    /// The thickness of the outline to be rendered.
    outline_thickness : f32,
    /// The width of the viewport/texture.
    width : u32,
    /// The height of the viewport/texture.
    height : u32
  }

  impl NarrowOutlinePass
  {
    /// Creates a new `NarrowOutlinePass` instance.
    pub fn new(
      gl : &gl::WebGl2RenderingContext,
      position_texture : Option< gl::web_sys::WebGlTexture >,
      object_color_texture : Option< gl::web_sys::WebGlTexture >,
      outline_thickness : f32,
      width : u32,
      height : u32
    ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../../shaders/post_processing/outline/narrow_outline.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let program_info = ProgramInfo::new( gl, &program, NarrowOutlineShader.dyn_clone() );

      {
        program_info.bind( gl );
        let locations = program_info.get_locations();

        let source_texture_loc = locations.get( "sourceTexture" ).unwrap().clone();
        let position_texture_loc = locations.get( "positionTexture" ).unwrap().clone();
        let object_color_texture_loc = locations.get( "objectColorTexture" ).unwrap().clone();

        gl.uniform1i( source_texture_loc.as_ref(), 0 );
        gl.uniform1i( position_texture_loc.as_ref(), 1 );
        gl.uniform1i( object_color_texture_loc.as_ref(), 2 );
        gl.use_program( None );
      }

      let pass = Self
      {
        program_info,
        position_texture,
        object_color_texture,
        outline_thickness,
        width,
        height
      };

      Ok( pass )
    }

    /// Sets the thickness of the outline.
    pub fn set_outline_thickness( &mut self, new_value : f32 )
    {
      self.outline_thickness = new_value;
    }
  }

  impl Pass for NarrowOutlinePass
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
      self.program_info.bind( gl );

      let locations = self.program_info.get_locations();

      let resolution_loc = locations.get( "resolution" ).unwrap().clone();
      let outline_thickness_loc = locations.get( "outlineThickness" ).unwrap().clone();

      gl::uniform::upload( gl, resolution_loc, &[ self.width as f32, self.height as f32 ] ).unwrap();
      gl::uniform::upload( gl, outline_thickness_loc, &[ self.outline_thickness ] ).unwrap();

      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        output_texture.as_ref(),
        0
      );

      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );

      gl.active_texture( gl::TEXTURE1 );
      gl.bind_texture( gl::TEXTURE_2D, self.position_texture.as_ref() );

      gl.active_texture( gl::TEXTURE2 );
      gl.bind_texture( gl::TEXTURE_2D, self.object_color_texture.as_ref() );

      gl.clear( GL::COLOR_BUFFER_BIT );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, [ -1.0, -1.0, -1.0, 1.0 ].as_slice() );

      gl.draw_arrays( GL::TRIANGLES, 0, 3 );

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
    NarrowOutlinePass
  };
}
