//! `LayerBehaviour.tint` through every compile pass that emits sprites.
//!
//! `TintBehaviour::Flat` and the compile-time `Masked` rejection are resolved
//! by one shared helper, but each pass calls it from its own emit site; a
//! pass that went back to passing the bare global tint would drop the layer
//! tint silently. Every case below places one object whose single layer is
//! drawn by a different pass, and checks every behaviour on it.

mod common;

use alloc::sync::Arc;
use tilemap_renderer::commands::{ RenderCommand, Sprite };
use tilemap_scene::{ Camera, CompileError, PathResolver, RenderSpec, Renderer, Scene, SceneSnapshot };

extern crate alloc;

/// One object per case: its anchor, the layer's `sprite_source` in RON, and
/// the scene placement (a `SceneSnapshot` field) that instantiates it.
struct PassCase
{
  pass : &'static str,
  anchor : &'static str,
  source : &'static str,
  placement : &'static str,
}

const CASES : &[ PassCase ] =
&[
  PassCase
  {
    pass : "hex instance",
    anchor : "Hex",
    source : r#"Static( ( "t", "0" ) )"#,
    placement : r#"tiles: [ ( pos: ( 0, 0 ), objects: [ "subject" ] ) ]"#,
  },
  PassCase
  {
    pass : "vertex corners",
    anchor : "Hex",
    source : r#"VertexCorners( patterns: [ ( corners: ( "subject", "void", "void" ), sprite_pattern: "0" ) ], asset: "t" )"#,
    placement : r#"tiles: [ ( pos: ( 0, 0 ), objects: [ "subject" ] ) ]"#,
  },
  PassCase
  {
    pass : "neighbor condition",
    anchor : "Hex",
    source : r#"NeighborCondition( condition: NoNeighbor, sides: [ N ], sprite_pattern: "0", asset: "t" )"#,
    placement : r#"tiles: [ ( pos: ( 0, 0 ), objects: [ "subject" ] ) ]"#,
  },
  PassCase
  {
    pass : "edge",
    anchor : "Edge",
    source : r#"Static( ( "t", "0" ) )"#,
    placement : r#"edges: [ ( at: ( hex: ( 0, 0 ), dir: N ), object: "subject" ) ]"#,
  },
  PassCase
  {
    pass : "free position",
    anchor : "FreePos",
    source : r#"Static( ( "t", "0" ) )"#,
    placement : r#"free_instances: [ ( pos: ( 10.0, 5.0 ), object: "subject" ) ]"#,
  },
  PassCase
  {
    pass : "viewport single",
    anchor : "Viewport",
    source : r#"ViewportTiled( content: Static( ( "t", "0" ) ), tiling: Center, anchor_point: Center )"#,
    placement : r#"viewport_instances: [ ( object: "subject" ) ]"#,
  },
  PassCase
  {
    pass : "viewport repeat",
    anchor : "Viewport",
    source : r#"ViewportTiled( content: Static( ( "t", "0" ) ), tiling: Repeat2D, anchor_point: Center )"#,
    placement : r#"viewport_instances: [ ( object: "subject" ) ]"#,
  },
];

/// `global_tint` is 50% grey at full strength; `half_blue` is pure blue at
/// strength 0.5, i.e. the multiplier `[ 0.5, 0.5, 1, 1 ]`; `glow` declares
/// the unsupported `Add` mode. The object has a
/// `priority` so the vertex pass reads it as the cell's terrain id.
fn spec_for( case : &PassCase, tint : &str ) -> RenderSpec
{
  let ron = format!
  (
    r##"RenderSpec(
      version: "0.2.0",
      assets: [ Asset( id: "t", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 4 ) ) ],
      tints: [
        Tint( id: "grey", color: "#808080", strength: 1.0 ),
        Tint( id: "half_blue", color: "#0000ff", strength: 0.5 ),
        Tint( id: "glow", color: "#ffffff", strength: 1.0, mode: Add ),
      ],
      objects: [
        Object(
          id: "subject",
          anchor: {anchor},
          global_layer: "main",
          priority: Some( 1 ),
          states: {{ "default": [ ( sprite_source: {source}, behaviour: ( tint: {tint} ) ) ] }},
        ),
      ],
      pipeline: (
        hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
        layers: [ ( id: "main" ) ],
        global_tint: Some( ( "grey" ) ),
      ),
    )"##,
    anchor = case.anchor,
    source = case.source,
  );
  RenderSpec::from_ron_str( &ron ).unwrap_or_else( | e | panic!( "{}: spec parses: {e}", case.pass ) )
}

/// Compiles without `RenderSpec::load`, so specs load-time validation would
/// reject (`Masked`, a non-`Multiply` tint) still reach the compile passes
/// under test.
fn render( case : &PassCase, spec : &RenderSpec ) -> Result< Vec< Sprite >, CompileError >
{
  let snapshot = SceneSnapshot::from_ron_str( &format!( "SceneSnapshot( meta: (), bounds: ( min: ( -2, -2 ), max: ( 2, 2 ) ), {} )", case.placement ) )
    .unwrap_or_else( | e | panic!( "{}: scene parses: {e}", case.pass ) );
  let mut renderer = Renderer::new( spec, &PathResolver )?;
  let scene = Scene::from_snapshot( &snapshot, Arc::new( spec.clone() ) ).expect( "scene" );
  let raw = renderer.render( &scene, &Camera::default() )?;
  // World-space passes emit `Sprite`, the viewport pass `ScreenSpaceSprite`.
  Ok( common::commands_to_sprites( raw ).into_iter().filter_map( | c | match c
  {
    RenderCommand::Sprite( s ) | RenderCommand::ScreenSpaceSprite( s ) => Some( s ),
    _ => None,
  }).collect() )
}

/// Asserts every sprite `case` emits under `tint` carries `expected`.
fn tint_assert( case : &PassCase, tint : &str, expected : [ f32; 4 ] )
{
  let sprites = render( case, &spec_for( case, tint ) )
    .unwrap_or_else( | e | panic!( "{}: render: {e}", case.pass ) );
  assert!( !sprites.is_empty(), "{}: the case must emit at least one sprite", case.pass );
  for sprite in &sprites
  {
    let close = sprite.tint.iter().zip( expected ).all( | ( a, e ) | ( a - e ).abs() < 1e-5 );
    assert!( close, "{}: tint {:?}, expected {expected:?}", case.pass, sprite.tint );
  }
}

/// `None` leaves the global tint unchanged on every pass.
#[ test ]
fn no_tint_keeps_global_tint_on_every_pass()
{
  let grey = 128.0 / 255.0;
  for case in CASES
  {
    tint_assert( case, "None", [ grey, grey, grey, 1.0 ] );
  }
}

/// `Flat` multiplies the named tint into the global tint on every pass, with
/// the tint's `strength` blending its colour towards identity:
/// grey `[ 128/255; 3 ]` × half-strength blue `[ 0.5, 0.5, 1, 1 ]`.
#[ test ]
fn flat_tint_composes_with_global_tint_on_every_pass()
{
  let grey = 128.0 / 255.0;
  let expected = [ grey * 0.5, grey * 0.5, grey, 1.0 ];
  for case in CASES
  {
    tint_assert( case, r#"Flat( ( "half_blue" ) )"#, expected );
  }
}

/// `Masked` is not implemented: every pass must fail with
/// `UnsupportedBehaviour` instead of drawing the layer with the global tint.
#[ test ]
fn masked_tint_is_rejected_on_every_pass()
{
  for case in CASES
  {
    let spec = spec_for( case, r#"Masked( mask: Static( ( "t", "1" ) ), tint: Ref( ( "half_blue" ) ) )"# );
    match render( case, &spec )
    {
      Err( CompileError::UnsupportedBehaviour { object, behaviour } ) =>
      {
        assert_eq!( object, "subject", "{}", case.pass );
        assert_eq!( behaviour, "Masked tint", "{}", case.pass );
      }
      other => panic!( "{}: expected UnsupportedBehaviour, got {other:?}", case.pass ),
    }
  }
}

/// A `Flat` tint whose `mode` is not `Multiply` cannot be folded into the
/// multiplicative sprite tint: every pass must fail with `UnsupportedTintMode`
/// instead of drawing it as a plain multiply.
#[ test ]
fn non_multiply_flat_tint_is_rejected_on_every_pass()
{
  for case in CASES
  {
    match render( case, &spec_for( case, r#"Flat( ( "glow" ) )"# ) )
    {
      Err( CompileError::UnsupportedTintMode { tint, .. } ) => assert_eq!( tint, "glow", "{}", case.pass ),
      other => panic!( "{}: expected UnsupportedTintMode, got {other:?}", case.pass ),
    }
  }
}

/// A `Flat` tint naming no declared tint reports the object whose layer
/// referenced it.
#[ test ]
fn unresolved_flat_tint_names_its_object()
{
  let case = &CASES[ 0 ];
  match render( case, &spec_for( case, r#"Flat( ( "ghost" ) )"# ) )
  {
    Err( CompileError::UnresolvedRef { kind : "tint", id, context } ) =>
    {
      assert_eq!( id, "ghost" );
      assert_eq!( context, r#"object "subject" layer tint"# );
    }
    other => panic!( "expected UnresolvedRef for tint 'ghost', got {other:?}" ),
  }
}
