//! Tests for `engraving_config.json` parsing and `SizingMode` resolution/validation, and for
//! the mip-chain sizes (`mip_dimensions`) the canvas and the GPU texture share.
//!
//! Unlike the other `engraving_*_tests.rs` files, this one exercises pure config logic with no
//! GL/DOM dependency, so it runs natively (not `wasm32`-gated like `engraving_shader_tests.rs`).

use renderer::webgl::engraving::{ EngravingConfig, SizingMode, mip_dimensions };

fn node_json( extra : &str ) -> String
{
  format!(
    r#"{{ "nodes": [ {{
      "nodeName": "Node",
      "uvChannel": 1,
      "aspectRatio": 4.0,
      "maxCharacters": 20,
      "defaultFont": "Roboto",
      "allowedFonts": [ "Roboto" ]
      {extra}
    }} ] }}"#,
    extra = if extra.is_empty() { String::new() } else { format!( ", {extra}" ) },
  )
}

#[ test ]
fn no_physical_fields_resolves_to_relative()
{
  let config = EngravingConfig::from_json( &node_json( "" ) ).expect( "valid config" );
  assert_eq!( config.nodes[ 0 ].resolved_sizing_mode(), SizingMode::Relative );
}

#[ test ]
fn both_physical_fields_resolve_to_hybrid_without_explicit_mode()
{
  let json = node_json( r#""stripHeightMm": 2.5, "defaultFontSizeMm": 1.5"# );
  let config = EngravingConfig::from_json( &json ).expect( "valid config" );
  assert_eq!( config.nodes[ 0 ].resolved_sizing_mode(), SizingMode::Hybrid );
}

#[ test ]
fn explicit_sizing_mode_overrides_auto_resolution()
{
  let json = node_json( r#""sizingMode": "PHYSICAL", "stripHeightMm": 2.5, "defaultFontSizeMm": 1.5"# );
  let config = EngravingConfig::from_json( &json ).expect( "valid config" );
  assert_eq!( config.nodes[ 0 ].resolved_sizing_mode(), SizingMode::Physical );
}

#[ test ]
fn physical_mode_without_physical_fields_fails_validation()
{
  let json = node_json( r#""sizingMode": "PHYSICAL""# );
  let err = EngravingConfig::from_json( &json ).expect_err( "PHYSICAL without stripHeightMm/defaultFontSizeMm must fail" );
  assert!( err.to_string().contains( "stripHeightMm" ) );
}

#[ test ]
fn hybrid_mode_without_physical_fields_fails_validation()
{
  let json = node_json( r#""sizingMode": "HYBRID""# );
  assert!( EngravingConfig::from_json( &json ).is_err() );
}

#[ test ]
fn min_font_size_exceeding_default_fails_validation()
{
  let json = node_json( r#""stripHeightMm": 2.5, "defaultFontSizeMm": 1.5, "minFontSizeMm": 2.0"# );
  let err = EngravingConfig::from_json( &json ).expect_err( "minFontSizeMm > defaultFontSizeMm must fail" );
  assert!( err.to_string().contains( "minFontSizeMm" ) );
}

#[ test ]
fn negative_strip_height_fails_validation()
{
  let json = node_json( r#""stripHeightMm": -1.0, "defaultFontSizeMm": 1.5"# );
  assert!( EngravingConfig::from_json( &json ).is_err() );
}

#[ test ]
fn font_size_ratio_out_of_range_fails_validation()
{
  let json = node_json( r#""fontSizeRatio": 1.5"# );
  assert!( EngravingConfig::from_json( &json ).is_err() );
}

#[ test ]
fn font_size_ratio_zero_fails_validation()
{
  let json = node_json( r#""fontSizeRatio": 0.0"# );
  assert!( EngravingConfig::from_json( &json ).is_err() );
}

#[ test ]
fn example_config_parses_and_resolves_each_node_independently()
{
  let json = include_str!( "../src/webgl/engraving/engraving_config.example.json" );
  let config = EngravingConfig::from_json( json ).expect( "engraving_config.example.json must stay valid against the schema" );

  let ring = config.node( "RingBand_Inner" ).expect( "RingBand_Inner node present" );
  assert_eq!( ring.resolved_sizing_mode(), SizingMode::Hybrid );

  let charm = config.node( "Charm_Bracelet_Link" ).expect( "Charm_Bracelet_Link node present" );
  assert_eq!( charm.resolved_sizing_mode(), SizingMode::Physical );
  // Different node, different physical strip height in the same file, resolved independently.
  assert_eq!( charm.strip_height_mm, Some( 1.2 ) );
  assert_ne!( charm.strip_height_mm, ring.strip_height_mm );

  let pendant = config.node( "PendantTag_Back" ).expect( "PendantTag_Back node present" );
  assert_eq!( pendant.resolved_sizing_mode(), SizingMode::Relative );
}

#[ test ]
fn valid_hybrid_config_with_min_font_size_parses()
{
  let json = node_json( r#""stripHeightMm": 2.5, "defaultFontSizeMm": 1.5, "minFontSizeMm": 0.8"# );
  let config = EngravingConfig::from_json( &json ).expect( "valid HYBRID config" );
  let node = &config.nodes[ 0 ];
  assert_eq!( node.resolved_sizing_mode(), SizingMode::Hybrid );
  assert_eq!( node.strip_height_mm, Some( 2.5 ) );
  assert_eq!( node.default_font_size_mm, Some( 1.5 ) );
  assert_eq!( node.min_font_size_mm, Some( 0.8 ) );
}

#[ test ]
fn zero_texture_height_fails_validation()
{
  let err = EngravingConfig::from_json( &node_json( r#""textureHeight": 0"# ) ).expect_err( "textureHeight 0 must fail" );
  assert!( err.to_string().contains( "textureHeight" ) );
}

#[ test ]
fn negative_engraving_strength_fails_validation()
{
  let err = EngravingConfig::from_json( &node_json( r#""engravingStrength": -0.5"# ) ).expect_err( "negative strength must fail" );
  assert!( err.to_string().contains( "engravingStrength" ) );
}

#[ test ]
fn engraving_roughness_out_of_range_fails_validation()
{
  let err = EngravingConfig::from_json( &node_json( r#""engravingRoughness": 1.5"# ) ).expect_err( "roughness above 1 must fail" );
  assert!( err.to_string().contains( "engravingRoughness" ) );
}

#[ test ]
fn engraving_darkening_out_of_range_fails_validation()
{
  let err = EngravingConfig::from_json( &node_json( r#""engravingDarkening": -0.1"# ) ).expect_err( "darkening below 0 must fail" );
  assert!( err.to_string().contains( "engravingDarkening" ) );
}

#[ test ]
fn duplicate_node_name_fails_validation()
{
  let node = r#"{ "nodeName": "Node", "uvChannel": 1, "aspectRatio": 4.0, "maxCharacters": 20,
    "defaultFont": "Roboto", "allowedFonts": [ "Roboto" ] }"#;
  let json = format!( r#"{{ "nodes": [ {node}, {node} ] }}"# );
  let err = EngravingConfig::from_json( &json ).expect_err( "a node listed twice must fail" );
  assert!( err.to_string().contains( "more than once" ) );
}

#[ test ]
fn mip_dimensions_halve_down_to_one_by_one()
{
  let levels : Vec< _ > = mip_dimensions( 10, 3 ).collect();
  assert_eq!( levels, vec![ ( 10, 3 ), ( 5, 1 ), ( 2, 1 ), ( 1, 1 ) ] );

  // floor( log2( max( w, h ) ) ) + 1 levels, ending at 1x1.
  let levels : Vec< _ > = mip_dimensions( 8038, 256 ).collect();
  assert_eq!( levels.len(), 13 );
  assert_eq!( levels.last(), Some( &( 1, 1 ) ) );

  assert_eq!( mip_dimensions( 1, 1 ).collect::< Vec< _ > >(), vec![ ( 1, 1 ) ] );
}
