//! Procedural sci-fi HUD diagram: animated star, orbit ring, and a Cartesian
//! grid, rendered by a fullscreen fragment shader with a real multi-pass
//! Unreal-style bloom, driven by parameterization uniforms adjustable live
//! via the keyboard.
//!
//! Every color, opacity, and radius not listed above as keyboard-live comes
//! from `scene.rhai` (see `scene` module) instead of being a shader
//! constant — edit that file and rebuild to restyle the diagram.

// Only the wasm32 path (`app_run()`) consumes the scene module here; the native
// path below is a stub, and the scene tests live in `tests/scene_test.rs`
// against the library target.
#[cfg( target_arch = "wasm32" )]
use minwebgl_sun_grid_lines::scene;

#[cfg( target_arch = "wasm32" )]
use minwebgl as gl;
#[cfg( target_arch = "wasm32" )]
use gl::GL;
#[cfg( target_arch = "wasm32" )]
use renderer::webgl::post_processing::{ Pass, SwapFramebuffer, UnrealBloomPass, BlendPass, ToSrgbPass };
#[cfg( target_arch = "wasm32" )]
use std::rc::Rc;
#[cfg( target_arch = "wasm32" )]
use core::cell::RefCell;
#[cfg( target_arch = "wasm32" )]
use web_sys::{ wasm_bindgen::prelude::*, KeyboardEvent };

#[cfg( target_arch = "wasm32" )]
const SIZE : i32 = 800; // square canvas, matching the reference composition

/// Live-adjustable scene parameters, shared between the animation loop and
/// the keyboard handler below.
#[cfg( target_arch = "wasm32" )]
struct Params
{
  seed : f32,
  node_count : i32,
  grid_density : f32
}

/// Allocates one linear-filtered, clamp-wrapped `RGBA16F` render target of
/// size `SIZE x SIZE`, matching the format `UnrealBloomPass` expects as
/// input.
#[cfg( target_arch = "wasm32" )]
fn target_make( gl : &GL ) -> Option< gl::web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::RGBA16F, SIZE, SIZE );
  gl::texture::d2::filter_linear( gl );
  gl::texture::d2::wrap_clamp( gl );
  gl.bind_texture( GL::TEXTURE_2D, None );
  texture
}

/// Uploads the static scene styling, loaded once from `scene.rhai` (see
/// `scene::SceneConfig`) and uploaded once at startup rather than every frame
/// like the 4 dynamic uniforms in `app_run` — unlike the WebGPU port's single
/// packed uniform buffer (rewritten wholesale each frame), this crate
/// uploads one GL uniform per field, so there's no bulk buffer write to
/// piggyback the per-frame values onto and no reason to re-upload
/// unchanging values every frame.
#[cfg( target_arch = "wasm32" )]
fn scene_styling_upload( gl : &GL, program : &gl::web_sys::WebGlProgram )
{
  let u_bg_top_loc = gl.get_uniform_location( program, "u_bg_top" );
  let u_bg_bottom_loc = gl.get_uniform_location( program, "u_bg_bottom" );
  let u_nebula_color_loc = gl.get_uniform_location( program, "u_nebula_color" );
  let u_nebula_opacity_loc = gl.get_uniform_location( program, "u_nebula_opacity" );
  let u_stars_color_loc = gl.get_uniform_location( program, "u_stars_color" );
  let u_stars_intensity_loc = gl.get_uniform_location( program, "u_stars_intensity" );
  let u_grid_color_loc = gl.get_uniform_location( program, "u_grid_color" );
  let u_grid_opacity_loc = gl.get_uniform_location( program, "u_grid_opacity" );
  let u_corona_inner_loc = gl.get_uniform_location( program, "u_corona_inner" );
  let u_corona_mid_loc = gl.get_uniform_location( program, "u_corona_mid" );
  let u_corona_outer_loc = gl.get_uniform_location( program, "u_corona_outer" );
  let u_disc_dark_loc = gl.get_uniform_location( program, "u_disc_dark" );
  let u_disc_mid_loc = gl.get_uniform_location( program, "u_disc_mid" );
  let u_disc_bright_loc = gl.get_uniform_location( program, "u_disc_bright" );
  let u_disc_base_radius_loc = gl.get_uniform_location( program, "u_disc_base_radius" );
  let u_ring_color_loc = gl.get_uniform_location( program, "u_ring_color" );
  let u_ring_radius_loc = gl.get_uniform_location( program, "u_ring_radius" );

  let scene = scene::SceneConfig::load();
  gl.use_program( Some( program ) );
  gl::uniform::upload( gl, u_bg_top_loc, &scene.background.top.to_array() ).unwrap();
  gl::uniform::upload( gl, u_bg_bottom_loc, &scene.background.bottom.to_array() ).unwrap();
  gl::uniform::upload( gl, u_nebula_color_loc, &scene.nebula.color.to_array() ).unwrap();
  gl::uniform::upload( gl, u_nebula_opacity_loc, &( scene.nebula.opacity as f32 ) ).unwrap();
  gl::uniform::upload( gl, u_stars_color_loc, &scene.stars.color.to_array() ).unwrap();
  gl::uniform::upload( gl, u_stars_intensity_loc, &( scene.stars.intensity as f32 ) ).unwrap();
  gl::uniform::upload( gl, u_grid_color_loc, &scene.grid.color.to_array() ).unwrap();
  gl::uniform::upload( gl, u_grid_opacity_loc, &( scene.grid.opacity as f32 ) ).unwrap();
  gl::uniform::upload( gl, u_corona_inner_loc, &scene.sun_corona.inner.to_array() ).unwrap();
  gl::uniform::upload( gl, u_corona_mid_loc, &scene.sun_corona.mid.to_array() ).unwrap();
  gl::uniform::upload( gl, u_corona_outer_loc, &scene.sun_corona.outer.to_array() ).unwrap();
  gl::uniform::upload( gl, u_disc_dark_loc, &scene.sun_disc.dark.to_array() ).unwrap();
  gl::uniform::upload( gl, u_disc_mid_loc, &scene.sun_disc.mid.to_array() ).unwrap();
  gl::uniform::upload( gl, u_disc_bright_loc, &scene.sun_disc.bright.to_array() ).unwrap();
  gl::uniform::upload( gl, u_disc_base_radius_loc, &( scene.sun_disc.base_radius as f32 ) ).unwrap();
  gl::uniform::upload( gl, u_ring_color_loc, &scene.orbit_ring.color.to_array() ).unwrap();
  gl::uniform::upload( gl, u_ring_radius_loc, &( scene.orbit_ring.radius as f32 ) ).unwrap();
}

#[cfg( target_arch = "wasm32" )]
fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );

  let canvas = gl::canvas::make()?;
  canvas.set_width( SIZE as u32 );
  canvas.set_height( SIZE as u32 );

  let gl = gl::context::from_canvas( &canvas )?;
  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  let vertex_shader_src = include_str!( "../shaders/scene.vert" );
  let fragment_shader_src = include_str!( "../shaders/scene.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;

  let u_time_loc = gl.get_uniform_location( &program, "u_time" );
  let u_seed_loc = gl.get_uniform_location( &program, "u_seed" );
  let u_node_count_loc = gl.get_uniform_location( &program, "u_node_count" );
  let u_grid_density_loc = gl.get_uniform_location( &program, "u_grid_density" );

  scene_styling_upload( &gl, &program );

  // Offscreen G-buffer-style framebuffer: attachment 0 receives the composed
  // scene color, attachment 1 receives emission only ( the subset of the
  // scene that should bloom ). Both feed the post-processing chain below.
  let scene_framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, scene_framebuffer.as_ref() );
  let main_color_texture = target_make( &gl );
  let emission_texture = target_make( &gl );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, main_color_texture.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, emission_texture.as_ref(), 0 );
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  // Bloom + blend + present chain, wired exactly as the renderer crate's own
  // Renderer uses it internally ( module/helper/renderer/src/webgl/renderer.rs ).
  let mut swap = SwapFramebuffer::new( &gl, SIZE as u32, SIZE as u32 );
  let bloom = UnrealBloomPass::new( &gl, SIZE as u32, SIZE as u32, GL::RGBA16F )?;
  let mut blend = BlendPass::new( &gl )?;
  let to_srgb = ToSrgbPass::new( &gl, true )?;

  let params = Rc::new( RefCell::new( Params { seed : 0.0, node_count : 1, grid_density : 10.0 } ) );

  // Keyboard controls demonstrate live re-parameterization of the shader:
  // Up/Down change how many nodes orbit the ring, Left/Right change grid
  // density, and Space reshuffles the star field and node layout.
  {
    let params = params.clone();
    let keydown_closure = move | e : KeyboardEvent |
    {
      let mut params = params.borrow_mut();
      match e.key().as_str()
      {
        "ArrowUp" => params.node_count = ( params.node_count + 1 ).min( 8 ),
        "ArrowDown" => params.node_count = ( params.node_count - 1 ).max( 1 ),
        "ArrowRight" => params.grid_density = ( params.grid_density + 2.0 ).min( 24.0 ),
        "ArrowLeft" => params.grid_density = ( params.grid_density - 2.0 ).max( 4.0 ),
        " " => params.seed = params.seed * 1.618_034 + 1.0,
        _ => {}
      }
    };
    let closure = Closure::< dyn FnMut( _ ) >::new( Box::new( keydown_closure ) );
    gl::web_sys::window().unwrap().set_onkeydown( Some( closure.as_ref().unchecked_ref() ) );
    closure.forget();
  }

  let update_and_draw = move | t : f64 |
  {
    let time = ( t / 1000.0 ) as f32;
    let ( seed, node_count, grid_density ) =
    {
      let params = params.borrow();
      ( params.seed, params.node_count, params.grid_density )
    };

    // Pass 1: draw the procedural scene into the offscreen color + emission
    // targets.
    gl.bind_framebuffer( GL::FRAMEBUFFER, scene_framebuffer.as_ref() );
    gl::drawbuffers::drawbuffers( &gl, &[ 0, 1 ] );
    gl.viewport( 0, 0, SIZE, SIZE );
    gl.use_program( Some( &program ) );
    gl::uniform::upload( &gl, u_time_loc.clone(), &time ).unwrap();
    gl::uniform::upload( &gl, u_seed_loc.clone(), &seed ).unwrap();
    gl::uniform::upload( &gl, u_node_count_loc.clone(), &node_count ).unwrap();
    gl::uniform::upload( &gl, u_grid_density_loc.clone(), &grid_density ).unwrap();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    // Pass 2: real multi-mip Unreal-style bloom on the emission target,
    // additively blended back onto the main color, then presented to the
    // screen with sRGB conversion.
    swap.reset();
    swap.bind( &gl );
    swap.input_set( emission_texture.clone() );
    bloom.render( &gl, swap.input_get(), swap.output_get() ).unwrap();
    blend.blend_texture_set( swap.output_get() );
    blend.render( &gl, None, main_color_texture.clone() ).unwrap();
    to_srgb.render( &gl, main_color_texture.clone(), None ).unwrap();

    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

#[cfg( target_arch = "wasm32" )]
fn main()
{
  app_run().unwrap();
}

// Stub main for native targets
#[cfg( not( target_arch = "wasm32" ) )]
fn main()
{
  println!( "This WebGL2 example only works on WebAssembly targets." );
  println!( "To run this example, compile for wasm32-unknown-unknown target:" );
  println!( "  cargo build --target wasm32-unknown-unknown" );
}
