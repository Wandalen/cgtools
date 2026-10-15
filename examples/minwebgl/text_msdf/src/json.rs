use std::collections::HashMap;

use serde::{ Deserialize, Serialize };
use crate::text::MSDFFont;


#[ derive( Debug, Serialize, Deserialize ) ]
pub struct CharInfo
{
  pub id : u8,
  pub width : f32,
  pub height : f32,
  pub xoffset : f32,
  pub yoffset : f32,
  pub xadvance : f32,
  pub chnl : u32,
  pub x : f32,
  pub y : f32,
  pub page : u32
}

#[ derive( Default, Debug, Serialize, Deserialize ) ]
#[ serde( default ) ]
pub struct FontInfo
{
  pub charset : Vec< char >,
}

#[ derive( Default, Debug, Serialize, Deserialize ) ]
#[ serde( default ) ]
pub struct CommonInfo
{
  #[ serde( rename = "scaleW" ) ]
  pub scale_w : f32,
  #[ serde( rename = "scaleH" ) ]
  pub scale_h : f32,
}

#[ derive( Default, Debug, Serialize, Deserialize ) ]
#[ serde( default ) ]
pub struct Kerning
{
  pub first : u8,
  pub second : u8,
  pub amount : f32
}

#[ derive( Default, Debug, Serialize, Deserialize ) ]
#[ serde( default ) ]
pub struct MSDFFontJSON
{
  pub pages : Vec< String >,
  pub chars : Vec< CharInfo >,
  pub info : FontInfo,
  pub common : CommonInfo,
  pub kernings : Vec< Kerning >
}

impl MSDFFontJSON 
{
  pub fn font_parse( font: &str ) -> MSDFFont
  {
    let res : Self = serde_json::from_str( font ).unwrap();

    let mut char_map = HashMap::new();
    
    // Build a map from available letters
    for c in res.chars
    {
      char_map.insert( c.id,  c );
    }

    let mut kerning_map : HashMap< u8, HashMap< u8, f32 > > = HashMap::new();

    // If present, build a map of offsets between possible pair of letters
    // Fix(BUG-467): the inner map was only ever looked up via `get_mut`, never
    // inserted -- no code path ever created an entry for a `first` key, so
    // `if let Some( map ) = ...` never matched and every kerning pair was
    // silently dropped. `entry(..).or_default()` creates the inner map on
    // first use of a given `first` key instead of requiring it to already exist.
    // Root cause: a lookup-only accessor (`get_mut`) used where an
    // upsert (`entry`/`or_default`) was needed -- the `if let Some` guard made
    // the always-`None` result look like an intentional "kerning data is
    // optional" check instead of dead code.
    // Pitfall: `HashMap<K, HashMap<K2, V>>` nested maps need `entry(..)
    // .or_default()` (or equivalent) at the outer level before inserting into
    // the inner map -- `get_mut` alone can never populate a key that was never
    // inserted, and silently no-ops instead of panicking, which hides the bug.
    for k in &res.kernings
    {
      kerning_map.entry( k.first ).or_default().insert( k.second, k.amount );
    }

    MSDFFont
    {
      chars : char_map,
      kernings : kerning_map,
      scale : [ res.common.scale_w, res.common.scale_h ]
    }
  }
}

// This crate is a `fn main()`-only WebGL demo binary with no `[lib]` target,
// so an external `tests/*.rs` integration test cannot reach `font_parse`
// regardless of its `pub` visibility -- per this workspace's rulebook.md
// "Test placement" rule, that makes an inline `#[cfg(test)] mod tests` block
// the correct home, not `tests/`.
#[ cfg( test ) ]
mod tests
{
  use super::*;

  // BUG-467 task/bug/completed/467_text_msdf_kerning_map_never_populated.md --
  // reproducer for `font_parse` silently dropping every kerning pair.
  // test_kind: bug_reproducer(BUG-467)
  #[ test ]
  fn font_parse_populates_kerning_map_for_every_pair()
  {
    // Two pairs share `first == 65` (to exercise the outer map's `entry`
    // being reused across pairs) and one pair uses a distinct `first == 66`
    // (to confirm outer keys stay independent). Every other `MSDFFontJSON`
    // field carries `#[serde(default)]`, so `chars`/`pages`/`info`/`common`
    // can be omitted from the fixture entirely.
    let font_json = r#"
    {
      "kernings":
      [
        { "first": 65, "second": 66, "amount": -1.5 },
        { "first": 65, "second": 67, "amount": -0.5 },
        { "first": 66, "second": 65, "amount": 2.0 }
      ]
    }
    "#;

    let font = MSDFFontJSON::font_parse( font_json );

    assert_eq!
    (
      font.kernings.len(), 2,
      "expected exactly 2 distinct `first` keys ( 65 and 66 ), got : {:?}", font.kernings
    );
    assert_eq!( font.kernings.get( &65 ).and_then( | m | m.get( &66 ) ), Some( &-1.5 ) );
    assert_eq!( font.kernings.get( &65 ).and_then( | m | m.get( &67 ) ), Some( &-0.5 ) );
    assert_eq!( font.kernings.get( &66 ).and_then( | m | m.get( &65 ) ), Some( &2.0 ) );
  }
}