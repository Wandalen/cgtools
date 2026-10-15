//! Simple N-body gravitational simulation.
//!
//! Bodies attract each other according to Newton's law of gravitation,
//! with a central restoring force that keeps them from drifting off screen.
//! The resulting trajectories are used as 3D line trail data.

use minwebgl as gl;

/// A point mass with position, velocity, and accumulated force.
#[ derive( Clone, Copy ) ]
pub struct Body
{
  /// Current position in 3D space.
  pub position : gl::F32x3,
  /// Current velocity vector.
  pub velocity : gl::F32x3,
  /// Mass of the body (affects gravitational attraction and inertia).
  pub mass : f32,
  /// Net force accumulated during the current simulation step.
  pub force : gl::F32x3
}

/// Runs the N-body gravitational simulation each frame.
pub struct Simulation
{
  /// The set of gravitationally interacting bodies.
  pub bodies : Vec< Body >
}

impl Simulation 
{
  /// Creates a simulation with `n_bodies` bodies at random positions and velocities.
  ///
  /// Positions are uniformly distributed in a small cube around the origin,
  /// velocities are unit-length in a random direction, and masses range from 1.0 to 2.0.
  pub fn new( n_bodies : usize ) -> Self
  {
    let mut bodies = Vec::with_capacity( n_bodies );

    for _ in 0..n_bodies
    {
      let pos = gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() ) * 2.0 - 1.0;
      let velocity = gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() ) * 2.0 - 1.0;

      let body = Body
      {
        position : pos / 5.0,
        velocity : velocity.normalize(),
        mass : fastrand::f32() * 1.0 + 1.0,
        force : gl::F32x3::default()
      };

      bodies.push( body );
    }

    Simulation
    {
      bodies
    }
  }  

  /// Advances the simulation by one step.
  ///
  /// The simulation proceeds in two phases:
  /// 1. **Force accumulation** — for each body, compute the net gravitational
  ///    pull from every other body (`F = G * m1 * m2 / r^2`). A repulsion
  ///    kick is applied when bodies are nearly overlapping (`dist < 1e-6`).
  ///    Forces are clamped to unit magnitude to prevent explosions, and a
  ///    restoring spring (`-5 * position`) pulls bodies back toward the origin.
  /// 2. **Integration** — velocity and position are updated via explicit Euler,
  ///    with velocity clamped to unit length for stability.
  pub fn simulate( &mut self, delta_time : f32 )
  {
    // Phase 1: accumulate forces.
    for i in 0..self.bodies.len()
    {
      let mut force = gl::F32x3::default();
      let body = &self.bodies[ i ];

      for k in 0..self.bodies.len()
      {
        if i == k { continue; }
        let other_body = self.bodies[ k ];

        let bb = other_body.position - body.position;
        let dist = bb.mag();

        if dist < 1e-6
        {
          // Fix(BUG-457): removed the unconditional `dir = bb.normalize()`
          // that previously ran before this guard.
          // Root cause: `bb.normalize()` on a near-zero vector is `0.0/0.0`
          // -> NaN; the repel branch below then multiplied that NaN through
          // `force`, and NaN comparisons are always false, so no later
          // magnitude clamp could ever catch it.
          // Pitfall: a "singularity guard" doesn't actually guard anything
          // if the branch it protects reads a value computed *before* the
          // guard ran -- check every value a guarded branch uses is itself
          // safe for the exact input the guard exists to catch.
          // Repel overlapping bodies along a fixed axis -- direction is
          // genuinely undefined for two coincident bodies, so an arbitrary
          // constant axis stands in for the (otherwise NaN) normalized
          // separation.
          force += gl::F32x3::new( 1.0, 0.0, 0.0 ) * 10.0;
        }
        else
        {
          let dir = bb.normalize();
          // Standard gravitational attraction (with a constant multiplier).
          force += 15.0 * dir * other_body.mass * body.mass / ( dist * dist );
        }
      }

      // Clamp force magnitude to prevent numerical explosions.
      if force.mag() > 1.0
      {
        force = force.normalize();
      }

      // Central restoring force keeps the system from drifting.
      force += body.position * -5.0;

      self.bodies[ i ].force = force;
    }

    // Phase 2: integrate velocity and position (explicit Euler).
    for i in 0..self.bodies.len()
    {
      let body = &mut self.bodies[ i ];

      let acc = body.force / body.mass;
      body.velocity += acc * delta_time * 15.0;

      // Clamp velocity for stability.
      if body.velocity.mag() > 1.0 { body.velocity = body.velocity.normalize(); }

      body.position += body.velocity  * delta_time * 15.0;
    }
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// ## Root Cause
  /// `dir = bb.normalize()` ran unconditionally before the `dist < 1e-6`
  /// "avoid singularity" guard. `normalize()` divides by magnitude with no
  /// zero-check, so two bit-exact-same-position bodies (`bb` = the zero
  /// vector, `dist` = `0.0`) produced `dir` = NaN in every component --
  /// poisoning the guard's own "repel" branch (`force += -dir * 10.0`),
  /// which existed specifically to handle this exact case. NaN then
  /// propagated through `force`, `body.velocity`, and `body.position`; later
  /// magnitude clamps (`if force.mag() > 1.0 { .. }`) never caught it
  /// because every NaN comparison is `false`.
  ///
  /// ## Why Not Caught
  /// The crate had zero tests before this one, and the defect only
  /// manifests for the specific degenerate input of two bodies occupying
  /// the exact same position -- never reached by `Simulation::new`'s random
  /// initial placement during normal demo use.
  ///
  /// ## Fix Applied
  /// Moved `dir = bb.normalize()` into the `else` (standard-attraction)
  /// branch, where `dist >= 1e-6` guarantees a safe division. The `if`
  /// (repel) branch no longer reads `dir` at all -- it pushes apart along a
  /// fixed axis instead, since direction is genuinely undefined for two
  /// exactly-coincident bodies.
  ///
  /// ## Prevention
  /// This test places two bodies at the exact same position and asserts
  /// `simulate()` produces no NaN/Inf in any resulting force, velocity, or
  /// position -- the general invariant the fix restores, not a pinned
  /// per-value expectation.
  ///
  /// ## Pitfall
  /// A "singularity guard" doesn't actually guard anything if the branch it
  /// protects reads a value computed *before* the guard ran -- always
  /// double-check every value a guarded branch uses is itself safe for the
  /// exact input the guard exists to catch.
  #[ test ]
  fn bug_reproducer_bug_457_coincident_bodies_no_nan()
  {
    let mut sim = Simulation
    {
      bodies : vec!
      [
        Body { position : gl::F32x3::new( 0.0, 0.0, 0.0 ), velocity : gl::F32x3::default(), mass : 1.0, force : gl::F32x3::default() },
        Body { position : gl::F32x3::new( 0.0, 0.0, 0.0 ), velocity : gl::F32x3::default(), mass : 1.0, force : gl::F32x3::default() },
      ]
    };

    sim.simulate( 0.016 );

    for ( i, body ) in sim.bodies.iter().enumerate()
    {
      assert!( body.position.mag().is_finite(), "body {i} position is NaN/Inf after simulate() on coincident bodies" );
      assert!( body.velocity.mag().is_finite(), "body {i} velocity is NaN/Inf after simulate() on coincident bodies" );
      assert!( body.force.mag().is_finite(), "body {i} force is NaN/Inf after simulate() on coincident bodies" );
    }
  }
}