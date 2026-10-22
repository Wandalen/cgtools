//! Pixel readback of the two pieces an AR overlay is built from: `ToSrgbPass` with alpha
//! forwarding, whose output must be valid premultiplied alpha for the canvas, and a depth-only
//! `Material::is_occluder` material drawn by the real `Renderer::render`, which must leave the
//! background showing where it stands in front of opaque geometry. `ColorGradingPass`, which
//! sits between them in an AR chain, must pass the coverage alpha through, and `ScenePicker`
//! must report the occluder, not what it hides. The whole chain, from `Renderer::render`
//! through ACES and colour grading to `ToSrgbPass`, is checked on the canvas too.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::{ GL, JsCast };
  use renderer::webgl::
  {
    post_processing::{ ColorGradingPass, Pass, ToSrgbPass, ToneMappingAces, ToneMappingPass },
    picking::{ IdBuffer, PickId, ScenePicker },
    material::PbrMaterial,
    AttributeInfo,
    Camera,
    Geometry,
    Material,
    MaterialUploadContext,
    Mesh,
    Node,
    Object3D,
    Primitive,
    Renderer,
    Sampler,
    Scene,
    ShaderProgram,
    Texture,
  };

  const SIZE : u32 = 64;

  /// `Renderer::new` allocates `RGBA16F` targets, and the sRGB tests sample one.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap();
    gl.get_extension( "EXT_color_buffer_float" )
    .expect( "get_extension does not throw" )
    .expect( "EXT_color_buffer_float is available" );
    gl
  }

  // --- ToSrgbPass ---

  /// Runs `pass` to the canvas on a 1x1 `RGBA16F` input holding `rgba` and returns the
  /// canvas's center pixel.
  fn to_srgb_pixel( gl : &GL, pass : &ToSrgbPass, rgba : [ f32; 4 ] ) -> [ u8; 4 ]
  {
    let source = gl.create_texture().expect( "create_texture succeeds" );
    gl.bind_texture( gl::TEXTURE_2D, Some( &source ) );
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
    (
      gl::TEXTURE_2D, 0, gl::RGBA16F as i32, 1, 1, 0, gl::RGBA, gl::FLOAT,
      Some( &gl::js_sys::Float32Array::from( rgba.as_slice() ) )
    ).expect( "tex_image_2d succeeds" );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32 );

    let ( width, height ) = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );
    gl.viewport( 0, 0, width, height );
    pass.render( gl, Some( source.clone() ), None ).expect( "ToSrgbPass renders" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the pass raises no GL error" );

    let mut pixel = [ 0_u8; 4 ];
    gl.read_pixels_with_opt_u8_array( width / 2, height / 2, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) )
    .expect( "read_pixels succeeds" );
    gl.delete_texture( Some( &source ) );
    pixel
  }

  fn assert_premultiplied( pixel : [ u8; 4 ], what : &str )
  {
    assert!
    (
      pixel[ 0 ] <= pixel[ 3 ] && pixel[ 1 ] <= pixel[ 3 ] && pixel[ 2 ] <= pixel[ 3 ],
      "{what}: rgb must not exceed alpha on a premultiplied canvas, got {pixel:?}"
    );
  }

  fn assert_near( actual : u8, expected : u8, what : &str )
  {
    assert!( actual.abs_diff( expected ) <= 2, "{what}: expected about {expected}, got {actual}" );
  }

  /// A partially covered edge pixel with a 0 clear holds `a * c`. The output must be
  /// `a * srgb( c )` with alpha `a`, so rgb never exceeds alpha. Encoding `a * c` directly
  /// gives `srgb( 0.25 ) = 0.54` against alpha 0.25 for white at a quarter coverage.
  #[ wasm_bindgen_test ]
  fn alpha_forward_output_is_premultiplied()
  {
    let gl = gl_init();
    let mut pass = ToSrgbPass::new( &gl, true ).expect( "ToSrgbPass compiles" );
    pass.alpha_forward_set( &gl, true ).expect( "the FORWARD_ALPHA variant compiles" );
    assert!( pass.alpha_forward() );

    // White at a quarter coverage: srgb( 1 ) * 0.25.
    let white = to_srgb_pixel( &gl, &pass, [ 0.25, 0.25, 0.25, 0.25 ] );
    assert_premultiplied( white, "white, a = 0.25" );
    assert_near( white[ 0 ], 64, "white, a = 0.25, red" );
    assert_near( white[ 3 ], 64, "white, a = 0.25, alpha" );

    // Linear 0.2 at half coverage: srgb( 0.2 ) = 0.484, times 0.5.
    let grey = to_srgb_pixel( &gl, &pass, [ 0.1, 0.1, 0.1, 0.5 ] );
    assert_premultiplied( grey, "grey, a = 0.5" );
    assert_near( grey[ 0 ], 62, "grey, a = 0.5, red" );
    assert_near( grey[ 3 ], 128, "grey, a = 0.5, alpha" );

    // A fully covered pixel is opaque, its colour the plain sRGB encoding: srgb( 0.25 ) = 0.537.
    let covered = to_srgb_pixel( &gl, &pass, [ 0.25, 0.25, 0.25, 1.0 ] );
    assert_near( covered[ 0 ], 137, "a = 1, red" );
    assert_eq!( covered[ 3 ], 255, "a = 1, alpha: {covered:?}" );

    // An uncovered pixel is fully transparent, whatever colour a pass left in it.
    assert_eq!( to_srgb_pixel( &gl, &pass, [ 0.3, 0.3, 0.3, 0.0 ] ), [ 0, 0, 0, 0 ] );
  }

  /// Off, the default: alpha is 1 and the colour is the plain sRGB encoding of the input.
  #[ wasm_bindgen_test ]
  fn default_output_is_opaque()
  {
    let gl = gl_init();
    let pass = ToSrgbPass::new( &gl, true ).expect( "ToSrgbPass compiles" );
    assert!( !pass.alpha_forward() );

    let pixel = to_srgb_pixel( &gl, &pass, [ 0.25, 0.25, 0.25, 0.25 ] );
    assert_eq!( pixel[ 3 ], 255, "opaque output, got {pixel:?}" );
    assert_near( pixel[ 0 ], 137, "srgb( 0.25 )" );
  }

  /// Toggling alpha forwarding compiles the other variant and deletes the program it
  /// replaces, so a page that switches AR on and off does not pile up programs. The new
  /// program must be the one drawing: the output follows the toggle each time.
  #[ cfg( feature = "test_internals" ) ]
  #[ wasm_bindgen_test ]
  fn alpha_forward_set_deletes_the_replaced_program()
  {
    let gl = gl_init();
    let mut pass = ToSrgbPass::new( &gl, true ).expect( "ToSrgbPass compiles" );
    let quarter = [ 0.25, 0.25, 0.25, 0.25 ];

    for alpha_forward in [ true, false, true ]
    {
      let replaced = pass.program_for_test();
      pass.alpha_forward_set( &gl, alpha_forward ).expect( "the variant compiles" );
      // The previous draw left the replaced program current, and GL defers deleting a
      // current program until it is unbound, as the next pass or frame does.
      gl.use_program( None );
      assert!( !gl.is_program( Some( &replaced ) ), "alpha_forward = {alpha_forward}: the replaced program is deleted" );
      assert!( gl.is_program( Some( &pass.program_for_test() ) ), "alpha_forward = {alpha_forward}: the new program is live" );

      let alpha = to_srgb_pixel( &gl, &pass, quarter )[ 3 ];
      let expected = if alpha_forward { 64 } else { 255 };
      assert_near( alpha, expected, &format!( "alpha_forward = {alpha_forward}: the new variant draws" ) );
    }

    let kept = pass.program_for_test();
    pass.alpha_forward_set( &gl, true ).expect( "setting the same value succeeds" );
    assert!( gl.is_program( Some( &kept ) ), "setting the current value keeps the program" );
  }

  // --- ColorGradingPass ---

  /// Runs `pass` on a 1x1 `RGBA16F` input holding `rgba`, into a 1x1 `RGBA8` texture, and
  /// returns that texel.
  fn color_grading_pixel( gl : &GL, pass : &ColorGradingPass, rgba : [ f32; 4 ] ) -> [ u8; 4 ]
  {
    let texture = | format : u32, kind : u32, data : Option< &gl::js_sys::Object > |
    {
      let texture = gl.create_texture().expect( "create_texture succeeds" );
      gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
      (
        gl::TEXTURE_2D, 0, format as i32, 1, 1, 0, gl::RGBA, kind, data
      ).expect( "tex_image_2d succeeds" );
      gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32 );
      gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32 );
      texture
    };
    let input = texture( gl::RGBA16F, gl::FLOAT, Some( &gl::js_sys::Float32Array::from( rgba.as_slice() ) ) );
    let output = texture( gl::RGBA8, gl::UNSIGNED_BYTE, None );

    let framebuffer = gl.create_framebuffer().expect( "create_framebuffer succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &framebuffer ) );
    gl.viewport( 0, 0, 1, 1 );
    pass.render( gl, Some( input.clone() ), Some( output.clone() ) ).expect( "ColorGradingPass renders" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the pass raises no GL error" );

    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, Some( &output ), 0 );
    let mut pixel = [ 0_u8; 4 ];
    gl.read_pixels_with_opt_u8_array( 0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) )
    .expect( "read_pixels succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( Some( &framebuffer ) );
    gl.delete_texture( Some( &input ) );
    gl.delete_texture( Some( &output ) );
    pixel
  }

  /// The pass sits between tone mapping and `ToSrgbPass` in an AR chain, so it must pass the
  /// coverage alpha through: an alpha of 1 here would make the whole canvas opaque again.
  #[ wasm_bindgen_test ]
  fn color_grading_forwards_alpha()
  {
    let gl = gl_init();
    let pass = ColorGradingPass::new( &gl ).expect( "ColorGradingPass compiles" );

    for alpha in [ 0.0_f32, 0.25, 0.5, 1.0 ]
    {
      let pixel = color_grading_pixel( &gl, &pass, [ 0.2 * alpha, 0.2 * alpha, 0.2 * alpha, alpha ] );
      assert_near( pixel[ 3 ], ( alpha * 255.0 ).round() as u8, &format!( "alpha {alpha}" ) );
    }
  }

  // --- Occluders in Renderer::render ---

  /// A depth-only occluder: a `PbrMaterial` that writes no colour.
  ///
  /// It also reports emission. The opaque queue's order without the occluder key is
  /// ( has emission, program ), so an emissive occluder would draw after the plain quad
  /// behind it, too late to hide it; drawing it first must come from `is_occluder` alone.
  #[ derive( Debug, Clone ) ]
  struct Occluder( PbrMaterial );

  impl Material for Occluder
  {
    fn id( &self ) -> uuid::Uuid { self.0.id() }
    fn needs_update( &self ) -> bool { self.0.needs_update() }
    fn needs_update_set( &self, value : bool ) { self.0.needs_update_set( value ); }
    fn ibl_base_texture_unit( &self ) -> Option< u32 > { self.0.ibl_base_texture_unit() }
    fn shader_program_make( &self, gl : &GL, program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >
    {
      self.0.shader_program_make( gl, program )
    }
    fn type_name( &self ) -> &'static str { "Occluder" }
    fn vertex_shader( &self ) -> String { self.0.vertex_shader() }
    fn fragment_shader( &self ) -> String { self.0.fragment_shader() }
    fn defines_str( &self ) -> &str { self.0.defines_str() }
    fn vertex_defines_str( &self ) -> &str { self.0.vertex_defines_str() }
    fn fragment_defines_str( &self ) -> &str { self.0.fragment_defines_str() }
    fn configure( &self, gl : &GL, ctx : &MaterialUploadContext< '_ > ) { self.0.configure( gl, ctx ); }
    fn upload_on_state_change( &self, gl : &GL, ctx : &MaterialUploadContext< '_ > ) -> Result< (), gl::WebglError >
    {
      self.0.upload_on_state_change( gl, ctx )
    }
    fn upload( &self, gl : &GL, ctx : &MaterialUploadContext< '_ > ) -> Result< (), gl::WebglError >
    {
      self.0.upload( gl, ctx )
    }
    fn bind( &self, gl : &GL ) { self.0.bind( gl ); }
    fn dyn_clone( &self ) -> Box< dyn Material > { Box::new( self.clone() ) }
    fn as_any( &self ) -> &dyn std::any::Any { self }
    fn as_any_mut( &mut self ) -> &mut dyn std::any::Any { self }
    fn color_write_mask( &self ) -> ( bool, bool, bool, bool ) { ( false, false, false, false ) }
    fn has_emission( &self ) -> bool { true }
    fn is_occluder( &self ) -> bool { true }
    fn needs_recompile( &self ) -> bool { self.0.needs_recompile() }
    fn recompile_flag_clear( &self ) { self.0.recompile_flag_clear(); }
  }

  /// A float buffer with `data`, bound to `slot`.
  fn attribute( gl : &GL, slot : u32, data : &[ f32 ] ) -> AttributeInfo
  {
    let buffer = gl.create_buffer().expect( "create_buffer succeeds" );
    gl::buffer::upload( gl, &buffer, data, GL::STATIC_DRAW );
    let descriptor = gl::BufferDescriptor::from_vector( mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ) )
    .offset( 0 )
    .stride( 0 );
    AttributeInfo { slot, buffer, descriptor, bounding_box : mingl::geometry::BoundingBox::default() }
  }

  /// A node holding an axis-aligned rectangle at depth `z`, facing +Z, drawn with `material`.
  fn rect_node( gl : &GL, [ x0, y0, x1, y1 ] : [ f32; 4 ], z : f32, material : Box< dyn Material > ) -> Rc< RefCell< Node > >
  {
    let positions =
    [
      x0, y0, z,  x1, y0, z,  x1, y1, z,
      x0, y0, z,  x1, y1, z,  x0, y1, z,
    ];
    let normals = [ 0.0, 0.0, 1.0 ].repeat( 6 );
    let mut geometry = Geometry::new( gl ).expect( "Geometry::new succeeds" );
    geometry.attribute_add( gl, "positions", attribute( gl, 0, &positions ) ).expect( "positions attach" );
    geometry.attribute_add( gl, "normals", attribute( gl, 1, &normals ) ).expect( "normals attach" );
    geometry.vertex_count = 6;

    let primitive = Primitive { geometry : Rc::new( RefCell::new( geometry ) ), material : Rc::new( RefCell::new( material ) ) };
    let mut mesh = Mesh::new();
    mesh.primitives.push( Rc::new( RefCell::new( primitive ) ) );
    let mut node = Node::new();
    node.object = Object3D::Mesh( Rc::new( RefCell::new( mesh ) ) );
    Rc::new( RefCell::new( node ) )
  }

  /// Looking down -Z from z = 2 with a 90° field of view: the z = 0 plane spans [ -2, 2 ].
  fn camera() -> Camera
  {
    Camera::new
    (
      gl::F32x3::from_array( [ 0.0, 0.0, 2.0 ] ),
      gl::F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
      gl::F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
      1.0,
      90.0_f32.to_radians(),
      0.1,
      100.0,
    ).expect( "valid camera" )
  }

  /// The linear RGBA the renderer resolved at pixel ( `x`, `y` ) of its main target.
  fn main_pixel( gl : &GL, renderer : &Renderer, x : i32, y : i32 ) -> [ f32; 4 ]
  {
    let framebuffer = gl.create_framebuffer().expect( "create_framebuffer succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &framebuffer ) );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, renderer.main_texture().as_ref(), 0 );
    let pixel = gl::js_sys::Float32Array::new_with_length( 4 );
    gl.read_pixels_with_opt_array_buffer_view( x, y, 1, 1, gl::RGBA, gl::FLOAT, Some( &pixel ) )
    .expect( "read_pixels succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( Some( &framebuffer ) );
    let mut out = [ 0.0; 4 ];
    pixel.copy_to( &mut out );
    out
  }

  /// An occluder in front of an opaque quad leaves the background there: the clear colour at
  /// alpha 0, which a forwarded-alpha canvas shows as transparent. Beside it the quad still draws.
  #[ wasm_bindgen_test ]
  fn occluder_hides_the_opaque_geometry_behind_it()
  {
    let gl = gl_init();
    let mut renderer = Renderer::new( &gl, SIZE, SIZE, 4 ).expect( "Renderer::new succeeds" );
    renderer.clear_color_set( gl::F32x3::from_array( [ 0.2, 0.4, 0.6 ] ) );

    let mut scene = Scene::new();
    // Added first, so only the occluder sort can put the occluder ahead of it.
    scene.add( rect_node( &gl, [ -5.0, -5.0, 5.0, 5.0 ], 0.0, Box::new( PbrMaterial::new( &gl ) ) ) );
    scene.add( rect_node( &gl, [ -5.0, -5.0, 0.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) ) );
    renderer.render( &gl, &mut scene, &camera() ).expect( "render succeeds" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the frame raises no GL error" );

    let mid = ( SIZE / 2 ) as i32;
    let quad = main_pixel( &gl, &renderer, mid + 16, mid );
    assert!( ( quad[ 3 ] - 1.0 ).abs() < 1e-3, "the quad beside the occluder draws, alpha 1: {quad:?}" );

    let occluded = main_pixel( &gl, &renderer, mid - 16, mid );
    assert!( occluded[ 3 ].abs() < 1e-3, "no colour behind the occluder, alpha 0: {occluded:?}" );
    for ( channel, clear ) in occluded.iter().zip( [ 0.2, 0.4, 0.6 ] )
    {
      assert!( ( channel - clear ).abs() < 1e-2, "the clear colour shows through: {occluded:?}" );
    }
  }

  /// A 1x1 opaque red `RGBA8` texture, for a skybox that draws red in every direction.
  fn red_skybox( gl : &GL ) -> Texture
  {
    let source = gl.create_texture().expect( "create_texture succeeds" );
    gl.bind_texture( gl::TEXTURE_2D, Some( &source ) );
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      gl::TEXTURE_2D, 0, gl::RGBA8 as i32, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, Some( &[ 255, 0, 0, 255 ] )
    ).expect( "tex_image_2d succeeds" );
    // The skybox is drawn with the GL texture's own parameters, and the default minifying
    // filter needs mipmaps this texture does not have.
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32 );
    Texture::owning( gl, gl::TEXTURE_2D, source, Sampler::default() )
  }

  /// A frame whose opaque queue holds only an occluder must not leave its colour mask
  /// behind, or the skybox, the post-processing passes and the next frame's clear write
  /// nothing.
  ///
  /// ## Root Cause
  /// The colour mask is per-material state, and the opaque pass left the last material's in
  /// place. An occluder's mask is all off.
  ///
  /// ## Why Not Caught
  /// No test drew an occluder last, and the depth tests read depth, not colour.
  ///
  /// ## Fix Applied
  /// `Renderer::render` turns colour writes back on after the opaque pass, before the
  /// skybox, and again after the transparent pass.
  ///
  /// ## Prevention
  /// The skybox draws right after the opaque pass, so only the first restoration can make
  /// it show: beside the occluder the background must be the skybox's red, not the clear
  /// colour. The mask must also be on once the frame is done.
  ///
  /// ## Pitfall
  /// Any per-material GL state the opaque pass applies has to be reset before a pass that
  /// does not set it itself.
  // test_kind: bug_reproducer(BUG-539)
  #[ wasm_bindgen_test ]
  fn an_occluder_drawn_last_does_not_leave_the_colour_mask_off()
  {
    let gl = gl_init();
    let mut renderer = Renderer::new( &gl, SIZE, SIZE, 4 ).expect( "Renderer::new succeeds" );
    renderer.clear_color_set( gl::F32x3::from_array( [ 0.2, 0.4, 0.6 ] ) );
    renderer.skybox_set( Some( red_skybox( &gl ) ) );
    let mut scene = Scene::new();
    // The left half only, so the skybox shows on the right.
    scene.add( rect_node( &gl, [ -5.0, -5.0, 0.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) ) );
    renderer.render( &gl, &mut scene, &camera() ).expect( "render succeeds" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the frame raises no GL error" );

    let mid = ( SIZE / 2 ) as i32;
    let sky = main_pixel( &gl, &renderer, mid + 16, mid );
    assert!( sky[ 0 ] > 0.9 && sky[ 1 ] < 0.05 && sky[ 2 ] < 0.05 && ( sky[ 3 ] - 1.0 ).abs() < 1e-3, "the skybox draws after the occluder: {sky:?}" );
    let occluded = main_pixel( &gl, &renderer, mid - 16, mid );
    assert!( occluded[ 3 ].abs() < 1e-3, "the occluder hides the skybox too, alpha 0: {occluded:?}" );

    let mask = gl.get_parameter( GL::COLOR_WRITEMASK ).expect( "get_parameter succeeds" )
    .dyn_into::< gl::js_sys::Array >().expect( "COLOR_WRITEMASK is an array" );
    let mask : Vec< bool > = mask.iter().map( | v | v.as_bool().expect( "a bool" ) ).collect();
    assert_eq!( mask, [ true; 4 ], "colour writes are on after the frame" );
  }

  /// Several occluders in one scene each hide the geometry behind them, wherever the scene
  /// lists them relative to it: before it, after it, or one on each side. Between them the
  /// geometry still draws.
  #[ wasm_bindgen_test ]
  fn every_occluder_hides_the_geometry_behind_it_in_any_order()
  {
    let gl = gl_init();
    let mut renderer = Renderer::new( &gl, SIZE, SIZE, 4 ).expect( "Renderer::new succeeds" );
    let mid = ( SIZE / 2 ) as i32;

    // 0: quad, occluders left and right; 1: occluders, then quad; 2: one on each side.
    for order in 0..3
    {
      let quad = rect_node( &gl, [ -5.0, -5.0, 5.0, 5.0 ], 0.0, Box::new( PbrMaterial::new( &gl ) ) );
      // At z = 1 the view spans [ -1, 1 ]: the occluders cover all of it but a band at x = 0.
      let left = rect_node( &gl, [ -5.0, -5.0, -0.25, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) );
      let right = rect_node( &gl, [ 0.25, -5.0, 5.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) );
      let nodes = match order
      {
        0 => [ quad, left, right ],
        1 => [ left, right, quad ],
        _ => [ left, quad, right ],
      };
      let mut scene = Scene::new();
      for node in nodes { scene.add( node ); }
      renderer.render( &gl, &mut scene, &camera() ).expect( "render succeeds" );

      for x in [ mid - 20, mid + 20 ]
      {
        let occluded = main_pixel( &gl, &renderer, x, mid );
        assert!( occluded[ 3 ].abs() < 1e-3, "order {order}, x = {x}: no colour behind an occluder: {occluded:?}" );
      }
      let between = main_pixel( &gl, &renderer, mid, mid );
      assert!( ( between[ 3 ] - 1.0 ).abs() < 1e-3, "order {order}: the quad draws between the occluders: {between:?}" );
    }
  }

  /// A `SIZE` x `SIZE` `RGBA16F` texture for one post-processing stage to write and the next
  /// to read.
  fn stage_texture( gl : &GL ) -> gl::web_sys::WebGlTexture
  {
    let texture = gl.create_texture().expect( "create_texture succeeds" );
    gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, SIZE as i32, SIZE as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32 );
    texture
  }

  /// The whole AR chain on a real frame: `Renderer::render` with a 0 clear, then
  /// `ToneMappingPass< ToneMappingAces >`, `ColorGradingPass` and an alpha-forwarding
  /// `ToSrgbPass` onto the canvas. Each stage is tested alone above; this one checks that
  /// they compose: where an occluder stands the canvas is fully transparent, so the camera
  /// image shows, and beside it the quad is opaque, premultiplied and not black.
  #[ wasm_bindgen_test ]
  fn the_ar_chain_keeps_the_occluder_transparent_on_the_canvas()
  {
    let gl = gl_init();
    let canvas = gl.canvas().expect( "the context has a canvas" ).dyn_into::< gl::web_sys::HtmlCanvasElement >().expect( "an HTML canvas" );
    canvas.set_width( SIZE );
    canvas.set_height( SIZE );

    let mut renderer = Renderer::new( &gl, SIZE, SIZE, 4 ).expect( "Renderer::new succeeds" );
    renderer.clear_color_set( gl::F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
    // A dielectric shows the PBR shader's ambient term without lights; the default metal is black.
    let mut quad_material = PbrMaterial::new( &gl );
    quad_material.metallic_factor = 0.0;
    let mut scene = Scene::new();
    scene.add( rect_node( &gl, [ -5.0, -5.0, 5.0, 5.0 ], 0.0, Box::new( quad_material ) ) );
    scene.add( rect_node( &gl, [ -5.0, -5.0, 0.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) ) );
    renderer.render( &gl, &mut scene, &camera() ).expect( "render succeeds" );

    let tone_mapping = ToneMappingPass::< ToneMappingAces >::new( &gl ).expect( "tone mapping compiles" );
    let color_grading = ColorGradingPass::new( &gl ).expect( "ColorGradingPass compiles" );
    let mut to_srgb = ToSrgbPass::new( &gl, true ).expect( "ToSrgbPass compiles" );
    to_srgb.alpha_forward_set( &gl, true ).expect( "the FORWARD_ALPHA variant compiles" );

    let ( tone_mapped, graded ) = ( stage_texture( &gl ), stage_texture( &gl ) );
    let framebuffer = gl.create_framebuffer().expect( "create_framebuffer succeeds" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &framebuffer ) );
    gl.viewport( 0, 0, SIZE as i32, SIZE as i32 );
    tone_mapping.render( &gl, renderer.main_texture(), Some( tone_mapped.clone() ) ).expect( "tone mapping renders" );
    color_grading.render( &gl, Some( tone_mapped.clone() ), Some( graded.clone() ) ).expect( "colour grading renders" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( Some( &framebuffer ) );
    gl.viewport( 0, 0, SIZE as i32, SIZE as i32 );
    to_srgb.render( &gl, Some( graded.clone() ), None ).expect( "ToSrgbPass renders" );
    assert_eq!( gl.get_error(), gl::NO_ERROR, "the chain raises no GL error" );

    let canvas_pixel = | x : i32 |
    {
      let mut pixel = [ 0_u8; 4 ];
      gl.read_pixels_with_opt_u8_array( x, ( SIZE / 2 ) as i32, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) )
      .expect( "read_pixels succeeds" );
      pixel
    };
    let mid = ( SIZE / 2 ) as i32;
    assert_eq!( canvas_pixel( mid - 16 ), [ 0, 0, 0, 0 ], "the occluder leaves the canvas fully transparent" );
    let quad = canvas_pixel( mid + 16 );
    assert_eq!( quad[ 3 ], 255, "the quad is opaque on the canvas: {quad:?}" );
    assert!( quad[ 0 ] > 0, "the quad is not black: {quad:?}" );
    assert_premultiplied( quad, "the quad" );

    gl.delete_texture( Some( &tone_mapped ) );
    gl.delete_texture( Some( &graded ) );
  }

  // --- Occluders in ScenePicker ---

  /// The pick id `ScenePicker` reports for each named node of the occluder scenes.
  fn pick_id_of( node : &Node ) -> PickId
  {
    match node.name_get().as_deref()
    {
      Some( "occluder" ) => 1,
      Some( "quad" ) => 2,
      _ => 0,
    }
  }

  /// `ScenePicker` draws an occluder into the id buffer like any mesh: depth-tested, and
  /// whatever its colour mask. So a pick in front of the geometry an occluder hides
  /// reports the occluder, never what is behind it, whichever of the two the scene lists
  /// first — the picking pass has no draw-order dependency to get wrong.
  #[ wasm_bindgen_test ]
  fn picking_reports_the_occluder_not_the_geometry_behind_it()
  {
    let gl = gl_init();
    let camera = camera();
    let view_projection = camera.projection_matrix_get() * camera.view_matrix_get();
    let picker = ScenePicker::new( &gl ).expect( "ScenePicker::new succeeds" );
    let mut buffer = IdBuffer::new( &gl ).expect( "IdBuffer::new succeeds" );
    buffer.size_set( SIZE, SIZE ).expect( "size_set succeeds" );
    let mid = ( SIZE / 2 ) as i32;

    for occluder_first in [ false, true ]
    {
      let quad = rect_node( &gl, [ -5.0, -5.0, 5.0, 5.0 ], 0.0, Box::new( PbrMaterial::new( &gl ) ) );
      quad.borrow_mut().name_set( "quad" );
      let occluder = rect_node( &gl, [ -5.0, -5.0, 0.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) );
      occluder.borrow_mut().name_set( "occluder" );

      let mut scene = Scene::new();
      if occluder_first
      {
        scene.add( occluder );
        scene.add( quad );
      }
      else
      {
        scene.add( quad );
        scene.add( occluder );
      }
      scene.world_matrix_update();
      picker.render( &buffer, &scene, view_projection, pick_id_of ).expect( "picking renders" );
      assert_eq!( gl.get_error(), gl::NO_ERROR, "the picking pass raises no GL error" );

      assert_eq!( buffer.read( mid - 16, mid ), Some( 1 ), "occluder first: {occluder_first}; the occluder covers the quad here" );
      assert_eq!( buffer.read( mid + 16, mid ), Some( 2 ), "occluder first: {occluder_first}; beside the occluder the quad is picked" );
    }
  }
}
