//! `WebGlBackend` draws that reference an image whose asynchronous decode has
//! not landed yet, against a live WebGL2 context with pixel read-back.
//!
//! wasm32-only for the same reason as `webgl_context_loss_test.rs`: the
//! subject compiles real shaders against a live `WebGl2RenderingContext`,
//! which only exists under the workspace's headless-browser wasm32 runner.
//!
//! An `ImageSource::Encoded` image is decoded by the browser in a later task,
//! so a draw submitted in the same task as `assets_load` always sees its
//! texture still 0×0 with no level-0 image. Sampling such a texture reads
//! opaque black, so every draw path that samples it must wait for the decode.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::webgl::{ f32_bytes, gl_init, pixel_read, sleep };
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, DataType, GeometryAsset, ImageAsset, ImageSource, Source };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::{ AddMeshInstance, BindBatch, Clear, CreateMeshBatch, DrawBatch, Mesh, MeshBatchParams, RenderCommand, UnbindBatch };
use tilemap_renderer::types::{ BlendMode, FillRef, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Topology, Transform, WrapMode };
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Opaque mid grey every case clears to; a skipped draw leaves it in place.
///
/// 0.5 · 255 = 127.5 is a rounding tie, and the float-to-normalized conversion
/// WebGL2 inherits from OpenGL ES 3.0 lets an implementation store either
/// neighbour, so the cases compare against the clear as read back
/// ( [`clear_pixel`] ) rather than against a fixed 127 or 128.
const BACKGROUND : [ f32; 4 ] = [ 0.5, 0.5, 0.5, 1.0 ];

/// A 1×1 opaque white RGBA PNG.
fn white_png() -> Vec< u8 >
{
  let mut bytes = Vec::new();
  let mut encoder = png::Encoder::new( &mut bytes, 1, 1 );
  encoder.set_color( png::ColorType::Rgba );
  encoder.set_depth( png::BitDepth::Eight );
  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data( &[ 255; 4 ] ).unwrap();
  writer.finish().unwrap();
  bytes
}

/// One encoded white image ( id 0 ) and one unit quad geometry ( id 0 ).
fn encoded_image_assets() -> Assets
{
  let unit_quad = [ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0 ];
  Assets
  {
    fonts : Vec::new(),
    images : vec!
    [
      ImageAsset
      {
        id : ResourceId::new( 0 ),
        source : ImageSource::Encoded( white_png() ),
        filter : SamplerFilter::Nearest,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
        premultiplied : false,
      }
    ],
    sprites : Vec::new(),
    geometries : vec!
    [
      GeometryAsset
      {
        id : ResourceId::new( 0 ),
        positions : Source::Bytes( f32_bytes( &unit_quad ) ),
        uvs : Some( Source::Bytes( f32_bytes( &unit_quad ) ) ),
        indices : None,
        data_type : DataType::U16,
      }
    ],
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

/// Clears to [`BACKGROUND`] alone and returns pixel `( 0, 0 )`: the bytes this
/// implementation stores for the clear, which a skipped draw leaves in place.
fn clear_pixel( backend : &mut WebGlBackend, gl : &gl::GL ) -> [ u8; 4 ]
{
  backend.submit( &[ RenderCommand::Clear( Clear { color : BACKGROUND } ) ] ).unwrap();
  pixel_read( gl )
}

/// Loads [`encoded_image_assets`], then in the same task clears to
/// [`BACKGROUND`], submits the commands `draw` builds and returns pixel
/// `( 0, 0 )` together with the clear alone as read back. `draw` receives the
/// transform that stretches the unit quad over the whole viewport.
fn pixel_before_decode( draw : impl FnOnce( Transform ) -> Vec< RenderCommand > ) -> ( [ u8; 4 ], [ u8; 4 ] )
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let full_view = Transform { scale : [ config.width as f32, config.height as f32 ], ..Transform::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( &encoded_image_assets() ).unwrap();
  let background = clear_pixel( &mut backend, &gl );

  let mut commands = vec![ RenderCommand::Clear( Clear { color : BACKGROUND } ) ];
  commands.extend( draw( full_view ) );
  backend.submit( &commands ).unwrap();

  ( pixel_read( &gl ), background )
}

/// A textured mesh whose image is still decoding is skipped, like a sprite on
/// a pending sheet, instead of sampling the empty texture as opaque black.
///
/// ## Root Cause
/// `Path` / `Encoded` images are registered as a 0×0 texture with no level-0
/// image until the decode lands, and only the sprite paths skipped such a
/// texture; both mesh paths bound it and sampled opaque black.
///
/// ## Why Not Caught
/// No test drew a mesh on an image that loads asynchronously.
///
/// ## Fix Applied
/// `cmd_mesh` and `MeshRenderer::batch_draw` return early on a 0×0 texture.
///
/// ## Prevention
/// Draws the mesh in the same task as `assets_load` and requires the clear.
///
/// ## Pitfall
/// Every draw path that binds an image texture has to check that it has pixels.
// test_kind: bug_reproducer(BUG-537)
#[ wasm_bindgen_test ]
fn textured_mesh_waits_for_its_image()
{
  let ( pixel, background ) = pixel_before_decode( | transform | vec!
  [
    RenderCommand::Mesh( Mesh
    {
      transform,
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0; 4 ] ),
      texture : Some( ResourceId::new( 0 ) ),
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    })
  ]);

  assert_eq!( pixel, background, "a pending image must not draw as black" );
}

/// The mesh batch path skips a pending texture the same way.
///
/// ## Root Cause
/// `Path` / `Encoded` images are registered as a 0×0 texture with no level-0
/// image until the decode lands, and only the sprite paths skipped such a
/// texture; both mesh paths bound it and sampled opaque black.
///
/// ## Why Not Caught
/// No test drew a mesh on an image that loads asynchronously.
///
/// ## Fix Applied
/// `cmd_mesh` and `MeshRenderer::batch_draw` return early on a 0×0 texture.
///
/// ## Prevention
/// Draws the mesh batch in the same task as `assets_load` and requires the
/// clear.
///
/// ## Pitfall
/// Every draw path that binds an image texture has to check that it has pixels.
// test_kind: bug_reproducer(BUG-537)
#[ wasm_bindgen_test ]
fn textured_mesh_batch_waits_for_its_image()
{
  let batch = ResourceId::new( 0 );
  let params = MeshBatchParams
  {
    transform : Transform::default(),
    geometry : ResourceId::new( 0 ),
    fill : FillRef::Solid( [ 1.0; 4 ] ),
    texture : Some( ResourceId::new( 0 ) ),
    topology : Topology::TriangleList,
    blend : BlendMode::Normal,
    clip : None,
  };
  let ( pixel, background ) = pixel_before_decode( | transform | vec!
  [
    RenderCommand::CreateMeshBatch( CreateMeshBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddMeshInstance( AddMeshInstance { transform, tint : [ 1.0; 4 ] } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ]);

  assert_eq!( pixel, background, "a pending image must not draw as black" );
}

/// The skip only lasts until the decode lands: the same textured mesh, drawn
/// again once the browser has decoded the image, shows the white texel.
///
/// ## Root Cause
/// `Path` / `Encoded` images are registered as a 0×0 texture with no level-0
/// image until the decode lands, and only the sprite paths skipped such a
/// texture; both mesh paths bound it and sampled opaque black.
///
/// ## Why Not Caught
/// No test drew a mesh on an image that loads asynchronously.
///
/// ## Fix Applied
/// `cmd_mesh` and `MeshRenderer::batch_draw` return early on a 0×0 texture.
///
/// ## Prevention
/// Waits for the decode and requires the white texel. Before the fix the
/// first frame already drew black, so the wait ended there and the check failed.
///
/// ## Pitfall
/// Every draw path that binds an image texture has to check that it has pixels.
// test_kind: bug_reproducer(BUG-537)
#[ wasm_bindgen_test ]
async fn textured_mesh_draws_once_its_image_decodes()
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let transform = Transform { scale : [ config.width as f32, config.height as f32 ], ..Transform::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( &encoded_image_assets() ).unwrap();
  // Read in the same task as `assets_load`, before the decode can land.
  let background = clear_pixel( &mut backend, &gl );
  let commands =
  [
    RenderCommand::Clear( Clear { color : BACKGROUND } ),
    RenderCommand::Mesh( Mesh
    {
      transform,
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0; 4 ] ),
      texture : Some( ResourceId::new( 0 ) ),
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ];

  let mut pixel = background;
  for _ in 0..200
  {
    sleep( 10 ).await;
    backend.submit( &commands ).unwrap();
    pixel = pixel_read( &gl );
    if pixel != background { break; }
  }

  assert_eq!( pixel, [ 255; 4 ], "the decoded image must draw" );
}
