//! Compiles a simulated [`crate::Frame`] into `tilemap_renderer::commands::RenderCommand`s —
//! the "compiling a script's per-frame output into `RenderCommand`s" glue
//! `docs/adr/003_d2_stack_hal_adoption.md` Decision #4 names. Example-local:
//! exactly one consumer exists, so this stays a plain function here rather
//! than a shared crate.

use crate::Frame;
use tilemap_renderer::
{
  assets::{ Assets, DataType, GeometryAsset, Source },
  commands::{ Mesh, RenderCommand },
  types::{ asset, BlendMode, FillRef, ResourceId, Topology, Transform },
};

/// Geometry id for the ball's unit quad.
pub const BALL_GEOMETRY : ResourceId< asset::Geometry > = ResourceId::new( 0 );
/// Geometry id for a paddle's unit quad.
pub const PADDLE_GEOMETRY : ResourceId< asset::Geometry > = ResourceId::new( 1 );

const BALL_HALF_SIZE : f32 = 10.0;
const PADDLE_HALF_WIDTH : f32 = 5.0;
const PADDLE_HALF_HEIGHT : f32 = 30.0;
const PADDLE_LEFT_X : f32 = -380.0;
const PADDLE_RIGHT_X : f32 = 380.0;

/// Flattens a centered `[-hw,-hh] .. [hw,hh]` quad (2 triangles, 6 vertices,
/// no index buffer) into the little-endian byte source `GeometryAsset::positions` expects.
fn quad_bytes( half_width : f32, half_height : f32 ) -> Vec< u8 >
{
  let verts : [ f32; 12 ] =
  [
    -half_width, -half_height,
     half_width, -half_height,
     half_width,  half_height,
    -half_width, -half_height,
     half_width,  half_height,
    -half_width,  half_height,
  ];
  verts.iter().flat_map( | v | v.to_le_bytes() ).collect()
}

/// Builds the static geometry assets `frame_to_commands`' output references
/// ([`BALL_GEOMETRY`], [`PADDLE_GEOMETRY`]). Must be loaded via
/// `Backend::assets_load` before submitting any compiled commands.
#[ must_use ]
pub fn render_assets() -> Assets
{
  Assets
  {
    fonts : Vec::new(),
    images : Vec::new(),
    sprites : Vec::new(),
    geometries : vec!
    [
      GeometryAsset
      {
        id : BALL_GEOMETRY,
        positions : Source::Bytes( quad_bytes( BALL_HALF_SIZE, BALL_HALF_SIZE ) ),
        uvs : None,
        indices : None,
        data_type : DataType::F32,
      },
      GeometryAsset
      {
        id : PADDLE_GEOMETRY,
        positions : Source::Bytes( quad_bytes( PADDLE_HALF_WIDTH, PADDLE_HALF_HEIGHT ) ),
        uvs : None,
        indices : None,
        data_type : DataType::F32,
      },
    ],
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

fn mesh_command( geometry : ResourceId< asset::Geometry >, position : [ f32; 2 ], color : [ f32; 4 ] ) -> RenderCommand
{
  RenderCommand::Mesh( Mesh
  {
    transform : Transform { position, ..Default::default() },
    geometry,
    fill : FillRef::Solid( color ),
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::Normal,
    clip : None,
  } )
}

/// Compiles one simulated `Frame` into the `RenderCommand`s that draw it:
/// the ball plus both paddles, in that order. Pure function — no I/O, no
/// shared state, output determined entirely by `frame`'s own fields.
#[ must_use ]
pub fn frame_to_commands( frame : &Frame ) -> Vec< RenderCommand >
{
  vec!
  [
    mesh_command( BALL_GEOMETRY, [ frame.ball.x(), frame.ball.y() ], [ 1.0, 1.0, 1.0, 1.0 ] ),
    mesh_command( PADDLE_GEOMETRY, [ PADDLE_LEFT_X, frame.paddle_left_y as f32 ], [ 0.2, 0.8, 1.0, 1.0 ] ),
    mesh_command( PADDLE_GEOMETRY, [ PADDLE_RIGHT_X, frame.paddle_right_y as f32 ], [ 1.0, 0.5, 0.2, 1.0 ] ),
  ]
}
