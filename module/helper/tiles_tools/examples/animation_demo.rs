//! Animation and tweening system demonstration.

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
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::cast_lossless)]
//!
//! This example demonstrates the comprehensive animation system including:
//! - Various easing functions and their effects
//! - Tween configuration (delay, repeat, yoyo)
//! - Timeline management for multiple animations
//! - Animating different value types (positions, colors, floats)
//! - Builder pattern for complex animations
//! - Performance characteristics

use tiles_tools::{
  animation::*,
  coordinates::square::{Coordinate as SquareCoord, FourConnected},
};
use std::time::Instant;

fn main() {
  println!("Animation and Tweening System Demonstration");
  println!("==========================================");

  // === Basic Easing Function Demonstration ===
  println!("\n=== Easing Functions Comparison ===");

  let easing_functions = vec![
  EasingFunction::Linear,
  EasingFunction::EaseIn,
  EasingFunction::EaseOut,
  EasingFunction::EaseInOut,
  EasingFunction::EaseInCubic,
  EasingFunction::EaseOutCubic,
  EasingFunction::BounceOut,
  EasingFunction::ElasticOut,
  EasingFunction::BackOut,
  ];

  println!("Comparing different easing functions at t=0.5:");
  for easing in easing_functions {
  let value = easing.apply(0.5);
  println!("  {:15} → {:.3}", format!("{:?}", easing), value);
  }

  // === Basic Tween Demonstration ===
  println!("\n=== Basic Float Animation ===");

  let mut float_tween = tween(0.0_f32, 100.0_f32, 1.0);
  println!("Animating float from 0.0 to 100.0 over 1 second:");

  for frame in 0..11 {
  let time = frame as f32 * 0.1;
  let value = float_tween.update(0.1);
  println!("  Frame {}: t={:.1}s, value={:.1}", frame, time, value);
  }

  // === Position Animation ===
  println!("\n=== Coordinate Animation ===");

  let start_pos = SquareCoord::<FourConnected>::new(0, 0);
  let end_pos = SquareCoord::<FourConnected>::new(20, 15);
  let mut pos_tween = tween_with_easing(start_pos, end_pos, 1.5, EasingFunction::EaseInOutCubic);

  println!("Animating position from (0,0) to (20,15) with EaseInOutCubic:");
  for i in 0..6 {
  let delta = 0.3;
  let pos = pos_tween.update(delta);
  println!("  Step {}: position=({}, {}), progress={:.1}%", 
          i, pos.x, pos.y, pos_tween.progress() * 100.0);
  }

  // === Color Animation ===
  println!("\n=== Color Animation ===");

  let red = Color::rgb(1.0, 0.0, 0.0);
  let blue = Color::rgb(0.0, 0.0, 1.0);
  let mut color_tween = tween_with_easing(red, blue, 1.0, EasingFunction::EaseInOut);

  println!("Animating color from red to blue:");
  for i in 0..6 {
  let color = color_tween.update(0.2);
  println!("  Step {}: RGB({:.2}, {:.2}, {:.2}), progress={:.1}%",
          i, color.r, color.g, color.b, color_tween.progress() * 100.0);
  }

  // === Advanced Tween Features ===
  println!("\n=== Advanced Tween Features ===");

  // Delay demonstration
  println!("\n--- Tween with Delay ---");
  let mut delayed_tween = animate(0.0_f32)
  .to(50.0, 0.8)
  .delay(0.4)
  .easing(EasingFunction::EaseOut)
  .build();

  println!("Tween with 0.4s delay, then 0.8s animation:");
  for i in 0..8 {
  let value = delayed_tween.update(0.2);
  println!("  Frame {}: value={:.1}, state={:?}", i, value, delayed_tween.state());
  }

  // Repeat demonstration
  println!("\n--- Tween with Repeat ---");
  let mut repeat_tween = animate(0.0_f32)
  .to(10.0, 0.5)
  .repeat(2)
  .build();

  println!("Tween repeating 2 times (3 total iterations):");
  for i in 0..8 {
  let value = repeat_tween.update(0.25);
  println!("  Frame {}: value={:.1}, completed={}", i, value, repeat_tween.is_completed());
  if repeat_tween.is_completed() {
    break;
  }
  }

  // Yoyo demonstration
  println!("\n--- Tween with Yoyo ---");
  let mut yoyo_tween = animate(0.0_f32)
  .to(20.0, 0.6)
  .repeat(1)
  .yoyo(true)
  .build();

  println!("Tween with yoyo mode (forward then backward):");
  for i in 0..10 {
  let value = yoyo_tween.update(0.2);
  println!("  Frame {}: value={:.1}, completed={}", i, value, yoyo_tween.is_completed());
  if yoyo_tween.is_completed() {
    break;
  }
  }

  // === Timeline Management ===
  println!("\n=== Timeline Management ===");

  let mut timeline = Timeline::new();

  // Add multiple animations to the timeline
  timeline.add_tween("position", 
  tween(SquareCoord::<FourConnected>::new(0, 0), 
        SquareCoord::<FourConnected>::new(10, 8), 
        1.2));
  
  timeline.add_tween("scale", 
  tween_with_easing(1.0_f32, 2.5_f32, 1.0, EasingFunction::EaseOutCubic));
  
  timeline.add_tween("rotation", 
  tween_with_easing(0.0_f32, 360.0_f32, 1.5, EasingFunction::Linear));

  timeline.add_tween("opacity", 
  tween_with_easing(0.0_f32, 1.0_f32, 0.8, EasingFunction::EaseIn));

  println!("Timeline with {} animations:", timeline.animation_count());

  for frame in 0..8 {
  timeline.update(0.2);
  
  let pos = timeline.get_value::<SquareCoord<FourConnected>>("position");
  let scale = timeline.get_value::<f32>("scale");
  let rotation = timeline.get_value::<f32>("rotation");
  let opacity = timeline.get_value::<f32>("opacity");

  println!("  Frame {}: pos=({},{}) scale={:.1} rot={:.0}° alpha={:.2}", 
          frame,
          pos.map(|p| p.x).unwrap_or(0),
          pos.map(|p| p.y).unwrap_or(0),
          scale.unwrap_or(0.0),
          rotation.unwrap_or(0.0),
          opacity.unwrap_or(0.0));

  if timeline.is_completed() {
    println!("  Timeline completed!");
    break;
  }
  }

  // === Complex Animation Patterns ===
  println!("\n=== Complex Animation Patterns ===");

  // Bouncing ball simulation
  println!("\n--- Bouncing Ball Animation ---");
  let mut ball_timeline = Timeline::new();

  // Horizontal movement (constant)
  ball_timeline.add_tween("x", 
  tween_with_easing(0.0_f32, 100.0_f32, 2.0, EasingFunction::Linear));

  // Vertical movement (bouncing)
  ball_timeline.add_tween("y", 
  animate(0.0_f32)
    .to(50.0, 0.4)
    .repeat(4)
    .yoyo(true)
    .easing(EasingFunction::BounceOut)
    .build());

  println!("Simulating bouncing ball:");
  for frame in 0..12 {
  ball_timeline.update(0.2);
  
  let x = ball_timeline.get_value::<f32>("x").unwrap_or(0.0);
  let y = ball_timeline.get_value::<f32>("y").unwrap_or(0.0);

  println!("  Frame {}: ball at ({:.1}, {:.1})", frame, x, y);

  if ball_timeline.is_completed() {
    break;
  }
  }

  // === Performance Test ===
  println!("\n=== Performance Test ===");

  let animation_count = 1000;
  let mut animations = Vec::new();

  // Create many simple animations
  for i in 0..animation_count {
  let start = (i as f32) * 0.1;
  let end = start + 10.0;
  let animation = tween_with_easing(start, end, 1.0, EasingFunction::EaseInOut);
  animations.push(animation);
  }

  println!("Performance test with {} animations:", animation_count);

  let start_time = Instant::now();
  
  // Update all animations
  for animation in &mut animations {
  animation.update(0.016); // ~60 FPS
  }
  
  let duration = start_time.elapsed();
  let microseconds = duration.as_micros();
  
  println!("Updated {} animations in {}µs", animation_count, microseconds);
  println!("Average time per animation: {:.2}µs", microseconds as f64 / animation_count as f64);
  println!("Theoretical max FPS with this load: {:.0}", 1_000_000.0 / microseconds as f64 * 60.0);

  // === Easing Function Showcase ===
  println!("\n=== Easing Function Showcase ===");

  let showcase_functions = vec![
  ("Linear", EasingFunction::Linear),
  ("Elastic", EasingFunction::ElasticOut),
  ("Bounce", EasingFunction::BounceOut),
  ("Back", EasingFunction::BackOut),
  ("Cubic", EasingFunction::EaseInOutCubic),
  ];

  for (name, easing) in showcase_functions {
  println!("\n{} Easing Progress:", name);
  let mut demo_tween = tween_with_easing(0.0_f32, 100.0_f32, 1.0, easing);
  
  print!("  ");
  for i in 0..11 {
    let value = demo_tween.update(0.1);
    print!("{:5.1}", value);
    if i < 10 { print!(" "); }
  }
  println!();
  }

  // === Animation State Management ===
  println!("\n=== Animation State Management ===");

  let mut state_demo = tween(0.0_f32, 100.0_f32, 1.0);
  
  println!("Demonstrating pause/resume functionality:");
  
  // Normal progress
  state_demo.update(0.3);
  println!("  After 0.3s: value={:.1}, state={:?}", state_demo.get_current_value(), state_demo.state());
  
  // Pause
  state_demo.pause();
  println!("  Animation paused");
  
  // Try to update while paused
  state_demo.update(0.5);
  println!("  After 0.5s (paused): value={:.1}, state={:?}", state_demo.get_current_value(), state_demo.state());
  
  // Resume
  state_demo.resume();
  println!("  Animation resumed");
  
  // Continue
  state_demo.update(0.4);
  println!("  After 0.4s more: value={:.1}, state={:?}", state_demo.get_current_value(), state_demo.state());

  println!("\n=== Animation System Demonstration Complete ===");
  println!("\nKey features demonstrated:");
  println!("- Multiple easing functions with different characteristics");
  println!("- Tween configuration (delay, repeat, yoyo mode)");
  println!("- Animation of various types (float, position, color)");
  println!("- Timeline management for coordinated animations");
  println!("- Builder pattern for fluent animation creation");
  println!("- Performance characteristics and optimization");
  println!("- Animation state management (pause/resume)");
  println!("- Complex animation patterns and effects");
}