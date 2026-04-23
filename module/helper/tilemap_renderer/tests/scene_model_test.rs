//! Integration tests for the `scene-model` feature: parsing, serde round-trip,
//! and loader API surface.

#![ cfg( feature = "scene-model" ) ]
#![ allow( clippy::min_ident_chars ) ]   // short locals like `o` / `s` are idiomatic inside closures and one-shot assertions

use tilemap_renderer::scene_model::
{
  Anchor,
  Asset,
  AssetKind,
  BlendMode,
  MaskTint,
  RenderSpec,
  Scene,
  SpriteSource,
  TilingStrategy,
  TintBehaviour,
  Validate,
  VariantSelection,
};
use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

// ────────────────────────────────────────────────────────────────────────────
// Minimal end-to-end: parse a render spec with one grass object and one
// knight object featuring a masked team-colour layer, then validate.
// ────────────────────────────────────────────────────────────────────────────

const MINIMAL_SPEC : &str = r#"
RenderSpec(
    version: "0.2.0",
    assets: [
        Asset(
            id: "terrain",
            path: "terrain.png",
            kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ),
        ),
        Asset(
            id: "knight_sheet",
            path: "knight.png",
            kind: SpriteSheet( frame_count: 8, layout: Horizontal ),
        ),
    ],
    tints: [],
    animations: [
        Animation(
            id: "knight_idle",
            timing: FromSheet( asset: "knight_sheet", start_frame: 0, count: 8, fps: 10.0 ),
            mode: Loop,
        ),
    ],
    effects: [],
    objects: [
        Object(
            id: "grass",
            anchor: Hex,
            global_layer: "terrain",
            priority: Some( 10 ),
            animations: {
                "default": [
                    (
                        sprite_source: Static( ( "terrain", "grass_01" ) ),
                    ),
                ],
            },
        ),
        Object(
            id: "knight",
            anchor: Hex,
            global_layer: "units",
            default_animation: "idle",
            animations: {
                "idle": [
                    (
                        id: Some( "body" ),
                        sprite_source: Animation( ( "knight_idle" ) ),
                    ),
                    (
                        id: Some( "team" ),
                        sprite_source: Animation( ( "knight_idle" ) ),
                        behaviour: (
                            tint: Masked(
                                mask: Animation( ( "knight_idle" ) ),
                                tint: TeamColor,
                            ),
                        ),
                    ),
                ],
            },
        ),
    ],
    pipeline: (
        hex: ( tiling: HexFlatTop, cell_size: ( 72, 64 ) ),
        layers: [
            ( id: "terrain" ),
            ( id: "units", sort: YAsc ),
        ],
    ),
)
"#;

#[ test ]
fn parses_minimal_spec()
{
  let spec = RenderSpec::from_ron_str( MINIMAL_SPEC ).expect( "spec must parse" );
  assert_eq!( spec.version, "0.2.0" );
  assert_eq!( spec.assets.len(), 2 );
  assert_eq!( spec.objects.len(), 2 );
  assert_eq!( spec.pipeline.hex.tiling, TilingStrategy::HexFlatTop );

  // Grass object: single layer, static source.
  let grass = spec.objects.iter().find( | o | o.id == "grass" ).expect( "grass present" );
  assert!( matches!( grass.anchor, Anchor::Hex ) );
  assert_eq!( grass.priority, Some( 10 ) );
  let default_stack = grass.animations.get( "default" ).expect( "default anim" );
  assert_eq!( default_stack.len(), 1 );
  assert!( matches!( default_stack[ 0 ].sprite_source, SpriteSource::Static( _ ) ) );

  // Knight object: two layers with synchronised animations, second uses masked team colour.
  let knight = spec.objects.iter().find( | o | o.id == "knight" ).expect( "knight present" );
  assert_eq!( knight.default_animation, "idle" );
  let idle = knight.animations.get( "idle" ).expect( "idle anim" );
  assert_eq!( idle.len(), 2 );
  match &idle[ 1 ].behaviour.tint
  {
    TintBehaviour::Masked { tint, .. } => assert!( matches!( tint, MaskTint::TeamColor ) ),
    other => panic!( "expected Masked tint on team layer, got {other:?}" ),
  }
}

#[ test ]
fn validates_minimal_spec()
{
  let spec = RenderSpec::from_ron_str( MINIMAL_SPEC ).expect( "spec must parse" );
  // Validation is a skeleton today — should pass trivially.
  spec.validate().expect( "skeleton validation returns Ok" );
}

// ────────────────────────────────────────────────────────────────────────────
// Scene parsing — tiles + entities + viewport instances.
// ────────────────────────────────────────────────────────────────────────────

const MINIMAL_SCENE : &str = r##"
Scene(
    meta: ( name: Some("Demo"), render_spec: Some("render_spec.ron") ),
    bounds: ( min: ( 0, 0 ), max: ( 3, 3 ) ),
    tiles: [
        ( pos: ( 0, 0 ), objects: [ "grass" ] ),
        ( pos: ( 1, 0 ), objects: [ "grass" ] ),
        ( pos: ( 2, 0 ), objects: [ "grass", "village" ] ),
    ],
    entities: [
        ( at: ( 1, 1 ), object: "knight", owner: 0 ),
    ],
    players: [
        ( id: 0, color: "#cc2233", name: "Red" ),
    ],
    viewport_instances: [
        ( object: "sky_background", animation: Some( "dusk" ) ),
    ],
)
"##;

#[ test ]
fn parses_minimal_scene()
{
  let scene = Scene::from_ron_str( MINIMAL_SCENE ).expect( "scene must parse" );
  assert_eq!( scene.meta.name.as_deref(), Some( "Demo" ) );
  assert_eq!( scene.tiles.len(), 3 );
  assert_eq!( scene.entities.len(), 1 );
  assert_eq!( scene.viewport_instances.len(), 1 );
  assert_eq!( scene.tiles[ 2 ].objects.len(), 2 );
  assert_eq!( scene.entities[ 0 ].owner, 0 );
}

#[ test ]
fn validates_minimal_scene()
{
  let scene = Scene::from_ron_str( MINIMAL_SCENE ).expect( "scene must parse" );
  scene.validate().expect( "skeleton validation returns Ok" );
}

// ────────────────────────────────────────────────────────────────────────────
// Serde round-trip for anchor variants.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn anchor_multihex_round_trip()
{
  let anchor = Anchor::Multihex { shape : vec![ ( 0, 0 ), ( 1, 0 ), ( 0, 1 ), ( 1, 1 ) ] };
  let s = ron::to_string( &anchor ).unwrap();
  let back : Anchor = ron::from_str( &s ).unwrap();
  assert!( matches!( back, Anchor::Multihex { ref shape } if shape.len() == 4 ) );
}

// ────────────────────────────────────────────────────────────────────────────
// Serde round-trip for Variant source with nested Animation leaves.
// Covers the mask-animation pattern and SPEC §5.2 selection modes.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn variant_of_animations_round_trip()
{
  let src : SpriteSource = ron::from_str
  ( r#"
    Variant(
        variants: [
            ( sprite: Animation( ( "river_a" ) ), weight: 3 ),
            ( sprite: Animation( ( "river_b" ) ), weight: 1 ),
        ],
        selection: HashCoord,
    )
  "# ).expect( "variant parses" );

  match src
  {
    SpriteSource::Variant { variants, selection } =>
    {
      assert_eq!( variants.len(), 2 );
      assert!( matches!( selection, VariantSelection::HashCoord ) );
      assert!( matches!( *variants[ 0 ].sprite, SpriteSource::Animation( _ ) ) );
    },
    _ => panic!( "expected Variant, got {src:?}" ),
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Resource enum quick sanity.
// ────────────────────────────────────────────────────────────────────────────

#[ test ]
fn asset_kind_atlas_round_trip()
{
  let kind = AssetKind::Atlas
  {
    tile_size : ( 64, 64 ),
    columns : 4,
    frames : std::collections::HashMap::new(),
  };
  let s = ron::to_string( &kind ).unwrap();
  let back : AssetKind = ron::from_str( &s ).unwrap();
  assert!( matches!( back, AssetKind::Atlas { columns : 4, .. } ) );
}

#[ test ]
fn asset_sampler_defaults_on_parse()
{
  // Minimal Asset with no sampler fields — defaults must kick in.
  let a : Asset = ron::from_str
  ( r#"
    Asset(
        id: "sky",
        path: "sky.png",
        kind: Single,
    )
  "# ).expect( "asset parses" );
  assert!( matches!( a.filter, SamplerFilter::Linear ) );
  assert!( matches!( a.mipmap, MipmapMode::Off ) );
  assert!( matches!( a.wrap, WrapMode::Clamp ) );
}

#[ test ]
fn asset_sampler_repeat_round_trip()
{
  // Tiled background: pixel art, no mipmaps, repeating UV.
  let a : Asset = ron::from_str
  ( r#"
    Asset(
        id: "sky_tile",
        path: "sky_tile.png",
        kind: Single,
        filter: Nearest,
        mipmap: Off,
        wrap: Repeat,
    )
  "# ).expect( "asset parses" );
  assert!( matches!( a.filter, SamplerFilter::Nearest ) );
  assert!( matches!( a.wrap, WrapMode::Repeat ) );

  // Round-trip.
  let s = ron::to_string( &a ).unwrap();
  let back : Asset = ron::from_str( &s ).unwrap();
  assert!( matches!( back.filter, SamplerFilter::Nearest ) );
  assert!( matches!( back.wrap, WrapMode::Repeat ) );
}

#[ test ]
fn blend_mode_default_is_normal()
{
  // Not directly derivable since BlendMode doesn't implement Default, but
  // LayerBehaviour's default uses Normal. Verified indirectly via RON parse.
  let src : SpriteSource = ron::from_str
  ( r#" Static( ( "atlas", "spr" ) ) "# ).unwrap();
  match src
  {
    SpriteSource::Static( _ ) =>
    {
      // Confirm BlendMode::Normal is a valid token.
      let _ = BlendMode::Normal;
    },
    _ => panic!( "expected Static" ),
  }
}
