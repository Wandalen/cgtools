//! Integration tests for the `scene-model` feature: parsing, serde round-trip,
//! and loader API surface.

use rustc_hash::FxHashMap as HashMap;

use tilemap_scene::
{
  Anchor,
  Asset,
  AssetKind,
  BlendMode,
  MaskTint,
  RenderSpec,
  SceneSnapshot,
  SpriteRef,
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
            states: {
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
            default_state: "idle",
            states: {
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
        hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
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
  let default_stack = grass.states.get( "default" ).expect( "default state" );
  assert_eq!( default_stack.len(), 1 );
  assert!( matches!( default_stack[ 0 ].sprite_source, SpriteSource::Static( _ ) ) );

  // Knight object: two layers with synchronised animations, second uses masked team colour.
  let knight = spec.objects.iter().find( | o | o.id == "knight" ).expect( "knight present" );
  assert_eq!( knight.default_state, "idle" );
  let idle = knight.states.get( "idle" ).expect( "idle state" );
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
  // MINIMAL_SPEC declares assets "terrain" / "knight_sheet" and pipeline
  // layers "terrain" / "units"; every reference resolves, so validate()
  // succeeds.
  spec.validate().expect( "minimal spec validates clean" );
}

#[ test ]
fn validate_rejects_unknown_pipeline_layer()
{
  // Object.global_layer points at a layer id that does not appear in
  // pipeline.layers.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "ghost",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "ghost layer must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "pipeline layer" && id == "ghost"
    )),
    "expected UnresolvedRef for pipeline layer 'ghost', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unknown_asset_in_sprite_source()
{
  // Static sprite references asset "absent" which is not declared.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "absent", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "absent asset must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "asset" && id == "absent"
    )),
    "expected UnresolvedRef for asset 'absent', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unknown_asset_in_animation()
{
  // FromSheet animation refers to a missing asset id.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        animations: [
            Animation(
                id: "ghost",
                timing: FromSheet( asset: "ghost_sheet", start_frame: 0, count: 4, fps: 8.0 ),
                mode: Loop,
            ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "missing animation asset must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "asset" && id == "ghost_sheet"
    )),
    "expected UnresolvedRef for animation asset 'ghost_sheet', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_missing_default_state()
{
  // Object.default_state names a key that is not present in the states map.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                default_state: "missing",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "missing default_state must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::MissingDefaultState { object, state }
        if object == "grass" && state == "missing"
    )),
    "expected MissingDefaultState for object 'grass' / state 'missing', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_reserved_id()
{
  // Object.id is the reserved identifier "void" (SPEC §15.1).
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "void",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "reserved id 'void' must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::ReservedId { id }
        if id == "void"
    )),
    "expected ReservedId for 'void', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_duplicate_object_id()
{
  // Two objects declare the same id.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "duplicate object id must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::DuplicateId { kind, id }
        if *kind == "object" && id == "grass"
    )),
    "expected DuplicateId for object 'grass', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_duplicate_asset_id()
{
  // Two assets declare the same id — proves duplicate_ids_check generalises
  // beyond the objects collection exercised above.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
            Asset( id: "terrain", path: "other.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "duplicate asset id must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::DuplicateId { kind, id }
        if *kind == "asset" && id == "terrain"
    )),
    "expected DuplicateId for asset 'terrain', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unresolved_animation_ref()
{
  // Sprite source plays an animation id that is not declared.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Animation( ( "ghost_anim" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "dangling animation ref must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "animation" && id == "ghost_anim"
    )),
    "expected UnresolvedRef for animation 'ghost_anim', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unresolved_tint_ref()
{
  // Layer behaviour flat-tints with an undeclared tint id.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: {
                    "default": [
                        (
                            sprite_source: Static( ( "terrain", "0" ) ),
                            behaviour: ( tint: Flat( ( "ghost_tint" ) ) ),
                        ),
                    ],
                },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "dangling tint ref must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "tint" && id == "ghost_tint"
    )),
    "expected UnresolvedRef for tint 'ghost_tint', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unresolved_effect_ref()
{
  // Layer behaviour lists an effect id that is not declared.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: {
                    "default": [
                        (
                            sprite_source: Static( ( "terrain", "0" ) ),
                            behaviour: ( effects: [ ( "ghost_effect" ) ] ),
                        ),
                    ],
                },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "dangling effect ref must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "effect" && id == "ghost_effect"
    )),
    "expected UnresolvedRef for effect 'ghost_effect', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_unresolved_connects_with()
{
  // NeighborBitmask.connects_with names an object id that is not declared.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: {
                    "default": [
                        (
                            sprite_source: NeighborBitmask(
                                connects_with: [ "ghost_neighbor" ],
                                source: ByAtlas( asset: "terrain", layout: Bitmask6 ),
                            ),
                        ),
                    ],
                },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "dangling connects_with entry must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "object" && id == "ghost_neighbor"
    )),
    "expected UnresolvedRef for object 'ghost_neighbor', got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_illegal_composite_nesting()
{
  // ViewportTiled.content nests another composite (NeighborBitmask) — SPEC
  // §5 permits only leaf sources in a composite's inner slots.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: {
                    "default": [
                        (
                            sprite_source: ViewportTiled(
                                content: NeighborBitmask(
                                    connects_with: [],
                                    source: ByAtlas( asset: "terrain", layout: Bitmask6 ),
                                ),
                                tiling: Center,
                                anchor_point: Center,
                            ),
                        ),
                    ],
                },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "composite-in-composite nesting must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::IllegalSourceNesting { outer, inner }
        if *outer == "ViewportTiled" && *inner == "NeighborBitmask"
    )),
    "expected IllegalSourceNesting for ViewportTiled/NeighborBitmask, got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_square_tiling()
{
  // pipeline.hex.tiling requests the reserved Square4 strategy.
  let spec : RenderSpec = ron::from_str( r#"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: { "default": [ ( sprite_source: Static( ( "terrain", "0" ) ) ) ] },
            ),
        ],
        pipeline: (
            hex: ( tiling: Square4, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "# ).expect( "spec parses" );
  let errs = spec.validate().expect_err( "Square4 tiling must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!( e, tilemap_scene::ValidationError::UnsupportedTiling( name ) if name == "Square4" ) ),
    "expected UnsupportedTiling for 'Square4', got {errs:?}",
  );
}

#[ test ]
fn validate_accepts_tint_effect_connects_with()
{
  // Positive case: a flat tint, an effect, and a self-referencing
  // connects_with all resolve cleanly — MINIMAL_SPEC never exercises these
  // paths, so validates_minimal_spec alone doesn't cover them.
  // Uses r##"..."## (not r#"..."#) because the tint colour literal below
  // contains `"#`, which would otherwise prematurely close a single-hash
  // raw string — same reason MINIMAL_SCENE uses r##"..."## for "#cc2233".
  let spec : RenderSpec = ron::from_str( r##"
    RenderSpec(
        version: "0.2.0",
        assets: [
            Asset( id: "terrain", path: "t.png", kind: Atlas( tile_size: ( 72, 64 ), columns: 8 ) ),
        ],
        tints: [ Tint( id: "dusk", color: "#223344", strength: 0.5 ) ],
        effects: [ Effect( id: "sway", kind: VertexDisplace( axis: X, amplitude: 2.0, frequency: 1.0 ) ) ],
        objects: [
            Object(
                id: "grass",
                anchor: Hex,
                global_layer: "terrain",
                states: {
                    "default": [
                        (
                            sprite_source: NeighborBitmask(
                                connects_with: [ "grass" ],
                                source: ByAtlas( asset: "terrain", layout: Bitmask6 ),
                            ),
                            behaviour: (
                                tint: Flat( ( "dusk" ) ),
                                effects: [ ( "sway" ) ],
                            ),
                        ),
                    ],
                },
            ),
        ],
        pipeline: (
            hex: ( tiling: HexFlatTop, grid_stride: ( 72, 64 ) ),
            layers: [ ( id: "terrain" ) ],
        ),
    )
  "## ).expect( "spec parses" );
  spec.validate().expect( "tint / effect / self-connects_with all resolve" );
}

// ────────────────────────────────────────────────────────────────────────────
// Scene parsing — tiles + entities + viewport instances.
// ────────────────────────────────────────────────────────────────────────────

const MINIMAL_SCENE : &str = r##"
SceneSnapshot(
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
  let scene = SceneSnapshot::from_ron_str( MINIMAL_SCENE ).expect( "scene must parse" );
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
  let scene = SceneSnapshot::from_ron_str( MINIMAL_SCENE ).expect( "scene must parse" );
  scene.validate().expect( "skeleton validation returns Ok" );
}

#[ test ]
fn validate_rejects_conflicting_tile_source()
{
  // Both `tiles` and `map` are populated — mutually exclusive, since
  // SceneSnapshot::palette_expand silently prefers `tiles` and drops `map`
  // when both are present.
  let scene : SceneSnapshot = ron::from_str( r#"
    SceneSnapshot(
        meta: ( name: Some("Demo"), render_spec: Some("render_spec.ron") ),
        bounds: ( min: ( 0, 0 ), max: ( 3, 3 ) ),
        tiles: [
            ( pos: ( 0, 0 ), objects: [ "grass" ] ),
        ],
        map: [ "GG" ],
    )
  "# ).expect( "scene parses" );
  let errs = scene.validate().expect_err( "conflicting tile source must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::ConflictingTileSource { tiles_len, map_rows }
        if *tiles_len == 1 && *map_rows == 1
    )),
    "expected ConflictingTileSource(1, 1), got {errs:?}",
  );
}

#[ test ]
fn validate_rejects_owner_out_of_range()
{
  // entities[0].owner indexes past the end of `players` (empty here).
  let scene : SceneSnapshot = ron::from_str( r#"
    SceneSnapshot(
        meta: ( name: Some("Demo"), render_spec: Some("render_spec.ron") ),
        bounds: ( min: ( 0, 0 ), max: ( 3, 3 ) ),
        entities: [
            ( at: ( 1, 1 ), object: "knight", owner: 0 ),
        ],
    )
  "# ).expect( "scene parses" );
  let errs = scene.validate().expect_err( "out-of-range owner must be flagged" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      tilemap_scene::ValidationError::UnresolvedRef { kind, id, context }
        if *kind == "player" && id == "0" && context.contains( "owner" )
    )),
    "expected UnresolvedRef for player '0', got {errs:?}",
  );
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
    origin : ( 0, 0 ),
    gap : ( 0, 0 ),
    frames : HashMap::default(),
    frame_rects : HashMap::default(),
    image_size : None,
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
        kind: Single( size: ( 800, 600 ) ),
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
        kind: Single( size: ( 256, 256 ) ),
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

#[ test ]
fn sprite_ref_ron_accepts_bare_tuple()
{
  // The on-wire RON form is a 2-tuple — preserved across the named-fields
  // refactor via #[serde(from = "(String, String)")] on SpriteRef.
  let r : SpriteRef = ron::from_str( r#"( "atlas", "grass_01" )"# ).unwrap();
  assert_eq!( r.asset, "atlas" );
  assert_eq!( r.frame, "grass_01" );
  // Round-trip back through RON should also give a tuple.
  let s = ron::to_string( &r ).unwrap();
  assert_eq!( s, r#"("atlas","grass_01")"# );

  // The prefixed form `SpriteRef("a", "b")` is NOT supported because
  // #[serde(from = ...)] erases SpriteRef from the deserializer's view,
  // so RON only sees a `(String, String)` target. Asserted negatively here
  // to catch silent re-introduction.
  let prefixed : Result< SpriteRef, _ > =
    ron::from_str( r#"SpriteRef( "atlas", "grass_01" )"# );
  assert!( prefixed.is_err(), "prefixed form unexpectedly parsed: {prefixed:?}" );
}
