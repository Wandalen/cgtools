use super::*;
use the_module::picking::{ pick_in_bounds, readback_to_pick_id, PICK_ID_NONE };

#[ test ]
fn pick_in_bounds_accepts_edges_and_rejects_outside()
{
  assert!( pick_in_bounds( 0, 0, 4, 3 ) );
  assert!( pick_in_bounds( 3, 2, 4, 3 ) );
  assert!( !pick_in_bounds( 4, 2, 4, 3 ), "x == width is out of bounds" );
  assert!( !pick_in_bounds( 3, 3, 4, 3 ), "y == height is out of bounds" );
  assert!( !pick_in_bounds( -1, 0, 4, 3 ) );
  assert!( !pick_in_bounds( 0, -1, 4, 3 ) );
  assert!( !pick_in_bounds( 0, 0, 0, 0 ), "a buffer without storage has no pixels" );
}

#[ test ]
fn readback_maps_sentinel_and_out_of_range_to_none()
{
  assert_eq!( readback_to_pick_id( u32::from( PICK_ID_NONE ) ), None );
  assert_eq!( readback_to_pick_id( 1 ), Some( 1 ) );
  assert_eq!( readback_to_pick_id( u32::from( u16::MAX ) ), Some( u16::MAX ) );
  assert_eq!( readback_to_pick_id( u32::from( u16::MAX ) + 1 ), None );
}

#[ cfg( target_arch = "wasm32" ) ]
mod gl_scene
{
  use super::*;
  use minwebgl as gl;
  use gl::GL;
  use std::{ cell::RefCell, rc::Rc };
  use the_module::
  {
    AttributeInfo, Geometry, Mesh, Node, Object3D, Primitive, Scene,
    material::PbrMaterial,
    picking::{ IdBuffer, ScenePicker },
  };

  const SIZE : u32 = 64;

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
  }

  /// An axis-aligned NDC rectangle at depth `z`, as a non-indexed two-triangle geometry.
  fn rect( gl : &GL, [ x0, y0, x1, y1 ] : [ f32; 4 ], z : f32 ) -> Rc< RefCell< Geometry > >
  {
    let positions : [ f32; 18 ] =
    [
      x0, y0, z,  x1, y0, z,  x1, y1, z,
      x0, y0, z,  x1, y1, z,  x0, y1, z,
    ];
    let buffer = gl.create_buffer().unwrap();
    gl::buffer::upload( gl, &buffer, positions.as_slice(), GL::STATIC_DRAW );

    let descriptor = gl::BufferDescriptor::from_vector( mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ) )
    .offset( 0 )
    .stride( 0 );

    let mut geometry = Geometry::new( gl ).unwrap();
    geometry.attribute_add
    (
      gl,
      "positions",
      AttributeInfo { slot : 0, buffer, descriptor, bounding_box : mingl::geometry::BoundingBox::default() }
    ).unwrap();
    geometry.vertex_count = 6;
    Rc::new( RefCell::new( geometry ) )
  }

  fn mesh_node( gl : &GL, name : &str, geometry : Rc< RefCell< Geometry > > ) -> Rc< RefCell< Node > >
  {
    let material : Box< dyn the_module::Material > = Box::new( PbrMaterial::new( gl ) );
    let primitive = Primitive { geometry, material : Rc::new( RefCell::new( material ) ) };
    let mut mesh = Mesh::new();
    mesh.primitives.push( Rc::new( RefCell::new( primitive ) ) );

    let mut node = Node::new();
    node.name_set( name );
    node.object = Object3D::Mesh( Rc::new( RefCell::new( mesh ) ) );
    Rc::new( RefCell::new( node ) )
  }

  fn id_for( node : &Node ) -> u16
  {
    match node.name_get().as_deref()
    {
      Some( "back" ) => 1,
      Some( "front" ) => 2,
      Some( "hidden" ) => 3,
      // "occluder" and anything else: drawn, never reported.
      _ => PICK_ID_NONE,
    }
  }

  /// `back` fills the viewport; `front` covers the right half nearer the camera;
  /// `occluder` covers the bottom-left quadrant nearest of all but has no id; `hidden`
  /// would cover everything nearest of all but is invisible. Identity view-projection,
  /// so NDC depth decides.
  fn scene( gl : &GL ) -> Scene
  {
    let mut scene = Scene::new();
    scene.add( mesh_node( gl, "back", rect( gl, [ -1.0, -1.0, 1.0, 1.0 ], 0.5 ) ) );
    scene.add( mesh_node( gl, "front", rect( gl, [ 0.0, -1.0, 1.0, 1.0 ], 0.0 ) ) );
    scene.add( mesh_node( gl, "occluder", rect( gl, [ -1.0, -1.0, 0.0, 0.0 ], -0.5 ) ) );
    let hidden = mesh_node( gl, "hidden", rect( gl, [ -1.0, -1.0, 1.0, 1.0 ], -0.9 ) );
    hidden.borrow_mut().visibility_set( false, true );
    scene.add( hidden );
    scene.world_matrix_update();
    scene
  }

  #[ wasm_bindgen_test::wasm_bindgen_test ]
  fn render_then_read_resolves_depth_occluders_and_visibility()
  {
    let gl = gl_init();
    let scene = scene( &gl );
    let picker = ScenePicker::new( &gl ).unwrap();
    let mut buffer = IdBuffer::new( &gl ).unwrap();
    buffer.size_set( SIZE, SIZE ).unwrap();

    picker.render( &buffer, &scene, gl::math::mat4x4::identity(), | n | id_for( n ) ).unwrap();

    assert_eq!( buffer.read( 10, 50 ), Some( 1 ), "top-left: only `back` is there" );
    assert_eq!( buffer.read( 50, 50 ), Some( 2 ), "top-right: nearer `front` wins over `back`" );
    assert_eq!( buffer.read( 50, 10 ), Some( 2 ), "bottom-right: `front`" );
    assert_eq!( buffer.read( 10, 10 ), None, "bottom-left: id-less occluder hides `back`" );
    assert_eq!( buffer.read( SIZE as i32, 0 ), None, "out of bounds" );
  }

  #[ wasm_bindgen_test::wasm_bindgen_test ]
  fn read_before_storage_and_render_without_storage()
  {
    let gl = gl_init();
    let scene = scene( &gl );
    let picker = ScenePicker::new( &gl ).unwrap();
    let buffer = IdBuffer::new( &gl ).unwrap();

    assert_eq!( buffer.size_get(), ( 0, 0 ) );
    assert_eq!( buffer.read( 0, 0 ), None );
    assert!( picker.render( &buffer, &scene, gl::math::mat4x4::identity(), | n | id_for( n ) ).is_err() );
  }

  #[ wasm_bindgen_test::wasm_bindgen_test ]
  fn render_restores_caller_gl_state()
  {
    let gl = gl_init();
    let scene = scene( &gl );
    let picker = ScenePicker::new( &gl ).unwrap();
    let mut buffer = IdBuffer::new( &gl ).unwrap();
    buffer.size_set( SIZE, SIZE ).unwrap();

    let caller_framebuffer = gl.create_framebuffer().unwrap();
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &caller_framebuffer ) );
    gl.viewport( 3, 4, 5, 6 );
    gl.enable( GL::BLEND );
    gl.enable( GL::CULL_FACE );
    gl.disable( GL::DEPTH_TEST );
    gl.depth_mask( false );

    picker.render( &buffer, &scene, gl::math::mat4x4::identity(), | n | id_for( n ) ).unwrap();

    use gl::JsCast;
    let framebuffer = gl.get_parameter( GL::FRAMEBUFFER_BINDING ).unwrap().dyn_into::< gl::web_sys::WebGlFramebuffer >().ok();
    assert_eq!( framebuffer.as_ref(), Some( &caller_framebuffer ), "framebuffer binding restored" );
    let viewport : gl::js_sys::Int32Array = gl.get_parameter( GL::VIEWPORT ).unwrap().dyn_into().unwrap();
    assert_eq!( viewport.to_vec(), vec![ 3, 4, 5, 6 ], "viewport restored" );
    assert!( gl.is_enabled( GL::BLEND ) );
    assert!( gl.is_enabled( GL::CULL_FACE ) );
    assert!( !gl.is_enabled( GL::DEPTH_TEST ) );
    assert_eq!( gl.get_parameter( GL::DEPTH_WRITEMASK ).unwrap().as_bool(), Some( false ) );
  }

  #[ wasm_bindgen_test::wasm_bindgen_test ]
  fn resize_reallocates_and_zero_releases()
  {
    let gl = gl_init();
    let scene = scene( &gl );
    let picker = ScenePicker::new( &gl ).unwrap();
    let mut buffer = IdBuffer::new( &gl ).unwrap();

    buffer.size_set( 8, 8 ).unwrap();
    buffer.size_set( SIZE, SIZE ).unwrap();
    assert_eq!( buffer.size_get(), ( SIZE, SIZE ) );
    picker.render( &buffer, &scene, gl::math::mat4x4::identity(), | n | id_for( n ) ).unwrap();
    assert_eq!( buffer.read( 50, 50 ), Some( 2 ) );

    buffer.size_set( 0, 0 ).unwrap();
    assert_eq!( buffer.size_get(), ( 0, 0 ) );
    assert_eq!( buffer.read( 0, 0 ), None );
  }
}
