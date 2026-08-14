mod private
{
  use bytemuck::Zeroable as _;

  /// Capacity of the point light array, mirrored by `shaders/main.wgsl`.
  pub const MAX_POINT_LIGHTS : usize = 8;
  /// Capacity of the directional light array, mirrored by `shaders/main.wgsl`.
  pub const MAX_DIRECT_LIGHTS : usize = 8;
  /// Capacity of the spot light array, mirrored by `shaders/main.wgsl`.
  pub const MAX_SPOT_LIGHTS : usize = 8;

  /// GPU layout of `PointLight` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct PointLightRaw
  {
    /// xyz — position; w — range.
    pub position_range : [ f32; 4 ],
    /// xyz — color; w — strength.
    pub color_strength : [ f32; 4 ]
  }

  /// GPU layout of `DirectLight` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct DirectLightRaw
  {
    /// xyz — unit direction from the surface toward the light; w — strength.
    pub direction_strength : [ f32; 4 ],
    /// xyz — color; w — unused.
    pub color : [ f32; 4 ]
  }

  /// GPU layout of `SpotLight` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct SpotLightRaw
  {
    /// xyz — position; w — range.
    pub position_range : [ f32; 4 ],
    /// xyz — unit cone axis, pointing away from the light; w — strength.
    pub direction_strength : [ f32; 4 ],
    /// xyz — color; w — inner cone angle ( radians ).
    pub color_inner : [ f32; 4 ],
    /// x — outer cone angle ( radians ); yzw — unused.
    pub outer : [ f32; 4 ]
  }

  /// GPU layout of `LightsUniform` in `shaders/main.wgsl`.
  #[ repr( C ) ]
  #[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
  pub struct LightsRaw
  {
    /// x — point count; y — direct count; z — spot count; w — unused.
    pub counts : [ u32; 4 ],
    /// Point lights; only the first `counts[ 0 ]` entries are meaningful.
    pub point : [ PointLightRaw; MAX_POINT_LIGHTS ],
    /// Directional lights; only the first `counts[ 1 ]` entries are meaningful.
    pub direct : [ DirectLightRaw; MAX_DIRECT_LIGHTS ],
    /// Spot lights; only the first `counts[ 2 ]` entries are meaningful.
    pub spot : [ SpotLightRaw; MAX_SPOT_LIGHTS ]
  }

  fn normalized( v : [ f32; 3 ] ) -> [ f32; 3 ]
  {
    let len = ( v[ 0 ] * v[ 0 ] + v[ 1 ] * v[ 1 ] + v[ 2 ] * v[ 2 ] ).sqrt();
    if len > 0.0
    {
      [ v[ 0 ] / len, v[ 1 ] / len, v[ 2 ] / len ]
    }
    else
    {
      v
    }
  }

  /// CPU-side light list, packed for the lights uniform of the opaque shader.
  #[ derive( Clone ) ]
  pub struct Lights
  {
    raw : LightsRaw
  }

  impl Default for Lights
  {
    fn default() -> Self
    {
      Self { raw : LightsRaw::zeroed() }
    }
  }

  impl Lights
  {
    /// An empty light list.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Removes every light.
    pub fn clear( &mut self )
    {
      self.raw.counts = [ 0; 4 ];
    }

    /// Adds a point light. Returns `false` — dropping the light — when the
    /// array is already at `MAX_POINT_LIGHTS`.
    #[ must_use ]
    pub fn point_push( &mut self, position : [ f32; 3 ], color : [ f32; 3 ], strength : f32, range : f32 ) -> bool
    {
      let i = self.raw.counts[ 0 ] as usize;
      if i >= MAX_POINT_LIGHTS
      {
        return false;
      }
      self.raw.point[ i ] = PointLightRaw
      {
        position_range : [ position[ 0 ], position[ 1 ], position[ 2 ], range ],
        color_strength : [ color[ 0 ], color[ 1 ], color[ 2 ], strength ]
      };
      self.raw.counts[ 0 ] += 1;
      true
    }

    /// Adds a directional light. `direction` points from the surface toward
    /// the light ( the WebGL uniform semantic ); normalized internally.
    /// Returns `false` — dropping the light — when the array is full.
    #[ must_use ]
    pub fn direct_push( &mut self, direction : [ f32; 3 ], color : [ f32; 3 ], strength : f32 ) -> bool
    {
      let i = self.raw.counts[ 1 ] as usize;
      if i >= MAX_DIRECT_LIGHTS
      {
        return false;
      }
      let direction = normalized( direction );
      self.raw.direct[ i ] = DirectLightRaw
      {
        direction_strength : [ direction[ 0 ], direction[ 1 ], direction[ 2 ], strength ],
        color : [ color[ 0 ], color[ 1 ], color[ 2 ], 0.0 ]
      };
      self.raw.counts[ 1 ] += 1;
      true
    }

    /// Adds a spot light. `direction` is the cone axis pointing away from the
    /// light; normalized internally. Cone angles are radians from the axis,
    /// `inner_cone_angle <= outer_cone_angle`. Returns `false` — dropping the
    /// light — when the array is full.
    #[ expect( clippy::too_many_arguments, reason = "a spot light irreducibly needs position, axis, color, strength, range, and both cone angles" ) ]
    #[ must_use ]
    pub fn spot_push
    (
      &mut self,
      position : [ f32; 3 ],
      direction : [ f32; 3 ],
      color : [ f32; 3 ],
      strength : f32,
      range : f32,
      inner_cone_angle : f32,
      outer_cone_angle : f32
    ) -> bool
    {
      let i = self.raw.counts[ 2 ] as usize;
      if i >= MAX_SPOT_LIGHTS
      {
        return false;
      }
      let direction = normalized( direction );
      self.raw.spot[ i ] = SpotLightRaw
      {
        position_range : [ position[ 0 ], position[ 1 ], position[ 2 ], range ],
        direction_strength : [ direction[ 0 ], direction[ 1 ], direction[ 2 ], strength ],
        color_inner : [ color[ 0 ], color[ 1 ], color[ 2 ], inner_cone_angle ],
        outer : [ outer_cone_angle, 0.0, 0.0, 0.0 ]
      };
      self.raw.counts[ 2 ] += 1;
      true
    }

    /// The packed uniform contents.
    #[ must_use ]
    pub fn as_raw( &self ) -> LightsRaw
    {
      self.raw
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    MAX_POINT_LIGHTS,
    MAX_DIRECT_LIGHTS,
    MAX_SPOT_LIGHTS,
    PointLightRaw,
    DirectLightRaw,
    SpotLightRaw,
    LightsRaw,
    Lights
  };
}
