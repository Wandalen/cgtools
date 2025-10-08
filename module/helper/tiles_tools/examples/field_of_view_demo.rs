//! Field of View demonstration showcasing different FOV algorithms.

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
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::duplicated_attributes ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::trivially_copy_pass_by_ref ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unnested_or_patterns ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::redundant_else ) ]
//! 
//! This example demonstrates the enhanced field-of-view capabilities including:
//! - Shadowcasting algorithm
//! - Ray casting algorithm  
//! - Bresenham line tracing
//! - Multi-source lighting

use tiles_tools::{
    field_of_view::{ FieldOfView, FOVAlgorithm, LightSource, LightingCalculator },
    coordinates::square::{ Coordinate as SquareCoord, EightConnected },
};

fn main()
{
    println!("Field of View Algorithm Demonstration");
    println!("=====================================");
    
    // Create different FOV calculators
    let shadowcasting_fov = FieldOfView::with_algorithm(FOVAlgorithm::Shadowcasting);
    let raycasting_fov = FieldOfView::with_algorithm(FOVAlgorithm::RayCasting);
    let bresenham_fov = FieldOfView::with_algorithm(FOVAlgorithm::Bresenham);
    let floodfill_fov = FieldOfView::with_algorithm(FOVAlgorithm::FloodFill);
    
    // Define a viewer position
    let viewer = SquareCoord::<EightConnected>::new(5, 5);
    let view_range = 8;
    
    // Create some obstacles
    let obstacles = [
        SquareCoord::<EightConnected>::new(3, 3),
        SquareCoord::<EightConnected>::new(3, 4),
        SquareCoord::<EightConnected>::new(3, 5),
        SquareCoord::<EightConnected>::new(7, 6),
        SquareCoord::<EightConnected>::new(7, 7),
        SquareCoord::<EightConnected>::new(7, 8),
    ];
    
    let blocks_sight = |coord: &SquareCoord<EightConnected>| {
        obstacles.contains(coord)
    };
    
    // Test each algorithm
    println!("\n=== Shadowcasting Algorithm ===");
    let shadowcast_visibility = shadowcasting_fov.calculate_fov(&viewer, view_range, blocks_sight);
    println!("Visible positions count: {}", shadowcast_visibility.visible_coordinates().len());
    
    println!("\n=== Ray Casting Algorithm ===");
    let raycast_visibility = raycasting_fov.calculate_fov(&viewer, view_range, blocks_sight);
    println!("Visible positions count: {}", raycast_visibility.visible_coordinates().len());
    
    println!("\n=== Bresenham Algorithm ===");
    let bresenham_visibility = bresenham_fov.calculate_fov(&viewer, view_range, blocks_sight);
    println!("Visible positions count: {}", bresenham_visibility.visible_coordinates().len());
    
    println!("\n=== Flood Fill Algorithm ===");
    let floodfill_visibility = floodfill_fov.calculate_fov(&viewer, view_range, blocks_sight);
    println!("Visible positions count: {}", floodfill_visibility.visible_coordinates().len());
    
    // Demonstrate line-of-sight checking
    println!("\n=== Line of Sight Tests ===");
    let targets = [
        SquareCoord::<EightConnected>::new(8, 8),
        SquareCoord::<EightConnected>::new(2, 2),
        SquareCoord::<EightConnected>::new(4, 3), // Behind obstacle
    ];
    
    for target in &targets {
        let has_los = shadowcasting_fov.line_of_sight(&viewer, target, blocks_sight);
        println!("Line of sight from ({}, {}) to ({}, {}): {}", 
                viewer.x, viewer.y, target.x, target.y, has_los);
    }
    
    // Demonstrate multi-source lighting
    println!("\n=== Multi-Source Lighting ===");
    let mut lighting_calculator = LightingCalculator::new();
    
    // Add light sources
    lighting_calculator.add_light_source(
        LightSource::new(SquareCoord::<EightConnected>::new(2, 2), 6, 1.0)
            .with_color(1.0, 0.8, 0.6) // Warm light
    );
    
    lighting_calculator.add_light_source(
        LightSource::new(SquareCoord::<EightConnected>::new(10, 10), 5, 0.8)
            .with_color(0.6, 0.8, 1.0) // Cool light
    );
    
    lighting_calculator.add_light_source(
        LightSource::new(SquareCoord::<EightConnected>::new(8, 3), 4, 0.6)
            .penetrating(true) // This light passes through walls
            .with_color(0.8, 1.0, 0.8) // Green light
    );
    
    let lighting_map = lighting_calculator.calculate_lighting(blocks_sight);
    println!("Lit positions count: {}", lighting_map.len());
    
    // Show some specific lighting values
    let test_positions = [
        SquareCoord::<EightConnected>::new(2, 2),
        SquareCoord::<EightConnected>::new(5, 5),
        SquareCoord::<EightConnected>::new(8, 8),
    ];
    
    for pos in &test_positions {
        let light_level = lighting_map.get(pos).copied().unwrap_or(0.0);
        println!("Light level at ({}, {}): {:.2}", pos.x, pos.y, light_level);
    }
    
    // Demonstrate visibility ranges
    println!("\n=== Visibility by Distance ===");
    let close_positions = shadowcast_visibility.coordinates_in_range(0, 3);
    let medium_positions = shadowcast_visibility.coordinates_in_range(4, 6);
    let far_positions = shadowcast_visibility.coordinates_in_range(7, view_range);
    
    println!("Close range (0-3): {} positions", close_positions.len());
    println!("Medium range (4-6): {} positions", medium_positions.len());
    println!("Far range (7-{}): {} positions", view_range, far_positions.len());
    
    println!("\nField of View demonstration complete!");
}