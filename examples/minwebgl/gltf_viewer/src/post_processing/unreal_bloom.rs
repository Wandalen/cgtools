
use minwebgl as gl;
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
    let output_texture = gl.create_texture();

    gl.bind_texture( gl::TEXTURE_2D, texture);
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

    Self 
    { 

    }
  }    
}