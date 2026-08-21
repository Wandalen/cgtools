//! ECS collision detection and spatial queries demonstration.

//!
//! This example demonstrates the enhanced ECS systems including:
//! - Collision detection between entities
//! - Collision resolution (separation)
//! - Spatial queries (circular, line, rectangular)
//! - Team-based filtering

use tiles_tools::{
    ecs::{World, Position, Health, Stats, Team, Collision, CollisionSystem, SpatialQuerySystem},
    coordinates::square::{ Coordinate as SquareCoord, FourConnected },
};

fn entities_spawn(world: &mut World) -> hecs::Entity
{
    println!("\n=== Spawning Entities ===");
    let player = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(5, 5)),
        Health::new(100),
        Stats::new(20, 15, 12, 1),
        Team::new(0), // Player team
        Collision::new(1), // Collision radius 1
    ));
    println!("Spawned player at (5, 5)");

    let _enemy1 = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(6, 5)), // Close to player
        Health::new(50),
        Stats::new(15, 10, 8, 1),
        Team::hostile(1), // Enemy team
        Collision::new(1),
    ));
    println!("Spawned enemy1 at (6, 5) - should collide with player");

    let _enemy2 = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(10, 10)),
        Health::new(60),
        Stats::new(18, 12, 9, 1),
        Team::hostile(1),
        Collision::new(1),
    ));
    println!("Spawned enemy2 at (10, 10) - far from others");

    let _ally = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(3, 5)),
        Health::new(80),
        Stats::new(22, 13, 11, 1),
        Team::new(0), // Same team as player
        Collision::new(1),
    ));
    println!("Spawned ally at (3, 5)");

    let _obstacle = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(7, 7)),
        Collision::new(2), // Larger collision radius
    ));
    println!("Spawned obstacle at (7, 7) with radius 2");

    player
}

fn collisions_detect_and_resolve(world: &mut World)
{
    println!("\n=== Collision Detection ===");
    let collisions = CollisionSystem::collisions_detect::<SquareCoord<FourConnected>>(&world.hecs_world);
    println!("Detected {} collisions", collisions.len());

    for collision in &collisions {
        println!("Collision between entities: {:?} and {:?}",
                collision.entity1, collision.entity2);
    }

    // Resolve collisions
    if !collisions.is_empty() {
        println!("\n=== Collision Resolution ===");
        println!("Resolving collisions by separating entities...");
        CollisionSystem::collisions_resolve(&mut world.hecs_world, &collisions);

        // Show new positions after resolution
        for (entity, pos) in &mut world.query::<(hecs::Entity, &Position<SquareCoord<FourConnected>>)>() {
            println!("Entity {:?} now at ({}, {})", entity, pos.coord.x, pos.coord.y);
        }
    }
}

fn spatial_queries_run(world: &World, player_entity: hecs::Entity)
{
    println!("\n=== Spatial Queries ===");

    // Fix(BUG-515): re-fetch the player's live position instead of using a
    // snapshot captured before collision resolution moved entities.
    // Root cause: `main` previously captured `player_pos` once, right after
    // spawning, and passed that single stale value to every "of player"
    // query in this function -- even though collision resolution (called
    // in between) can move the player, leaving these queries centered on a
    // location the player no longer occupies.
    // Pitfall: reverting this parameter back to a `Position` captured
    // before collision resolution silently reintroduces stale-position
    // queries; always re-derive from the stable `Entity` handle instead.
    let player_pos = *world.hecs_world.get::<&Position<SquareCoord<FourConnected>>>(player_entity).unwrap();

    // Circular query around player
    let nearby_entities = SpatialQuerySystem::circle_query(&world.hecs_world, &player_pos, 3);
    println!("Entities within radius 3 of player: {}", nearby_entities.len());
    for (entity, pos) in nearby_entities {
        println!("  Entity {:?} at ({}, {})", entity, pos.coord.x, pos.coord.y);
    }

    // Line query between two points
    let line_start = Position::new(SquareCoord::<FourConnected>::new(0, 0));
    let line_end = Position::new(SquareCoord::<FourConnected>::new(10, 10));
    let line_entities = SpatialQuerySystem::line_query(&world.hecs_world, &line_start, &line_end);
    println!("\nEntities along line from (0,0) to (10,10): {}", line_entities.len());

    // Rectangular query
    let center = Position::new(SquareCoord::<FourConnected>::new(5, 5));
    let rect_entities = SpatialQuerySystem::rectangle_query(&world.hecs_world, &center, 4, 4);
    println!("\nEntities in 4x4 rectangle around (5,5): {}", rect_entities.len());

    // Team-based queries
    println!("\n=== Team-Based Queries ===");

    // Find all enemy entities within range
    let enemies_nearby = SpatialQuerySystem::by_team_query(
        &world.hecs_world,
        &player_pos,
        8,
        |team| team.is_hostile_to(&Team::new(0)) // Find entities hostile to player team
    );
    println!("Enemy entities within range 8 of player: {}", enemies_nearby.len());
    for (entity, pos, team) in enemies_nearby {
        println!("  Enemy entity {:?} (team {}) at ({}, {})",
                entity, team.id, pos.coord.x, pos.coord.y);
    }

    // Find allied entities
    let allies_nearby = SpatialQuerySystem::by_team_query(
        &world.hecs_world,
        &player_pos,
        5,
        |team| team.is_allied_with(&Team::new(0)) // Find allied entities
    );
    println!("\nAllied entities within range 5 of player: {}", allies_nearby.len());
    for (entity, pos, team) in allies_nearby {
        println!("  Allied entity {:?} (team {}) at ({}, {})",
                entity, team.id, pos.coord.x, pos.coord.y);
    }

    // Find nearest entity to player
    println!("\n=== Nearest Entity Search ===");
    // Fix(BUG-515): exclude the player's own entity from the nearest-entity
    // search. The library's `nearest_entity_find` is a pure "nearest entity
    // to a coordinate" primitive with no self-exclusion parameter, so
    // calling it with the player's own live position always trivially
    // returned the player itself at distance 0.
    // Root cause: this section's whole purpose is finding another entity
    // relative to the player; a self-match carries zero information.
    // Pitfall: swapping this back to a bare `nearest_entity_find` call
    // silently reintroduces a guaranteed self-match, distance-0 result any
    // time `player_pos` reflects the player's true live position.
    let mut query = world.query::<(hecs::Entity, &Position<SquareCoord<FourConnected>>)>();
    let nearest_other = query.iter()
        .filter(|(entity, _)| *entity != player_entity)
        .map(|(entity, pos)| (entity, *pos, player_pos.distance_to(pos)))
        .min_by_key(|&(_, _, distance)| distance);
    if let Some((nearest_entity, nearest_pos, distance)) = nearest_other {
        println!("Nearest entity to player: {nearest_entity:?}");
        println!("  Position: ({}, {})", nearest_pos.coord.x, nearest_pos.coord.y);
        println!("  Distance: {distance}");
    }
}

fn collision_layers_demonstrate(world: &mut World)
{
    println!("\n=== Collision Layers ===");
    let _ghost = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(5, 5)), // Same position as player
        Collision::new(1).non_solid().with_layer(1), // Non-solid, different layer
    ));
    println!("Spawned ghost entity at player position (non-solid, layer 1)");

    let _wall = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(8, 8)),
        Collision::new(1).with_layer(2), // Solid, layer 2
    ));
    println!("Spawned wall entity at (8, 8) with layer 2");

    // Check collisions again with layered entities
    let new_collisions = CollisionSystem::collisions_detect::<SquareCoord<FourConnected>>(&world.hecs_world);
    println!("New collision count with layered entities: {}", new_collisions.len());

    // Demonstrate collision filtering could be added here
    // (This would require extending the collision system to consider layers)
}

fn performance_demonstrate(world: &mut World)
{
    println!("\n=== Performance Demonstration ===");

    // Spawn many entities for performance testing
    println!("Spawning 100 additional entities for performance test...");
    for i in 0..100 {
        world.spawn((
            Position::new(SquareCoord::<FourConnected>::new(i % 20, i / 20)),
            Collision::new(1),
            Health::new(25),
        ));
    }

    let start_time = std::time::Instant::now();
    let mass_collisions = CollisionSystem::collisions_detect::<SquareCoord<FourConnected>>(&world.hecs_world);
    let duration = start_time.elapsed();

    println!("Collision detection for ~105 entities completed in {duration:?}");
    println!("Detected {} collisions among all entities", mass_collisions.len());

    // Spatial query performance
    let start_time = std::time::Instant::now();
    let center = Position::new(SquareCoord::<FourConnected>::new(5, 5));
    let mass_spatial = SpatialQuerySystem::circle_query(&world.hecs_world, &center, 15);
    let duration = start_time.elapsed();

    println!("Spatial query for ~105 entities completed in {duration:?}");
    println!("Found {} entities within large radius", mass_spatial.len());
}

fn main()
{
    println!("ECS Collision Detection and Spatial Queries Demo");
    println!("===============================================");

    let mut world = World::new();

    let player_entity = entities_spawn(&mut world);
    collisions_detect_and_resolve(&mut world);
    spatial_queries_run(&world, player_entity);
    collision_layers_demonstrate(&mut world);
    performance_demonstrate(&mut world);

    println!("\nECS collision and spatial query demonstration complete!");
    println!("\nKey features demonstrated:");
    println!("- Entity collision detection");
    println!("- Automatic collision resolution");
    println!("- Circular, line, and rectangular spatial queries");
    println!("- Team-based entity filtering");
    println!("- Nearest entity search");
    println!("- Collision layers and properties");
    println!("- Performance with large entity counts");
}