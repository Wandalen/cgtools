//! Verifies `RenderSpec::from_ron_str` / `SceneSnapshot::from_ron_str`
//! return `LoadError::Ron` — not a panic, not a silently-defaulted value —
//! when given syntactically-invalid RON input, and pins the boundary
//! against `LoadError::Validation`: syntactically-valid RON that fails a
//! SPEC §16 rule must parse fine and only fail at the separate
//! `.validate()` step.

use tilemap_scene::{ LoadError, RenderSpec, SceneSnapshot, Validate, ValidationError };

const UNCLOSED_PAREN_SPEC : &str = r"RenderSpec( assets: [";

#[ test ]
fn from_ron_str_unclosed_paren_yields_ron_error()
{
  let result = RenderSpec::from_ron_str( UNCLOSED_PAREN_SPEC );
  assert!
  (
    matches!( result, Err( LoadError::Ron( _ ) ) ),
    "expected LoadError::Ron for unclosed-paren input, got {result:?}",
  );
}

const BARE_TOKEN_SCENE : &str = r"totally_not_a_struct";

#[ test ]
fn from_ron_str_bare_token_yields_ron_error()
{
  let result = SceneSnapshot::from_ron_str( BARE_TOKEN_SCENE );
  assert!
  (
    matches!( result, Err( LoadError::Ron( _ ) ) ),
    "expected LoadError::Ron for bare-token input, got {result:?}",
  );
}

// Mirrors `scene_model_test.rs`'s `validate_rejects_unknown_pipeline_layer`
// fixture: Object.global_layer points at a layer id absent from
// pipeline.layers. Syntactically valid RON — parses cleanly — but fails
// the separate validate() pass, so it must surface as
// LoadError::Validation, never LoadError::Ron.
const GHOST_LAYER_SPEC : &str = r#"
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
"#;

#[ test ]
fn syntactically_valid_ron_failing_validation_is_not_a_ron_error()
{
  let spec = RenderSpec::from_ron_str( GHOST_LAYER_SPEC )
    .expect( "syntactically valid RON must parse regardless of validation outcome" );
  let errs = spec.validate().expect_err( "ghost layer must fail validation" );
  assert!
  (
    errs.iter().any( | e | matches!
    (
      e,
      ValidationError::UnresolvedRef { kind, id, .. }
        if *kind == "pipeline layer" && id == "ghost"
    )),
    "expected UnresolvedRef for pipeline layer 'ghost', got {errs:?}",
  );
}
