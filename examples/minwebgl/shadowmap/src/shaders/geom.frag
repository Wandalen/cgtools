#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;

uniform vec3 u_light_dir;
uniform vec3 u_view_pos;
uniform vec3 u_light_color;
uniform vec3 u_object_color;
uniform float u_is_orthographic;
uniform sampler2D u_shadow_map;

out vec4 frag_color;

float inverse_lerp( float v, float min_value, float max_value )
{
  return ( v - min_value ) / ( max_value  - min_value );
}

float remap( float v, float in_min, float in_max, float out_min, float out_max )
{
  float t = inverse_lerp( v, in_min, in_max );
  return mix( out_min, out_max, t );
}

// Vogel disk sampling for better distribution than grid or circular patterns
vec2 vogel_disk_sample( int sample_index, int num_samples, float angle_offset )
{
  const float golden_angle = 2.399963229728653;
  float r = sqrt( float( sample_index ) + 0.5 ) / sqrt( float( num_samples ) );
  float theta = float( sample_index ) * golden_angle + angle_offset;
  return vec2( r * cos( theta ), r * sin( theta ) );
}

// High-quality noise for dithering
float interleaved_gradient_noise( vec2 position )
{
  vec3 magic = vec3( 0.06711056, 0.00583715, 52.9829189 );
  return fract( magic.z * fract( dot( position, magic.xy ) ) );
}

// Find average blocker depth for PCSS (contact hardening)
float find_blocker_distance( vec2 shadow_uv, float receiver_depth, float search_radius, int num_samples )
{
  float blocker_sum = 0.0;
  float blocker_count = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Use different angle offset for each pixel to reduce banding
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * 6.283185307;

  for( int i = 0; i < num_samples; ++i )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip if outside shadow map
    if( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    float shadow_depth = texture( u_shadow_map, sample_uv ).r;

    // Only consider depths that would block (closer than receiver)
    if( shadow_depth < receiver_depth )
    {
      blocker_sum += shadow_depth;
      blocker_count += 1.0;
    }
  }

  if( blocker_count < 0.1 )
  {
    return -1.0; // No blockers found
  }

  return blocker_sum / blocker_count;
}

// PCSS filtering with variable penumbra
float pcss_shadow_filter( vec2 shadow_uv, float receiver_depth, float filter_radius, int num_samples )
{
  float shadow_sum = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Adaptive bias based on surface angle and projection type
  // Using back-face shadow mapping, so we can use much smaller bias
  vec3 normal = normalize( v_normal );
  vec3 light_dir = u_light_dir;  // Use actual light direction from shadow frustum
  float cos_theta = max( dot( normal, light_dir ), 0.0 );

  // Orthographic needs less bias than perspective
  float base_bias = mix( 0.001, 0.0002, u_is_orthographic );
  float slope_bias = mix( 0.003, 0.0008, u_is_orthographic );
  float bias = mix( slope_bias, base_bias, cos_theta );

  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * 6.283185307;

  for( int i = 0; i < num_samples; ++i )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    if( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      shadow_sum += 0.0; // Fully lit outside shadow map
      continue;
    }

    float shadow_depth = texture( u_shadow_map, sample_uv ).r;
    shadow_sum += ( receiver_depth - bias ) > shadow_depth ? 1.0 : 0.0;
  }

  return shadow_sum / float( num_samples );
}

// Main PCSS shadow calculation with contact hardening (supports both ortho and perspective)
float calculate_shadow( vec4 light_space_pos )
{
  // Perspective divide (for perspective projection, w=1 for orthographic)
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;

  // Transform to [0,1] range
  proj_coords = proj_coords * 0.5 + 0.5;

  // Check if outside shadow map
  if( proj_coords.z > 1.0 || proj_coords.z < 0.0 )
  {
    return 0.0; // No shadow beyond far plane
  }

  // Clamp to valid texture coordinates
  if( proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 )
  {
    return 0.0;
  }

  float receiver_depth = proj_coords.z;

  // === Step 1: Blocker Search ===
  // Light size in world space units (represents physical light source size)
  // For orthographic: this is the actual light radius
  // For perspective: this scales with distance
  const float light_world_size = 1.0;  // Adjust this to control shadow softness

  // Convert world space light size to texel space
  // For orthographic: fixed relationship
  // For perspective: varies with depth
  float search_radius;
  if( u_is_orthographic > 0.0 )
  {
    // Orthographic: constant search radius
    search_radius = light_world_size * 20.0;  // Texels
  }
  else
  {
    // Perspective: search radius grows with distance from light
    search_radius = light_world_size * 40.0;
  }

  const int blocker_search_samples = 32;

  float avg_blocker_depth = find_blocker_distance
  (
    proj_coords.xy,
    receiver_depth,
    search_radius,
    blocker_search_samples
  );

  // No blockers found = fully lit
  if( avg_blocker_depth < 0.0 )
  {
    return 0.0;
  }

  // === Step 2: Penumbra Size Estimation ===
  float penumbra_width;

  if( u_is_orthographic > 0.0 )
  {
    // Orthographic: penumbra size is constant (no perspective)
    // Based on light size only
    penumbra_width = light_world_size * 20.0;
  }
  else
  {
    // Perspective: contact hardening based on distance
    // Penumbra = (receiver - blocker) * light_size / blocker
    float blocker_distance = receiver_depth - avg_blocker_depth;
    penumbra_width = blocker_distance * search_radius / max( avg_blocker_depth, 0.01 );
  }

  // Clamp to reasonable range
  penumbra_width = clamp( penumbra_width, 2.0, 150.0 );

  // === Step 3: Percentage Closer Filtering ===
  // More samples for better quality (since it's precomputed once)
  const int pcf_samples = 128;

  float shadow = pcss_shadow_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    pcf_samples
  );

  return shadow;
}

void main()
{
  // Calculate normalized surface normal (needed for both lighting and shadow bias)
  vec3 norm = normalize( v_normal );

  float shadow = calculate_shadow( v_light_space_pos );
  // shadow = 0.0;
  // Diffuse (use actual light direction from shadow frustum)
  vec3 light_dir_diffuse = -u_light_dir;
  float diff = max( dot( norm, light_dir_diffuse ), 0.0 );
  vec3 diffuse = diff * u_light_color;

  // Hemi-light
  vec3 sky_color = vec3( 0.0, 0.2, 0.4 );
  vec3 ground_color = vec3( 0.1, 0.05, 0.0 );
  float hemi_mix = remap( norm.y, -1.0, 1.0, 0.0, 1.0 );
  vec3 hemi = mix( ground_color, sky_color, hemi_mix );
  hemi = vec3( 0.0 );
  // Specular (Blinn-Phong)
  float specular_strength = 0.1;
  vec3 view_dir = normalize( u_view_pos - v_world_pos );
  vec3 halfway_dir = normalize( light_dir_diffuse + view_dir );
  float spec = pow( max( dot( norm, halfway_dir ), 0.0 ), 4.0 );
  vec3 specular = specular_strength * spec * u_light_color;

  // Apply shadow (only affects direct lighting, not hemi)
  vec3 result = ( hemi + ( diffuse + specular ) * ( 1.0 - shadow ) ) * u_object_color;
  frag_color = vec4( pow( result, vec3( 1.0 / 2.2 ) ), 1.0 );
}
