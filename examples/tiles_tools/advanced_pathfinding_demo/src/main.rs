//! Simplified advanced pathfinding demonstration to avoid clippy issues.

use tiles_tools::
{
  pathfind::{ astar, astar_multi_goal, astar_with_edge_costs, PathfindingConfig, astar_advanced },
  coordinates::
  {
    hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
    triangular::{ Coordinate as TriCoord, FlatSided },
    isometric::{ Coordinate as IsoCoord, Diamond },
    square::{ Coordinate as SquareCoord, FourConnected, EightConnected },
  },
};
use std::collections::HashSet;

fn main()
{
    println!("Advanced Pathfinding Demonstration");
    println!("=================================");

    basic_pathfinding_demonstrate();
    advanced_pathfinding_demonstrate();
    multi_goal_pathfinding_demonstrate();
    edge_cost_pathfinding_demonstrate();
    hexagonal_pathfinding_demonstrate();
    triangular_pathfinding_demonstrate();
    isometric_pathfinding_demonstrate();

    println!("\n🎉 Advanced Pathfinding Demo Complete!");
    println!("Key features demonstrated:");
    println!("- Multiple coordinate systems");
    println!("- Obstacle avoidance");
    println!("- Variable terrain costs");
    println!("- Multi-goal pathfinding");
    println!("- Performance characteristics");
}

fn basic_pathfinding_demonstrate()
{
    println!("\n=== Basic A* Pathfinding ===");
    let start = SquareCoord::<FourConnected>::new(0, 0);
    let goal = SquareCoord::<FourConnected>::new(5, 5);

    let obstacles: HashSet<_> = [
        SquareCoord::<FourConnected>::new(2, 1),
        SquareCoord::<FourConnected>::new(2, 2),
        SquareCoord::<FourConnected>::new(2, 3),
        SquareCoord::<FourConnected>::new(3, 3),
    ].into_iter().collect();

    if let Some((path, cost)) = astar(
        &start,
        &goal,
        |coord| !obstacles.contains(coord),
        |_coord| 1,
    ) {
        println!("Basic path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No path found");
    }
}

fn advanced_pathfinding_demonstrate()
{
    println!("\n=== Advanced A* with Configuration ===");
    let start = SquareCoord::<FourConnected>::new(0, 0);
    let goal = SquareCoord::<FourConnected>::new(5, 5);

    let config = PathfindingConfig::new()
        .with_max_distance(20)
        .with_terrain_cost(SquareCoord::<FourConnected>::new(1, 1), 3)
        .with_terrain_cost(SquareCoord::<FourConnected>::new(1, 2), 3)
        .with_base_cost(1);

    if let Some((path, cost)) = astar_advanced(&start, &goal, &config) {
        println!("Advanced path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No advanced path found");
    }
}

fn multi_goal_pathfinding_demonstrate()
{
    println!("\n=== Multi-Goal Pathfinding ===");
    let ai_position = SquareCoord::<FourConnected>::new(2, 2);
    let possible_targets = [
        SquareCoord::<FourConnected>::new(0, 0),
        SquareCoord::<FourConnected>::new(5, 1),
        SquareCoord::<FourConnected>::new(1, 5),
        SquareCoord::<FourConnected>::new(4, 4),
    ];

    if let Some((path, cost, chosen_target)) = astar_multi_goal(
        &ai_position,
        &possible_targets,
        |_coord| true,
        |_coord| 1,
    ) {
        println!("AI chose target: {chosen_target:?}");
        println!("Best path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No multi-goal path found");
    }
}

fn edge_cost_pathfinding_demonstrate()
{
    println!("\n=== Edge Cost Pathfinding (8-Connected) ===");
    let start_8 = SquareCoord::<EightConnected>::new(0, 0);
    let goal_8 = SquareCoord::<EightConnected>::new(4, 3);

    if let Some((path, cost)) = astar_with_edge_costs(
        &start_8,
        &goal_8,
        |_coord| true,
        |from, to| {
            let dx = (to.x - from.x).abs();
            let dy = (to.y - from.y).abs();
            if dx == 1 && dy == 1 {
                14// ~1.414 * 10 for diagonal
            } else {
                10// Standard orthogonal movement
            }
        },
    ) {
        println!("Edge cost path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No edge cost path found");
    }
}

fn hexagonal_pathfinding_demonstrate()
{
    println!("\n=== Hexagonal Grid Pathfinding ===");
    let hex_start = HexCoord::<Axial, Pointy>::new(-2, 2);
    let hex_goal = HexCoord::<Axial, Pointy>::new(3, -1);

    if let Some((path, cost)) = astar(
        &hex_start,
        &hex_goal,
        |_coord| true,
        |_coord| 1,
    ) {
        println!("Hexagonal path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No hexagonal path found");
    }
}

fn triangular_pathfinding_demonstrate()
{
    println!("\n=== Triangular Grid Pathfinding ===");
    let tri_start = TriCoord::< FlatSided >::new( 0, 1, 0 ).unwrap();
    let tri_goal = TriCoord::< FlatSided >::new( 1, 2, -1 ).unwrap();

    if let Some((path, cost)) = astar(
        &tri_start,
        &tri_goal,
        |_coord| true,
        |_coord| 1,
    ) {
        println!("Triangular path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No triangular path found");
    }
}

fn isometric_pathfinding_demonstrate()
{
    println!("\n=== Isometric Grid Pathfinding ===");
    let iso_start = IsoCoord::<Diamond>::new(0, 0);
    let iso_goal = IsoCoord::<Diamond>::new(3, 2);

    if let Some((path, cost)) = astar(
        &iso_start,
        &iso_goal,
        |_coord| true,
        |_coord| 1,
    ) {
        println!("Isometric path found: {} steps, cost: {}", path.len(), cost);
        println!("Path: {path:?}");
    } else {
        println!("No isometric path found");
    }
}
