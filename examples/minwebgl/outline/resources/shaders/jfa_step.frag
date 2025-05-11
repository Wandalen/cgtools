#version 300 es
precision highp float;
in vec2 v_tex_coord;
out vec4 FragColor; // Outputting vec4 for RGBA32F texture
uniform sampler2D u_jfa_texture; // Input: JFA texture from previous step
uniform vec2 u_resolution;
uniform vec2 u_step_size; // Current jump distance

void main() 
{
  float best_distance = 1e20;
  vec2 best_coord = vec2( -1.0 );

  for ( int y = -1; y <= 1; ++y ) 
  {
    for ( int x = -1; x <= 1; ++x ) 
    {   
      vec2 offset = ceil( vec2( float( x ), float( y ) ) * u_step_size ) / u_resolution;
      
      vec2 sample_coord = v_tex_coord + offset;

      vec2 seed_coord = texture( u_jfa_texture, sample_coord ).xy;

      if ( seed_coord.x > -0.01 && seed_coord.x < 0.01 )
      {
        continue;
      }

      float dist = distance( seed_coord * u_resolution * 100.0, v_tex_coord * u_resolution * 100.0 );

      if ( dist < best_distance )
      {
        best_distance = dist;
        best_coord = seed_coord;
      } 
    }
  }

  FragColor = vec4( best_coord, 0.0, 1.0 );
}