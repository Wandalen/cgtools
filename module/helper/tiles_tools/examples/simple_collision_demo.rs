//! Simple ECS collision detection demonstration.

#![allow(clippy::needless_return)]
#![allow(clippy::implicit_return)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::similar_names)]
#![allow(clippy::duplicated_attributes)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_else)]

use tiles_tools::{
    ecs::{World, Position, Health, Collision, CollisionSystem, SpatialQuerySystem},
    coordinates::square::{ Coordinate as SquareCoord, FourConnected },
};

fn main() {
    println!("Simple ECS Collision Detection Demo");
    println!("==================================");
    
    // Create a world
    let mut world = World::new();
    
    // Spawn entities with collision components
    println!("\nSpawning entities...");
    let _player = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(5, 5)),
        Health::new(100),
        Collision::new(1),
    ));
    
    let _enemy = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(6, 5)), // Adjacent to player
        Health::new(50),
        Collision::new(1),
    ));
    
    let _distant_entity = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(10, 10)),
        Health::new(75),
        Collision::new(1),
    ));
    
    // Check for collisions
    println!("\nDetecting collisions...");
    let collisions = CollisionSystem::detect_collisions::<SquareCoord<FourConnected>>(&world.hecs_world);
    println!("Found {} collisions", collisions.len());
    
    if !collisions.is_empty() {
        println!("Resolving collisions...");
        CollisionSystem::resolve_collisions(&mut world.hecs_world, &collisions);
        println!("Collisions resolved");
    }
    
    // Test spatial queries
    println!("\nTesting spatial queries...");
    let center = Position::new(SquareCoord::<FourConnected>::new(5, 5));
    let nearby = SpatialQuerySystem::query_circle(&world.hecs_world, &center, 3);
    println!("Found {} entities near center", nearby.len());
    
    println!("\nDemo complete!");
}