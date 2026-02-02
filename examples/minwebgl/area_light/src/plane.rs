use minwebgl as gl;
use gl::GL;
use web_sys::{ WebGlTexture, WebGlVertexArrayObject };

pub fn plane_material
(
  gl : &GL,
  base_color : [ u8; 4 ],
  ao : f32,
  roughness : f32,
  metalness : f32,
) -> ( Option< WebGlTexture >, Option< WebGlTexture > )
{
  let base_color_tex = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, base_color_tex.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset
  (
    gl::TEXTURE_2D,
    0,
    gl::RGBA8 as i32,
    1,
    1,
    0,
    gl::RGBA,
    gl::UNSIGNED_BYTE,
    base_color.as_slice(),
    0
  ).unwrap();
  gl::texture::d2::filter_nearest( &gl );
  gl::texture::d2::wrap_clamp( &gl );

  let arm =
  [
    ( ao        * u8::MAX as f32 ).round() as u8,
    ( roughness * u8::MAX as f32 ).round() as u8,
    ( metalness * u8::MAX as f32 ).round() as u8,
    0,
  ];
  let arm_tex = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, arm_tex.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset
  (
    gl::TEXTURE_2D,
    0,
    gl::RGBA8 as i32,
    1,
    1,
    0,
    gl::RGBA,
    gl::UNSIGNED_BYTE,
    arm.as_slice(),
    0
  ).unwrap();
  gl::texture::d2::filter_nearest( &gl );
  gl::texture::d2::wrap_clamp( &gl );

  ( base_color_tex, arm_tex )
}

pub fn plane_vao( gl : &GL ) -> Result< WebGlVertexArrayObject, gl::WebglError >
{
  let plane_vertices : &[ f32 ] =
  &[
    // position         // normal         // texcoord
    -1.0, 0.0,  1.0,    0.0, 1.0, 0.0,    0.0, 0.0,
     1.0, 0.0,  1.0,    0.0, 1.0, 0.0,    0.0, 1.0,
    -1.0, 0.0, -1.0,    0.0, 1.0, 0.0,    1.0, 0.0,
     1.0, 0.0, -1.0,    0.0, 1.0, 0.0,    1.0, 0.0,
  ];

  let vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  let vbo = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &vbo, plane_vertices, gl::STATIC_DRAW );

  gl::BufferDescriptor::new::< [ f32; 3 ] >()
  .offset( 0 )
  .stride( 8 )
  .attribute_pointer( gl, 0, &vbo )?;

  gl::BufferDescriptor::new::< [ f32; 3 ] >()
  .offset( 3 )
  .stride( 8 )
  .attribute_pointer( gl, 1, &vbo )?;

  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .offset( 6 )
  .stride( 8 )
  .attribute_pointer( gl, 2, &vbo )?;

  Ok( vao )
}
