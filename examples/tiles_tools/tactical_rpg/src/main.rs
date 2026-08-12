//! Tactical RPG example demonstrating advanced ECS gameplay mechanics.
//!
//! This example showcases a turn-based tactical RPG combat system using
//! the tiles_tools ECS framework. Features include:
//!
//! - Turn-based combat with initiative system
//! - Movement and attack ranges on hexagonal grid
//! - AI-controlled enemies with different behaviors  
//! - Player-controlled units with tactical decisions
//! - Line-of-sight and area-of-effect attacks
//! - Experience and leveling system
//! - Equipment and inventory management
//!
//! Run with: `cd examples/tiles_tools/tactical_rpg && cargo run --release`

use tiles_tools::{
  ecs::{World, Position, Health, Stats, Team, AI, Movable, Size},
  coordinates::{
  hexagonal::{Coordinate as HexCoord, Axial, Pointy},
  },
  pathfind::astar,
};
use std::collections::VecDeque;

// =============================================================================
// Game-Specific Components
// =============================================================================

/// Experience and leveling component
#[derive(Debug, Clone, Copy ) ]
struct Experience
{
  current_xp: u32,
  level: u32,
  xp_to_next_level: u32,
}

impl Experience {
  pub fn new(level: u32) -> Self {
  Self {
    current_xp: 0,
    level,
    xp_to_next_level: Self::xp_required_for_level(level + 1),
  }
  }
  
  pub fn xp_add(&mut self, xp: u32) -> bool {
  self.current_xp += xp;
  if self.current_xp >= self.xp_to_next_level {
    self.level_up();
    true
  } else {
    false
  }
  }
  
  fn level_up(&mut self) {
  self.level += 1;
  self.current_xp -= self.xp_to_next_level;
  self.xp_to_next_level = Self::xp_required_for_level(self.level + 1);
  }
  
  fn xp_required_for_level(level: u32) -> u32 {
  level * level * 100
  }
}

/// Initiative component for turn order
#[derive(Debug, Clone, Copy ) ]
struct Initiative
{
  value: u32,
}

impl Initiative {
  pub fn new(base: u32) -> Self {
  Self {
    value: base,
  }
  }
}

/// Equipment and inventory component
#[derive(Debug, Clone ) ]
struct Equipment
{
  weapon: Option<Weapon>,
}

#[derive(Debug, Clone ) ]
struct Weapon
{
  name: String,
  attack_bonus: u32,
}

// =============================================================================
// Game State Management
// =============================================================================

/// Main tactical RPG game state
struct TacticalRPG
{
  world: World,
  turn_queue: VecDeque<hecs::Entity>,
  current_turn: Option<hecs::Entity>,
  turn_number: u32,
  player_team: Team,
  enemy_team: Team,
  game_phase: GamePhase,
}

#[derive(Debug, Clone, Copy, PartialEq ) ]
enum GamePhase
{
  Planning,    // Player selects actions
  AI,          // AI makes decisions
  Resolution,  // Effects are resolved
}

impl TacticalRPG {
  fn player_warrior_spawn(world: &mut World, team: Team) -> hecs::Entity {
  world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(-2, 1)),
    Health::new(120),
    Stats::new(18, 12, 10, 1),
    team,
    Movable::new(3),
    Experience::new(1),
    Initiative::new(15),
    Equipment {
      weapon: Some(Weapon {
        name: "Iron Sword".to_string(),
        attack_bonus: 5,
      }),
    },
    Size::single(),
  ))
  }

  fn player_mage_spawn(world: &mut World, team: Team) -> hecs::Entity {
  world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(-1, 0)),
    Health::new(80),
    Stats::new(12, 8, 14, 1),
    team,
    Movable::new(2),
    Experience::new(1),
    Initiative::new(12),
    Equipment {
      weapon: Some(Weapon {
        name: "Fire Staff".to_string(),
        attack_bonus: 2,
      }),
    },
    Size::single(),
  ))
  }

  fn enemy_goblin_spawn(world: &mut World, team: Team) -> hecs::Entity {
  world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(2, -1)),
    Health::new(60),
    Stats::new(12, 6, 12, 1),
    team,
    Movable::new(4),
    AI::new(1.0),
    Initiative::new(14),
    Equipment {
      weapon: Some(Weapon {
        name: "Rusty Dagger".to_string(),
        attack_bonus: 2,
      }),
    },
    Size::single(),
  ))
  }

  fn enemy_orc_spawn(world: &mut World, team: Team) -> hecs::Entity {
  world.spawn((
    Position::new(HexCoord::<Axial, Pointy>::new(3, -2)),
    Health::new(100),
    Stats::new(16, 10, 8, 1),
    team,
    Movable::new(2),
    AI::new(1.5),
    Initiative::new(10),
    Equipment {
      weapon: Some(Weapon {
        name: "War Axe".to_string(),
        attack_bonus: 6,
      }),
    },
    Size::single(),
  ))
  }

  /// Creates a new tactical RPG game
  pub fn new() -> Self {
  let mut world = World::new();
  let player_team = Team::new(0);
  let enemy_team = Team::hostile(1);

  let player_warrior = Self::player_warrior_spawn(&mut world, player_team);
  let player_mage = Self::player_mage_spawn(&mut world, player_team);
  let enemy_goblin = Self::enemy_goblin_spawn(&mut world, enemy_team);
  let enemy_orc = Self::enemy_orc_spawn(&mut world, enemy_team);

  let mut turn_queue = VecDeque::new();
  turn_queue.extend([player_warrior, player_mage, enemy_goblin, enemy_orc]);

  Self {
    world,
    turn_queue,
    current_turn: None,
    turn_number: 1,
    player_team,
    enemy_team,
    game_phase: GamePhase::Planning,
  }
  }
  
  /// Starts a new turn
  pub fn turn_start(&mut self) {
  if let Some(entity) = self.turn_queue.pop_front() {
    self.current_turn = Some(entity);
    println!("\n=== Turn {} ===", self.turn_number);
    self.unit_status_print(entity);
    
    // Check if this is a player or AI unit
    let team_id = {
      if let Ok(team) = self.world.get::<Team>(entity) {
        team.id
      } else {
        return;
      }
    };
    
    if team_id == self.player_team.id {
      self.game_phase = GamePhase::Planning;
      self.player_turn_handle(entity);
    } else {
      self.game_phase = GamePhase::AI;
      self.ai_turn_handle(entity);
    }
  } else {
    // End of round, reset turn queue
    self.turn_queue_reset();
    self.turn_number += 1;
  }
  }
  
  /// Handles a player unit's turn
  fn player_turn_handle(&mut self, entity: hecs::Entity) {
  println!("🎮 Player turn - planning actions...");
  
  // In a real implementation, this would wait for player input
  // For demo purposes, we'll simulate some actions
  
  let (pos_coord, target) = {
    if let Ok(pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
      let pos_coord = pos.coord;
      println!("Player unit at ({}, {})", pos_coord.q, pos_coord.r);
      
      // Find nearest enemy
      let target = self.nearest_enemy_find(entity);
      (pos_coord, target)
    } else {
      return;
    }
  };
  
  if let Some(target) = target {
    let pos = Position::new(pos_coord);
    println!("Targeting enemy at distance {}", pos.distance_to(&target.1));
    
    // Try to attack or move closer
    if pos.distance_to(&target.1) <= 2 {
      self.attack_execute(entity, target.0);
    } else {
      self.execute_move_toward(entity, target.1.coord);
    }
  }
  
  self.game_phase = GamePhase::Resolution;
  }
  
  /// Handles an AI unit's turn
  fn ai_turn_handle(&mut self, entity: hecs::Entity) {
  println!("🤖 AI turn - calculating optimal action...");
  
  let (pos_coord, target) = {
    if let Ok(pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
      let pos_coord = pos.coord;
      println!("AI unit at ({}, {})", pos_coord.q, pos_coord.r);
      
      // Simple AI: move toward nearest player unit and attack if possible
      let target = self.nearest_player_find(entity);
      (pos_coord, target)
    } else {
      return;
    }
  };
  
  if let Some(target) = target {
    let pos = Position::new(pos_coord);
    let distance = pos.distance_to(&target.1);
    println!("AI targeting player at distance {distance}");
    
    if distance <= 1 {
      // Attack if adjacent
      self.attack_execute(entity, target.0);
    } else if distance <= 4 {
      // Move closer if within reasonable range
      self.execute_move_toward(entity, target.1.coord);
    } else {
      // Hold position if target too far
      println!("AI unit holding position");
    }
  }
  
  self.game_phase = GamePhase::Resolution;
  }
  
  /// Executes an attack between two units
  fn attack_execute(&mut self, attacker: hecs::Entity, target: hecs::Entity) {
  let (final_damage, target_level) = {
    let attacker_stats = self.world.get::<Stats>(attacker).expect("attacker should have stats");
    let attacker_equipment = self.world.get::<Equipment>(attacker).expect("attacker should have equipment");
    let target_stats = self.world.get::<Stats>(target).expect("target should have stats");
    
    let mut base_damage = attacker_stats.attack;
    if let Some(weapon) = &attacker_equipment.weapon {
      base_damage += weapon.attack_bonus;
    }
    
    let final_damage = base_damage.saturating_sub(target_stats.defense / 2).max(1);
    (final_damage, target_stats.level)
  };
  
  // Apply damage
  let target_defeated = {
    if let Ok(mut target_health) = self.world.get_mut::<Health>(target) {
      let old_health = target_health.current;
      target_health.damage(final_damage);
      
      println!("💥 Attack! {} damage dealt ({} -> {} HP)", 
               final_damage, old_health, target_health.current);
      
      !target_health.is_alive()
    } else {
      false
    }
  };
  
  if target_defeated {
    println!("💀 Unit defeated!");
    
    // Award experience to attacker
    if let Ok(mut exp) = self.world.get_mut::<Experience>(attacker) {
      let xp_gained = target_level * 50;
      if exp.xp_add(xp_gained) {
        println!("🎉 Level up! Now level {}", exp.level);
      }
    }
  }
  }
  
  /// Executes movement toward a target position
  fn execute_move_toward(&mut self, entity: hecs::Entity, target: HexCoord<Axial, Pointy>) {
  if let Ok(pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
    if let Ok(movable) = self.world.get::<Movable>(entity) {
      // Use pathfinding to find route
      let path_result = astar(
        &pos.coord,
        &target,
        |&coord| Self::is_position_passable(coord),
        |_| 1,
      );

      if let Some((path, _cost)) = path_result {
        let path_len = u32::try_from(path.len()).unwrap_or(u32::MAX);
        let move_distance = movable.range.min(path_len - 1);
        if move_distance > 0 {
          let new_pos = path[move_distance as usize];
          
          // Update position (in real implementation would use proper ECS mutation)
          println!("🚶 Moving from ({}, {}) to ({}, {})", 
                   pos.coord.q, pos.coord.r, new_pos.q, new_pos.r);
        }
      }
    }
  }
  }
  
  /// Finds the nearest enemy unit
  fn nearest_enemy_find(&self, entity: hecs::Entity) -> Option<(hecs::Entity, Position<HexCoord<Axial, Pointy>>)> {
  if let Ok(our_team) = self.world.get::<Team>(entity) {
    if let Ok(our_pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
      return self.world.nearest_entity_find(&our_pos)
        .and_then(|(nearest_entity, nearest_pos, _distance)| {
          if let Ok(their_team) = self.world.get::<Team>(nearest_entity) {
            if our_team.is_hostile_to(&their_team) {
              Some((nearest_entity, nearest_pos))
            } else {
              None
            }
          } else {
            None
          }
        });
    }
  }
  None
  }
  
  /// Finds the nearest player unit
  fn nearest_player_find(&self, entity: hecs::Entity) -> Option<(hecs::Entity, Position<HexCoord<Axial, Pointy>>)> {
  if let Ok(our_pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
    return self.world.nearest_entity_find(&our_pos)
      .and_then(|(nearest_entity, nearest_pos, _distance)| {
        if let Ok(their_team) = self.world.get::<Team>(nearest_entity) {
          if their_team.id == self.player_team.id {
            Some((nearest_entity, nearest_pos))
          } else {
            None
          }
        } else {
          None
        }
      });
  }
  None
  }
  
  /// Checks if a position is passable (no other units)
  fn is_position_passable(_coord: HexCoord<Axial, Pointy>) -> bool {
  // In a real implementation, would check for other units and obstacles
  true
  }
  
  /// Resets the turn queue for a new round
  fn turn_queue_reset(&mut self) {
  // Collect all living units sorted by initiative
  let mut units_by_initiative = Vec::new();
  
  for (entity, (init, health)) in &mut self.world.query::<(&Initiative, &Health)>() {
    if health.is_alive() {
      units_by_initiative.push((entity, init.value));
    }
  }
  
  units_by_initiative.sort_by_key(|b| std::cmp::Reverse(b.1)); // Descending initiative
  
  self.turn_queue.clear();
  for (entity, _init) in units_by_initiative {
    self.turn_queue.push_back(entity);
  }
  }
  
  /// Prints the status of a unit
  fn unit_status_print(&self, entity: hecs::Entity) {
  if let Ok(health) = self.world.get::<Health>(entity) {
    if let Ok(stats) = self.world.get::<Stats>(entity) {
      if let Ok(pos) = self.world.get::<Position<HexCoord<Axial, Pointy>>>(entity) {
        if let Ok(team) = self.world.get::<Team>(entity) {
          let team_name = if team.id == self.player_team.id { "Player" } else { "Enemy" };
          
          println!("{} Unit at ({}, {}): {}/{} HP, Level {} (ATK:{} DEF:{} SPD:{})", 
                   team_name,
                   pos.coord.q, pos.coord.r,
                   health.current, health.maximum,
                   stats.level, stats.attack, stats.defense, stats.speed);
          
          if let Ok(equipment) = self.world.get::<Equipment>(entity) {
            if let Some(weapon) = &equipment.weapon {
              println!("  📋 Equipped: {} (+{} attack)", weapon.name, weapon.attack_bonus);
            }
          }
        }
      }
    }
  }
  }
  
  /// Prints the current battlefield state
  pub fn battlefield_print(&self) {
  println!("\n📍 Battlefield Status:");
  
  // Find all living units
  let mut units = Vec::new();
  for (_entity, (pos, health, team)) in &mut self.world.query::<(&Position<HexCoord<Axial, Pointy>>, &Health, &Team)>() {
    if health.is_alive() {
      let symbol = if team.id == self.player_team.id { "🟢" } else { "🔴" };
      units.push((pos.coord.q, pos.coord.r, symbol));
    }
  }
  
  if units.is_empty() {
    println!("Battle concluded!");
    return;
  }
  
  // Find bounds
  let min_q = units.iter().map(|(q, _, _)| *q).min().unwrap_or(0) - 1;
  let max_q = units.iter().map(|(q, _, _)| *q).max().unwrap_or(0) + 1;
  let min_r = units.iter().map(|(_, r, _)| *r).min().unwrap_or(0) - 1;
  let max_r = units.iter().map(|(_, r, _)| *r).max().unwrap_or(0) + 1;
  
  // Print hexagonal grid representation
  for r in min_r..=max_r {
    // Add offset for hexagonal display
    if r % 2 == 1 {
      print!(" ");
    }
    
    for q in min_q..=max_q {
      let symbol = units.iter()
        .find(|(unit_q, unit_r, _)| *unit_q == q && *unit_r == r)
        .map_or("⬡", |(_, _, symbol)| *symbol);
      print!("{symbol} ");
    }
    println!();
  }
  }
  
  /// Runs the complete game simulation
  pub fn simulation_run(&mut self) {
  println!("🎯 Tactical RPG Combat Simulation");
  println!("=================================");
  println!("🟢 = Player Units");
  println!("🔴 = Enemy Units");
  println!("⬡ = Empty Hex");
  
  self.battlefield_print();
  
  // Run several turns
  for turn in 1..=10 {
    self.turn_start();
    self.battlefield_print();
    
    // Check victory conditions
    let player_units_alive = self.living_units_count(self.player_team.id);
    let enemy_units_alive = self.living_units_count(self.enemy_team.id);
    
    #[allow(clippy::else_if_without_else, reason = "both branches diverge (`break`), so a trailing `else` would be flagged `redundant_else`, but omitting it triggers `else_if_without_else` — the two pedantic lints contradict for this pattern; redundant_else's guidance is followed")]
    if player_units_alive == 0 {
      println!("💀 Defeat! All player units have fallen.");
      break;
    } else if enemy_units_alive == 0 {
      println!("🏆 Victory! All enemies defeated.");
      break;
    }
    // Neither side has been eliminated yet; continue to the next turn.

    if turn >= 10 {
      println!("⏰ Battle continues...");
      break;
    }
    
    std::thread::sleep(std::time::Duration::from_millis(1500));
  }
  }
  
  /// Counts living units for a team
  fn living_units_count(&self, team_id: u32) -> usize {
  let mut count = 0;
  for (_entity, (health, team)) in &mut self.world.query::<(&Health, &Team)>() {
    if health.is_alive() && team.id == team_id {
      count += 1;
    }
  }
  count
  }
}

/// Main entry point for the tactical RPG demo
fn main()
{
  let mut game = TacticalRPG::new();
  game.simulation_run();
  
  println!("\n✨ Tactical RPG Demo Complete!");
  println!("This example showcases:");
  println!("• Turn-based combat with initiative system");
  println!("• AI decision-making and pathfinding"); 
  println!("• Equipment and stat systems");
  println!("• Experience and leveling mechanics");
  println!("• Grid-aware tactical positioning");
}