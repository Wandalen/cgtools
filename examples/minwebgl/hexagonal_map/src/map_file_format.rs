use std::rc::Rc;
use nutype::nutype;
use serde::{ Deserialize, Serialize };
use crate::{ triaxial::TriAxial, Axial, Tile };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MapFile
{
  pub metadata : Metadata,
  pub map : MapSerde,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MapSerde
{
  pub tile_map : Vec< ( Axial, Tile ) >,
  pub river_map : Vec< TriAxial >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Metadata
{
  pub config : Config,
  pub bounds : [ Axial; 2 ],
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Config
{
  pub player_colors : Vec< [ u8; 3 ] >,
  pub tiles_config : Vec< TileConfig >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct TileConfig
{
  pub name : TileName,
  pub attributes : serde_json::Map< String, serde_json::Value >,
  pub sprite : Sprite
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Sprite
{
  pub source : Box< str >,
  pub width : u32,
  pub height : u32,
  pub x_offset : u32,
  pub y_offset : u32,
  pub scale : f32,
}

#[ derive( Debug, Serialize, Deserialize, PartialEq, Default ) ]
pub struct Attributes
{
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub is_static : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub is_constructable : bool,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub area : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub cost : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub wage : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub strength : u32,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub grows_in_random_direction : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub appears_on_dead_body : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub can_exist_only_near_sea : bool,
}

fn is_default_bool( value : &bool ) -> bool
{
  *value == false
}

fn is_default_u32( value : &u32 ) -> bool
{
  *value == 0
}

#[ nutype( derive( Debug, PartialEq, AsRef, Deref, Clone ) ) ]
pub struct TileName( Rc< str > );

impl Serialize for TileName
{
  fn serialize< S >( &self, serializer : S ) -> Result< S::Ok, S::Error >
  where
    S : serde::Serializer
  {
    serializer.collect_str( self.as_ref() )
  }
}

impl< 'de > Deserialize< 'de > for TileName
{
  fn deserialize< D >( deserializer : D ) -> Result< Self, D::Error >
  where
    D : serde::Deserializer< 'de >
  {
    let s = String::deserialize( deserializer )?;
    Ok( TileName::new( s.into() ) )
  }
}

#[ nutype( derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ) ]
pub struct TileId( usize );
