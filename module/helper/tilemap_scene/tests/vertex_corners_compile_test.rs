//! `SpriteSource::VertexCorners` through the compile pipeline: dual-mesh
//! pattern matching and wildcards, `orient_to_grid` frame selection,
//! `corner_source` channels and `offset`.

#![ expect( clippy::float_cmp, reason = "assertions check exact pass-through of constant rotations; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]

use rustc_hash::FxHashMap as HashMap;

mod common;
use common::compile::{ at_time_compile, atlas_with_frames, minimal_scene_3x3, minimal_spec, sprite_commands, try_compile };

use tilemap_renderer::commands::RenderCommand;
use tilemap_scene::
{
  Anchor,
  Asset,
  Camera,
  LayerBehaviour,
  MipmapMode,
  Object,
  ObjectLayer,
  PathResolver,
  PipelineLayer,
  RenderSpec,
  SamplerFilter,
  SceneSnapshot,
  SortMode,
  SortYSource,
  SpriteRef,
  SpriteSource,
  Tile,
  TilingStrategy,
  TriBlendPattern,
  WrapMode,
  assets_compile,
};

#[ test ]
#[ expect( clippy::too_many_lines, reason = "linear fixture-build, compile, assert scenario; splitting would scatter the scenario steps across helpers" ) ]
fn vertex_corners_three_way_blend()
{
  // Three tiles surrounding a vertex: grass at (0,0), sand at (1,-1), water at (0,-1).
  // These three hexes share exactly one dual-mesh triangle (by construction).
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "blends".into(),
      path : "blends.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "tri_gsw_0", ( 0, 0 ) ),
          ( "tri_gsw_1", ( 1, 0 ) ),
          ( "tri_gsw_2", ( 2, 0 ) ),
        ],
      ),
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
    }
  );

  // Terrains grass/sand/water.
  for ( id, prio ) in [ ( "sand", 8 ), ( "water", 5 ) ]
  {
    spec.objects.push( Object
    {
      id : id.into(),
      anchor : Anchor::Hex,
      global_layer : "terrain".into(),
      priority : Some( prio ),
      sort_y_source : SortYSource::default(),
      pivot : ( 0.5, 0.5 ),
      default_state : "default".into(),
      states :
      {
        let mut m = HashMap::default();
        m.insert
        (
          "default".into(),
          vec!
          [
            ObjectLayer
            {
              id : None,
              sprite_source : SpriteSource::Static( SpriteRef { asset : "terrain".into(), frame : "0".into() } ),
              behaviour : LayerBehaviour::default(),
              z_in_object : 0,
              pipeline_layer : None,
            },
          ],
        );
        m
      },
    });
  }

  // VertexCorners object — its own default animation has a single layer
  // with a pattern that matches the gss/sand/water triple.
  spec.objects.push( Object
  {
    id : "blend".into(),
    anchor : Anchor::Hex,   // anchor type of the owning object doesn't matter for VertexCorners pass
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern
                {
                  corners : ( "grass".into(), "sand".into(), "water".into() ),
                  sprite_pattern : "tri_gsw_{rot}".into(),
                  priority : 10,
                  animation : None,
                },
              ],
              asset : "blends".into(),
              orient_to_grid : false,
              corner_source : None,
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  // Instantiate the blend object once on any tile so the VertexCorners pass
  // finds it during bucket emission. (Object presence is what matters, not
  // the tile — vertex sprites are global per-bucket.)
  let scene = SceneSnapshot
  {
    tiles : vec!
    [
      Tile { pos : ( 0,  0 ), objects : vec![ "grass".into(), "blend".into() ] },
      Tile { pos : ( 1, -1 ), objects : vec![ "sand".into() ] },
      Tile { pos : ( 0, -1 ), objects : vec![ "water".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  let sprite_ids : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  // The triangle surrounding the shared vertex should have produced a
  // tri_gsw_<rot> sprite for some rotation in 0..3.
  let any_rot_emitted = ( 0..3 ).any( | r |
  {
    let id = compiled.ids.sprite( "blends", &format!( "tri_gsw_{r}" ) );
    id.is_some() && sprite_ids.contains( &id.unwrap() )
  });
  assert!( any_rot_emitted, "expected any rotation of tri_gsw to emit; sprite_ids = {sprite_ids:?}" );
}

#[ test ]
fn vertex_corners_wildcard_edge_fade()
{
  // An isolated tile of grass — every dual triangle has 2 void corners.
  // A wildcard pattern ("*", "*", "void") should cover each.
  let mut spec = minimal_spec();
  spec.assets.push
  (
    Asset
    {
      id : "fades".into(),
      path : "fades.png".into(),
      kind : atlas_with_frames
      (
        8,
        &[
          ( "edge_fade_0", ( 0, 0 ) ),
          ( "edge_fade_1", ( 1, 0 ) ),
          ( "edge_fade_2", ( 2, 0 ) ),
        ],
      ),
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
    }
  );
  spec.objects.push( Object
  {
    id : "fade".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : None,
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern
                {
                  corners : ( "*".into(), "*".into(), "void".into() ),
                  sprite_pattern : "edge_fade_{rot}".into(),
                  priority : 0,
                  animation : None,
                },
              ],
              asset : "fades".into(),
              orient_to_grid : false,
              corner_source : None,
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });

  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "grass".into(), "fade".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  let emitted : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let tilemap_renderer::commands::RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  // Six triangles around the isolated hex, all with 2 void corners → all
  // should match the wildcard fade pattern. We just assert at least one
  // edge_fade_* sprite emitted.
  let any_fade = ( 0..3 ).any( | r |
  {
    let id = compiled.ids.sprite( "fades", &format!( "edge_fade_{r}" ) );
    id.is_some() && emitted.contains( &id.unwrap() )
  });
  assert!( any_fade, "expected wildcard fade to match at least one triangle; emitted = {emitted:?}" );
}

/// Build the single-terrain dual-grid spec used by the orient_to_grid tests:
/// one `hexagon` object that both marks terrain (`priority`) and carries the
/// `VertexCorners` layer (`orient_to_grid: true`) routing the canonical
/// full/edge/corner frames.
fn dual_orient_spec() -> RenderSpec
{
  let mut spec = minimal_spec();
  spec.pipeline.hex.grid_stride = ( 96, 111 );
  spec.objects.clear();
  // Pre-baked oriented frames: full has 2 (▲/▽), edge and corner have 6 each
  // (the void / lone-hex points in any of 6 directions). Positions are
  // irrelevant to these tests — only that each frame exists in the atlas.
  let frames : Vec< ( &str, ( u32, u32 ) ) > = vec!
  [
    ( "dual_full_0", ( 0, 0 ) ), ( "dual_full_1", ( 1, 0 ) ),
    ( "dual_edge_0", ( 2, 0 ) ), ( "dual_edge_1", ( 3, 0 ) ),
    ( "dual_edge_2", ( 0, 1 ) ), ( "dual_edge_3", ( 1, 1 ) ),
    ( "dual_edge_4", ( 2, 1 ) ), ( "dual_edge_5", ( 3, 1 ) ),
    ( "dual_corner_0", ( 0, 2 ) ), ( "dual_corner_1", ( 1, 2 ) ),
    ( "dual_corner_2", ( 2, 2 ) ), ( "dual_corner_3", ( 3, 2 ) ),
    ( "dual_corner_4", ( 0, 3 ) ), ( "dual_corner_5", ( 1, 3 ) ),
  ];
  spec.assets.push
  (
    Asset
    {
      id : "dual".into(),
      path : "dual.png".into(),
      kind : atlas_with_frames( 4, &frames ),
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
    }
  );
  spec.objects.push( Object
  {
    id : "hexagon".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 10 ),
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern { corners : ( "hexagon".into(), "hexagon".into(), "hexagon".into() ), sprite_pattern : "dual_full_{rot}".into(),   priority : 30, animation : None },
                TriBlendPattern { corners : ( "hexagon".into(), "hexagon".into(), "void".into() ),    sprite_pattern : "dual_edge_{rot}".into(),   priority : 20, animation : None },
                TriBlendPattern { corners : ( "hexagon".into(), "void".into(), "void".into() ),       sprite_pattern : "dual_corner_{rot}".into(), priority : 10, animation : None },
              ],
              asset : "dual".into(),
              orient_to_grid : true,
              corner_source : None,
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  spec
}

/// orient_to_grid (Path B): a lone hex in void emits exactly the six
/// surrounding `dual_corner` triangles, each picking a DISTINCT pre-baked
/// orientation frame (`dual_corner_0..5` — all six present). No runtime sprite
/// rotation: `transform.rotation` stays 0 and the frame index carries the
/// orientation. This pins the discrete frame-selection geometry; the absolute
/// base angle (texture v-flip) is calibrated visually in-browser.
#[ test ]
fn vertex_corners_orient_to_grid_single_hex_six_orientations()
{
  let spec = dual_orient_spec();
  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  // Map each of the six baked corner frame ids back to its orientation index.
  let corner_ids : Vec< _ > = ( 0..6 )
    .map( | o | compiled.ids.sprite( "dual", &format!( "dual_corner_{o}" ) ).expect( "corner frame allocated" ) )
    .collect();

  let mut seen = std::collections::HashSet::new();
  for c in &cmds
  {
    if let RenderCommand::Sprite( s ) = c
      && let Some( o ) = corner_ids.iter().position( | id | *id == s.sprite )
    {
      seen.insert( o );
      assert_eq!( s.transform.rotation, 0.0, "orient mode must not rotate sprites at runtime" );
    }
  }
  // Six triangles around the lone hex, each a distinct 60°-orientation frame.
  assert_eq!( seen.len(), 6, "lone hex must emit all six distinct corner orientations; got {seen:?}" );
}

/// orient_to_grid (Path B), pointy-top tiling: the same lone-hex invariant as
/// `_single_hex_six_orientations`, but with `TilingStrategy::HexPointyTop`.
/// `enumerate_triangles` dispatches to a DIFFERENT coordinate pair for
/// pointy-top (`Pointy`/`FlatTopped`) than for flat-top (`Flat`/`FlatSided`),
/// so the pointy-top corner geometry and `dual_orientation_index` bearings are
/// an independent code path — a regression there would not be caught by the
/// flat-top tests. A lone hex must still emit all six distinct pre-baked corner
/// orientations with no runtime rotation.
#[ test ]
fn vertex_corners_orient_to_grid_pointy_top_six_orientations()
{
  let mut spec = dual_orient_spec();
  spec.pipeline.hex.tiling = TilingStrategy::HexPointyTop;
  // dual_orient_spec's (96,111) stride is a *regular* flat-top hex
  // (cw = ch·√3/2). Pointy-top regularity needs the inverse ratio
  // (ch = cw·√3/2), so swap to (111,96); otherwise the hexes are stretched and
  // the six 60°-spaced bearings collapse onto fewer discrete orientations.
  spec.pipeline.hex.grid_stride = ( 111, 96 );
  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  let corner_ids : Vec< _ > = ( 0..6 )
    .map( | o | compiled.ids.sprite( "dual", &format!( "dual_corner_{o}" ) ).expect( "corner frame allocated" ) )
    .collect();

  let mut seen = std::collections::HashSet::new();
  for c in &cmds
  {
    if let RenderCommand::Sprite( s ) = c
      && let Some( o ) = corner_ids.iter().position( | id | *id == s.sprite )
    {
      seen.insert( o );
      assert_eq!( s.transform.rotation, 0.0, "orient mode must not rotate sprites at runtime" );
    }
  }
  assert_eq!( seen.len(), 6, "pointy-top lone hex must emit all six distinct corner orientations; got {seen:?}" );
}

/// orient_to_grid (Path B): a solid patch yields full interior triangles, and
/// the up-pointing (▲) and down-pointing (▽) duals select DIFFERENT pre-baked
/// frames (`dual_full_0` vs `dual_full_1` — a solid triangle is 3-fold
/// symmetric, so only the ▲/▽ parity distinguishes them). The legacy 120°-only
/// path collapsed both to one frame.
#[ test ]
fn vertex_corners_orient_to_grid_up_down_distinct()
{
  // 7-hex flower: centre + 6 flat-top neighbours. The six triangles around the
  // centre all have three `hexagon` corners → `dual_full`.
  let neigh = [ ( 0, -1 ), ( 1, -1 ), ( 1, 0 ), ( 0, 1 ), ( -1, 1 ), ( -1, 0 ) ];
  let mut tiles = vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ];
  for ( q, r ) in neigh { tiles.push( Tile { pos : ( q, r ), objects : vec![ "hexagon".into() ] } ); }

  let spec = dual_orient_spec();
  let scene = SceneSnapshot { tiles, ..minimal_scene_3x3() };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  let full_0 = compiled.ids.sprite( "dual", "dual_full_0" ).expect( "dual_full_0 allocated" );
  let full_1 = compiled.ids.sprite( "dual", "dual_full_1" ).expect( "dual_full_1 allocated" );

  let ( mut n0, mut n1 ) = ( 0_u32, 0_u32 );
  for c in &cmds
  {
    if let RenderCommand::Sprite( s ) = c
    {
      if s.sprite == full_0 { n0 += 1; assert_eq!( s.transform.rotation, 0.0 ); }
      if s.sprite == full_1 { n1 += 1; assert_eq!( s.transform.rotation, 0.0 ); }
    }
  }

  assert!( n0 + n1 >= 6, "flower interior must emit ≥6 full triangles; got {}", n0 + n1 );
  // Both parities must appear — the ▲/▽ distinction the legacy path lost.
  assert!( n0 > 0 && n1 > 0, "both ▲ and ▽ full frames must appear; got full_0={n0}, full_1={n1}" );
}

/// orient_to_grid edge tiles (two present corners): the six rim edge triangles
/// of a 7-hex flower each point their absent corner a different way, so they
/// must hit six distinct `dual_edge_*` frames. One is pinned to its frame: the
/// easternmost rim triangle (between the NE and SE neighbours) has its absent
/// corner due east of its centroid, bearing 0°, so with the 300° edge base it
/// takes frame `round( ( 300° − 0° ) / 60° ) = 5`. A wrong base angle or
/// distinguishing corner in the edge branch moves that index.
#[ test ]
fn vertex_corners_orient_to_grid_flower_edges()
{
  let flower = [ ( 0, 0 ), ( 0, -1 ), ( 1, -1 ), ( 1, 0 ), ( 0, 1 ), ( -1, 1 ), ( -1, 0 ) ];
  let tiles = flower.iter().map( | &pos | Tile { pos, objects : vec![ "hexagon".into() ] } ).collect();
  let spec = dual_orient_spec();
  let scene = SceneSnapshot { tiles, ..minimal_scene_3x3() };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );

  let edge_ids : Vec< _ > = ( 0..6 )
    .map( | o | compiled.ids.sprite( "dual", &format!( "dual_edge_{o}" ) ).expect( "edge frame allocated" ) )
    .collect();
  // `( frame index, x )` of every edge sprite.
  let edges : Vec< ( usize, f32 ) > = sprite_commands( &cmds ).into_iter()
    .filter_map( | s | edge_ids.iter().position( | id | *id == s.sprite ).map( | o | ( o, s.transform.position[ 0 ] ) ) )
    .collect();

  assert_eq!( edges.len(), 6, "a flower has six rim edge triangles; got {edges:?}" );
  let distinct : std::collections::HashSet< usize > = edges.iter().map( | ( o, _ ) | *o ).collect();
  assert_eq!( distinct.len(), 6, "rim edges must hit six distinct dual_edge frames; got {edges:?}" );
  let east = edges.iter().max_by( | a, b | a.1.total_cmp( &b.1 ) ).expect( "edges" );
  assert_eq!( east.0, 5, "easternmost rim edge (absent corner at 0°) must take dual_edge_5; got {edges:?}" );
}

/// Without a solid `( X, X, X )` pattern an orient layer has no self id, so
/// every non-void corner counts as present. The frame pick must not depend on
/// how the object's id sorts against `"void"`: `"water"` sorts after it, so a
/// sort-based reading would take `( water, water, void )` for a corner. Over a
/// 7-hex flower, the edge and corner frames of the layer without a solid
/// pattern must match `dual_orient_spec`'s (self id `hexagon`) for an id
/// sorting either side of `"void"`.
#[ test ]
fn vertex_corners_orient_without_solid_pattern_ignores_id_order()
{
  let neigh = [ ( 0, 0 ), ( 0, -1 ), ( 1, -1 ), ( 1, 0 ), ( 0, 1 ), ( -1, 1 ), ( -1, 0 ) ];
  // `position → frame name` of every edge / corner sprite `id`'s flower emits.
  let frame_map = | spec : &RenderSpec, id : &str |
  {
    let tiles = neigh.iter().map( | &pos | Tile { pos, objects : vec![ id.into() ] } ).collect();
    let scene = SceneSnapshot { tiles, ..minimal_scene_3x3() };
    let compiled = assets_compile( spec, &PathResolver ).expect( "assets" );
    let names : std::collections::HashMap< _, String > = ( 0..6 )
      .flat_map( | o | [ format!( "dual_edge_{o}" ), format!( "dual_corner_{o}" ) ] )
      .filter_map( | n | compiled.ids.sprite( "dual", &n ).map( | sid | ( sid, n ) ) )
      .collect();
    let cmds = at_time_compile( spec, &scene, &Camera::default(), 0.0 );
    sprite_commands( &cmds ).into_iter()
      .filter_map( | s | names.get( &s.sprite ).map( | n |
        ( format!( "{:.2},{:.2}", s.transform.position[ 0 ], s.transform.position[ 1 ] ), n.clone() ) ) )
      .collect::< std::collections::BTreeMap< _, _ > >()
  };
  // `dual_orient_spec` with its solid pattern dropped and `hexagon` renamed.
  let without_solid = | id : &str |
  {
    let mut spec = dual_orient_spec();
    let object = &mut spec.objects[ 0 ];
    object.id = id.into();
    let layer = &mut object.states.get_mut( "default" ).expect( "default state" )[ 0 ];
    let SpriteSource::VertexCorners { patterns, .. } = &mut layer.sprite_source
    else { panic!( "dual_orient_spec layer 0 must be VertexCorners" ) };
    patterns.retain( | p | p.self_id().is_none() );
    for p in patterns.iter_mut()
    {
      for c in [ &mut p.corners.0, &mut p.corners.1, &mut p.corners.2 ]
      {
        if c == "hexagon" { *c = id.into(); }
      }
    }
    spec
  };

  let with_self_id = frame_map( &dual_orient_spec(), "hexagon" );
  assert!( with_self_id.values().any( | n | n.starts_with( "dual_edge_" ) ), "flower rim must emit edge tiles; got {with_self_id:?}" );
  assert!( with_self_id.values().any( | n | n.starts_with( "dual_corner_" ) ), "flower rim must emit corner tiles; got {with_self_id:?}" );
  for id in [ "hexagon", "water" ]
  {
    assert_eq!( frame_map( &without_solid( id ), id ), with_self_id, "{id}: a layer without a solid pattern must orient like one with it" );
  }
}

/// Regression: a bare `("*","*","*")` wildcard pattern with `orient_to_grid:
/// true` must still compile. The wildcard is excluded from `self_id` detection,
/// so the layer has no self id, counts non-void corners as present and can pick
/// `{rot}` up to 5. Pre-allocation must therefore reserve six frames, not the
/// two it would for a genuine fully-symmetric (all-equal) pattern — otherwise
/// rendering hits `CompileError::UnresolvedRef`.
#[ test ]
fn vertex_corners_orient_to_grid_triple_wildcard_allocates_six()
{
  let mut spec = minimal_spec();
  spec.pipeline.hex.grid_stride = ( 96, 111 );
  spec.objects.clear();
  let frames : Vec< ( &str, ( u32, u32 ) ) > = ( 0..6 )
    .map( | o | ( [ "w_0", "w_1", "w_2", "w_3", "w_4", "w_5" ][ o ], ( o as u32, 0 ) ) )
    .collect();
  spec.assets.push( Asset
  {
    id : "wild".into(),
    path : "wild.png".into(),
    kind : atlas_with_frames( 6, &frames ),
    filter : SamplerFilter::default(),
    mipmap : MipmapMode::default(),
    wrap : WrapMode::default(),
  });
  spec.objects.push( Object
  {
    id : "hexagon".into(),
    anchor : Anchor::Hex,
    global_layer : "terrain".into(),
    priority : Some( 10 ),
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern { corners : ( "*".into(), "*".into(), "*".into() ), sprite_pattern : "w_{rot}".into(), priority : 0, animation : None },
              ],
              asset : "wild".into(),
              orient_to_grid : true,
              corner_source : None,
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });

  // All six `{rot}` frames must be pre-allocated.
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  for o in 0..6
  {
    assert!
    (
      compiled.ids.sprite( "wild", &format!( "w_{o}" ) ).is_some(),
      "wildcard orient pattern must reserve frame w_{o}",
    );
  }

  // A lone hex's surrounding corner triangles orient by their non-void corner and can pick
  // `{rot}` up to 5 — rendering must resolve every frame, not error.
  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  assert!( try_compile( &spec, &scene, &Camera::default() ).is_ok(), "triple-wildcard orient scene must compile without UnresolvedRef" );
}

/// `offset: Some((dx,dy))` must shift every emitted VertexCorners sprite's
/// position by exactly that world delta — and nothing else. Same scene with and
/// without the offset must pick the SAME frames (offset does not touch corner
/// resolution / orient frame selection), only their positions move. With the
/// default camera (zoom 1, no rotation, no Y-flip in `project`) a world offset
/// maps 1:1 to a screen-position delta.
#[ test ]
fn vertex_corners_offset_shifts_sprite_position()
{
  let base_spec = dual_orient_spec();
  let mut off_spec = dual_orient_spec();
  // Apply the offset to the (only) VertexCorners layer.
  let layer = &mut off_spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" )[ 0 ];
  if let SpriteSource::VertexCorners { offset, .. } = &mut layer.sprite_source
  {
    *offset = Some( ( 32.0, -16.0 ) );
  }
  else
  {
    panic!( "dual_orient_spec layer 0 must be VertexCorners" );
  }

  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let base = at_time_compile( &base_spec, &scene, &Camera::default(), 0.0 );
  let off  = at_time_compile( &off_spec,  &scene, &Camera::default(), 0.0 );
  let base = sprite_commands( &base );
  let off  = sprite_commands( &off );

  assert!( !base.is_empty(), "lone hex must emit corner sprites" );
  assert_eq!( base.len(), off.len(), "offset must not change the number of sprites" );
  for ( b, o ) in base.iter().zip( off.iter() )
  {
    assert_eq!( b.sprite, o.sprite, "offset must not change the chosen frame (un-shifted geometry invariant)" );
    let dx = o.transform.position[ 0 ] - b.transform.position[ 0 ];
    let dy = o.transform.position[ 1 ] - b.transform.position[ 1 ];
    assert!( ( dx - 32.0 ).abs() < 1e-4, "x must shift by offset dx=32; got {dx}" );
    assert!( ( dy + 16.0 ).abs() < 1e-4, "y must shift by offset dy=-16; got {dy}" );
  }
}

/// `VertexCorners.offset` moves only the drawn sprite, not its depth-sort key:
/// a sorted bucket orders an offset tile by its un-shifted triangle centroid,
/// like every other pass orders by the anchor rather than the drawn position.
/// One object carries the same dual grid twice in a `YAsc` bucket — layer 0
/// plain, layer 1 shifted far down. Sorting by the centroid keeps each shifted
/// copy directly after its un-shifted twin (stable sort, same key, emission
/// order = `z_in_object`); sorting by the shifted position would instead move
/// every shifted copy ahead of all plain tiles.
#[ test ]
fn vertex_corners_offset_sorts_at_unshifted_centroid()
{
  const DY : f32 = -10_000.0;
  let mut spec = dual_orient_spec();
  spec.pipeline.layers[ 0 ].sort = SortMode::YAsc;
  let stack = spec.objects[ 0 ].states.get_mut( "default" ).expect( "default state" );
  let mut shifted = stack[ 0 ].clone();
  if let SpriteSource::VertexCorners { offset, .. } = &mut shifted.sprite_source
  {
    *offset = Some( ( 0.0, DY ) );
  }
  else
  {
    panic!( "dual_orient_spec layer 0 must be VertexCorners" );
  }
  shifted.z_in_object = 1;
  stack.push( shifted );

  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into() ] } ],
    ..minimal_scene_3x3()
  };
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );
  let sprites = sprite_commands( &cmds );

  assert_eq!( sprites.len(), 12, "lone hex: six triangles, each drawn plain and shifted" );
  for [ plain, copy ] in sprites.as_chunks::< 2 >().0
  {
    assert_eq!( plain.sprite, copy.sprite, "shifted copy must directly follow its own triangle's plain tile" );
    let dy = copy.transform.position[ 1 ] - plain.transform.position[ 1 ];
    assert!( ( dy - DY ).abs() < 1e-3, "second of each pair must be the shifted copy (dy = {DY}); got {dy}" );
  }
}

/// `corner_source`: two independent dual grids in ONE scene. A cell carrying
/// BOTH a terrain object (default channel = terrain id) and a region object
/// (channel = its `global_layer`, "region") must emit tiles from BOTH assets —
/// proving the per-layer corner resolution keeps the channels isolated (the
/// region layer reads "region_0", not "hexagon", on the shared cell).
#[ test ]
fn vertex_corners_corner_source_isolates_channels()
{
  let mut spec = dual_orient_spec(); // hexagon (terrain) + "dual" asset.

  // A second atlas + object whose dual grid reads the "region" channel.
  let region_frames : Vec< ( &str, ( u32, u32 ) ) > = vec!
  [
    ( "r_full_0", ( 0, 0 ) ), ( "r_full_1", ( 1, 0 ) ),
    ( "r_corner_0", ( 0, 2 ) ), ( "r_corner_1", ( 1, 2 ) ), ( "r_corner_2", ( 2, 2 ) ),
    ( "r_corner_3", ( 3, 2 ) ), ( "r_corner_4", ( 0, 3 ) ), ( "r_corner_5", ( 1, 3 ) ),
  ];
  spec.assets.push( Asset
  {
    id : "region".into(),
    path : "region.png".into(),
    kind : atlas_with_frames( 4, &region_frames ),
    filter : SamplerFilter::default(),
    mipmap : MipmapMode::default(),
    wrap : WrapMode::default(),
  });
  spec.objects.push( Object
  {
    id : "region_0".into(),
    anchor : Anchor::Hex,
    global_layer : "region".into(),
    priority : Some( 10 ),
    sort_y_source : SortYSource::default(),
    pivot : ( 0.5, 0.5 ),
    default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert
      (
        "default".into(),
        vec!
        [
          ObjectLayer
          {
            id : None,
            sprite_source : SpriteSource::VertexCorners
            {
              patterns : vec!
              [
                TriBlendPattern { corners : ( "region_0".into(), "region_0".into(), "region_0".into() ), sprite_pattern : "r_full_{rot}".into(),   priority : 30, animation : None },
                TriBlendPattern { corners : ( "region_0".into(), "*".into(), "*".into() ),                sprite_pattern : "r_corner_{rot}".into(), priority : 10, animation : None },
              ],
              asset : "region".into(),
              orient_to_grid : true,
              corner_source : Some( "region".into() ),
              offset : None,
            },
            behaviour : LayerBehaviour::default(),
            z_in_object : 0,
            pipeline_layer : None,
          },
        ],
      );
      m
    },
  });
  spec.pipeline.layers.push( PipelineLayer { id : "region".into(), sort : SortMode::None, tint_mask : None } );

  // One lone cell carrying BOTH objects → terrain channel sees [hexagon,void,
  // void] and region channel sees [region_0,void,void] on the same triangles.
  let scene = SceneSnapshot
  {
    tiles : vec![ Tile { pos : ( 0, 0 ), objects : vec![ "hexagon".into(), "region_0".into() ] } ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );
  let cmds = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );
  let emitted : std::collections::HashSet< _ > = cmds.iter().filter_map( | c |
    if let RenderCommand::Sprite( s ) = c { Some( s.sprite ) } else { None }
  ).collect();

  let any_dual = ( 0..6 ).any( | o |
    compiled.ids.sprite( "dual", &format!( "dual_corner_{o}" ) ).is_some_and( | id | emitted.contains( &id ) ) );
  let any_region = ( 0..6 ).any( | o |
    compiled.ids.sprite( "region", &format!( "r_corner_{o}" ) ).is_some_and( | id | emitted.contains( &id ) ) );
  assert!( any_dual, "terrain channel must emit dual corner tiles; emitted = {emitted:?}" );
  assert!( any_region, "region channel must emit region corner tiles from the SAME cell; emitted = {emitted:?}" );

  // Channel isolation, not just presence. The lone cell carries hexagon
  // (terrain channel) AND region_0 (region channel), each surrounded by void,
  // so each dual grid must emit ONLY its own single-corner family:
  //   - the terrain/dual layer resolves "hexagon" → `dual_corner_*` only
  //     (never a `region`-asset frame),
  //   - the region layer resolves "region_0" → `r_corner_*` only (never a
  //     `dual`-asset frame, and in particular never a `dual_corner_*`).
  // A bug that ignored `corner_source` and read the region layer off the
  // terrain channel would emit no region frame at all (caught above), or — if
  // it leaked the other way — would surface a cross-family frame here.
  //
  // Reverse-map every emitted vertex sprite id back to its frame name via the
  // public id lookup over the known frame families.
  let dual_names : Vec< String > = ( 0..6 ).map( | o | format!( "dual_corner_{o}" ) )
    .chain( ( 0..6 ).map( | o | format!( "dual_edge_{o}" ) ) )
    .chain( ( 0..2 ).map( | o | format!( "dual_full_{o}" ) ) )
    .collect();
  let region_names : Vec< String > = ( 0..6 ).map( | o | format!( "r_corner_{o}" ) )
    .chain( ( 0..2 ).map( | o | format!( "r_full_{o}" ) ) )
    .collect();
  let dual_emitted : Vec< &str > = dual_names.iter()
    .filter( | n | compiled.ids.sprite( "dual", n ).is_some_and( | id | emitted.contains( &id ) ) )
    .map( String::as_str ).collect();
  let region_emitted : Vec< &str > = region_names.iter()
    .filter( | n | compiled.ids.sprite( "region", n ).is_some_and( | id | emitted.contains( &id ) ) )
    .map( String::as_str ).collect();

  assert!( !dual_emitted.is_empty(), "terrain channel must emit dual frames" );
  assert!( !region_emitted.is_empty(), "region channel must emit region frames" );
  // Each channel resolved its OWN single id surrounded by void → corner family
  // only, never the other channel's frames.
  assert!
  (
    dual_emitted.iter().all( | n | n.starts_with( "dual_corner_" ) ),
    "terrain/dual layer leaked a non-corner or region frame: {dual_emitted:?}",
  );
  assert!
  (
    region_emitted.iter().all( | n | n.starts_with( "r_corner_" ) ),
    "region layer leaked a non-corner or dual frame: {region_emitted:?}",
  );
}

/// `corner_source` compile-time fallback (`format/005`): a misspelled layer
/// name matches no object's `global_layer`, so every corner resolves to
/// `VOID_ID` — exactly as for an off-map corner. `validate()` rejects such a
/// spec at load (`validate_rejects_unknown_corner_source_layer`); this test
/// compiles without loading to pin what compilation itself does when handed
/// one: no error, and the dual grid matches none of its region patterns (all
/// require `region_1`), so it emits nothing rather than panicking.
#[ test ]
fn vertex_corners_corner_source_invalid_layer_falls_back_to_void()
{
  let mut spec = region_boundary_spec();

  // A small region_1 patch. With the correct channel its dual grid emits region
  // frames; with a misspelled channel every corner is void and it emits none.
  let scene = SceneSnapshot
  {
    tiles : vec!
    [
      Tile { pos : ( 0, 0 ), objects : vec![ "region_1".into() ] },
      Tile { pos : ( 1, 0 ), objects : vec![ "region_1".into() ] },
      Tile { pos : ( 0, 1 ), objects : vec![ "region_1".into() ] },
    ],
    ..minimal_scene_3x3()
  };
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );

  // Build the set of region-asset sprite ids (all frame families) once; both
  // runs allocate the same frames (allocation is independent of corner_source).
  let region_names : Vec< String > = ( 0..2 ).map( | o | format!( "r_full_{o}" ) )
    .chain( ( 0..6 ).map( | o | format!( "r_edge_{o}" ) ) )
    .chain( ( 0..6 ).map( | o | format!( "r_corner_{o}" ) ) )
    .collect();
  let region_ids : std::collections::HashSet< _ > = region_names.iter()
    .filter_map( | n | compiled.ids.sprite( "region", n ) )
    .collect();
  let count_region = | cmds : &[ RenderCommand ] | cmds.iter()
    .filter( | c | matches!( c, RenderCommand::Sprite( s ) if region_ids.contains( &s.sprite ) ) )
    .count();

  // Baseline: the correct channel emits region frames.
  let cmds_ok = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );
  assert!( count_region( &cmds_ok ) > 0, "correct corner_source must emit region frames" );

  // Misspell the corner_source — names no object's global_layer.
  let region_1 = spec.objects.iter_mut().find( | o | o.id == "region_1" ).expect( "region_1 object" );
  let layer = &mut region_1.states.get_mut( "default" ).expect( "default state" )[ 0 ];
  if let SpriteSource::VertexCorners { corner_source, .. } = &mut layer.sprite_source
  {
    *corner_source = Some( "regionn".into() );   // typo: "regionn" vs "region"
  }
  else
  {
    panic!( "region_1 layer 0 must be VertexCorners" );
  }

  // Silent fallback: compile succeeds and emits no region frames at all.
  let cmds_bad = at_time_compile( &spec, &scene, &Camera::default(), 0.0 );
  assert_eq!
  (
    count_region( &cmds_bad ), 0,
    "misspelled corner_source must silently resolve every corner to void → no region frames",
  );
}

/// Build a two-region scene spec: `region_1` carries an `orient_to_grid` dual
/// grid keyed off the `"region"` channel, `region_0` is a foreign region that
/// only marks that channel (no dual grid of its own). Used by the cross-region
/// boundary regression below.
fn region_boundary_spec() -> RenderSpec
{
  let mut spec = minimal_spec();
  spec.pipeline.hex.grid_stride = ( 96, 111 );

  let region_frames : Vec< ( &str, ( u32, u32 ) ) > = vec!
  [
    ( "r_full_0", ( 0, 0 ) ), ( "r_full_1", ( 1, 0 ) ),
    ( "r_edge_0", ( 0, 1 ) ), ( "r_edge_1", ( 1, 1 ) ), ( "r_edge_2", ( 2, 1 ) ),
    ( "r_edge_3", ( 3, 1 ) ), ( "r_edge_4", ( 0, 2 ) ), ( "r_edge_5", ( 1, 2 ) ),
    ( "r_corner_0", ( 2, 2 ) ), ( "r_corner_1", ( 3, 2 ) ), ( "r_corner_2", ( 0, 3 ) ),
    ( "r_corner_3", ( 1, 3 ) ), ( "r_corner_4", ( 2, 3 ) ), ( "r_corner_5", ( 3, 3 ) ),
  ];
  spec.assets.push( Asset
  {
    id : "region".into(), path : "region.png".into(), kind : atlas_with_frames( 4, &region_frames ),
    filter : SamplerFilter::default(), mipmap : MipmapMode::default(), wrap : WrapMode::default(),
  });
  spec.assets.push( Asset
  {
    id : "marker".into(), path : "marker.png".into(), kind : atlas_with_frames( 1, &[ ( "0", ( 0, 0 ) ) ] ),
    filter : SamplerFilter::default(), mipmap : MipmapMode::default(), wrap : WrapMode::default(),
  });

  spec.objects.push( Object
  {
    id : "region_1".into(), anchor : Anchor::Hex, global_layer : "region".into(), priority : Some( 10 ),
    sort_y_source : SortYSource::default(), pivot : ( 0.5, 0.5 ), default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert( "default".into(), vec!
      [
        ObjectLayer
        {
          id : None,
          sprite_source : SpriteSource::VertexCorners
          {
            patterns : vec!
            [
              TriBlendPattern { corners : ( "region_1".into(), "region_1".into(), "region_1".into() ), sprite_pattern : "r_full_{rot}".into(),   priority : 30, animation : None },
              TriBlendPattern { corners : ( "region_1".into(), "region_1".into(), "*".into() ),         sprite_pattern : "r_edge_{rot}".into(),   priority : 20, animation : None },
              TriBlendPattern { corners : ( "region_1".into(), "*".into(), "*".into() ),                sprite_pattern : "r_corner_{rot}".into(), priority : 10, animation : None },
            ],
            asset : "region".into(), orient_to_grid : true, corner_source : Some( "region".into() ), offset : None,
          },
          behaviour : LayerBehaviour::default(), z_in_object : 0, pipeline_layer : None,
        },
      ]);
      m
    },
  });
  spec.objects.push( Object
  {
    id : "region_0".into(), anchor : Anchor::Hex, global_layer : "region".into(), priority : Some( 10 ),
    sort_y_source : SortYSource::default(), pivot : ( 0.5, 0.5 ), default_state : "default".into(),
    states :
    {
      let mut m = HashMap::default();
      m.insert( "default".into(), vec!
      [
        ObjectLayer
        {
          id : None,
          sprite_source : SpriteSource::Static( SpriteRef { asset : "marker".into(), frame : "0".into() } ),
          behaviour : LayerBehaviour::default(), z_in_object : 0, pipeline_layer : None,
        },
      ]);
      m
    },
  });
  spec.pipeline.layers.push( PipelineLayer { id : "region".into(), sort : SortMode::None, tint_mask : None } );
  spec
}

/// Cross-region boundary regression — the scenario that motivated the self_id
/// fix. At a boundary, `region_1`'s edge triangle has corners
/// `(region_1, region_1, region_0)`; because `"region_0" < "region_1"` the old
/// canonical-sort orientation sorted the foreign id first and misread the edge
/// as a corner pointing at the neighbour. The fix counts corners equal to
/// `region_1`'s own id, so a FOREIGN region is treated exactly like void.
///
/// Invariant pinned here: `region_1`'s dual grid orients identically whether its
/// non-self neighbours are `region_0` or empty (void). We compare the full
/// `position → frame` map of `region_1`'s emitted sprites between the two
/// scenes; the geometry is unchanged, so any difference would be a
/// misclassification. (Under the pre-fix canonical-sort path the boundary
/// triangle's `{rot}` differs between the foreign-region and void cases.)
#[ test ]
fn vertex_corners_orient_foreign_region_matches_void()
{
  let spec = region_boundary_spec();
  let compiled = assets_compile( &spec, &PathResolver ).expect( "assets" );

  // region_1's sprites all live in the "region" asset — build id → frame-name
  // over its known frame families via the public id lookup.
  let region_names : Vec< String > = ( 0..2 ).map( | o | format!( "r_full_{o}" ) )
    .chain( ( 0..6 ).map( | o | format!( "r_edge_{o}" ) ) )
    .chain( ( 0..6 ).map( | o | format!( "r_corner_{o}" ) ) )
    .collect();
  let region_ids : std::collections::HashMap< _, String > = region_names.iter()
    .filter_map( | n | compiled.ids.sprite( "region", n ).map( | id | ( id, n.clone() ) ) )
    .collect();

  // Two adjacent region_1 hexes share a dual triangle with the cell at (1,0).
  let region1_cells = [ ( 0, 0 ), ( 1, -1 ) ];
  let make_scene = | with_foreign : bool |
  {
    let mut tiles : Vec< Tile > = region1_cells.iter()
      .map( | &pos | Tile { pos, objects : vec![ "region_1".into() ] } ).collect();
    if with_foreign
    {
      tiles.push( Tile { pos : ( 1, 0 ), objects : vec![ "region_0".into() ] } );
    }
    SceneSnapshot { tiles, ..minimal_scene_3x3() }
  };

  // Build `quantized-position → frame-name` for region_1's emitted sprites.
  let region_map = | snap : &SceneSnapshot |
  {
    let cmds = at_time_compile( &spec, snap, &Camera::default(), 0.0 );
    let mut map : std::collections::BTreeMap< String, String > = std::collections::BTreeMap::new();
    for s in sprite_commands( &cmds )
    {
      if let Some( name ) = region_ids.get( &s.sprite )
      {
        // Quantise the position into a stable string key (avoids float ordering
        // / hashing) so the two scenes' triangles line up by geometry.
        let key = format!( "{:.2},{:.2}", s.transform.position[ 0 ], s.transform.position[ 1 ] );
        map.insert( key, name.clone() );
      }
    }
    map
  };

  let foreign_map = region_map( &make_scene( true ) );
  let void_map    = region_map( &make_scene( false ) );

  // The boundary edge triangle must actually be present (otherwise the test is
  // vacuous): at least one `r_edge_*` frame is emitted.
  assert!
  (
    foreign_map.values().any( | n | n.starts_with( "r_edge_" ) ),
    "boundary must produce an edge triangle; got {foreign_map:?}",
  );
  assert!( !foreign_map.is_empty(), "region_1 must emit some dual sprites" );

  // A foreign neighbour must orient region_1 identically to void.
  assert_eq!
  (
    foreign_map, void_map,
    "region_1 orientation must be identical whether the neighbour is region_0 or void",
  );
}
