//! Beginner Tutorial: Building Your First Tile-Based Game

#![ allow( clippy::needless_return ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::items_after_statements ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::format_in_format_args ) ]
#![ allow( clippy::cast_precision_loss ) ]
//! 
//! This tutorial walks you through creating a simple tile-based game using tiles_tools.
//! We'll build a basic dungeon explorer where a player moves around a grid, collects
//! items, and encounters enemies. This demonstrates the core concepts and systems
//! in an approachable way.
//!
//! # What You'll Learn
//! 
//! - Working with coordinate systems and tile grids
//! - Basic pathfinding and movement
//! - Entity management with ECS components  
//! - Simple game state and turn management
//! - Visual debugging and grid rendering
//! - Event handling between game systems

use tiles_tools::
{
  coordinates::{ square::{ Coordinate as SquareCoord, FourConnected }, Distance, Neighbors },
  pathfind::astar,
  ecs::{ World, Position, Health },
  debug::{ GridRenderer, GridStyle, DebugColor },
  game_systems::{ TurnBasedGame, ResourceManager, GameStateMachine, GameState, GameStateEvent },
  events::{ EventBus, EventResult },
};

fn main()
{
    println!("🎮 Tiles Tools Beginner Tutorial");
    println!("=================================");
    println!("Welcome to your first tile-based game!");
    println!("We'll build a simple dungeon explorer step by step.\n");

    // Step 1: Understanding Coordinates
    tutorial_step_1_coordinates();
    
    // Step 2: Creating a Simple Map
    tutorial_step_2_map_creation();
    
    // Step 3: Adding a Player
    tutorial_step_3_player_basics();
    
    // Step 4: Movement and Pathfinding
    tutorial_step_4_movement();
    
    // Step 5: Adding Game Entities
    tutorial_step_5_entities();
    
    // Step 6: Simple Combat System
    tutorial_step_6_combat();
    
    // Step 7: Visual Debugging
    tutorial_step_7_debugging();
    
    // Step 8: Complete Game Example
    tutorial_step_8_complete_game();

    println!("\n🎉 Tutorial Complete!");
    println!("You've learned the fundamentals of tile-based game development with tiles_tools!");
    println!("\nNext steps:");
    println!("• Check out the other examples for advanced features");
    println!("• Experiment with different coordinate systems (hexagonal, triangular)");
    println!("• Try the animation system for smooth movement");
    println!("• Explore the behavior tree system for advanced AI");
    println!("• Add serialization for save/load functionality");
}

// Step 1: Understanding coordinate systems
fn tutorial_step_1_coordinates()
{
    println!("📍 Step 1: Understanding Coordinates");
    println!("------------------------------------");
    println!("Tiles_tools supports multiple coordinate systems. Let's start with square tiles.");
    
    // Create some coordinates
    let start = SquareCoord::<FourConnected>::new(0, 0);
    let destination = SquareCoord::<FourConnected>::new(3, 2);
    
    println!("Starting position: ({}, {})", start.x, start.y);
    println!("Destination: ({}, {})", destination.x, destination.y);
    
    // Calculate distance
    let distance = start.distance(&destination);
    println!("Manhattan distance: {}", distance);
    
    // Get neighbors (4-connected square grid)
    let neighbors = start.neighbors();
    println!("Neighbors of start position:");
    for (i, neighbor) in neighbors.iter().enumerate() {
        println!("  {}: ({}, {})", i + 1, neighbor.x, neighbor.y);
    }
    
    println!("✅ Coordinates are the foundation of tile-based games!\n");
}

// Step 2: Creating a simple map with obstacles
fn tutorial_step_2_map_creation()
{
    println!("🗺️ Step 2: Creating a Simple Map");
    println!("--------------------------------");
    
    // Define our dungeon map (true = walkable, false = wall)
    let dungeon_map = [
        [false, false, false, false, false, false, false, false],
        [false, true,  true,  true,  false, true,  true,  false],
        [false, true,  false, true,  false, true,  false, false],
        [false, true,  true,  true,  true,  true,  true,  false],
        [false, false, false, true,  false, false, true,  false],
        [false, true,  true,  true,  false, true,  true,  false],
        [false, false, false, false, false, false, false, false],
    ];
    
    println!("Created a {}x{} dungeon map:", dungeon_map[0].len(), dungeon_map.len());
    println!("Legend: # = wall, . = floor");
    
    // Display the map in a simple ASCII format
    for row in &dungeon_map {
        print!("  ");
        for &cell in row {
            print!("{}", if cell { "." } else { "#" });
        }
        println!();
    }
    
    // Helper function to check if a coordinate is walkable
    let is_walkable = |coord: &SquareCoord<FourConnected>| -> bool {
        let x = coord.x as usize;
        let y = coord.y as usize;
        if y >= dungeon_map.len() || x >= dungeon_map[0].len() {
            return false;
        }
        dungeon_map[y][x]
    };
    
    // Test some positions
    let test_positions = [
        SquareCoord::<FourConnected>::new(1, 1), // Should be walkable
        SquareCoord::<FourConnected>::new(0, 0), // Should be wall
        SquareCoord::<FourConnected>::new(3, 3), // Should be walkable
    ];
    
    println!("\nTesting walkability:");
    for pos in &test_positions {
        println!("  ({}, {}): {}", pos.x, pos.y, 
                if is_walkable(pos) { "walkable" } else { "blocked" });
    }
    
    println!("✅ Map creation is essential for defining your game world!\n");
}

// Step 3: Adding a player character
fn tutorial_step_3_player_basics()
{
    println!("🧙 Step 3: Adding a Player Character");
    println!("------------------------------------");
    
    // Create ECS world for managing entities
    let mut world = World::new();
    
    // Create the player entity
    let player_position = Position::new(SquareCoord::<FourConnected>::new(1, 1));
    let player_health = Health::new(100);
    
    let player = world.spawn((player_position, player_health));
    println!("Created player entity with ID: {:?}", player);
    
    // Query the player's data
    for (entity, (pos, health)) in world.query::<(&Position<SquareCoord<FourConnected>>, &Health)>().iter() {
        if entity == player {
            println!("Player stats:");
            println!("  Position: ({}, {})", pos.coord.x, pos.coord.y);
            println!("  Health: {}/{}", health.current, health.maximum);
        }
    }
    
    // Demonstrate updating the player's position
    // In a real game, you'd use proper ECS systems for this
    println!("Player moved to (2, 1) - position updates would be handled by game systems");
    
    println!("✅ Entities are the building blocks of your game objects!\n");
}

// Step 4: Basic movement and pathfinding
fn tutorial_step_4_movement()
{
    println!("🚶 Step 4: Movement and Pathfinding");
    println!("-----------------------------------");
    
    // Set up a simple obstacle map for pathfinding
    let obstacles: Vec<SquareCoord<FourConnected>> = vec![
        SquareCoord::new(2, 1),
        SquareCoord::new(2, 2),
        SquareCoord::new(2, 3),
        SquareCoord::new(4, 2),
        SquareCoord::new(4, 3),
    ];
    
    // Define start and goal positions
    let start = SquareCoord::<FourConnected>::new(1, 1);
    let goal = SquareCoord::<FourConnected>::new(6, 3);
    
    println!("Finding path from ({}, {}) to ({}, {})", start.x, start.y, goal.x, goal.y);
    println!("Obstacles at: {:?}", obstacles);
    
    // Use A* pathfinding to find the route
    let path_result = astar(
        &start,
        &goal,
        |coord| !obstacles.contains(coord), // Accessibility function
        |_coord| 1, // Uniform cost function
    );
    
    match path_result {
        Some((path, cost)) => {
            println!("✅ Path found! {} steps with cost {}:", path.len(), cost);
            for (i, step) in path.iter().enumerate() {
                println!("  Step {}: ({}, {})", i, step.x, step.y);
            }
            
            // Demonstrate simple movement validation
            println!("\nValidating movement:");
            for i in 1..path.len().min(4) { // Show first few moves
                let from = &path[i-1];
                let to = &path[i];
                let is_valid = from.distance(to) == 1; // Adjacent squares only
                println!("  {} -> {}: {}", 
                    format!("({}, {})", from.x, from.y),
                    format!("({}, {})", to.x, to.y),
                    if is_valid { "valid" } else { "invalid" });
            }
        },
        None => {
            println!("❌ No path found to destination!");
        }
    }
    
    println!("✅ Pathfinding enables intelligent movement in your game!\n");
}

// Step 5: Adding different types of game entities
fn tutorial_step_5_entities()
{
    println!("👾 Step 5: Adding Game Entities");
    println!("-------------------------------");
    
    let mut world = World::new();
    
    // Create different types of entities
    
    // Player
    let player = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(1, 1)),
        Health::new(100),
    ));
    
    // Enemies
    let goblin = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(5, 2)),
        Health::new(30),
    ));
    
    let orc = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(6, 4)),
        Health::new(60),
    ));
    
    // Treasure chest (no health, just position)
    let treasure = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(3, 5)),
    ));
    
    println!("Created entities:");
    println!("  Player: {:?}", player);
    println!("  Goblin: {:?}", goblin);
    println!("  Orc: {:?}", orc);  
    println!("  Treasure: {:?}", treasure);
    
    // Count entities by type
    let mut entity_count = 0;
    let mut entities_with_health = 0;
    
    // Query all entities with positions
    for (entity, (pos, health)) in world.query::<(&Position<SquareCoord<FourConnected>>, Option<&Health>)>().iter() {
        entity_count += 1;
        if health.is_some() {
            entities_with_health += 1;
        }
        
        let entity_type = if entity == player {
            "Player"
        } else if entity == goblin {
            "Goblin"  
        } else if entity == orc {
            "Orc"
        } else if entity == treasure {
            "Treasure"
        } else {
            "Unknown"
        };
        
        if let Some(hp) = health {
            println!("  {} at ({}, {}): {}/{} HP", 
                entity_type, pos.coord.x, pos.coord.y, hp.current, hp.maximum);
        } else {
            println!("  {} at ({}, {}): No health", 
                entity_type, pos.coord.x, pos.coord.y);
        }
    }
    
    println!("\nEntity summary:");
    println!("  Total entities: {}", entity_count);
    println!("  Living entities: {}", entities_with_health);
    
    println!("✅ Different entity types add variety to your game world!\n");
}

// Step 6: Simple combat system
fn tutorial_step_6_combat()
{
    println!("⚔️ Step 6: Simple Combat System");
    println!("-------------------------------");
    
    // Set up entities for combat demo
    let mut world = World::new();
    let mut resource_manager = ResourceManager::new();
    
    // Create combatants
    let player = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(2, 2)),
        Health::new(100),
    ));
    
    let goblin = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(3, 2)),
        Health::new(30),
    ));
    
    // Add to resource manager for easier health management
    resource_manager.add_entity(player.id() as u32, 100.0, 50.0);
    resource_manager.add_entity(goblin.id() as u32, 30.0, 0.0);
    
    println!("Combat scenario: Player vs Goblin");
    println!("Player at (2,2) - Goblin at (3,2)");
    
    // Check if entities are adjacent (can attack)
    let mut player_pos = None;
    let mut goblin_pos = None;
    
    for (entity, pos) in world.query::<&Position<SquareCoord<FourConnected>>>().iter() {
        if entity == player {
            player_pos = Some(pos.coord);
        } else if entity == goblin {
            goblin_pos = Some(pos.coord);
        } else {
            // No action needed for other entities
        }
    }
    
    if let (Some(player_coord), Some(goblin_coord)) = (player_pos, goblin_pos) {
        let distance = player_coord.distance(&goblin_coord);
        println!("Distance between combatants: {}", distance);
        
        if distance == 1 {
            println!("Combatants are adjacent - combat can begin!");
            
            // Simulate a few rounds of combat
            for round in 1..=3 {
                println!("\n--- Round {} ---", round);
                
                // Player attacks goblin
                let player_damage = 15;
                resource_manager.modify_health(goblin.id() as u32, -(player_damage as f32));
                
                if let Some(goblin_resources) = resource_manager.get_resources(goblin.id() as u32) {
                    println!("Player attacks for {} damage!", player_damage);
                    println!("Goblin health: {}/{}", 
                        goblin_resources.health.current, 
                        goblin_resources.health.maximum);
                    
                    if goblin_resources.health.current <= 0.0 {
                        println!("💀 Goblin defeated!");
                        break;
                    }
                }
                
                // Goblin attacks back (if alive)
                if let Some(goblin_resources) = resource_manager.get_resources(goblin.id() as u32) {
                    if goblin_resources.health.current > 0.0 {
                        let goblin_damage = 8;
                        resource_manager.modify_health(player.id() as u32, -(goblin_damage as f32));
                        
                        if let Some(player_resources) = resource_manager.get_resources(player.id() as u32) {
                            println!("Goblin attacks for {} damage!", goblin_damage);
                            println!("Player health: {}/{}", 
                                player_resources.health.current, 
                                player_resources.health.maximum);
                        }
                    }
                }
            }
        } else {
            println!("Combatants are too far apart for melee combat");
        }
    }
    
    println!("✅ Combat systems bring challenge and interaction to your game!\n");
}

// Step 7: Visual debugging to see what's happening
fn tutorial_step_7_debugging()
{
    println!("🔍 Step 7: Visual Debugging");
    println!("---------------------------");
    println!("Debugging tools help you visualize and understand your game state.");
    
    // Create a debug renderer for our game world
    let mut debug_renderer = GridRenderer::new()
        .with_size(10, 8)
        .with_style(GridStyle::Square4);
    
    // Add player
    debug_renderer.add_colored_marker((2, 2), "P", "Player", DebugColor::Green, 20);
    
    // Add enemies
    debug_renderer.add_colored_marker((5, 3), "G", "Goblin", DebugColor::Red, 15);
    debug_renderer.add_colored_marker((7, 5), "O", "Orc", DebugColor::Red, 15);
    
    // Add treasure
    debug_renderer.add_colored_marker((8, 1), "T", "Treasure", DebugColor::Yellow, 10);
    
    // Add walls/obstacles
    let walls = vec![(3, 1), (3, 2), (3, 3), (6, 4), (6, 5), (6, 6)];
    for (x, y) in walls {
        debug_renderer.add_colored_marker((x, y), "#", "Wall", DebugColor::Gray, 5);
    }
    
    // Show player's movement path
    let path = vec![(2, 2), (1, 2), (1, 3), (1, 4), (2, 4), (4, 4), (5, 4), (5, 3)];
    debug_renderer.add_path(path, "Player Path", DebugColor::Blue);
    
    println!("Game world visualization:");
    println!("{}", debug_renderer.render_ascii());
    
    // Demonstrate debug annotations
    debug_renderer.clear(); // Reset for cleaner view
    
    // Add just key elements with annotations
    debug_renderer.add_colored_marker((2, 2), "P", "Player (HP: 85/100)", DebugColor::Green, 20);
    debug_renderer.add_colored_marker((5, 3), "G", "Goblin (HP: 15/30)", DebugColor::Red, 15);
    debug_renderer.add_annotation((2, 1), "Start", DebugColor::Blue);
    debug_renderer.add_annotation((8, 1), "Goal", DebugColor::Yellow);
    
    println!("\nDetailed game state with annotations:");
    println!("{}", debug_renderer.render_ascii());
    
    println!("✅ Visual debugging is crucial for understanding complex game states!\n");
}

// Step 8: Putting it all together in a complete mini-game
fn tutorial_step_8_complete_game()
{
    println!("🎯 Step 8: Complete Mini-Game");
    println!("-----------------------------");
    println!("Let's put everything together into a working game!");
    
    // Game components
    let mut world = World::new();
    let mut turn_game = TurnBasedGame::new();
    let mut resource_manager = ResourceManager::new();
    let mut event_bus = EventBus::new();
    let mut state_machine = GameStateMachine::new(GameState::Playing);
    
    // Game events
    #[derive(Debug, Clone ) ]
    struct PlayerMoved {
        from: (i32, i32),
        to: (i32, i32),
    }
    
    #[derive(Debug, Clone)] 
    struct EnemyDefeated {
        enemy_type: String,
        position: (i32, i32),
    }
    
    #[derive(Debug, Clone ) ]
    struct TreasureFound {
        position: (i32, i32),
        value: u32,
    }
    
    // Events automatically implement Event trait via blanket impl
    
    // Set up event listeners
    event_bus.subscribe(|event: &PlayerMoved| {
        println!("📍 Player moved from ({}, {}) to ({}, {})", 
            event.from.0, event.from.1, event.to.0, event.to.1);
        EventResult::Continue
    });
    
    event_bus.subscribe(|event: &EnemyDefeated| {
        println!("💀 {} defeated at ({}, {})!", 
            event.enemy_type, event.position.0, event.position.1);
        EventResult::Continue
    });
    
    event_bus.subscribe(|event: &TreasureFound| {
        println!("💰 Found treasure worth {} gold at ({}, {})!", 
            event.value, event.position.0, event.position.1);
        EventResult::Continue
    });
    
    // Create game entities
    let player = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(1, 1)),
        Health::new(100),
    ));
    
    let goblin = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(4, 2)),
        Health::new(25),
    ));
    
    let _treasure = world.spawn((
        Position::new(SquareCoord::<FourConnected>::new(6, 3)),
    ));
    
    // Add to game systems
    turn_game.add_participant(player.id() as u32, 100); // Player goes first
    turn_game.add_participant(goblin.id() as u32, 80);  // Goblin second
    
    resource_manager.add_entity(player.id() as u32, 100.0, 30.0);
    resource_manager.add_entity(goblin.id() as u32, 25.0, 0.0);
    
    println!("🎮 Mini Dungeon Explorer Started!");
    println!("Player Goal: Defeat the goblin and find the treasure");
    
    // Game state tracking
    let mut player_gold = 0;
    let mut enemies_defeated = 0;
    
    // Simulate simple gameplay
    println!("\n=== Game Simulation ===");
    
    // Player moves towards goblin
    let player_start = SquareCoord::<FourConnected>::new(1, 1);
    let goblin_pos = SquareCoord::<FourConnected>::new(4, 2);
    
    event_bus.publish(PlayerMoved { 
        from: (player_start.x, player_start.y), 
        to: (goblin_pos.x - 1, goblin_pos.y) 
    });
    
    // Combat occurs
    println!("⚔️ Player attacks goblin!");
    resource_manager.modify_health(goblin.id() as u32, -25.0); // Defeat goblin
    
    event_bus.publish(EnemyDefeated {
        enemy_type: "Goblin".to_string(),
        position: (goblin_pos.x, goblin_pos.y),
    });
    enemies_defeated += 1;
    
    // Player finds treasure
    let treasure_pos = SquareCoord::<FourConnected>::new(6, 3);
    event_bus.publish(TreasureFound {
        position: (treasure_pos.x, treasure_pos.y),
        value: 100,
    });
    player_gold += 100;
    
    // Victory condition
    state_machine.process_event(GameStateEvent::VictoryAchieved);
    println!("🎉 Victory! Game completed!");
    
    event_bus.process_events();
    
    // Final game summary
    println!("\n📋 Game Summary:");
    println!("   Final State: {:?}", state_machine.current_state());
    println!("   Player Gold: {}", player_gold);
    println!("   Enemies Defeated: {}", enemies_defeated);
    
    if state_machine.current_state() == GameState::Victory {
        println!("🏆 Congratulations! You've mastered the basics of tiles_tools!");
    } else {
        println!("⏰ Game ended - but you've learned the fundamentals!");
    }
    
    println!("✅ You've built a complete tile-based game using tiles_tools!\n");
}