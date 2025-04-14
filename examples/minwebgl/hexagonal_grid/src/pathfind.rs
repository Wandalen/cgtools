use minwebgl as min;
use std::collections::HashMap;
use tiles_tools::{ coordinates::hexagonal::*, layout::RectangularGrid };

pub fn run( rect : &RectangularGrid<Odd, Pointy> )
{
  let mut map : HashMap< Coordinate< Axial, _ >, _ > = HashMap::from_iter( rect.coordinates().map( | c | ( c.into(), true ) ) );
  map.entry( Coordinate::< Axial, Pointy >::new( 1, 0 ) ).and_modify( | v | *v = false );

  let start = Coordinate::< Axial, Pointy >::new( 0, 0 );

  let directions =
  [
    (  1,  0 ),
    (  1, -1 ),
    (  0, -1 ),
    ( -1,  0 ),
    ( -1,  1 ),
    (  0,  1 ),
  ];

  let goal = Coordinate::< Axial, Pointy >::new( 2, 3 );

  let path = pathfinding::prelude::astar
  (
    &start.into(),
    | &( x, y ) |
    {
      directions
      .iter()
      .map( move | ( q, r ) | ( x + q, y + r ).into() )
      .filter( | c | map.get( &c ).copied().unwrap_or_default() )
      .map( | c | ( ( c.q, c.r ), 1 ) )
    },
    | &( x, y ) |
    {
      let c = Coordinate::< Axial, Pointy >::new( x, y );
      goal.distance( c )
    },
    | &p | p == goal.into()
  );
  min::info!( "{path:?}" );
}
