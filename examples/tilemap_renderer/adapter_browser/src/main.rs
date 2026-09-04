//! Drives `tilemap_renderer`'s `adapter-webgpu` / `adapter-webgl` `Backend`
//! impls through a real browser canvas — the browser-side counterpart to
//! `tilemap_renderer/tests/native_backend_test.rs`, which proves the same
//! construct → assets_load → submit → output flow on the native backend via
//! offscreen GPU readback instead. Reuses that test's exact 8x8 solid-red
//! sprite asset and centered-sprite command shape.
//!
//! `adapter-webgpu`'s `WebGpuBackend::new` is async (mirrors
//! `gpu_hal::Device::new_webgpu`) and `adapter-webgl`'s `WebGlBackend::new`
//! is not, so one `main()` can only drive one backend per build — pick one
//! via Cargo features:
//! ```bash
//! trunk serve --release                                         # webgpu ( default )
//! trunk serve --release --no-default-features --features webgl  # webgl
//! ```
//!
//! **Historical note (superseded by task 218, re-confirmed live):** at the
//! time this example was written, the two backends were NOT expected to
//! paint the same pixel — `adapter-webgl` uploaded real pixel bytes and
//! painted the sprite's configured solid red, while `adapter-webgpu` had no
//! texture-upload path wired and painted an opaque **black** quad instead.
//! Task 218 wired `adapter-webgpu`'s own real pixel upload
//! (`tilemap_renderer::assets::to_rgba8`, shared with `adapter-native`), and
//! this has now been re-confirmed live in Firefox: both backends paint the
//! identical solid red; see `tilemap_renderer/tests/manual/readme.md`
//! Scenario 2.

#[ cfg( target_arch = "wasm32" ) ]
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
#[ cfg( target_arch = "wasm32" ) ]
use tilemap_renderer::backend::{ Backend, Output };
#[ cfg( target_arch = "wasm32" ) ]
use tilemap_renderer::commands::{ Clear, RenderCommand, Sprite };
#[ cfg( target_arch = "wasm32" ) ]
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };

/// Clear color — **not** black, unlike `native_backend_test.rs`'s own
/// `CLEAR`: at authoring time `adapter-webgpu`'s sprite itself rendered
/// black (see the module doc comment on `WebGpuBackend`), so a black clear
/// would have made the corner (clear) and center (sprite) pixels
/// indistinguishable there, defeating the bounded-draw check (`AF2` in the
/// governing task file). Left blue rather than reverted after task 218 wired
/// real pixel upload — blue stays distinct from the now-predicted solid-red
/// sprite too, and re-deriving offsets/colors for a live-unconfirmed change
/// is unwarranted churn.
#[ cfg( target_arch = "wasm32" ) ]
const CLEAR : [ f32; 4 ] = [ 0.0, 0.0, 1.0, 1.0 ];
/// Sprite source color — solid red, distinct from `CLEAR`. Matches
/// `native_backend_test.rs`'s own `SPRITE_RGBA` exactly; since task 218,
/// `adapter-webgpu` uploads these exact bytes too (previously ignored).
#[ cfg( target_arch = "wasm32" ) ]
const SPRITE_RGBA : [ u8; 4 ] = [ 255, 0, 0, 255 ];
/// Proportion of the viewport's extent the centered sprite should visually
/// occupy — mirrors `native_backend_test.rs`'s own `0.375`.
#[ cfg( target_arch = "wasm32" ) ]
const SPRITE_PROPORTION : f32 = 0.375;
/// `solid_sprite_assets`'s region size (its 8x8 asset). Needed by
/// `centered_sprite_command`: `Transform::scale` multiplies this region
/// size, not the sprite's final on-screen size — see that function's doc
/// comment.
#[ cfg( target_arch = "wasm32" ) ]
const SPRITE_REGION_SIZE : f32 = 8.0;

/// Builds the same 8x8 solid-red `Assets` set as
/// `native_backend_test.rs::solid_sprite_assets`: one image, one sprite
/// covering the full sheet.
#[ cfg( target_arch = "wasm32" ) ]
fn solid_sprite_assets() -> Assets
{
  Assets
  {
    fonts : vec![],
    images : vec!
    [
      ImageAsset
      {
        id : ResourceId::new( 0 ),
        source : ImageSource::Bitmap
        {
          bytes : SPRITE_RGBA.repeat( 8 * 8 ),
          width : 8,
          height : 8,
          format : PixelFormat::Rgba8,
        },
        filter : SamplerFilter::default(),
        mipmap : MipmapMode::default(),
        wrap : WrapMode::default(),
      }
    ],
    sprites : vec![ SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 8.0, 8.0 ] } ],
    geometries : vec![],
    gradients : vec![],
    patterns : vec![],
    clip_masks : vec![],
    paths : vec![],
  }
}

/// A `Sprite` command centered in a `width x height` viewport, occupying
/// `SPRITE_PROPORTION` of it.
///
/// `Transform::to_mat3()` places `position` as the quad's *starting corner*,
/// not its center — both `sprite.vert` and `webgpu.rs`'s WGSL shader compute
/// `world = transform * ( quad * region_size )` for a raw `[0,1]` unit
/// `quad`, so `scale` multiplies `SPRITE_REGION_SIZE`, not the sprite's
/// final on-screen size. `native_backend_test.rs::centered_sprite_command`'s
/// raw `position`/`scale` numbers rely on this same corner+extent math, but
/// its two pixel assertions can't tell a small centered square from a
/// quadrant-filling quad apart — copying those numbers verbatim produces
/// the latter here. Solving for the corner and scale that actually center a
/// `SPRITE_PROPORTION`-sized square avoids that trap.
#[ cfg( target_arch = "wasm32" ) ]
fn centered_sprite_command( width : f32, height : f32 ) -> RenderCommand
{
  let half_proportion = SPRITE_PROPORTION / 2.0;
  RenderCommand::Sprite( Sprite
  {
    transform : Transform
    {
      position : [ width * ( 0.5 - half_proportion ), height * ( 0.5 - half_proportion ) ],
      scale : [ width * SPRITE_PROPORTION / SPRITE_REGION_SIZE, height * SPRITE_PROPORTION / SPRITE_REGION_SIZE ],
      ..Default::default()
    },
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  })
}

/// Loads `solid_sprite_assets`, submits a clear plus a centered sprite —
/// identical scene shape for both backends, so any pixel difference between
/// them traces to the adapter, not the scene.
#[ cfg( target_arch = "wasm32" ) ]
fn scene_render< B : Backend >( backend : &mut B, width : f32, height : f32 )
{
  backend.assets_load( &solid_sprite_assets() ).expect( "assets_load failed" );
  let commands = [ RenderCommand::Clear( Clear { color : CLEAR } ), centered_sprite_command( width, height ) ];
  backend.submit( &commands ).expect( "submit failed" );
  match backend.output().expect( "output failed" )
  {
    Output::Presented => {},
    other => panic!( "expected Output::Presented, got {other:?}" ),
  }
}

/// `webgpu` build: async context creation. `RenderConfig`'s width/height are
/// read back from the real canvas — the render pass always covers the full
/// surface regardless (`WebGpuBackend::submit` sets no explicit viewport),
/// but matching them keeps the scene's logical space square with the actual
/// canvas for a predictable center/corner layout.
#[ cfg( all( target_arch = "wasm32", feature = "webgpu" ) ) ]
async fn app_run()
{
  use tilemap_renderer::adapters::webgpu::WebGpuBackend;

  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let config = RenderConfig { width : canvas.width(), height : canvas.height(), ..Default::default() };
  let mut backend = WebGpuBackend::new( config, &canvas ).await
  .expect( "WebGpuBackend construction failed — does this browser support WebGPU?" );
  scene_render( &mut backend, config.width as f32, config.height as f32 );
}

/// `webgl` build: synchronous context creation. `WebGlBackend::new` issues
/// an explicit `gl.viewport(0, 0, config.width, config.height)` — unlike the
/// webgpu path, this one MUST match the real drawing buffer size, or the
/// render would be confined to a sub-region of the canvas instead of
/// covering it fully. `drawing_buffer_width`/`height` read that real size
/// directly from the GL context, no separate canvas lookup needed.
#[ cfg( all( target_arch = "wasm32", feature = "webgl" ) ) ]
fn app_run()
{
  use tilemap_renderer::adapters::webgl::WebGlBackend;

  let gl = minwebgl::context::retrieve_or_make().expect( "WebGL2 context retrieval failed" );
  let ( width, height ) = ( gl.drawing_buffer_width() as u32, gl.drawing_buffer_height() as u32 );
  let config = RenderConfig { width, height, ..Default::default() };
  let mut backend = WebGlBackend::new( config, gl ).expect( "WebGlBackend construction failed" );
  scene_render( &mut backend, width as f32, height as f32 );
}

#[ cfg( all( target_arch = "wasm32", feature = "webgpu" ) ) ]
fn main()
{
  wasm_bindgen_futures::spawn_local( app_run() );
}

#[ cfg( all( target_arch = "wasm32", feature = "webgl" ) ) ]
fn main()
{
  app_run();
}

// Stub main for native targets
#[ cfg( not( target_arch = "wasm32" ) ) ]
fn main()
{
  println!( "This tilemap_renderer adapter example only works on WebAssembly targets." );
  println!( "To run it, compile for wasm32-unknown-unknown with one backend feature:" );
  println!( "  cargo build --target wasm32-unknown-unknown --features webgpu" );
  println!( "  cargo build --target wasm32-unknown-unknown --no-default-features --features webgl" );
}
