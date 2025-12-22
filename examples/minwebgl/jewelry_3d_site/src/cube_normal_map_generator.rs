use std::{ rc::Rc, cell::RefCell };
use minwebgl as gl;
use gl::
{
  GL,
  WebglError,
  F32x4x4,
  WebGlProgram
};
use renderer::webgl::
{
  MagFilterMode,
  MinFilterMode,
  Node,
  Object3D,
  Sampler,
  TextureInfo,
  Texture,
  ShaderProgram,
  WrappingMode,
  ProgramInfo
};
use renderer::impl_locations;
use rustc_hash::FxHashMap;
use web_sys::WebGlFramebuffer;

// CubeNormalMapGenerator shader program
impl_locations!
(
  CubeNormalMapGeneratorShader,
  "worldMatrix",
  "normalMatrix",
  "viewMatrix",
  "projectionMatrix",
  "offsetMatrix",

  "maxDistance"
);

// Given from here
// https://github.com/mrdoob/three.js/blob/master/src/cameras/CubeCamera.js
fn make_cube_camera() -> [ gl::F32x4x4; 6 ]
{
  let px = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::X, gl::F32x3::NEG_Y );
  let nx = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_X, gl::F32x3::NEG_Y );

  let py = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Y, gl::F32x3::Z );
  let ny = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Y, gl::F32x3::NEG_Z );

  let pz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Z, gl::F32x3::NEG_Y );
  let nz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Z, gl::F32x3::NEG_Y );

  [ px, nx, py, ny, pz, nz ]
}

fn gen_cube_texture( gl : &GL, width: i32, height: i32 ) -> Option< gl::web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, texture.as_ref() );

  for i in 0..6
  {
    gl. tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
      0,
      gl::RGBA16F as i32,
      width as i32,
      height as i32,
      0,
      gl::RGBA,
      gl::FLOAT,
      None
    ).expect( "Failed to upload data to texture" );
  }

  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

  texture
}

/// Cube normal map generator. This maps can be used for
/// generating realistic reflections inside gem geometry
pub struct CubeNormalMapGenerator
{
  /// Generator shader program info
  program : CubeNormalMapGeneratorShader,
  /// Framebuffer used for rendering cube maps
  framebuffer : WebGlFramebuffer,
  /// View matrices for each side of renderer cube map
  cube_camera : [ F32x4x4; 6 ],
  /// Cube map one side width
  texture_width : u32,
  /// Cube map one side height
  texture_height : u32,
}

impl CubeNormalMapGenerator
{
  /// Creates new [`CubeNormalMapGenerator`]
  pub fn new( gl : &GL ) -> Result< Self, WebglError >
  {
    let vertex_shader_src = include_str!( "../shaders/gen_cube_map.vert" );
    let fragment_shader_src = include_str!( "../shaders/gen_cube_map.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
    let program = CubeNormalMapGeneratorShader::new( gl, &program );

    let framebuffer = gl.create_framebuffer().ok_or( WebglError::FailedToAllocateResource( "Framebuffer" ) )?;
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &framebuffer ) );
    gl.viewport( 0, 0, 512, 512 );
    gl.clear_color( 1.0, 0.0, 0.0, 1.0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER , None );

    let cube_camera = make_cube_camera();

    Ok
    (
      Self
      {
        program,
        framebuffer,
        cube_camera,
        texture_width : 512,
        texture_height : 512
      }
    )
  }

  /// Sets cube normal map resolution
  pub fn set_texture_size( &mut self, gl : &GL, width : u32, height : u32 )
  {
    self.texture_width = width;
    self.texture_height = height;
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.viewport( 0, 0, self.texture_width as i32, self.texture_height as i32 );
    gl.bind_framebuffer( gl::FRAMEBUFFER , None );
  }

  /// Generates cube normal maps only for [`Node`]'s that have [`Mesh`] as [`Node::object`]
  pub fn generate( &self, gl : &GL, node : &Rc< RefCell< Node > > ) -> Option< TextureInfo >
  {
    let Object3D::Mesh( mesh ) = &node.borrow().object
    else
    {
      return None;
    };

    let bb = node.borrow().bounding_box();
    let c = bb.center();
    let max_distance = ( bb.max - c ).mag().max( ( bb.min - c ).mag() );

    let offset_matrix = gl::math::mat3x3h::translation( -c );
    let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl
    (
      90.0f32.to_radians(),
      1.0,
      0.01,
      max_distance * 16.0
    );

    let locations = self.program.locations();
    let projection_matrix_location = locations.get( "projectionMatrix" ).unwrap().clone();
    let view_matrix_location = locations.get( "viewMatrix" ).unwrap();
    let max_distance_location = locations.get( "maxDistance" ).unwrap().clone();
    let offset_matrix_location = locations.get( "offsetMatrix" ).unwrap().clone();

    self.program.bind( gl );

    node.borrow().upload( gl, locations );
    gl::uniform::matrix_upload( &gl, projection_matrix_location, &perspective_matrix.to_array(), true ).ok()?;
    gl::uniform::matrix_upload( &gl, offset_matrix_location, &offset_matrix.to_array(), true ).ok()?;
    gl::uniform::upload( &gl, max_distance_location, &max_distance ).ok()?;

    let cube_texture = gen_cube_texture( &gl, self.texture_width as i32, self.texture_height as i32 );

    // Render to our cube texture using custom frame buffer
    // All the needed buffers were setup above
    gl.bind_framebuffer( gl::FRAMEBUFFER , Some( &self.framebuffer ) );

    for i in 0..6
    {
      let view_matrix = &self.cube_camera[ i ].to_array();
      gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), view_matrix, true ).ok()?;
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
        cube_texture.as_ref(),
        0
      );
      gl.clear( gl::COLOR_BUFFER_BIT );

      for primitive in &mesh.borrow().primitives
      {
        let primitive_ref = primitive.borrow();
        let geometry_ref = primitive_ref.geometry.borrow();
        geometry_ref.upload( gl ).ok()?;
        geometry_ref.draw( gl );
        gl.bind_vertex_array( None );
      }
    }

    gl.bind_framebuffer( gl::FRAMEBUFFER , None );

    let sampler = Sampler::former()
    .min_filter( MinFilterMode::Nearest )
    .mag_filter( MagFilterMode::Nearest )
    .wrap_r( WrappingMode::Repeat )
    .wrap_s( WrappingMode::ClampToEdge )
    .wrap_t( WrappingMode::ClampToEdge )
    .end();

    let texture = Texture::former()
    .target( GL::TEXTURE_CUBE_MAP )
    .source( cube_texture? )
    .sampler( sampler )
    .end();

    let texture_info = TextureInfo
    {
      texture : Rc::new( RefCell::new( texture ) ),
      uv_position : 0,
    };

    Some( texture_info )
  }
}
