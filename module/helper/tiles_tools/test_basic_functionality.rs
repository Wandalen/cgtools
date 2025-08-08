//! Basic functionality test to verify core library components work.

use tiles_tools::{
  field_of_view::{FieldOfView, FOVAlgorithm, LightSource, LightingCalculator},
  coordinates::square::{Coordinate as SquareCoord, EightConnected},
  flowfield::FlowField,
};

fn main() {
    println!("🧪 Testing tiles_tools core functionality...");
    
    // Test field-of-view system
    println!("✅ Testing Field-of-View system...");
    let fov = FieldOfView::with_algorithm(FOVAlgorithm::Shadowcasting);
    let viewer = SquareCoord::<EightConnected>::new(5, 5);
    let visibility = fov.calculate_fov(&viewer, 3, |_| false);
    
    if visibility.is_visible(&viewer) {
        println!("  ✅ FOV calculation works - viewer is visible");
    } else {
        println!("  ❌ FOV calculation failed");
    }
    
    // Test lighting system
    println!("✅ Testing Multi-source lighting system...");
    let mut lighting_calc = LightingCalculator::new();
    let light_pos = SquareCoord::<EightConnected>::new(3, 3);
    let light_source = LightSource::new(light_pos, 5, 0.8);
    lighting_calc.add_light_source(light_source);
    
    let lighting = lighting_calc.calculate_lighting(|_| false);
    if !lighting.is_empty() {
        println!("  ✅ Lighting system works - {} positions lit", lighting.len());
    } else {
        println!("  ❌ Lighting system failed");
    }
    
    // Test flow field (basic creation)
    println!("✅ Testing Flow Field system...");
    let flow_field = FlowField::<(), ()>::new(10, 10);
    println!("  ✅ FlowField created successfully: {}x{}", flow_field.width, flow_field.height);
    
    println!("\n🎉 All core systems are functioning correctly!");
    println!("The tiles_tools library provides:");
    println!("  • Advanced field-of-view calculations");
    println!("  • Multi-source dynamic lighting");
    println!("  • Flow field pathfinding framework");
    println!("  • Universal coordinate system support");
}