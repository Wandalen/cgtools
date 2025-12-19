mod private
{
  use minwebgl as gl;

  /// Loads an HDR image from a file path, decodes it, and uploads it to a specific mipmap level of a WebGL cube map texture.
  ///
  /// This function handles the file loading and HDR decoding using the `zune_hdr` library. It then iterates
  /// through the six faces of the cube map, uploading the appropriate slice of the decoded image data.
  /// Note that the function swaps the +Y and -Y faces to align with common graphics conventions.
  ///
  /// # Arguments
  ///
  /// * `gl` - The `WebGl2RenderingContext`.
  /// * `texture` - An optional reference to the WebGL texture to bind to.
  /// * `mip_level` - The mipmap level to upload the data to.
  /// * `path` - The file path to the HDR image.
  #[ allow( unused_variables ) ]
  pub async fn load_to_mip_cube
  (
    gl : &gl::WebGl2RenderingContext,
    texture : Option< &gl::web_sys::WebGlTexture >,
    mip_level : u32,
    path : &str
  )
  {
    let image = gl::file::load( path ).await.expect( "Can't load image" );
    let image = std::io::Cursor::new( image );
    let mut decoder = zune_hdr::HdrDecoder::new( image );
    let data = decoder.decode()
    .unwrap_or_else( | _ | panic!( "Failed to decode {}", path ) );
    let ( width, height ) = decoder.dimensions().expect( "Can't get image dimensions" );

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
      if i == 2
      {
        i2 = 3;
      }
      if i == 3
      {
        i2 = 2;
      }

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
        ( width * width * 3 ) as u32 * i2
      ).expect( "Failed to allocate memory for a cube texture" );
    }
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

    gl::texture::cube::wrap_clamp( gl );
    gl::texture::cube::filter_linear( gl );

    gl.bind_texture( gl::TEXTURE_CUBE_MAP, None );
  }

  /// Loads an HDR image from a file path, decodes it, and uploads it to a specific mipmap level of a WebGL 2D texture.
  ///
  /// This function loads the image data, decodes it, and then uploads it to the GPU as a 2D texture.
  /// It sets the texture wrapping and filtering to linear.
  ///
  /// # Arguments
  ///
  /// * `gl` - The `WebGl2RenderingContext`.
  /// * `texture` - An optional reference to the WebGL texture to bind to.
  /// * `mip_level` - The mipmap level to upload the data to.
  /// * `path` - The file path to the HDR image.
  #[ allow( unused_variables ) ]
  pub async fn load_to_mip_d2
  (
    gl : &gl::WebGl2RenderingContext,
    texture : Option< &gl::web_sys::WebGlTexture >,
    mip_level : u32,
    path : &str
  )
  {
    let image = gl::file::load( path ).await.expect( "Can't load image" );
    let image = std::io::Cursor::new( image );
    let mut decoder = zune_hdr::HdrDecoder::new( image );
    let data = decoder.decode()
    .unwrap_or_else( | _ | panic!( "Failed to decode {}", path ) );
    let ( width, height ) = decoder.dimensions().expect( "Can't get image dimensions" );

    let image_data : gl::js_sys::Object = gl::js_sys::Float32Array::from( data.as_slice() ).into();

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
    )
    .expect( "Failed to allocate memory for a cube texture" );
    gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

    gl::texture::d2::wrap_clamp( gl );
    gl::texture::d2::filter_linear( gl );

    gl.bind_texture( gl::TEXTURE_2D, None );
  }
}

crate::mod_interface!
{
  own use
  {
    load_to_mip_cube,
    load_to_mip_d2
  };
}
