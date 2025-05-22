
mod private
{
  use minwebgl as gl;

  const KERNEL_RADIUS : [ i32; 5 ] = [ 3, 5, 7, 9, 11 ];

  pub struct UnrealBloom
  {

  }

  impl UnrealBloom 
  {
    pub fn new
    ( 
      gl : &gl::WebGl2RenderingContext, 
      width : u32,
      height : u32
    ) -> Self
    {
      let frame_buffer = gl.create_framebuffer();

      let mut horizontal_target = Vec::new();
      let mut vertical_target = Vec::new();

      let allocate = | t : Option< &gl::web_sys::WebGlTexture >, width, height |
      {
        gl.bind_texture( gl::TEXTURE_2D, t );
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
        (
          gl::TEXTURE_2D,
          0,
          gl::RGB16F as i32,
          width as i32,
          height as i32,
          0,
          gl::RGB,
          gl::FLOAT,
          &gl::js_sys::Float32Array::from( [].as_slice() ).into(),
          0
        ).expect( "Failed to allocate memory for a cube texture" );
      };

      let mut size = ( width / 2, height / 2 );
      // Generate textures for blurs at different mipmap levels.
      // The blur will go in two passes : image > horizontal > vertical
      for i in 0..5
      {
        let horizontal = gl.create_texture();
        let vertical = gl.create_texture();

        allocate( horizontal.as_ref(), size.0, size.1 );
        allocate( vertical.as_ref(), size.0, size.1 );

        horizontal_target.push( horizontal );
        vertical_target.push( vertical );

        size.0 /= 2;
        size.1 /= 2;
      }

      let vs_shader = include_str!( "../shaders/big_triangle.vert" );
      let filter_shader = include_str!( "../shaders/gaussian_filter.frag" )

      

      Self 
      { 

      }
    }    
  }
}

crate::mod_interface!
{
  
}