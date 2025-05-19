use minwebgl as gl;

pub fn load_to_mip
(
  gl : &gl::WebGl2RenderingContext, 
  texture : Option< &gl::web_sys::WebGlTexture >,
  mip_level : u32,
  width : u32,
  height : u32,
  color : [ u8; 3 ]
)
{
  let image_data = vec![ color; ( width * height ) as usize ];

  gl.bind_texture( gl::TEXTURE_2D, texture );

  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset
  (
    gl::TEXTURE_2D,
    mip_level as i32,
    gl::RGB as i32,
    width as i32,
    height as i32,
    0,
    gl::RGB,
    gl::UNSIGNED_BYTE,
    image_data.as_flattened(),
    0
  ).expect( "Failed to allocate memory for a cube texture" );

  gl.bind_texture( gl::TEXTURE_2D, None );
}

