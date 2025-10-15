//! Text surface rendering example demonstrating curve-based text rendering with WebGL.
#![ doc( html_root_url = "https://docs.rs/text_rendering/latest/text_rendering/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders text as set of 3D glyph meshes" ) ]

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::manual_midpoint ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::std_instead_of_alloc ) ]

use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  GL,
  WebGl2RenderingContext,
  web_sys::HtmlCanvasElement,
  VectorDataType
};
use renderer::webgl::
{
  Mesh,
  Object3D,
  Node,
  Geometry,
  IndexInfo,
  geometry::AttributeInfo,
  loaders::gltf::GLTF,
  post_processing::
  {
    self, Pass, SwapFramebuffer
  },
  Camera,
  Material,
  Primitive,
  Renderer,
  Scene
};
use std::rc::Rc;
use std::any::type_name_of_val;

mod text;

fn make_buffer_attribute_info
(
  buffer : &web_sys::WebGlBuffer,
  offset : i32,
  stride : i32,
  slot : u32,
  normalized : bool,
  vector: gl::VectorDataType
) -> Result< AttributeInfo, gl::WebglError >
{
  let descriptor = match vector.scalar
  {
    gl::DataType::U8 => gl::BufferDescriptor::new::< [ u8; 1 ] >(),
    gl::DataType::I8 => gl::BufferDescriptor::new::< [ i8; 1 ] >(),
    gl::DataType::U16 => gl::BufferDescriptor::new::< [ u16; 1 ] >(),
    gl::DataType::I16 => gl::BufferDescriptor::new::< [ i16; 1 ] >(),
    gl::DataType::U32 => gl::BufferDescriptor::new::< [ u32; 1 ] >(),
    gl::DataType::F32 => gl::BufferDescriptor::new::< [ f32; 1 ] >(),
    _ => return Err( gl::WebglError::NotSupportedForType( type_name_of_val( &vector.scalar ) ) )
  };

  let descriptor = descriptor
  .offset( offset )
  .normalized( normalized )
  .stride( stride )
  .vector( vector );

  Ok(
    AttributeInfo
    {
      slot,
      buffer : buffer.clone(),
      descriptor,
      bounding_box : Default::default()
    }
  )
}

#[ derive( Debug, Clone ) ]
struct Transform
{
  translation : [ f32; 3 ],
  rotation : [ f32; 3 ],
  scale : [ f32; 3 ],
}

impl Default for Transform
{
  fn default() -> Self
  {
    Self
    {
      translation : [ 0.0; 3 ],
      rotation : [ 0.0; 3 ],
      scale : [ 1.0; 3 ]
    }
  }
}

impl Transform
{
  fn set_node_transform( &self, node : Rc< RefCell< Node > > )
  {
    let t = self.translation;
    let r = self.rotation;
    let s = self.scale;
    let mut node_mut = node.borrow_mut();
    node_mut.set_translation( [ t[ 0 ], t[ 1 ], t[ 2 ] ] );
    let q = gl::QuatF32::from_euler_xyz( r );
    node_mut.set_rotation( q );
    node_mut.set_scale( [ s[ 0 ], s[ 1 ], s[ 2 ] ] );
    node_mut.update_local_matrix();
  }
}

struct AttributesData
{
  positions : Vec< [ f32; 3 ] >,
  normals : Vec< [ f32; 3 ] >,
  indices : Vec< u32 >
}

#[ derive( Clone ) ]
struct PrimitiveData
{
  attributes : Rc< RefCell< AttributesData > >,
  material : Rc< RefCell< Material > >,
  transform : Transform
}

fn primitives_data_to_gltf
(
  gl : &GL,
  primitives_data : Vec< PrimitiveData >,
  materials : Vec< Rc< RefCell< Material > > >
) -> GLTF
{
  let mut scenes = vec![];
  let mut nodes = vec![];
  let mut gl_buffers = vec![];
  let mut meshes = vec![];

  scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

  let position_buffer = gl.create_buffer().unwrap();
  let normal_buffer = gl.create_buffer().unwrap();

  for buffer in
  [
    position_buffer.clone(),
    normal_buffer.clone()
  ]
  {
    gl_buffers.push( buffer );
  }

  let attribute_infos =
  [
    (
      "positions",
      make_buffer_attribute_info
      (
        &position_buffer,
        0,
        3,
        0,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    ),
    (
      "normals",
      make_buffer_attribute_info
      (
        &normal_buffer,
        0,
        3,
        1,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    )
  ];

  let index_buffer = gl.create_buffer().unwrap();
  gl_buffers.push( index_buffer.clone() );

  let mut index_info = IndexInfo
  {
    buffer : index_buffer.clone(),
    count : 0,
    offset : 0,
    data_type : GL::UNSIGNED_INT
  };

  let mut positions = vec![];
  let mut normals = vec![];
  let mut indices = vec![];

  for primitive_data in primitives_data
  {
    let last_positions_count = positions.len() as u32;
    positions.extend( primitive_data.attributes.borrow().positions.clone() );
    normals.extend( primitive_data.attributes.borrow().normals.clone() );
    let primitive_indices = primitive_data.attributes.borrow().indices.iter()
    .map( | i | i + last_positions_count )
    .collect::< Vec< _ > >();
    let offset = indices.len() as u32 * 4;
    indices.extend( primitive_indices );

    index_info.offset = offset;
    index_info.count = primitive_data.attributes.borrow().indices.len() as u32;

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      panic!( "Can't create new Geometry struct" );
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
    }

    geometry.add_index( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = primitive_data.attributes.borrow().positions.len() as u32;

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : primitive_data.material.clone(),
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
    primitive_data.transform.set_node_transform( node.clone() );

    nodes.push( node.clone() );
    meshes.push( mesh );
    scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : Rc::new( RefCell::new( vec![] ) ),
    textures : vec![],
    materials,
    meshes,
    animations : vec![]
  }
}

fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

fn init_camera( canvas : &HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera.bind_controls( &canvas );

  camera
}

async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init_context();

  let font_names = [
    "Roboto-Regular".to_string(),
    "Caveat".to_string(),
    "HennyPenny-Regular".to_string(),
    "Parisienne-Regular".to_string()
  ];

  let fonts_ufo_3d = text::ufo::load_fonts_3d( font_names.as_slice() ).await;
  let fonts_ttf_3d = text::ttf::load_fonts_3d( font_names.as_slice() ).await;

  let text = "CGTools".to_string();

  let material = Rc::new( RefCell::new( Material::default() ) );
  let materials = vec![ material.clone() ];

  let mut primitives_data = vec![];
  let mut transform_ufo = Transform::default();
  transform_ufo.translation[ 1 ] += (font_names.len() as f32 + 1.0 ) / 2.0 + 0.5;
  transform_ufo.translation[ 0 ] -= 1.8;
  let mut transform_ttf = Transform::default();
  transform_ttf.translation[ 1 ] += (font_names.len() as f32 + 1.0 ) / 2.0 + 0.5;
  transform_ttf.translation[ 0 ] += 1.8;
  for font_name in font_names
  {
    transform_ufo.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ufo::text_to_mesh( &text, fonts_ufo_3d.get( &font_name ).unwrap(), &transform_ufo );
    for p in text_mesh.iter_mut()
    {
      p.material = material.clone()
    }
    primitives_data.extend( text_mesh );

    transform_ttf.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ttf::text_to_mesh( &text, fonts_ttf_3d.get( &font_name ).unwrap(), &transform_ttf );
    for p in text_mesh.iter_mut()
    {
      p.material = material.clone()
    }
    primitives_data.extend( text_mesh );
  }

  let gltf = primitives_data_to_gltf( &gl, primitives_data, materials );
  let scenes = gltf.scenes.clone();

  scenes[ 0 ].borrow_mut().update_world_matrix();
  let camera = init_camera( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.get_main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _t = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render to srgb pass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
