//! Live WebGL2-context pass-cycle test for `renderer::webgl::Renderer::render()` -- the
//! top-level per-frame orchestration method ( collect -> clear -> upload uniforms -> draw
//! opaque -> draw transparent -> composite ) -- following `fbo_pass_cycle_test.rs`'s pattern:
//! structural, not pixel-level. `webgl_frame_orchestration_test.rs` only unit-tests the pure
//! `frame_attachments( bool, bool )` helper with zero `WebGl2RenderingContext` calls anywhere in
//! its body, so it never actually invokes `render()`; this file closes that gap by driving the
//! real method end-to-end against a real WebGL2 context.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use std::{ cell::RefCell, rc::Rc };
  use renderer::webgl::
  {
    Camera, Geometry, Material, Mesh, Node, Object3D, Primitive, Renderer, Scene,
    material::PbrMaterial,
  };

  /// Creates a headless WebGL2 context with `EXT_color_buffer_float` -- unlike
  /// `fbo_pass_cycle_test.rs`'s `gl_init`, `Renderer::new` unconditionally builds its
  /// `FramebufferContext` with `RGBA16F` ( main / emission / transparent-accumulate ) and `R16F`
  /// ( transparent-revealage ) multisample color attachments regardless of what the scene
  /// contains, so the extension is mandatory here even for an otherwise-minimal render --
  /// mirroring `unreal_bloom_tests.rs`'s own `gl_init` rather than `fbo_pass_cycle_test.rs`'s.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default();
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

    gl.get_extension( "EXT_color_buffer_float" )
      .expect( "get_extension call should not throw" )
      .expect( "EXT_color_buffer_float must be available in the test environment" );

    gl
  }

  /// `Camera::new`'s known-good parameter set, mirroring `tests/webgl/camera.rs`'s and
  /// `fbo_pass_cycle_test.rs`'s own fixture.
  fn camera_make() -> Camera
  {
    Camera::new
    (
      gl::F32x3::from_array( [ 0.0, 1.0, 3.0 ] ),
      gl::F32x3::from_array( [ 0.0, 1.0, 0.0 ] ),
      gl::F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
      16.0 / 9.0,
      70.0f32.to_radians(),
      0.1,
      1000.0,
    ).expect( "valid_args camera parameters must construct successfully" )
  }

  /// A real opaque `Node` holding one `PbrMaterial` primitive backed by an empty
  /// ( zero-attribute, zero-vertex-count ) `Geometry` -- enough to drive `render()`'s real
  /// shader-compile-and-cache path ( `primitive_register`, compiling the actual production
  /// `main.vert`/`main.frag` pair ) and real per-frame uniform-upload path
  /// ( `per_program_uniforms_upload` / `opaque_draw`, looking up `PBRShader`'s real uniform
  /// locations ) without needing real vertex-buffer contents -- `Geometry::draw`'s
  /// `gl.draw_arrays( mode, 0, 0 )` on a zero-vertex-count geometry is a well-defined no-op draw
  /// call, matching `fbo_pass_cycle_test.rs`'s own "zero primitives/geometry, real GL call"
  /// minimalism one level further down the call stack ( primitive-level here vs. mesh-level
  /// there ).
  fn opaque_node_make( gl : &GL ) -> Rc< RefCell< Node > >
  {
    let geometry = Geometry::new( gl ).expect( "Geometry::new should succeed on a valid context" );
    let material : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( PbrMaterial::new( gl ) ) ) );
    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material
    };

    let mut mesh = Mesh::new();
    mesh.primitive_add( Rc::new( RefCell::new( primitive ) ) );

    let mut node = Node::default();
    node.object = Object3D::Mesh( Rc::new( RefCell::new( mesh ) ) );

    Rc::new( RefCell::new( node ) )
  }

  /// `Renderer::render` must complete without panicking, without an incomplete-framebuffer
  /// error, and without a missing-uniform failure against a scene holding one real opaque
  /// `PbrMaterial` primitive -- this is the real risk this test targets: `PbrMaterial::configure`
  /// and `Node`/`Camera::upload` all `.unwrap()` several uniform-location lookups against the
  /// real compiled `main.vert`/`main.frag` pair and `PBRShader`'s real ( not test-fixture )
  /// static uniform-name list, so a future rename mismatch between that list and any of these
  /// lookup call sites would panic here exactly as it would in production.
  #[ wasm_bindgen_test( async ) ]
  async fn render_completes_on_an_opaque_pbr_primitive()
  {
    let gl = gl_init();
    let mut renderer = Renderer::new( &gl, 64, 64, 4 )
    .expect( "Renderer::new should succeed on a valid context" );

    let mut scene = Scene::new();
    scene.add( opaque_node_make( &gl ) );
    let camera = camera_make();

    let result = renderer.render( &gl, &mut scene, &camera );

    assert!( result.is_ok(), "Renderer::render should succeed on a scene with one opaque PBR primitive -- got {:?}", result.err() );
  }

  /// `Renderer::render` must also complete on an empty scene ( `nodes_collect` registers no
  /// primitive and no light at all -- this isolates the real multisample bind/clear/resolve FBO
  /// cycle on its own, independent of whether any material shader ever compiles ).
  #[ wasm_bindgen_test( async ) ]
  async fn render_completes_on_an_empty_scene()
  {
    let gl = gl_init();
    let mut renderer = Renderer::new( &gl, 64, 64, 4 )
    .expect( "Renderer::new should succeed on a valid context" );

    let mut scene = Scene::new();
    let camera = camera_make();

    let result = renderer.render( &gl, &mut scene, &camera );

    assert!( result.is_ok(), "Renderer::render should succeed on an empty scene -- got {:?}", result.err() );
  }
}
