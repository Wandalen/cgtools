//! Tests for the `spatial` module — bounds arithmetic, spatial entities, and
//! quadtree insert/query/remove/stats driven purely through the public surface.
//!
//! Relocated from `src/spatial.rs` by task 072 (bodies verbatim, re-indented).

#![ cfg( feature = "enabled" ) ]

use tiles_tools::spatial::*;
use tiles_tools::coordinates::square::{Coordinate as SquareCoord, FourConnected};

#[test]
fn test_spatial_bounds_creation() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  assert_eq!(bounds.width(), 100);
  assert_eq!(bounds.height(), 100);
  assert_eq!(bounds.area(), 10000);
  assert_eq!(bounds.center(), (50, 50));
}

#[test]
fn test_spatial_bounds_contains() {
  let bounds = SpatialBounds::new(10, 10, 50, 50);
  assert!(bounds.contains_point(25, 25));
  assert!(!bounds.contains_point(5, 5));
  assert!(!bounds.contains_point(60, 60));
}

#[test]
fn test_spatial_bounds_intersects() {
  let bounds1 = SpatialBounds::new(0, 0, 50, 50);
  let bounds2 = SpatialBounds::new(25, 25, 75, 75);
  let bounds3 = SpatialBounds::new(100, 100, 150, 150);

  assert!(bounds1.intersects(&bounds2));
  assert!(!bounds1.intersects(&bounds3));
}

#[test]
fn test_spatial_entity_creation() {
  let pos = SquareCoord::<FourConnected>::new(10, 20);
  let entity = SpatialEntity::new(1, pos, 5);

  assert_eq!(entity.id, 1);
  assert_eq!(entity.radius, 5);

  let bounds = entity.bounds();
  assert_eq!(bounds.center(), (10, 20));
}

#[test]
fn test_quadtree_basic_operations() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 4);

  // Insert entities
  let entity1 = SpatialEntity::new(1, SquareCoord::<FourConnected>::new(25, 25), 1);
  let entity2 = SpatialEntity::new(2, SquareCoord::<FourConnected>::new(75, 75), 1);

  assert!(quadtree.insert(entity1));
  assert!(quadtree.insert(entity2));

  // Query all entities
  let all_entities = quadtree.all_entities();
  assert_eq!(all_entities.len(), 2);

  // Query specific region
  let query_bounds = SpatialBounds::new(0, 0, 50, 50);
  let region_entities = quadtree.region_query(&query_bounds);
  assert_eq!(region_entities.len(), 1);
  assert_eq!(region_entities[0].id, 1);
}

#[test]
fn test_quadtree_subdivision() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 2); // Low capacity to force subdivision

  // Insert enough entities to trigger subdivision
  for i in 0..10 {
    let entity = SpatialEntity::new(i, SquareCoord::<FourConnected>::new((i * 10) as i32, (i * 10) as i32), 1);
    assert!(quadtree.insert(entity));
  }

  let stats = quadtree.stats();
  assert!(stats.max_depth > 0); // Should have subdivided
  assert_eq!(stats.total_entities, 10);
}

#[test]
fn test_quadtree_circular_query() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 10);

  // Insert entities in a pattern
  assert!(quadtree.insert(SpatialEntity::new(1, SquareCoord::<FourConnected>::new(50, 50), 1))); // Center
  assert!(quadtree.insert(SpatialEntity::new(2, SquareCoord::<FourConnected>::new(52, 50), 1))); // Close
  assert!(quadtree.insert(SpatialEntity::new(3, SquareCoord::<FourConnected>::new(80, 80), 1))); // Far

  // Query circle around center
  let nearby = quadtree.circle_query(50, 50, 5);
  assert_eq!(nearby.len(), 2); // Should find entities 1 and 2, not 3
}

#[test]
fn test_quadtree_remove() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 10);

  let entity1 = SpatialEntity::new(1, SquareCoord::<FourConnected>::new(25, 25), 1);
  let entity2 = SpatialEntity::new(2, SquareCoord::<FourConnected>::new(75, 75), 1);

  assert!(quadtree.insert(entity1));
  assert!(quadtree.insert(entity2));

  // Remove entity
  let removed = quadtree.remove(1);
  assert_eq!(removed.len(), 1);
  assert_eq!(removed[0].id, 1);

  // Verify removal
  let remaining = quadtree.all_entities();
  assert_eq!(remaining.len(), 1);
  assert_eq!(remaining[0].id, 2);
}

#[test]
fn test_quadtree_stats() {
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 5);

  // Insert entities to create interesting stats
  for i in 0..20 {
    let entity = SpatialEntity::new(i, SquareCoord::<FourConnected>::new((i * 5) as i32, (i * 5) as i32), 1);
    assert!(quadtree.insert(entity));
  }

  let stats = quadtree.stats();
  assert_eq!(stats.total_entities, 20);
  assert!(stats.average_entities_per_leaf() > 0.0);
  assert!(stats.fill_ratio() > 0.0);
}

// test_kind: bug_reproducer(BUG-134)
/// ## Root Cause
/// `Quadtree::insert` never validated that an entity's position actually
/// fell within the tree's own declared bounds before routing it -- the
/// recursive quadrant split picks a quadrant via unbounded center-point
/// comparisons, which always succeeds for any position, in or out of
/// bounds. An out-of-bounds entity got filed into a leaf whose real bounds
/// don't contain it, and remained structurally present (visible via
/// `all_entities()`) while every spatially-scoped query for the region it
/// actually occupies returned nothing, because `region_query`'s pruning
/// walks from the tree's fixed, never-growing `self.bounds`.
/// ## Why Not Caught
/// Every existing test inserted entities well within the declared bounds --
/// none exercised a position outside them.
/// ## Fix Applied
/// `insert` now checks `self.bounds.contains_point` before routing, and
/// returns `false` (leaving the tree unchanged) for an out-of-bounds
/// position instead of silently misfiling it.
/// ## Prevention
/// n/a -- covered by this test.
/// ## Pitfall
/// Invisible under any workload that only ever inserts positions already
/// known to be within the tree's declared bounds -- the defect only
/// surfaces once a caller's own bounds-computation and the tree's own
/// bounds can disagree (e.g. an entity that moved, or a tree sized from a
/// stale world extent).
#[test]
fn test_quadtree_insert_rejects_out_of_bounds_entity()
{
  let bounds = SpatialBounds::new(0, 0, 100, 100);
  let mut quadtree = Quadtree::new(bounds, 1); // Low capacity to force subdivision

  let in_bounds = SpatialEntity::new(1, SquareCoord::<FourConnected>::new(5, 5), 0);
  let out_of_bounds = SpatialEntity::new(2, SquareCoord::<FourConnected>::new(1000, 1000), 0);

  assert!(quadtree.insert(in_bounds));
  assert!(
    !quadtree.insert(out_of_bounds),
    "insert() must reject a position outside the quadtree's own declared bounds"
  );

  // The rejected entity must never appear in any view of the tree -- not
  // structurally present (all_entities), and not reachable by any query for
  // the region it would have silently occupied pre-fix.
  let all = quadtree.all_entities();
  assert_eq!(all.len(), 1);
  assert_eq!(all[0].id, 1);

  let far_query = SpatialBounds::new(900, 900, 1100, 1100);
  assert!(quadtree.region_query(&far_query).is_empty());
}
