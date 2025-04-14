use tiles_tools::coordinates::hexagonal::*;

pub fn find_path< F, System, Orientation >
(
  start : Coordinate< System, Orientation >,
  goal : Coordinate< System, Orientation >,
  is_accessible : F
) -> Option< ( Vec< Coordinate< System, Orientation > >, i32 ) >
where
  F : Fn( Coordinate< Axial, Orientation > ) -> bool,
  Coordinate< System, Orientation > : Into< Coordinate< Axial, Orientation > > + From< Coordinate< Axial, Orientation > >,
{
  // TODO:
  // try to abstact neighbor getting process,
  // maybe use trait?
  let directions =
  [
    (  1,  0 ),
    (  1, -1 ),
    (  0, -1 ),
    ( -1,  0 ),
    ( -1,  1 ),
    (  0,  1 ),
  ];

  pathfinding::prelude::astar
  (
    &start,
    | &coord |
    {
      directions
      .iter()
      .map
      (
        move | &offset |
        {
          let coord : Coordinate::< Axial, Orientation > = coord.into();
          coord + ( offset ).into()
        }
      )
      .filter( | coord | is_accessible( *coord ) )
      .map( | coord | ( coord.into(), 1 ) )
    },
    | &coord |
    {
      let coord : Coordinate::< Axial, Orientation > = coord.into();
      let goal : Coordinate::< Axial, Orientation > = goal.into();
      goal.distance( coord )
    },
    | &p | p == goal
  )
}
