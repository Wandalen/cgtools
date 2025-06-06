
mod private
{
  use minwebgl as gl;
  use web_sys::WebGlBuffer;
  use gl::GL;
  use crate::webgl::{ post_processing::{ Pass, VS_TRIANGLE }, program::NarrowOutlineShader, ProgramInfo };

  pub const MAX_OBJECT_COUNT : usize = 1024;

  pub struct NarrowOutlinePass
  {
    program_info : ProgramInfo< NarrowOutlineShader >,
    depth_texture : Option< gl::web_sys::WebGlTexture >,
    object_id_texture : Option< gl::web_sys::WebGlTexture >,
    outline_thickness : f32,
    _object_colors : Option< Vec< [ f32; 4 ] > >,
    object_colors_buffer : WebGlBuffer,
    width : u32,
    height : u32
  }

  impl NarrowOutlinePass 
  {
    /// Creates a new `NarrowOutlinePass` instance.
    pub fn new( 
      gl : &gl::WebGl2RenderingContext, 
      depth_texture : Option< gl::web_sys::WebGlTexture >,
      object_id_texture : Option< gl::web_sys::WebGlTexture >,
      outline_thickness : f32,
      width : u32, 
      height : u32 
    ) -> Result< Self, gl::WebglError >
    {
      let fs_shader = include_str!( "../../shaders/post_processing/outline/narrow_outline.frag" );
      let program = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let program_info = ProgramInfo::< NarrowOutlineShader >::new( gl, program.clone() );

      {
        let locations = program_info.get_locations();

        let source_texture_loc = locations.get( "sourceTexture" ).unwrap().clone();
        let depth_texture_loc = locations.get( "depthTexture" ).unwrap().clone();
        let object_id_texture_loc = locations.get( "objectIdTexture" ).unwrap().clone();

        gl.uniform1i( source_texture_loc.as_ref(), 0 );
        gl.uniform1i( depth_texture_loc.as_ref(), 1 );
        gl.uniform1i( object_id_texture_loc.as_ref(), 2 );
      }

      let object_colors_buffer = gl::buffer::create( &gl )?;
      let object_colors_loc = gl.get_uniform_block_index( &program, "ObjectColorBlock" );
      gl.uniform_block_binding( &program, object_colors_loc, 0 );
      gl.bind_buffer_base( GL::UNIFORM_BUFFER, 0, Some( &object_colors_buffer ) );
      gl.bind_buffer( GL::UNIFORM_BUFFER, Some( &object_colors_buffer ) );
      gl.buffer_data_with_i32( GL::UNIFORM_BUFFER, MAX_OBJECT_COUNT as i32 * 16, GL::DYNAMIC_DRAW );

      let mut pass = Self
      {
        program_info,
        depth_texture,
        object_id_texture,
        outline_thickness,
        _object_colors : None,
        object_colors_buffer,
        width,
        height
      };

      pass.set_object_colors( gl, vec![ [ 0.0; 4 ]; MAX_OBJECT_COUNT ] );

      Ok( pass )
    }    

    pub fn set_outline_thickness( &mut self, new_value : f32 )
    {
      self.outline_thickness = new_value;
    }

    pub fn set_object_colors( &mut self, gl : &gl::WebGl2RenderingContext, object_colors: Vec< [ f32; 4 ] > )
    {
      self._object_colors = Some( object_colors.clone() );
      let object_colors = object_colors.into_iter().flatten().collect::< Vec< _ > >();
      gl::ubo::upload( &gl, &self.object_colors_buffer, 0, &object_colors[ .. ], GL::DYNAMIC_DRAW );
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

      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, input_texture.as_ref() );
      
      gl.active_texture( gl::TEXTURE1 );
      gl.bind_texture( gl::TEXTURE_2D, self.depth_texture.as_ref() );

      gl.active_texture( gl::TEXTURE2 );
      gl.bind_texture( gl::TEXTURE_2D, self.object_id_texture.as_ref() );
      
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        output_texture.as_ref(), 
        0
      );

      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

      gl.draw_arrays( GL::TRIANGLES, 0, 3 );

      // --- Cleanup ---
      // Unbind the texture and framebuffer attachment to restore default state.
      gl.bind_vertex_array( None );
      gl.bind_texture( gl::TEXTURE_2D, None );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        None, 
        0
      );

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
    NarrowOutlinePass,
    MAX_OBJECT_COUNT
  };
}