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
  gl::texture::d2::filter_nearest( gl );
  gl::texture::d2::wrap_clamp( gl );

  let arm =
  [
    ( ao        * f32::from(u8::MAX) ).round() as u8,
    ( roughness * f32::from(u8::MAX) ).round() as u8,
    ( metalness * f32::from(u8::MAX) ).round() as u8,
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
  gl::texture::d2::filter_nearest( gl );
  gl::texture::d2::wrap_clamp( gl );

  ( base_color_tex, arm_tex )
}

#[ repr( C ) ]
#[ derive( Debug, Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
struct Vertex
{
  position : [ f32; 3 ],
  normal : [ f32; 3 ],
  texcoord : [ f32; 2 ],
}

impl mingl::Attribute for Vertex
{
  fn describe() -> Vec< mingl::VertexAttribute >
  {
    vec!
    [
      mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ),
      mingl::VertexAttribute::new( 1, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 3 ),
      mingl::VertexAttribute::new( 2, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 6 ),
    ]
  }
}

pub fn plane_vao( gl : &GL ) -> Result< WebGlVertexArrayObject, gl::WebglError >
{
  // Fix(BUG-321): vertex 3's texcoord was `( 1.0, 0.0 )`, a duplicate of vertex 2's —
  // breaking the bilinear UV grid the other 3 vertices establish ( uv.x tracks -z,
  // uv.y tracks x ), which requires vertex 3 ( x=1, z=-1, the corner diagonal from
  // vertex 0 ) to be `( 1.0, 1.0 )`. Invisible today only because `plane_material`
  // currently fills both textures with a single constant 1x1 texel ( any UV samples the
  // same color under `wrap_clamp`/`filter_nearest` ), but wrong the moment a real
  // ( non-1x1 ) texture is bound here.
  // Root cause: vertex 3's texcoord row was copy-pasted from vertex 2's instead of being
  // computed for its own corner.
  // Pitfall: don't "fix" this by touching vertices 0/1/2 — they already form a correct,
  // consistent grid; only vertex 3's row was wrong.
  let plane_vertices : &[ Vertex ] =
  &[
    Vertex { position : [ -1.0, 0.0,  1.0 ], normal : [ 0.0, 1.0, 0.0 ], texcoord : [ 0.0, 0.0 ] },
    Vertex { position : [  1.0, 0.0,  1.0 ], normal : [ 0.0, 1.0, 0.0 ], texcoord : [ 0.0, 1.0 ] },
    Vertex { position : [ -1.0, 0.0, -1.0 ], normal : [ 0.0, 1.0, 0.0 ], texcoord : [ 1.0, 0.0 ] },
    Vertex { position : [  1.0, 0.0, -1.0 ], normal : [ 0.0, 1.0, 0.0 ], texcoord : [ 1.0, 1.0 ] },
  ];

  let vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  let vbo = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &vbo, plane_vertices, gl::STATIC_DRAW );

  let plane_layout = mingl::VertexBufferLayout::from_attribute::< Vertex >( 8 );
  gl::vertex_buffer_layout_bind( gl, &vbo, &plane_layout )?;

  Ok( vao )
}
