//! Simple rendering with PBR lighting and shadowmapping

use minwebgl as gl;
use gl::{ JsCast as _, math::mat3x3h, GL };
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use core::cell::RefCell;
use renderer::webgl::
{
  Light,
  Node,
  Object3D,
  SpotLight,
  Texture,
  TextureInfo,
  loaders::gltf,
  post_processing,
  shadow,
  cast_unchecked_material_to_ref_mut,
  material::PbrMaterial
};
use post_processing::{ Pass, SwapFramebuffer, ShadowToColorPass };
use shadow::{ ShadowBaker, ShadowMap };

fn main()
{
  gl::browser::setup( gl::browser::Config::default() );
  gl::spawn_local( async { gl::info!( "{:?}", app_run().await ) } );
}

/// Creates the scene camera sized to the canvas and binds its controls.
fn camera_setup( canvas : &HtmlCanvasElement, width : i32, height : i32 ) -> renderer::webgl::Camera
{
  let aspect = width as f32 / height as f32;

  let mut camera = renderer::webgl::Camera::new
  (
    [ 0.0, 1.5, 5.0 ].into(),
    [ 0.0, 1.0, 0.0 ].into(),
    [ 0.0, 0.0, 0.0 ].into(),
    aspect,
    45.0_f32.to_radians(),
    0.1,
    100.0
  ).expect( "camera parameters are valid" );
  camera.window_size_set( [ width as f32, height as f32 ].into() );
  camera.controls_bind( canvas );

  camera
}

/// Creates a spot light node wrapping the given spot light description.
fn spot_light_node( spot : SpotLight ) -> Node
{
  let mut node = Node::new();
  node.object = Object3D::Light( Light::Spot( spot ) );
  node
}

/// Builds the shadow-mapping projection light for the given spot light description, via the
/// canonical `SpotLight` -> `shadow::Light` conversion.
///
/// Fix(BUG-XXX-D): this crate previously constructed `shadow::Light` manually --
/// `mat3x3h::perspective_rh_gl( 60.0_f32.to_radians(), 1.0, 0.1, 30.0 )` with a hardcoded
/// `light_size` of `0.5` -- instead of using `renderer::webgl::shadow`'s
/// `impl From< SpotLight > for Light`, which correctly doubles `outer_cone_angle` into a full
/// FOV, derives `far` from the spot light's own `range`, and derives `light_size` from the cone
/// angle. Using the raw ( undoubled ) `outer_cone_angle` as the shadow map's `fovy` made the
/// shadow-map frustum only half as wide as the spot light's actual illumination cone, leaving
/// the outer half of every lit surface -- including a visible band of this scene's floor plane
/// -- without valid shadow-map depth data.
/// Root cause: manual re-derivation of a conversion the `shadow` module already provides
/// correctly, using the cone half-angle directly as a full FOV instead of doubling it.
/// Pitfall: a shadow-casting light's projection FOV must cover at least the light's own visible
/// cone/angle -- reusing one of the light's own angle fields as the shadow camera's FOV without
/// checking whether that field is a half-angle or full-angle silently under-covers the lit area.
fn shadow_light_from_spot( spot : SpotLight ) -> shadow::Light
{
  spot.into()
}

/// Marks every mesh in the scene as a shadow caster.
fn shadow_casters_mark( scene : &renderer::webgl::Scene )
{
  _ = scene.traverse
  (
    &mut | node |
    {
      let node = node.borrow();
      if let Object3D::Mesh( mesh ) = &node.object
      {
        let mut mesh = mesh.borrow_mut();
        mesh.is_shadow_caster = true;
      }
      Ok( () )
    }
  );
}

/// Applies the baked colored shadow texture as the floor's base color texture.
fn floor_texture_apply( floor_node : &Rc< RefCell< Node > >, colored_texture : Option< web_sys::WebGlTexture > )
{
  if let Object3D::Mesh( mesh ) = &floor_node.borrow().object
  {
    let mut texture = Texture::new();
    texture.source = colored_texture;
    let texture_info = TextureInfo
    {
      texture : Rc::new( RefCell::new( texture ) ),
      uv_position : 0,
    };
    let mesh_borrow = mesh.borrow_mut();
    let primitive = &mesh_borrow.primitives[ 0 ];
    let primitive_borrow = primitive.borrow_mut();
    let material_ref = primitive_borrow.material.borrow_mut();
    let mut pbr_material = cast_unchecked_material_to_ref_mut::< PbrMaterial >( material_ref );
    pbr_material.base_color_texture_set( Some( texture_info ) );
  }
}

async fn app_run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );

  let canvas = gl.canvas()
  .unwrap()
  .dyn_into::< HtmlCanvasElement >()
  .unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  let camera = camera_setup( &canvas, width, height );

  // EXT_color_buffer_float is required for rendering to float framebuffer attachments (RGBA16F/RGBA32F).
  gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to query EXT_color_buffer_float" )
  .expect( "EXT_color_buffer_float is required for float framebuffer attachments" );

  let mut renderer = renderer::webgl::Renderer::new( &gl, width as u32, height as u32, 4 )?;
  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;
  let mut swap_buffer = SwapFramebuffer::new( &gl, width as u32, height as u32 );

  let mesh = gltf::load( &document, "static/skull_salazar_downloadable.glb", &gl ).await?;

  let cube_mesh = gltf::load( &document, "static/plane.glb", &gl ).await?;
  let cube_model = mat3x3h::translation( [ 0.0, -1.0, 0.0 ] ) * mat3x3h::scale( [ 8.0, 1.0, 8.0 ] );

  let mut main_scene = renderer::webgl::Scene::new();

  for scene in mesh.scenes
  {
    let mut scene = scene.borrow_mut();
    for node in core::mem::take( &mut scene.children )
    {
      main_scene.add( node );
    }
  }

  let floor_node = cube_mesh.scenes[ 0 ].borrow().children[ 0 ].clone();
  main_scene.add( floor_node.clone() );
  floor_node.borrow_mut().local_matrix_set( cube_model );
  main_scene.world_matrix_update();

  let light_pos = gl::F32x3::from_array( [ 0.0, 3.0, 3.0 ] );
  let light_dir = gl::F32x3::from_array( [ 0.0, -1.0, -1.0 ] ).normalize();

  let spot_light = SpotLight
  {
    position : light_pos,
    direction : light_dir,
    color : [ 1.0, 1.0, 1.0 ].into(),
    strength : 300.0,
    range : 100.0,
    inner_cone_angle : 40.0_f32.to_radians(),
    outer_cone_angle : 60.0_f32.to_radians(),
    use_light_map : true
  };

  main_scene.add( Rc::new( RefCell::new( spot_light_node( spot_light ) ) ) );

  shadow_casters_mark( &main_scene );

  let light = shadow_light_from_spot( spot_light );

  let shadowmap_res = 2048;
  let lightmap_res = 2048;
  let shadowmap = ShadowMap::new( &gl, shadowmap_res )?;
  shadowmap.render( &main_scene, light )?;
  let shadow_texture = texture_create( &gl, lightmap_res, gl::R8 );
  let shadow_baker = ShadowBaker::new( &gl )?;
  shadow_baker.soft_shadow_render( &floor_node.borrow(), shadow_texture.as_ref(), lightmap_res, lightmap_res, &shadowmap, light )?;

  // Convert shadow texture to colored base color texture
  let base_color = [ 0.8, 0.8, 0.8 ];
  let shadow_to_color_pass = ShadowToColorPass::new( &gl, base_color )?;
  let colored_texture = texture_create( &gl, lightmap_res, gl::RGB8 );

  // Create a framebuffer for rendering
  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );

  // Apply the shadow-to-color conversion
  shadow_to_color_pass.render( &gl, shadow_texture.clone(), colored_texture.clone() )?;

  // Unbind framebuffer
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  floor_texture_apply( &floor_node, colored_texture );

  let update = move | _ |
  {
    renderer.render( &gl, &mut main_scene, &camera ).expect( "Failed to render" );

    swap_buffer.reset();
    swap_buffer.bind( &gl );
    swap_buffer.input_set( renderer.main_texture() );

    let t = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
    .expect( "Failed to render tonemapping pass" );

    swap_buffer.output_set( t );
    swap_buffer.swap();

    let _ = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
    .expect( "Failed to render ToSrgbPass" );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}

fn texture_create( gl : &GL, res : u32, format : u32 ) -> Option< web_sys::WebGlTexture >
{
  let ret = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, ret.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, 1, format, res as i32, res as i32 );
  gl::texture::d2::filter_linear( gl );
  gl::texture::d2::wrap_clamp( gl );

  ret
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// The exact spot light description used by `app_run` for this demo's shadow-casting light.
  fn demo_spot_light() -> SpotLight
  {
    SpotLight
    {
      position : gl::F32x3::from_array( [ 0.0, 3.0, 3.0 ] ),
      direction : gl::F32x3::from_array( [ 0.0, -1.0, -1.0 ] ).normalize(),
      color : [ 1.0, 1.0, 1.0 ].into(),
      strength : 300.0,
      range : 100.0,
      inner_cone_angle : 40.0_f32.to_radians(),
      outer_cone_angle : 60.0_f32.to_radians(),
      use_light_map : true
    }
  }

  /// ## Root Cause
  /// `shadow_light_from_spot` ( formerly inlined in `app_run` ) previously built the
  /// shadow-mapping projection via `mat3x3h::perspective_rh_gl( 60.0_f32.to_radians(), 1.0, 0.1,
  /// 30.0 )` -- reusing the spot light's `outer_cone_angle` ( a half-angle from the light's
  /// direction to the edge of its cone, per the glTF spot light convention `SpotLight` follows )
  /// directly as `fovy` ( a full angle ). The shadow camera's frustum was therefore only half as
  /// wide as the spot light's actual illumination cone, and its far plane ( a disconnected
  /// hardcoded `30.0` ) didn't track the light's own declared `range` ( `100.0` ) either.
  ///
  /// ## Why Not Caught
  /// This crate has no test file -- it is a `fn main()`-only WebGL demo binary, verified by
  /// running it in a browser. The undersized shadow frustum only manifests as incorrect/missing
  /// shadow data on the outer portion of the lit floor ( geometrically, floor points beyond
  /// roughly 30 degrees off the light's axis but still within its actual 60-degree half-angle
  /// cone ), which reads as a plausible soft-shadow falloff rather than an obviously wrong
  /// result, especially at a glance in a browser.
  ///
  /// ## Fix Applied
  /// Replaced the manual `shadow::Light::new( ..., perspective_rh_gl( 60.0_f32.to_radians(), ...
  /// ), 0.5 )` construction with the canonical `impl From< SpotLight > for shadow::Light`
  /// ( `module/helper/renderer/src/webgl/shadow.rs` ), which doubles `outer_cone_angle` into a
  /// full FOV, derives `far` from the spot light's own `range`, and derives `light_size` from
  /// the cone angle -- reusing the exact same `SpotLight` value for both the scene's light node
  /// and the shadow-mapping light instead of re-declaring its parameters twice.
  ///
  /// ## Prevention
  /// `test_shadow_light_projection_matches_canonical_doubled_fov` pins the fixed projection
  /// against the canonical doubled-FOV/range-derived formula computed independently in the test,
  /// rather than merely checking the code compiles and runs.
  ///
  /// ## Pitfall
  /// When a light's own struct already carries a "canonical conversion" to a dependent type
  /// ( here, `SpotLight -> shadow::Light` ), manually re-deriving that conversion at a call site
  /// -- instead of using the conversion -- silently drops whatever correctness work the
  /// canonical version already encodes ( here, doubling a half-angle into a full FOV ). Re-using
  /// one of a light's own fields as an unrelated parameter (a shadow camera's FOV) requires
  /// checking whether the units/semantics actually match, not just whether the value "looks
  /// like" a plausible angle.
  // Fix(BUG-XXX-D): reproducer for the shadow-mapping spot light's FOV being built from the raw,
  // undoubled `outer_cone_angle` instead of the canonical `outer_cone_angle * 2.0`.
  // test_kind: bug_reproducer(BUG-XXX-D)
  #[ test ]
  fn test_shadow_light_projection_matches_canonical_doubled_fov()
  {
    let spot = demo_spot_light();
    let light = shadow_light_from_spot( spot );

    let expected = mat3x3h::perspective_rh_gl( spot.outer_cone_angle * 2.0, 1.0, 0.1, spot.range );
    let actual = light.projection();

    for ( a, e ) in actual.raw_slice().iter().zip( expected.raw_slice().iter() )
    {
      assert!( ( a - e ).abs() < 1e-6, "projection element mismatch: {a} vs {e}" );
    }
  }

  /// Pins that the fixed projection is *not* what the pre-fix ( undoubled FOV, disconnected
  /// far=30.0 ) formula would have produced, confirming the bug was real and the fix actually
  /// changes behavior rather than being a no-op refactor.
  // Fix(BUG-XXX-D): reproducer confirming the fixed projection diverges from the pre-fix formula.
  // test_kind: bug_reproducer(BUG-XXX-D)
  #[ test ]
  fn test_pre_fix_undoubled_fov_formula_would_have_diverged()
  {
    let spot = demo_spot_light();
    let light = shadow_light_from_spot( spot );

    // The pre-fix code used the raw ( undoubled ) cone angle and a hardcoded far=30.0.
    let pre_fix_projection = mat3x3h::perspective_rh_gl( spot.outer_cone_angle, 1.0, 0.1, 30.0 );
    let actual = light.projection();

    let any_element_differs = actual.raw_slice().iter()
    .zip( pre_fix_projection.raw_slice().iter() )
    .any( | ( a, p ) | ( a - p ).abs() > 1e-6 );

    assert!( any_element_differs, "fixed projection unexpectedly matches the pre-fix (undoubled FOV) formula" );
  }

  /// Structural check: `spot_light_node` must wrap the exact `SpotLight` it is given -- not
  /// silently rebuild one with different parameters -- since the scene's visible light and the
  /// shadow-mapping light must stay derived from the same single source of truth.
  #[ allow( clippy::float_cmp, reason = "wrapped is spot moved through spot_light_node with no arithmetic in between, so its fields must be bit-identical copies, not merely close" ) ]
  #[ test ]
  fn test_spot_light_node_preserves_the_given_spot_light_parameters()
  {
    let spot = demo_spot_light();
    let node = spot_light_node( spot );

    match node.object
    {
      Object3D::Light( Light::Spot( wrapped ) ) =>
      {
        assert_eq!( wrapped.position, spot.position );
        assert_eq!( wrapped.outer_cone_angle, spot.outer_cone_angle );
        assert_eq!( wrapped.range, spot.range );
      },
      _ => panic!( "spot_light_node must produce an Object3D::Light( Light::Spot( .. ) )" ),
    }
  }
}
