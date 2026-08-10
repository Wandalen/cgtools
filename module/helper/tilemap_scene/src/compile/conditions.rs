//! Evaluation of [`Condition`] against a single neighbour.
//!
//! Used by `NeighborCondition` sprite sources (skirts, Wesnoth edge blends)
//! to decide whether to emit a sprite for a given side of the current tile.

mod private
{
  use crate::compile::neighbors::{ NeighborState, VOID_ID };
  use crate::source::Condition;

  /// Evaluate a `Condition` against `neighbour`.
  ///
  /// `current_priority` is the [`crate::compile::neighbors::tile_max_priority`]
  /// of the tile whose sprite source is firing — compared to the neighbour's
  /// priority by [`Condition::NeighborPriorityLower`].
  #[ must_use ]
  pub fn evaluate_condition
  (
    condition : &Condition,
    neighbour : &NeighborState< '_ >,
    current_priority : Option< i32 >,
  ) -> bool
  {
    match condition
    {
      Condition::NeighborIs( ids ) =>
      {
        let is_void_match = ids.iter().any( | i | i == VOID_ID ) && neighbour.object_ids.is_empty();
        is_void_match
          || ids.iter().any( | wanted |
            neighbour.object_ids.iter().any( | present | present == wanted )
          )
      },
      Condition::NoNeighbor => neighbour.object_ids.is_empty(),
      Condition::NeighborPriorityLower =>
      {
        // True when the current tile has strictly higher priority than the
        // neighbour. If either side has no priority at all, the comparison
        // can't succeed (matches the Wesnoth idiom where only "terrains
        // with priority" participate).
        match ( current_priority, neighbour.max_priority )
        {
          ( Some( c ), Some( n ) ) => c > n,
          _ => false,
        }
      },
      Condition::AnyOf( sub ) =>
        sub.iter().any( | c | evaluate_condition( c, neighbour, current_priority ) ),
      Condition::AllOf( sub ) =>
        sub.iter().all( | c | evaluate_condition( c, neighbour, current_priority ) ),
      Condition::Not( inner ) =>
        !evaluate_condition( inner, neighbour, current_priority ),
    }
  }
}

mod_interface::mod_interface!
{
  exposed use evaluate_condition;
}
