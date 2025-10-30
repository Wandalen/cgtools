#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_depth_texture;
uniform float u_near;
uniform float u_far;

out vec4 frag_color;

// Linearize depth for better visualization
float linearize_depth( float depth, float near, float far )
{
  // For perspective projection
  float z = depth * 2.0 - 1.0; // Back to NDC
  return ( 2.0 * near * far ) / ( far + near - z * ( far - near ) );
}

// Apply color gradient for better depth perception
vec3 depth_to_color( float depth )
{
  // Blue (near) -> Cyan -> Green -> Yellow -> Red (far)
  vec3 color;

  if ( depth < 0.25 )
  {
    // Blue to Cyan
    float t = depth / 0.25;
    color = mix( vec3( 0.0, 0.0, 1.0 ), vec3( 0.0, 1.0, 1.0 ), t );
  }
  else if ( depth < 0.5 )
  {
    // Cyan to Green
    float t = ( depth - 0.25 ) / 0.25;
    color = mix( vec3( 0.0, 1.0, 1.0 ), vec3( 0.0, 1.0, 0.0 ), t );
  }
  else if ( depth < 0.75 )
  {
    // Green to Yellow
    float t = ( depth - 0.5 ) / 0.25;
    color = mix( vec3( 0.0, 1.0, 0.0 ), vec3( 1.0, 1.0, 0.0 ), t );
  }
  else
  {
    // Yellow to Red
    float t = ( depth - 0.75 ) / 0.25;
    color = mix( vec3( 1.0, 1.0, 0.0 ), vec3( 1.0, 0.0, 0.0 ), t );
  }

  return color;
}

void main()
{
  // Sample raw depth
  float depth = texture( u_depth_texture, v_uv ).r;

  // === LIGHTMAP MODE: Display shadow values directly ===
  // Shadow values: 0 = lit (white), 1 = shadowed (black)
  // Invert so lit areas appear bright
  vec3 color = vec3( 1.0 - depth );

  // === OLD: Depth visualization (commented out) ===
  // Linearize for better visualization (perspective projection)
  // float linear_depth = linearize_depth( depth, u_near, u_far );
  // linear_depth = linear_depth / u_far; // Normalize to [0, 1]

  // Option 1: Grayscale (simple)
  // vec3 color = vec3( linear_depth );

  // Option 2: Color gradient (better depth perception)
  // vec3 color = depth_to_color( depth );

  // Option 3: Raw depth (for orthographic)
  // vec3 color = vec3( depth );

  frag_color = vec4( color, 1.0 );
}
