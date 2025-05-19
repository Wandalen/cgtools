use minwebgl as gl;

pub struct CubeTextureRenderer2
{
  cube_map : Option< gl::web_sys::WebGlTexture >,
}

impl CubeTextureRenderer2
{
  pub async fn new( gl : &gl::WebGl2RenderingContext  ) -> Result< Self, gl::WebglError >
  {
    let image = gl::file::load( "specular_1_7.hdr" ).await.unwrap();
    //let image = gl::file::load( "diffuse.hdr" ).await.unwrap();

    let image = std::io::Cursor::new( image );
    let mut decoder = zune_hdr::HdrDecoder::new( image );
    let data = decoder.decode().expect( &format!( "Failed to decode {}.hdr", "specular_1_0" ) );
    let ( width, height ) = decoder.dimensions().unwrap();
    gl::info!( "{:?}", data );

    // gl::info!( "{}", data.len() );
    // gl::info!( "{}", width * height * 3 );
    gl::info!( "{}", data.len() / 3 );
    gl::info!( "{}|{}", width, height );

    let image_slice = | i : usize |
    {
      let start = width * width * 3 * i;
      let end = start + width * width * 3;
      start..end
    };


    let image_data : gl::js_sys::Object = gl::js_sys::Float32Array::from( data.as_slice() ).into();

    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
    let cube_map = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, cube_map.as_ref() );
    for i in 0..6
    {
      let mut i2 = i;
      if i == 2 { i2 = 3; }
      if i == 3 { i2 = 2; }

      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
      (
        gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
        0,
        gl::RGB16F as i32,
        width as i32,
        width as i32,
        0,
        gl::RGB,
        gl::FLOAT,
        &gl::js_sys::Float32Array::from( &data[ image_slice( i2 as usize ) ] ).into(),
        //&image_data,
        0,
        //( width * width * 3  ) as u32 * i2
      ).expect( "Failed to allocate memory for a cube texture" );
    }
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );

    Ok
    (
      Self
      {
        cube_map,
      }
    )
  }

  pub fn bind_texture( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, self.cube_map.as_ref() );
  }
}

const CUBE_VERTICES : [ f32; 108 ] =
[
  // front
  -1.0, -1.0,  1.0,
   1.0, -1.0,  1.0,
  -1.0,  1.0,  1.0,
 
  -1.0,  1.0,  1.0,
   1.0, -1.0,  1.0,
   1.0,  1.0,  1.0,
  // right
   1.0, -1.0,  1.0,
   1.0, -1.0, -1.0,
   1.0,  1.0,  1.0,
 
   1.0,  1.0,  1.0,
   1.0, -1.0, -1.0,
   1.0,  1.0, -1.0,
  // back
   1.0, -1.0, -1.0,
  -1.0, -1.0, -1.0,
   1.0,  1.0, -1.0,
 
   1.0,  1.0, -1.0,
  -1.0, -1.0, -1.0,
  -1.0,  1.0, -1.0,
  // left
  -1.0, -1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0,  1.0, -1.0,
 
  -1.0,  1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0,  1.0,  1.0,
  // top
   1.0,  1.0, -1.0,
  -1.0,  1.0, -1.0,
   1.0,  1.0,  1.0,
 
   1.0,  1.0,  1.0,
  -1.0,  1.0, -1.0,
  -1.0,  1.0,  1.0,
  // bottom
   1.0, -1.0,  1.0,
  -1.0, -1.0,  1.0,
   1.0, -1.0, -1.0,
 
   1.0, -1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0, -1.0, -1.0,
];