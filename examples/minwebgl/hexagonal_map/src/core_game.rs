use serde_with::serde_as;
use serde::{ Deserialize, Serialize };
use rustc_hash::{ FxHashMap, FxHashSet };
use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate, Flat };
use crate::triaxial;

pub type Coord = Coordinate< Axial, Flat >;

#[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default ) ]
pub struct ObjectIndex( pub u32 );

#[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default ) ]
pub struct TerraintIndex( pub u32 );

#[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default ) ]
pub struct PlayerIndex( pub u32 );

#[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq ) ]
pub struct Tile
{
  pub object_index : Option< ObjectIndex >,
  pub terrain_index : TerraintIndex,
  pub owner_index : PlayerIndex,
  pub coord : Coord,
}

#[ serde_as ]
#[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
pub struct Map
{
  #[ serde_as( as = "Vec<(_, _)>" ) ]
  pub tiles : FxHashMap< Coord, Tile >,
  pub rivers : FxHashSet< [ triaxial::TriAxial; 2 ] >
}

#[ derive( Debug, Serialize, Deserialize, Default ) ]
pub struct Config
{
  pub player_colors : Vec< [ u8; 3 ] >,
  pub object_props : Vec< Properties >,
  pub terrain_props : Vec< Properties >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Properties
{
  pub name : String,
  pub attributes : serde_json::Map< String, serde_json::Value >,
  pub sprite : Option< Sprite >
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Sprite
{
  pub source : String,
  pub width : u32,
  pub height : u32,
  pub x_offset : u32,
  pub y_offset : u32,
  pub scale : f32,
}
