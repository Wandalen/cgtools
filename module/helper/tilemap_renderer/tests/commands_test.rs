//! Commands tests

use tilemap_renderer::types::*;
use tilemap_renderer::commands::*;

/// All command types must be Copy — this is a core design invariant.
/// If any type loses Copy, this test fails at compile time.
fn assert_copy< T : Copy >() {}

/// Verifies at compile time that every command struct and the `RenderCommand`
/// enum implement `Copy`. Losing `Copy` on any command type breaks the
/// zero-allocation command-stream contract.
#[ test ]
fn all_commands_are_copy()
{
  assert_copy::< Clear >();
  assert_copy::< BeginPath >();
  assert_copy::< MoveTo >();
  assert_copy::< LineTo >();
  assert_copy::< QuadTo >();
  assert_copy::< CubicTo >();
  assert_copy::< ArcTo >();
  assert_copy::< ClosePath >();
  assert_copy::< EndPath >();
  assert_copy::< BeginText >();
  assert_copy::< Char >();
  assert_copy::< EndText >();
  assert_copy::< Mesh >();
  assert_copy::< Sprite >();
  assert_copy::< CreateSpriteBatch >();
  assert_copy::< CreateMeshBatch >();
  assert_copy::< BindBatch >();
  assert_copy::< AddSpriteInstance >();
  assert_copy::< AddMeshInstance >();
  assert_copy::< SetSpriteInstance >();
  assert_copy::< SetMeshInstance >();
  assert_copy::< RemoveInstance >();
  assert_copy::< SetSpriteBatchParams >();
  assert_copy::< SetMeshBatchParams >();
  assert_copy::< UnbindBatch >();
  assert_copy::< DrawBatch >();
  assert_copy::< DeleteBatch >();
  assert_copy::< BeginGroup >();
  assert_copy::< EndGroup >();
  assert_copy::< Effect >();
  assert_copy::< RenderCommand >();
}

/// Verifies that `RenderCommand` fits within 256 bytes. The enum must remain
/// cache-friendly; unexpected growth here signals a layout regression.
#[ test ]
fn render_command_size_reasonable()
{
  // RenderCommand is an enum — should fit in a cache-friendly size.
  // If this grows unexpectedly large, investigate.
  let size = core::mem::size_of::< RenderCommand >();
  assert!( size <= 256, "RenderCommand is {size} bytes, expected <= 256" );
}

/// Verifies that a representative command stream can be built as a `Vec`
/// covering path, group, and clear commands without type errors or panics.
#[ test ]
fn command_stream_construction()
{
  // Verify a typical command stream can be built as a Vec
  let cmds : Vec< RenderCommand > = vec![
    RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ),
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      stroke_color : [ 0.0, 0.0, 0.0, 1.0 ],
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
    RenderCommand::LineTo( LineTo( 100.0, 100.0 ) ),
    RenderCommand::ClosePath( ClosePath ),
    RenderCommand::EndPath( EndPath ),
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform::default(),
      clip : None,
      effect : Some( Effect::Opacity( 0.5 ) ),
    }),
    RenderCommand::EndGroup( EndGroup ),
  ];
  assert_eq!( cmds.len(), 8 );
}

/// Verifies that `SpriteBatchParams` and `MeshBatchParams` can be
/// constructed with default field values and that resource ids round-trip.
#[ test ]
fn batch_params_defaults()
{
  let sp = SpriteBatchParams
  {
    transform : Transform::default(),
    sheet : ResourceId::new( 0 ),
    blend : BlendMode::Normal,
    clip : None,
  };
  assert_eq!( sp.sheet.inner(), 0 );

  let mp = MeshBatchParams
  {
    transform : Transform::default(),
    geometry : ResourceId::new( 1 ),
    fill : FillRef::None,
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::Normal,
    clip : None,
  };
  assert_eq!( mp.geometry.inner(), 1 );
}
