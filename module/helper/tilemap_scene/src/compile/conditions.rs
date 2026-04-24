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

#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use crate::compile::neighbors::NeighborState;
  use crate::source::Condition;

  fn state( ids : &[ &str ], max_priority : Option< i32 > ) -> ( Vec< String >, Option< i32 > )
  {
    ( ids.iter().map( | s | ( *s ).into() ).collect(), max_priority )
  }

  #[ test ]
  fn neighbour_is_matches_any_present_id()
  {
    let ( ids, _ ) = state( &[ "water" ], None );
    let n = NeighborState { object_ids : &ids, max_priority : None };
    assert!( evaluate_condition( &Condition::NeighborIs( vec![ "water".into() ] ), &n, None ) );
    assert!( !evaluate_condition( &Condition::NeighborIs( vec![ "lava".into() ] ), &n, None ) );
  }

  #[ test ]
  fn neighbour_is_void_matches_empty()
  {
    let n = NeighborState { object_ids : &[], max_priority : None };
    assert!( evaluate_condition( &Condition::NeighborIs( vec![ "void".into() ] ), &n, None ) );
  }

  #[ test ]
  fn no_neighbor_matches_empty()
  {
    let ( ids, _ ) = state( &[], None );
    let n = NeighborState { object_ids : &ids, max_priority : None };
    assert!( evaluate_condition( &Condition::NoNeighbor, &n, None ) );

    let ( ids, _ ) = state( &[ "grass" ], None );
    let n = NeighborState { object_ids : &ids, max_priority : None };
    assert!( !evaluate_condition( &Condition::NoNeighbor, &n, None ) );
  }

  #[ test ]
  fn priority_lower_only_with_both_priorities()
  {
    let ( ids, _ ) = state( &[ "sand" ], Some( 5 ) );
    let n = NeighborState { object_ids : &ids, max_priority : Some( 5 ) };
    assert!( evaluate_condition( &Condition::NeighborPriorityLower, &n, Some( 10 ) ) );
    assert!( !evaluate_condition( &Condition::NeighborPriorityLower, &n, Some( 5 ) ) );
    assert!( !evaluate_condition( &Condition::NeighborPriorityLower, &n, None ) );
  }

  #[ test ]
  fn any_of_and_not_compose()
  {
    let ( ids, _ ) = state( &[ "water" ], None );
    let n = NeighborState { object_ids : &ids, max_priority : None };
    let cond = Condition::AnyOf( vec!
    [
      Condition::NeighborIs( vec![ "lava".into() ] ),
      Condition::NeighborIs( vec![ "water".into() ] ),
    ]);
    assert!( evaluate_condition( &cond, &n, None ) );

    let negated = Condition::Not( Box::new( cond ) );
    assert!( !evaluate_condition( &negated, &n, None ) );
  }
}

mod_interface::mod_interface!
{
  exposed use evaluate_condition;
}
