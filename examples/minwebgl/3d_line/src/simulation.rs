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
        let dir = bb.normalize();

        if dist < 1e-6
        {
          // Repel overlapping bodies to avoid singularity.
          force += -dir * 10.0;
        }
        else
        {
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