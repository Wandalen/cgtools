use minwebgl as gl;

pub async fn load_to_mip_cube
(
  gl : &gl::WebGl2RenderingContext, 
  texture : Option< &gl::web_sys::WebGlTexture >,
  mip_level : u32,
  path : &str
)
{
  let image = gl::file::load( path ).await.unwrap();

  let image = std::io::Cursor::new( image );
  let mut decoder = zune_hdr::HdrDecoder::new( image );
  let data = decoder.decode().expect( &format!( "Failed to decode {}", path ) );
  let ( width, height ) = decoder.dimensions().unwrap();

  let image_slice = | i : usize |
  {
    let start = width * width * 3 * i;
    let end = start + width * width * 3;
    start..end
  };


  let image_data : gl::js_sys::Object = gl::js_sys::Float32Array::from( data.as_slice() ).into();

  gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, texture );
  for i in 0..6
  {
    // +Y and -Y need to be swapped
    let mut i2 = i;
    if i == 2 { i2 = 3; }
    if i == 3 { i2 = 2; }

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
    (
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
      mip_level as i32,
      gl::RGB16F as i32,
      width as i32,
      width as i32,
      0,
      gl::RGB,
      gl::FLOAT,
      &image_data,
      ( width * width * 3  ) as u32 * i2
    ).expect( "Failed to allocate memory for a cube texture" );
  }
  gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

  gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );
}

pub async fn load_to_mip_d2
(
  gl : &gl::WebGl2RenderingContext, 
  texture : Option< &gl::web_sys::WebGlTexture >,
  mip_level : u32,
  path : &str
)
{
  let image = gl::file::load( path ).await.unwrap();

  let image = std::io::Cursor::new( image );
  let mut decoder = zune_hdr::HdrDecoder::new( image );
  let data = decoder.decode().expect( &format!( "Failed to decode {}", path ) );
  let ( width, height ) = decoder.dimensions().unwrap();

  let image_data : gl::js_sys::Object = gl::js_sys::Float32Array::from( data.as_slice() ).into();

  //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
  gl.bind_texture( gl::TEXTURE_2D, texture );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
  (
    gl::TEXTURE_2D,
    mip_level as i32,
    gl::RGB16F as i32,
    width as i32,
    height as i32,
    0,
    gl::RGB,
    gl::FLOAT,
    &image_data,
    0
  ).expect( "Failed to allocate memory for a cube texture" );
  gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

  gl.bind_texture( gl::TEXTURE_2D, None );
}
