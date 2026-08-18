//! Live WebGL2-context FBO pass-cycle tests for `ShadowMap` ( `renderer::webgl::shadow` ) and
//! `GBuffer` ( `renderer::webgl::post_processing` ) -- structural tests following the
//! `pmrem_tests.rs` pattern: they do not verify pixel-level output, but they catch panics,
//! incomplete-framebuffer failures, and missing-uniform regressions in the bind/clear/render
//! pass cycle without a human in the loop.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use rustc_hash::FxHashMap;
  use std::{ cell::RefCell, rc::Rc };
  use renderer::webgl::
  {
    Camera, Mesh, Node, Object3D, Scene, SpotLight,
    post_processing::{ GBuffer, GBufferAttachment },
    shadow::{ Light, ShadowMap },
  };

  /// Creates a headless WebGL2 context -- neither pass under test needs an extension beyond
  /// core WebGL2 ( `ShadowMap` uses `DEPTH_COMPONENT32F`; `GBuffer` here is scoped to the
  /// `RGBA8`-only attachments below ), unlike `pmrem_tests.rs`'s `EXT_color_buffer_float` need.
  /// Synchronous ( unlike `pmrem_tests.rs`'s `gl_init` ) -- nothing here awaits.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default();
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  /// A well-formed `SpotLight`, mirroring `tests/webgl/shadow.rs`'s own fixture values.
  fn spot_light_make() -> SpotLight
  {
    SpotLight
    {
      position : gl::F32x3::from_array( [ 0.0, 5.0, 0.0 ] ),
      direction : gl::F32x3::from_array( [ 0.0, -1.0, 0.0 ] ),
      color : gl::F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
      strength : 1.0,
      range : 10.0,
      inner_cone_angle : 0.1,
      outer_cone_angle : 0.5,
      use_light_map : false,
    }
  }

  /// `Camera::new`'s known-good parameter set, mirroring `tests/webgl/camera.rs`'s own fixture.
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

  /// `ShadowMap::bind`/`clear`/`render` must complete without panicking against a scene holding
  /// one shadow-casting mesh ( zero primitives -- enough to exercise the real `mvp_upload` GL
  /// call inside `render`'s traverse closure without needing actual geometry ).
  #[ wasm_bindgen_test( async ) ]
  async fn shadow_map_bind_clear_render_completes_on_a_shadow_casting_mesh()
  {
    let gl = gl_init();
    let shadow_map = ShadowMap::new( &gl, 64 ).expect( "ShadowMap::new should succeed on a valid context" );

    shadow_map.bind();
    shadow_map.clear();

    let mesh = Rc::new( RefCell::new( Mesh::default() ) );
    mesh.borrow_mut().is_shadow_caster = true;
    let mut node = Node::default();
    node.object = Object3D::Mesh( mesh );
    let mut scene = Scene::new();
    scene.add( Rc::new( RefCell::new( node ) ) );

    let light = Light::from( spot_light_make() );
    let result = shadow_map.render( &scene, light );

    assert!( result.is_ok(), "ShadowMap::render should succeed on a shadow-casting mesh with no primitives -- got {:?}", result.err() );
  }

  /// `ShadowMap::render` must also complete on an empty scene ( the early-return,
  /// non-shadow-caster path through the traverse closure never fires at all ).
  #[ wasm_bindgen_test( async ) ]
  async fn shadow_map_render_completes_on_an_empty_scene()
  {
    let gl = gl_init();
    let shadow_map = ShadowMap::new( &gl, 64 ).expect( "ShadowMap::new should succeed on a valid context" );
    let scene = Scene::new();
    let light = Light::from( spot_light_make() );

    let result = shadow_map.render( &scene, light );

    assert!( result.is_ok(), "ShadowMap::render should succeed on an empty scene -- got {:?}", result.err() );
  }

  /// `GBuffer::bind`/`render` must complete without panicking on an empty scene, using the
  /// minimal attachment set ( `Albedo` + `PbrInfo` + `Uv1` ) that avoids both `PbrMaterial`
  /// construction ( reserved for a follow-up task ) and the `EXT_color_buffer_float` extension --
  /// `ObjectColor`/`Position`/`Normal` need `RGBA16F`, but `Albedo`/`PbrInfo` only need `RGBA8`,
  /// and `Uv1` contributes no texture attachment at all ( vertex-attribute metadata only ).
  #[ wasm_bindgen_test( async ) ]
  async fn gbuffer_bind_render_completes_on_an_empty_scene()
  {
    let gl = gl_init();

    let mut attachment_buffers : FxHashMap< GBufferAttachment, Vec< gl::web_sys::WebGlBuffer > > = FxHashMap::default();
    attachment_buffers.insert( GBufferAttachment::Albedo, vec![] );
    attachment_buffers.insert( GBufferAttachment::PbrInfo, vec![] );
    attachment_buffers.insert( GBufferAttachment::Uv1, vec![] );

    let mut gbuffer = GBuffer::new( &gl, 64, 64, attachment_buffers )
    .expect( "GBuffer::new should succeed on a valid context with a minimal attachment set" );

    gbuffer.bind( &gl );

    let mut scene = Scene::new();
    let camera = camera_make();
    let result = gbuffer.render( &gl, &mut scene, None, &camera );

    assert!( result.is_ok(), "GBuffer::render should succeed on an empty scene -- got {:?}", result.err() );
  }
}
