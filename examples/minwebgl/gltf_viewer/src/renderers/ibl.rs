use minwebgl as gl;

pub struct IBLManager
{

}

impl IBLManager 
{
  pub async fn new( gl : &gl::WebGl2RenderingContext, path : &str ) -> Result< Self, gl::WebglError >
  {
    let load_texture = async | name |
    {
      let file = gl::file::load( &format!( "{}/{}.hdr", path, name ) ).await.expect( "Failed to load gltf file" );
      let mut decoder = zune_hdr::HdrDecoder::new( &file );
      let ( width, height ) = decoder.get_dimensions().unwrap();
      let data = decoder.decode().expect( &format!( "Failed to decode {}.hdr", name ) );

      let texture = gl.create_texture();
      gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
      (
        gl::TEXTURE_2D,
        0,
        gl::RGB32F as i32,
        width as i32,
        height as i32,
        0,
        gl::RGB,
        gl::FLOAT,
        Some( bytemuck::cast_slice( &data ) )
      ).unwrap();

      texture
    };

    let diffuse_texture = load_texture( "diffuse" ).await;
    let specular_1_0_texture = load_texture( "specular_1_0" ).await;
    let specular_1_1_texture = load_texture( "specular_1_1" ).await;
    let specular_1_2_texture = load_texture( "specular_1_2" ).await;
    let specular_1_3_texture = load_texture( "specular_1_3" ).await;
    let specular_1_4_texture = load_texture( "specular_1_4" ).await;
    let specular_2_texture = load_texture( "specular_2" ).await;

    Ok
    (
      Self
      {

      }
    )
  }    
}