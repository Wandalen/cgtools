use minwebgl as gl;

#[ derive( Clone, Copy ) ]
pub struct Body
{
  pub position : gl::F32x3,
  pub velocity : gl::F32x3,
  pub mass : f32,
  pub force : gl::F32x3
}
pub struct Simulation
{
  pub bodies : Vec< Body > 
}

impl Simulation 
{
  pub fn new( n_bodies : usize ) -> Self
  {
    let mut bodies = Vec::with_capacity( n_bodies );

    for i in 0..n_bodies
    {
      let body = Body
      {
        position : gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() ),
        velocity : gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() ),
        mass : fastrand::f32() + 0.5,
        force : gl::F32x3::default()
      };

      bodies.push( body );
    }

    Simulation
    {
      bodies
    }
  }  

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
        let dist = bb.mag().max( 0.1 );
        let dir = bb.normalize();

        if dist < 1e-6
        {
          force += -dir * 100.0;
        }
        else 
        {
          force += dir * other_body.mass * body.mass / ( dist * dist );
        }
      }

      force += -body.position * 10.0;
      self.bodies[ i ].force = force;
    }

    for i in 0..self.bodies.len()
    {
      let body = &mut self.bodies[ i ];

      let acc = body.force / body.mass;
      body.velocity += acc * delta_time;
      body.position += body.velocity  * delta_time;
    }
  }  
}