use std::hash::Hash;
use crate::coordinates::{ Distance, Neighbors };

/// Find the shortest path between two coordinates using the A* algorithm.
/// Returns a path and the total cost, or None if no path exists.
pub fn astar< C, Fa, Fc >
(
  start : &C,
  goal : &C,
  mut is_accessible : Fa,
  mut cost : Fc
)
-> Option< ( Vec< C >, u32 ) >
where
  C : Distance + Neighbors + Eq + Clone + Hash,
  Fa : FnMut( &C ) -> bool,
  Fc : FnMut( &C ) -> u32
{
  pathfinding::prelude::astar
  (
    start,
    // origin coord
    | coord | coord.neighbors()
                   .iter()
                   .filter( | coord | is_accessible( coord ) )
                   .map( | coord | ( coord.clone(), cost( coord ) ) ) // TODO: pass origin coord and destination coord
                   .collect::< Vec< _ > >(),
    | coord | goal.distance( coord ),
    | p | *p == *goal
  )
}
