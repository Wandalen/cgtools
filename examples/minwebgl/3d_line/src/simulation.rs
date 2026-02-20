use minwebgl as gl;

/// A body for physical interaction
#[ derive( Clone, Copy ) ]
pub struct Body
{
  /// Position of the body
  pub position : gl::F32x3,
  /// Velocity of the body
  pub velocity : gl::F32x3,
  /// Mass of the body
  pub mass : f32,
  /// Force being apllied to the body
  pub force : gl::F32x3
}

/// Controls the simulation logic
pub struct Simulation
{
  /// Bodies that attract eachother
  pub bodies : Vec< Body > 
}

impl Simulation 
{
  /// Initializes the simulation
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

  /// Simulation the interaction between the bodies
  pub fn simulate( &mut self, delta_time : f32 )
  {
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
          force += -dir * 10.0;
        }
        else 
        {
          force += 15.0 * dir * other_body.mass * body.mass / ( dist * dist );
        }
      }

      if force.mag() > 1.0
      {
        force = force.normalize();
      }

      force += body.position * -5.0;

      self.bodies[ i ].force = force;
    }

    for i in 0..self.bodies.len()
    {
      let body = &mut self.bodies[ i ];

      let acc = body.force / body.mass;
      body.velocity += acc * delta_time * 15.0;
      
      if body.velocity.mag() > 1.0 { body.velocity = body.velocity.normalize(); }

      body.position += body.velocity  * delta_time * 15.0;
    }
  }  
}