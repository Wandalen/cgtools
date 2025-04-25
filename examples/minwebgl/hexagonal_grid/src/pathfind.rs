use std::hash::Hash;
use tiles_tools::coordinates::{ Distance, Neigbors };

pub fn find_path< C, F >
(
  start : &C,
  goal : &C,
  is_accessible : F
)
-> Option< ( Vec< C >, u32 ) >
where
  C : Distance + Neigbors + Eq + Clone + Hash,
  F : Fn( &C ) -> bool,
{
  pathfinding::prelude::astar
  (
    start,
    | coord | coord.neighbors()
                   .iter()
                   .filter( | coord | is_accessible( coord ) )
                   .map( | coord | ( coord.clone(), 1 ) )
                   .collect::< Vec< _ > >(),
    | coord | goal.distance( coord ),
    | p | *p == *goal
  )
}
