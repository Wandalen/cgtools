use std::collections::HashMap;

use serde::{ Deserialize, Serialize };
use minwebgl as gl;
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
  pub fn parse_font( font: &str) -> MSDFFont
  {
    let res : Self = serde_json::from_str( font ).unwrap();

    let mut char_map = HashMap::new();
    
    for c in res.chars
    {
      char_map.insert( c.id,  c );
    }

    let mut kerning_map : HashMap< u8, HashMap< u8, f32 > > = HashMap::new();

    for k in res.kernings.iter()
    {
      if let Some( map ) = kerning_map.get_mut( &k.first )
      {
        map.insert( k.second, k.amount );
      }
    }

    MSDFFont
    {
      chars : char_map,
      kernings : kerning_map,
      scale : [ res.common.scale_w, res.common.scale_h ]
    }
  }   
}