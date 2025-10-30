#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;
in vec2 v_texcoord;

uniform vec3 u_light_dir;        // Used for orthographic (directional light)
uniform vec3 u_light_position;   // Used for perspective (point light)
uniform float u_is_orthographic;
uniform sampler2D u_shadow_map;

out vec4 frag_color;

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

  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip if outside shadow map
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    float shadow_depth = texture( u_shadow_map, sample_uv ).r;

    // Only consider depths that would block (closer than receiver)
    if ( shadow_depth < receiver_depth )
    {
      blocker_sum += shadow_depth;
      blocker_count += 1.0;
    }
  }

  if ( blocker_count < 0.1 )
  {
    return -1.0; // No blockers found
  }

  return blocker_sum / blocker_count;
}

// PCSS filtering with variable penumbra
float pcss_shadow_filter( vec2 shadow_uv, float receiver_depth, float filter_radius, int num_samples, vec3 light_dir_to_use )
{
  float shadow_sum = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Adaptive bias based on surface angle and projection type
  vec3 normal = normalize( v_normal );
  float cos_theta = max( dot( normal, light_dir_to_use ), 0.0 );

  // Orthographic needs less bias than perspective
  float base_bias = mix( 0.001, 0.0002, u_is_orthographic );
  float slope_bias = mix( 0.003, 0.0008, u_is_orthographic );
  float bias = mix( slope_bias, base_bias, cos_theta );

  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * 6.283185307;

  for ( int i = 0; i < num_samples; ++i )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      shadow_sum += 0.0; // Fully lit outside shadow map
      continue;
    }

    float shadow_depth = texture( u_shadow_map, sample_uv ).r;
    shadow_sum += ( receiver_depth - bias ) > shadow_depth ? 1.0 : 0.0;
  }

  return shadow_sum / float( num_samples );
}

// Main PCSS shadow calculation with contact hardening
float calculate_shadow( vec4 light_space_pos )
{
  // Calculate light direction based on projection type
  vec3 light_dir_to_use;
  if ( u_is_orthographic > 0.0 )
  {
    // Orthographic: uniform directional light
    light_dir_to_use = u_light_dir;
  }
  else
  {
    // Perspective: point light with per-fragment direction
    light_dir_to_use = normalize( u_light_position - v_world_pos );
  }

  // Perspective divide
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;

  // Transform to [0,1] range
  proj_coords = proj_coords * 0.5 + 0.5;

  // Check if outside shadow map
  if ( proj_coords.z > 1.0 || proj_coords.z < 0.0 )
  {
    return 0.0; // No shadow beyond far plane
  }

  // Clamp to valid texture coordinates
  // if( proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 )
  // {
  //   return 0.0;
  // }

  float receiver_depth = proj_coords.z;

  // === Step 1: Blocker Search ===
  const float light_world_size = 20.0;

  float search_radius;
  if ( u_is_orthographic > 0.0 )
  {
    search_radius = light_world_size * 20.0;
  }
  else
  {
    search_radius = light_world_size * 40.0;
  }

  const int blocker_search_samples = 64;

  float avg_blocker_depth = find_blocker_distance
  (
    proj_coords.xy,
    receiver_depth,
    search_radius,
    blocker_search_samples
  );

  // No blockers found = fully lit
  if ( avg_blocker_depth < 0.0 )
  {
    return 0.0;
  }

  // === Step 2: Penumbra Size Estimation ===
  float penumbra_width;

  if ( u_is_orthographic > 0.0 )
  {
    penumbra_width = light_world_size * 20.0;
  }
  else
  {
    float blocker_distance = receiver_depth - avg_blocker_depth;
    penumbra_width = blocker_distance * search_radius / max( avg_blocker_depth, 0.01 );
  }

  penumbra_width = clamp( penumbra_width, 2.0, 150.0 );

  // === Step 3: Percentage Closer Filtering ===
  const int pcf_samples = 128;

  float shadow = pcss_shadow_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    pcf_samples,
    light_dir_to_use
  );

  return shadow;
}

void main()
{
  // Calculate shadow value: 0 = lit, 1 = shadowed
  float shadow = calculate_shadow( v_light_space_pos );

  frag_color = vec4( shadow, 0.0, 0.0, 1.0 );
}
