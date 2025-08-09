//! Advanced pathfinding demonstration showcasing enhanced A* features.
//! 
//! This example demonstrates:
//! - Basic A* pathfinding
//! - Advanced pathfinding with terrain costs and obstacles
//! - Multi-goal pathfinding for AI decision making
//! - Edge cost pathfinding for realistic movement

use tiles_tools::{
    pathfind::{ astar, astar_advanced, astar_multi_goal, astar_with_edge_costs, PathfindingConfig },
    coordinates::square::{ Coordinate as SquareCoord, FourConnected, EightConnected },
};
use std::collections::{ HashMap, HashSet };

fn main() {
    println!("Advanced Pathfinding Demonstration");
    println!("=================================");
    
    // Basic pathfinding example
    println!("\n=== Basic A* Pathfinding ===");
    let start = SquareCoord::<FourConnected>::new(0, 0);
    let goal = SquareCoord::<FourConnected>::new(5, 5);
    
    // Create some obstacles
    let obstacles: HashSet<_> = [
        SquareCoord::<FourConnected>::new(2, 1),
        SquareCoord::<FourConnected>::new(2, 2),
        SquareCoord::<FourConnected>::new(2, 3),
        SquareCoord::<FourConnected>::new(3, 3),
    ].into_iter().collect();
    
    if let Some((path, cost)) = astar(
        &start,
        &goal,
        |coord| !obstacles.contains(coord), // Accessibility function
        |_coord| 1, // Uniform cost
    ) {
        println!("Basic path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {:?}", path);
    }
    
    // Advanced pathfinding with configuration
    println!("\n=== Advanced A* with Configuration ===");
    let mut terrain_costs = HashMap::new();
    terrain_costs.insert(SquareCoord::<FourConnected>::new(1, 1), 3); // Difficult terrain
    terrain_costs.insert(SquareCoord::<FourConnected>::new(1, 2), 3);
    terrain_costs.insert(SquareCoord::<FourConnected>::new(4, 4), 2); // Moderately difficult
    terrain_costs.insert(SquareCoord::<FourConnected>::new(4, 5), 2);
    
    let config = PathfindingConfig::new()
        .with_max_distance(20)
        .with_obstacles(obstacles.iter().cloned())
        .with_terrain_cost(SquareCoord::<FourConnected>::new(1, 1), 3)
        .with_terrain_cost(SquareCoord::<FourConnected>::new(1, 2), 3)
        .with_terrain_cost(SquareCoord::<FourConnected>::new(4, 4), 2)
        .with_terrain_cost(SquareCoord::<FourConnected>::new(4, 5), 2)
        .with_blocking_entity(SquareCoord::<FourConnected>::new(3, 2), 42) // Entity ID 42
        .with_base_cost(1);
    
    if let Some((path, cost)) = astar_advanced(&start, &goal, &config) {
        println!("Advanced path found: {} steps, cost: {}", path.len(), cost);
        println!("Path avoiding obstacles and considering terrain: {:?}", path);
    }
    
    // Edge cost pathfinding (diagonal movement costs more)
    println!("\n=== Edge Cost Pathfinding (8-Connected) ===");
    let start_8 = SquareCoord::<EightConnected>::new(0, 0);
    let goal_8 = SquareCoord::<EightConnected>::new(4, 3);
    
    if let Some((path, cost)) = astar_with_edge_costs(
        &start_8,
        &goal_8,
        |_coord| true, // All positions accessible
        |from, to| {
            // Diagonal movement costs more (realistic movement)
            let dx = (to.x - from.x).abs();
            let dy = (to.y - from.y).abs();
            if dx == 1 && dy == 1 {
                14 // ~1.414 * 10 for diagonal
            } else {
                10 // Standard orthogonal movement
            }
        },
    ) {
        println!("Edge cost path found: {} steps, cost: {}", path.len(), cost);
        println!("Path with realistic diagonal costs: {:?}", path);
    }
    
    // Multi-goal pathfinding for AI
    println!("\n=== Multi-Goal Pathfinding (AI Decision Making) ===");
    let ai_position = SquareCoord::<FourConnected>::new(2, 2);
    let possible_targets = [
        SquareCoord::<FourConnected>::new(0, 0), // Resource point
        SquareCoord::<FourConnected>::new(5, 1), // Enemy base
        SquareCoord::<FourConnected>::new(1, 5), // Defensive position
        SquareCoord::<FourConnected>::new(4, 4), // Strategic point
    ];
    
    if let Some((path, cost, chosen_target)) = astar_multi_goal(
        &ai_position,
        &possible_targets,
        |coord| !obstacles.contains(coord),
        |coord| terrain_costs.get(coord).copied().unwrap_or(1),
    ) {
        println!("AI chose target: {:?}", chosen_target);
        println!("Best path found: {} steps, cost: {}", path.len(), cost);
        println!("Optimal route: {:?}", path);
    }
    
    // Demonstrate pathfinding failure cases
    println!("\n=== Pathfinding Limitations ===");
    
    // Completely blocked goal
    let blocked_obstacles: HashSet<_> = [
        SquareCoord::<FourConnected>::new(4, 4),
        SquareCoord::<FourConnected>::new(4, 5),
        SquareCoord::<FourConnected>::new(4, 6),
        SquareCoord::<FourConnected>::new(5, 4),
        SquareCoord::<FourConnected>::new(5, 6),
        SquareCoord::<FourConnected>::new(6, 4),
        SquareCoord::<FourConnected>::new(6, 5),
        SquareCoord::<FourConnected>::new(6, 6),
    ].into_iter().collect();
    
    let blocked_goal = SquareCoord::<FourConnected>::new(5, 5);
    match astar(
        &start,
        &blocked_goal,
        |coord| !blocked_obstacles.contains(coord),
        |_| 1,
    ) {
        Some((path, cost)) => {
            println!("Path to blocked goal: {} steps, cost: {}", path.len(), cost);
        }
        None => {
            println!("No path found to blocked goal - pathfinding correctly detected impossibility");
        }
    }
    
    // Distance-limited pathfinding
    let distant_goal = SquareCoord::<FourConnected>::new(15, 15);
    let limited_config = PathfindingConfig::new()
        .with_max_distance(10); // Limit search distance
    
    match astar_advanced(&start, &distant_goal, &limited_config) {
        Some((path, cost)) => {
            println!("Path to distant goal: {} steps, cost: {}", path.len(), cost);
        }
        None => {
            println!("No path found within distance limit - target too far");
        }
    }
    
    // Performance comparison
    println!("\n=== Performance Characteristics ===");
    let large_start = SquareCoord::<FourConnected>::new(0, 0);
    let large_goal = SquareCoord::<FourConnected>::new(20, 20);
    
    let start_time = std::time::Instant::now();
    let result = astar(
        &large_start,
        &large_goal,
        |_| true, // Open field
        |_| 1,
    );
    let duration = start_time.elapsed();
    
    match result {
        Some((path, cost)) => {
            println!("Large-scale pathfinding completed in {:?}", duration);
            println!("Path length: {}, cost: {}", path.len(), cost);
            println!("Manhattan distance: {}", (large_goal.x - large_start.x).abs() + (large_goal.y - large_start.y).abs());
        }
        None => {
            println!("Large-scale pathfinding failed");
        }
    }
    
    println!("\nAdvanced pathfinding demonstration complete!");
    println!("\nKey features demonstrated:");
    println!("- Obstacle avoidance");
    println!("- Terrain cost consideration");
    println!("- Realistic diagonal movement costs");
    println!("- Multi-goal AI decision making");
    println!("- Pathfinding limitations and failure cases");
    println!("- Performance characteristics");
}