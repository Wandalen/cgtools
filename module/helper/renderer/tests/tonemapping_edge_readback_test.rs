//! Pixel readback of `ToneMappingPass` ( ACES ) over real `Renderer` frames, at the edge of a
//! quad seen against the clear-colour background.
//!
//! Given the renderer's transparent-coverage texture, the pass reconstructs partially covered
//! pixels from their background neighbours. The tests check the two kinds of partial coverage it
//! sees: an MSAA silhouette of an opaque quad, whose edge must not turn into a light fringe, and a
//! transparent ( WBOIT ) quad, whose outline must not be tone mapped differently from its
//! interior; and that without the texture the pass keeps the plain blend. The quad is unlit apart
//! from the PBR shader's ambient term ( 0.1 of its white base colour ), which the exposure scales
//! to an HDR value, so the comparisons only depend on how the pass treats each pixel, not on the
//! shading.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::
  {
    material::PbrMaterial,
    post_processing::{ Pass, ToneMappingAces, ToneMappingPass },
    AlphaMode, AttributeInfo, Camera, Geometry, Material, Mesh, Node, Object3D, Primitive, Renderer, Scene,
  };

  /// Side of the square render target, in pixels.
  const SIZE : i32 = 64;
  /// The column the quad's left edge falls on: the quad starts at world x = 0, the centre of
  /// the view, which is the boundary between columns `EDGE - 1` and `EDGE`.
  const EDGE : usize = SIZE as usize / 2;
  /// A column well inside the quad.
  const INTERIOR : usize = SIZE as usize * 3 / 4;
  /// A column well outside the quad.
  const BACKGROUND : usize = SIZE as usize / 4;
  /// Half the view's width at the quad's depth: the camera is 3 units away with a 70° fov.
  const HALF_WIDTH : f32 = 2.100_6;

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap();
    gl.get_extension( "EXT_color_buffer_float" )
    .expect( "get_extension call should not throw" )
    .expect( "EXT_color_buffer_float must be available in the test environment" );
    gl
  }

  fn camera_make() -> Camera
  {
    Camera::new
    (
      gl::F32x3::from_array( [ 0.0, 0.0, 3.0 ] ),
      gl::F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
      gl::F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
      1.0,
      70.0f32.to_radians(),
      0.1,
      100.0,
    ).expect( "valid camera parameters" )
  }

  fn attribute( gl : &GL, slot : u32, data : &[ f32 ] ) -> AttributeInfo
  {
    let buffer = gl.create_buffer().unwrap();
    gl::buffer::upload( gl, &buffer, data, GL::STATIC_DRAW );
    let descriptor = gl::BufferDescriptor::from_vector( mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ) )
    .offset( 0 )
    .stride( 0 );
    AttributeInfo { slot, buffer, descriptor, bounding_box : mingl::geometry::BoundingBox::default() }
  }

  /// A quad facing the camera from world x = `left` to beyond the right edge of the view, over
  /// the full height, in a grey PBR material with the given alpha mode and alpha.
  fn quad_node( gl : &GL, left : f32, alpha_mode : AlphaMode, alpha : f32 ) -> Rc< RefCell< Node > >
  {
    let ( x0, x1, y0, y1 ) = ( left, 10.0, -10.0, 10.0 );
    let positions = [ x0, y0, 0.0, x1, y0, 0.0, x1, y1, 0.0, x0, y0, 0.0, x1, y1, 0.0, x0, y1, 0.0 ];
    let normals = [ 0.0, 0.0, 1.0 ].repeat( 6 );
    let mut geometry = Geometry::new( gl ).unwrap();
    geometry.attribute_add( gl, "positions", attribute( gl, 0, &positions ) ).unwrap();
    geometry.attribute_add( gl, "normals", attribute( gl, 1, &normals ) ).unwrap();
    geometry.vertex_count = 6;

    let mut material = PbrMaterial::new( gl );
    material.base_color_factor = gl::F32x4::from( [ 1.0, 1.0, 1.0, alpha ] );
    material.metallic_factor = 0.0;
    material.alpha_mode_set( alpha_mode );
    let material : Box< dyn Material > = Box::new( material );

    let mut mesh = Mesh::new();
    mesh.primitive_add( Rc::new( RefCell::new( Primitive { geometry : Rc::new( RefCell::new( geometry ) ), material : Rc::new( RefCell::new( material ) ) } ) ) );
    let mut node = Node::default();
    node.object = Object3D::Mesh( Rc::new( RefCell::new( mesh ) ) );
    Rc::new( RefCell::new( node ) )
  }

  /// A renderer drawing over a grey `background`, 4x MSAA; quads come out `0.1 * 2^exposure`.
  fn renderer_make( gl : &GL, background : f32, exposure : f32 ) -> Renderer
  {
    let mut renderer = Renderer::new( gl, SIZE as u32, SIZE as u32, 4 ).expect( "Renderer::new succeeds" );
    renderer.clear_color_set( gl::F32x3::from( [ background; 3 ] ) );
    renderer.exposure_set( exposure );
    renderer
  }

  fn draw( gl : &GL, renderer : &mut Renderer, node : Rc< RefCell< Node > > )
  {
    let mut scene = Scene::new();
    scene.add( node );
    renderer.render( gl, &mut scene, &camera_make() ).expect( "render succeeds" );
  }

  /// World x of an opaque quad edge half a pixel right of the centre, so the edge column is
  /// partially covered.
  fn half_pixel_left() -> f32
  {
    0.5 / ( SIZE as f32 / 2.0 ) * HALF_WIDTH
  }

  /// Tone maps the renderer's frame, with `coverage` as the pass's transparent-coverage
  /// texture, and returns the red channel of the middle row.
  fn tone_mapped_row( gl : &GL, renderer : &Renderer, coverage : Option< gl::web_sys::WebGlTexture > ) -> Vec< u8 >
  {
    let mut pass = ToneMappingPass::< ToneMappingAces >::new( gl ).expect( "tone mapping program compiles" );
    pass.transparent_coverage_texture_set( coverage );

    let output = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, output.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA8, SIZE, SIZE );
    let framebuffer = gl.create_framebuffer();
    gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
    gl.viewport( 0, 0, SIZE, SIZE );
    pass.render( gl, renderer.main_texture(), output.clone() ).expect( "tone mapping renders" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "tone mapping raises no GL error" );

    // `render` detaches its output; attach it again to read it.
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, output.as_ref(), 0 );
    let mut row = vec![ 0_u8; SIZE as usize * 4 ];
    gl.read_pixels_with_opt_u8_array( 0, SIZE / 2, SIZE, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut row ) )
    .expect( "read_pixels succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( framebuffer.as_ref() );
    gl.delete_texture( output.as_ref() );
    row.chunks( 4 ).map( | pixel | pixel[ 0 ] ).collect()
  }

  /// An opaque silhouette pixel is half quad, half background. With the coverage texture
  /// wired, tone mapping it must give a value between the two. Unwired, the pass keeps the
  /// plain blend, which lets half of the quad's raw HDR value ( 3.2 here ) through and lights
  /// the edge up beyond the quad's own tone-mapped value: the fix is opt-in.
  #[ wasm_bindgen_test ]
  fn opaque_silhouette_has_no_light_fringe_when_wired()
  {
    let gl = gl_init();
    let mut renderer = renderer_make( &gl, 0.5, 5.0 );
    draw( &gl, &mut renderer, quad_node( &gl, half_pixel_left(), AlphaMode::Opaque, 1.0 ) );

    let row = tone_mapped_row( &gl, &renderer, renderer.transparent_coverage_texture() );
    let ( background, edge, interior ) = ( row[ BACKGROUND ], row[ EDGE ], row[ INTERIOR ] );
    assert!( background < interior, "precondition: the quad is brighter than the background: {row:?}" );
    assert!( edge > background && edge < interior, "the silhouette pixel must lie between background and quad: {row:?}" );

    let unwired = tone_mapped_row( &gl, &renderer, None );
    assert!( unwired[ EDGE ] > unwired[ INTERIOR ], "unwired, the pass keeps the plain blend and its fringe: {unwired:?}" );
  }

  /// The coverage texture outlives frames: after a transparent frame, a frame without
  /// transparent primitives must not inherit its coverage, or the opaque silhouette below
  /// the old glass would fall back to the plain blend and its fringe.
  #[ wasm_bindgen_test ]
  fn coverage_is_reset_by_a_frame_without_transparency()
  {
    let gl = gl_init();
    let mut renderer = renderer_make( &gl, 0.5, 5.0 );
    draw( &gl, &mut renderer, quad_node( &gl, -10.0, AlphaMode::Blend, 0.5 ) );
    draw( &gl, &mut renderer, quad_node( &gl, half_pixel_left(), AlphaMode::Opaque, 1.0 ) );

    let row = tone_mapped_row( &gl, &renderer, renderer.transparent_coverage_texture() );
    let ( background, edge, interior ) = ( row[ BACKGROUND ], row[ EDGE ], row[ INTERIOR ] );
    assert!( edge > background && edge < interior, "the silhouette pixel must lie between background and quad: {row:?}" );
  }

  /// A transparent quad over the background, its edge on a pixel boundary: the edge column is
  /// as covered as the interior, so it must be tone mapped to the same value, wired or not.
  /// Reconstructing the edge column, which has background neighbours, as an MSAA silhouette
  /// would draw a darker outline. The glass ( 1.6, half covering a 0.2 background ) is kept
  /// dim enough that neither value saturates.
  #[ wasm_bindgen_test ]
  fn transparent_edge_matches_its_interior()
  {
    let gl = gl_init();
    let mut renderer = renderer_make( &gl, 0.2, 4.0 );
    draw( &gl, &mut renderer, quad_node( &gl, 0.0, AlphaMode::Blend, 0.5 ) );

    let row = tone_mapped_row( &gl, &renderer, renderer.transparent_coverage_texture() );
    let ( background, edge, interior ) = ( row[ BACKGROUND ], row[ EDGE ], row[ INTERIOR ] );
    assert_eq!( row[ EDGE - 1 ], background, "precondition: the column left of the edge is background: {row:?}" );
    assert!( interior > background && interior < 250, "precondition: the glass is brighter than the background, unsaturated: {row:?}" );
    assert!( edge.abs_diff( interior ) <= 1, "wired, the glass edge must match its interior: {row:?}" );

    let unwired = tone_mapped_row( &gl, &renderer, None );
    assert_eq!( unwired, row, "unwired, the glass takes the same plain blend: {unwired:?}" );
  }
}
