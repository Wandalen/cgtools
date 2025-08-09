//! Spatial partitioning and optimization structures for tile-based games.
//!
//! This module provides efficient spatial data structures for managing large numbers
//! of entities in tile-based games. It includes quadtree implementation for fast
//! collision detection, spatial queries, and entity management.
//!
//! # Spatial Partitioning
//!
//! Spatial partitioning divides game space into hierarchical regions to enable:
//! - **Fast Collision Detection**: O(log n) instead of O(n²) for entity pairs
//! - **Efficient Spatial Queries**: Quick area-based entity searches
//! - **Level-of-Detail**: Different processing levels based on proximity
//! - **Culling**: Skip processing for entities outside view regions
//!
//! # Quadtree Implementation
//!
//! The quadtree recursively subdivides 2D space into four quadrants:
//! - **Automatic Subdivision**: Splits when node capacity exceeded
//! - **Dynamic Updates**: Handles moving entities efficiently
//! - **Query Optimization**: Fast rectangular and circular queries
//! - **Memory Efficient**: Only allocates nodes as needed
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::spatial::{ Quadtree, SpatialBounds, SpatialEntity };
//! use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, FourConnected };
//!
//! // Create a quadtree for a 100x100 game world
//! let bounds = SpatialBounds::new(0, 0, 100, 100);
//! let mut quadtree = Quadtree::new(bounds, 10); // Max 10 entities per node
//!
//! // Add entities to the quadtree
//! let entity_id = 1;
//! let position = SquareCoord::<FourConnected>::new(25, 25);
//! quadtree.insert(SpatialEntity::new(entity_id, position, 1)); // radius 1
//!
//! // Query entities in a region
//! let query_bounds = SpatialBounds::new(20, 20, 30, 30);
//! let nearby_entities = quadtree.query_region(&query_bounds);
//! println!("Found {} entities in region", nearby_entities.len());
//! ```

use crate::coordinates::Distance;

/// Represents a rectangular spatial boundary for quadtree operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialBounds {
    /// Left boundary (minimum x)
    pub left: i32,
    /// Top boundary (minimum y)  
    pub top: i32,
    /// Right boundary (maximum x)
    pub right: i32,
    /// Bottom boundary (maximum y)
    pub bottom: i32,
}

impl SpatialBounds {
    /// Creates a new spatial boundary.
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }

    /// Creates a boundary from center point and dimensions.
    pub fn from_center_size(center_x: i32, center_y: i32, width: i32, height: i32) -> Self {
        let half_width = width / 2;
        let half_height = height / 2;
        Self {
            left: center_x - half_width,
            top: center_y - half_height,
            right: center_x + half_width,
            bottom: center_y + half_height,
        }
    }

    /// Returns the width of this boundary.
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Returns the height of this boundary.
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// Returns the area of this boundary.
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    /// Checks if this boundary contains a point.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }

    /// Checks if this boundary intersects with another boundary.
    pub fn intersects(&self, other: &SpatialBounds) -> bool {
        !(self.right < other.left || 
          self.left > other.right || 
          self.bottom < other.top || 
          self.top > other.bottom)
    }

    /// Checks if this boundary completely contains another boundary.
    pub fn contains(&self, other: &SpatialBounds) -> bool {
        self.left <= other.left && 
        self.right >= other.right && 
        self.top <= other.top && 
        self.bottom >= other.bottom
    }

    /// Returns the center point of this boundary.
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }
}

/// Represents an entity with spatial properties for quadtree storage.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialEntity<C> {
    /// Unique identifier for this entity
    pub id: u32,
    /// Position in coordinate space
    pub position: C,
    /// Collision/interaction radius
    pub radius: i32,
}

impl<C> SpatialEntity<C> {
    /// Creates a new spatial entity.
    pub fn new(id: u32, position: C, radius: i32) -> Self {
        Self { id, position, radius }
    }

    /// Gets the spatial bounds of this entity.
    pub fn bounds(&self) -> SpatialBounds
    where
        C: SpatialCoordinate,
    {
        let (x, y) = self.position.to_spatial_coords();
        SpatialBounds::from_center_size(x, y, self.radius * 2, self.radius * 2)
    }

    /// Checks if this entity intersects with a boundary.
    pub fn intersects_bounds(&self, bounds: &SpatialBounds) -> bool
    where
        C: SpatialCoordinate,
    {
        self.bounds().intersects(bounds)
    }

    /// Checks if this entity intersects with another entity.
    pub fn intersects_entity(&self, other: &SpatialEntity<C>) -> bool
    where
        C: Distance,
    {
        let distance = self.position.distance(&other.position);
        distance <= (self.radius + other.radius) as u32
    }
}

/// Trait for coordinate types that can be used in spatial partitioning.
pub trait SpatialCoordinate {
    /// Converts this coordinate to spatial (x, y) integers.
    fn to_spatial_coords(&self) -> (i32, i32);
    
    /// Creates a coordinate from spatial (x, y) integers.
    fn from_spatial_coords(x: i32, y: i32) -> Self;
}

/// Quadtree node for hierarchical spatial partitioning.
#[derive(Debug)]
enum QuadtreeNode<C> {
    /// Leaf node containing entities
    Leaf {
        entities: Vec<SpatialEntity<C>>,
    },
    /// Internal node with four child quadrants
    Internal {
        northeast: Box<QuadtreeNode<C>>,
        northwest: Box<QuadtreeNode<C>>,
        southeast: Box<QuadtreeNode<C>>,
        southwest: Box<QuadtreeNode<C>>,
    },
}

impl<C> QuadtreeNode<C> {
    /// Creates a new empty leaf node.
    fn new_leaf() -> Self {
        QuadtreeNode::Leaf {
            entities: Vec::new(),
        }
    }
}

/// Quadtree for efficient spatial partitioning and queries.
#[derive(Debug)]
pub struct Quadtree<C> {
    /// Root node of the quadtree
    root: QuadtreeNode<C>,
    /// Spatial boundary of the entire quadtree
    bounds: SpatialBounds,
    /// Maximum entities per leaf node before subdivision
    max_entities: usize,
    /// Current depth of the quadtree
    max_depth: usize,
}

impl<C> Quadtree<C>
where
    C: SpatialCoordinate + Clone,
{
    /// Creates a new quadtree with the specified bounds and capacity.
    pub fn new(bounds: SpatialBounds, max_entities: usize) -> Self {
        Self {
            root: QuadtreeNode::new_leaf(),
            bounds,
            max_entities,
            max_depth: 0,
        }
    }

    /// Inserts an entity into the quadtree.
    pub fn insert(&mut self, entity: SpatialEntity<C>) {
        let bounds = self.bounds;
        let max_entities = self.max_entities;
        Self::insert_recursive_static(&mut self.root, entity, &bounds, 0, max_entities, &mut self.max_depth);
    }

    /// Removes all entities with the specified ID from the quadtree.
    pub fn remove(&mut self, entity_id: u32) -> Vec<SpatialEntity<C>> {
        let mut removed = Vec::new();
        Self::remove_recursive_static(&mut self.root, entity_id, &mut removed);
        removed
    }

    /// Queries all entities that intersect with the specified boundary.
    pub fn query_region(&self, query_bounds: &SpatialBounds) -> Vec<SpatialEntity<C>> {
        let mut results = Vec::new();
        self.query_recursive(&self.root, query_bounds, &self.bounds, &mut results);
        results
    }

    /// Queries all entities within a circular area.
    pub fn query_circle(&self, center_x: i32, center_y: i32, radius: i32) -> Vec<SpatialEntity<C>>
    where
        C: Distance,
    {
        // First get candidates from rectangular query
        let query_bounds = SpatialBounds::from_center_size(center_x, center_y, radius * 2, radius * 2);
        let candidates = self.query_region(&query_bounds);

        // Filter by actual circular distance
        let center_coord = C::from_spatial_coords(center_x, center_y);
        candidates.into_iter()
            .filter(|entity| {
                let distance = entity.position.distance(&center_coord);
                distance <= (radius as u32)
            })
            .collect()
    }

    /// Gets all entities stored in the quadtree.
    pub fn all_entities(&self) -> Vec<SpatialEntity<C>> {
        let mut entities = Vec::new();
        self.collect_all_entities(&self.root, &mut entities);
        entities
    }

    /// Clears all entities from the quadtree.
    pub fn clear(&mut self) {
        self.root = QuadtreeNode::new_leaf();
        self.max_depth = 0;
    }

    /// Returns statistics about the quadtree structure.
    pub fn stats(&self) -> QuadtreeStats {
        let mut stats = QuadtreeStats::default();
        self.calculate_stats(&self.root, 0, &mut stats);
        stats
    }

    // Private implementation methods

    fn insert_recursive_static(
        node: &mut QuadtreeNode<C>, 
        entity: SpatialEntity<C>, 
        bounds: &SpatialBounds,
        depth: usize,
        max_entities: usize,
        current_max_depth: &mut usize,
    ) {
        *current_max_depth = (*current_max_depth).max(depth);

        match node {
            QuadtreeNode::Leaf { entities } => {
                entities.push(entity);
                
                // Check if we need to subdivide
                if entities.len() > max_entities && depth < 16 { // Max depth limit
                    Self::subdivide_node_static(node, bounds, depth, max_entities, current_max_depth);
                }
            }
            QuadtreeNode::Internal { northeast, northwest, southeast, southwest } => {
                let (center_x, center_y) = bounds.center();
                let (entity_x, entity_y) = entity.position.to_spatial_coords();

                // Determine which quadrant(s) the entity belongs to
                let in_north = entity_y <= center_y;
                let in_east = entity_x >= center_x;

                match (in_north, in_east) {
                    (true, true) => {
                        Self::insert_recursive_static(northeast, entity, 
                            &SpatialBounds::new(center_x, bounds.top, bounds.right, center_y), 
                            depth + 1, max_entities, current_max_depth);
                    }
                    (true, false) => {
                        Self::insert_recursive_static(northwest, entity,
                            &SpatialBounds::new(bounds.left, bounds.top, center_x, center_y), 
                            depth + 1, max_entities, current_max_depth);
                    }
                    (false, true) => {
                        Self::insert_recursive_static(southeast, entity,
                            &SpatialBounds::new(center_x, center_y, bounds.right, bounds.bottom), 
                            depth + 1, max_entities, current_max_depth);
                    }
                    (false, false) => {
                        Self::insert_recursive_static(southwest, entity,
                            &SpatialBounds::new(bounds.left, center_y, center_x, bounds.bottom), 
                            depth + 1, max_entities, current_max_depth);
                    }
                };
            }
        }
    }

    fn subdivide_node_static(
        node: &mut QuadtreeNode<C>, 
        bounds: &SpatialBounds, 
        depth: usize,
        max_entities: usize,
        current_max_depth: &mut usize,
    ) {
        if let QuadtreeNode::Leaf { entities } = node {
            let entities_to_redistribute = std::mem::take(entities);
            
            // Create four child nodes
            *node = QuadtreeNode::Internal {
                northeast: Box::new(QuadtreeNode::new_leaf()),
                northwest: Box::new(QuadtreeNode::new_leaf()),
                southeast: Box::new(QuadtreeNode::new_leaf()),
                southwest: Box::new(QuadtreeNode::new_leaf()),
            };

            // Redistribute entities to child nodes
            for entity in entities_to_redistribute {
                Self::insert_recursive_static(node, entity, bounds, depth, max_entities, current_max_depth);
            }
        }
    }


    fn remove_recursive_static(
        node: &mut QuadtreeNode<C>,
        entity_id: u32,
        removed: &mut Vec<SpatialEntity<C>>
    ) {
        match node {
            QuadtreeNode::Leaf { entities } => {
                let _original_len = entities.len();
                entities.retain(|e| {
                    if e.id == entity_id {
                        removed.push(e.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            QuadtreeNode::Internal { northeast, northwest, southeast, southwest } => {
                Self::remove_recursive_static(northeast, entity_id, removed);
                Self::remove_recursive_static(northwest, entity_id, removed);
                Self::remove_recursive_static(southeast, entity_id, removed);
                Self::remove_recursive_static(southwest, entity_id, removed);
            }
        }
    }

    fn query_recursive(
        &self,
        node: &QuadtreeNode<C>,
        query_bounds: &SpatialBounds,
        node_bounds: &SpatialBounds,
        results: &mut Vec<SpatialEntity<C>>
    ) {
        if !query_bounds.intersects(node_bounds) {
            return;
        }

        match node {
            QuadtreeNode::Leaf { entities } => {
                for entity in entities {
                    if entity.intersects_bounds(query_bounds) {
                        results.push(entity.clone());
                    }
                }
            }
            QuadtreeNode::Internal { northeast, northwest, southeast, southwest } => {
                let (center_x, center_y) = node_bounds.center();
                
                self.query_recursive(
                    northeast, query_bounds,
                    &SpatialBounds::new(center_x, node_bounds.top, node_bounds.right, center_y),
                    results
                );
                self.query_recursive(
                    northwest, query_bounds,
                    &SpatialBounds::new(node_bounds.left, node_bounds.top, center_x, center_y),
                    results
                );
                self.query_recursive(
                    southeast, query_bounds,
                    &SpatialBounds::new(center_x, center_y, node_bounds.right, node_bounds.bottom),
                    results
                );
                self.query_recursive(
                    southwest, query_bounds,
                    &SpatialBounds::new(node_bounds.left, center_y, center_x, node_bounds.bottom),
                    results
                );
            }
        }
    }

    fn collect_all_entities(&self, node: &QuadtreeNode<C>, entities: &mut Vec<SpatialEntity<C>>) {
        match node {
            QuadtreeNode::Leaf { entities: node_entities } => {
                entities.extend_from_slice(node_entities);
            }
            QuadtreeNode::Internal { northeast, northwest, southeast, southwest } => {
                self.collect_all_entities(northeast, entities);
                self.collect_all_entities(northwest, entities);
                self.collect_all_entities(southeast, entities);
                self.collect_all_entities(southwest, entities);
            }
        }
    }

    fn calculate_stats(&self, node: &QuadtreeNode<C>, depth: usize, stats: &mut QuadtreeStats) {
        stats.total_nodes += 1;
        stats.max_depth = stats.max_depth.max(depth);

        match node {
            QuadtreeNode::Leaf { entities } => {
                stats.leaf_nodes += 1;
                stats.total_entities += entities.len();
                stats.max_entities_per_node = stats.max_entities_per_node.max(entities.len());
                if entities.is_empty() {
                    stats.empty_nodes += 1;
                }
            }
            QuadtreeNode::Internal { northeast, northwest, southeast, southwest } => {
                stats.internal_nodes += 1;
                self.calculate_stats(northeast, depth + 1, stats);
                self.calculate_stats(northwest, depth + 1, stats);
                self.calculate_stats(southeast, depth + 1, stats);
                self.calculate_stats(southwest, depth + 1, stats);
            }
        }
    }
}

/// Statistics about quadtree structure and performance.
#[derive(Debug, Default, Clone)]
pub struct QuadtreeStats {
    /// Total number of nodes in the quadtree
    pub total_nodes: usize,
    /// Number of leaf nodes
    pub leaf_nodes: usize,
    /// Number of internal nodes
    pub internal_nodes: usize,
    /// Number of empty leaf nodes
    pub empty_nodes: usize,
    /// Maximum depth of the tree
    pub max_depth: usize,
    /// Total number of entities stored
    pub total_entities: usize,
    /// Maximum entities in any single node
    pub max_entities_per_node: usize,
}

impl QuadtreeStats {
    /// Calculates the average entities per leaf node.
    pub fn average_entities_per_leaf(&self) -> f32 {
        if self.leaf_nodes > 0 {
            self.total_entities as f32 / self.leaf_nodes as f32
        } else {
            0.0
        }
    }

    /// Calculates the fill ratio (non-empty nodes / total nodes).
    pub fn fill_ratio(&self) -> f32 {
        if self.total_nodes > 0 {
            (self.total_nodes - self.empty_nodes) as f32 / self.total_nodes as f32
        } else {
            0.0
        }
    }
}

// Implement SpatialCoordinate for common coordinate types
impl SpatialCoordinate for (i32, i32) {
    fn to_spatial_coords(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    fn from_spatial_coords(x: i32, y: i32) -> Self {
        (x, y)
    }
}

impl<T> SpatialCoordinate for crate::coordinates::square::Coordinate<T> {
    fn to_spatial_coords(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn from_spatial_coords(x: i32, y: i32) -> Self {
        Self::new(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinates::square::{Coordinate as SquareCoord, FourConnected};

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
        
        quadtree.insert(entity1);
        quadtree.insert(entity2);

        // Query all entities
        let all_entities = quadtree.all_entities();
        assert_eq!(all_entities.len(), 2);

        // Query specific region
        let query_bounds = SpatialBounds::new(0, 0, 50, 50);
        let region_entities = quadtree.query_region(&query_bounds);
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
            quadtree.insert(entity);
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
        quadtree.insert(SpatialEntity::new(1, SquareCoord::<FourConnected>::new(50, 50), 1)); // Center
        quadtree.insert(SpatialEntity::new(2, SquareCoord::<FourConnected>::new(52, 50), 1)); // Close
        quadtree.insert(SpatialEntity::new(3, SquareCoord::<FourConnected>::new(80, 80), 1)); // Far

        // Query circle around center
        let nearby = quadtree.query_circle(50, 50, 5);
        assert_eq!(nearby.len(), 2); // Should find entities 1 and 2, not 3
    }

    #[test]
    fn test_quadtree_remove() {
        let bounds = SpatialBounds::new(0, 0, 100, 100);
        let mut quadtree = Quadtree::new(bounds, 10);

        let entity1 = SpatialEntity::new(1, SquareCoord::<FourConnected>::new(25, 25), 1);
        let entity2 = SpatialEntity::new(2, SquareCoord::<FourConnected>::new(75, 75), 1);
        
        quadtree.insert(entity1);
        quadtree.insert(entity2);

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
            quadtree.insert(entity);
        }

        let stats = quadtree.stats();
        assert_eq!(stats.total_entities, 20);
        assert!(stats.average_entities_per_leaf() > 0.0);
        assert!(stats.fill_ratio() > 0.0);
    }
}