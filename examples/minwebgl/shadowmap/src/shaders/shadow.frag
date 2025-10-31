#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;

uniform vec3 u_light_dir;        // Used for orthographic (directional light)
uniform vec3 u_light_position;   // Used for perspective (point light)
uniform float u_is_orthographic;
uniform float u_light_size;      // World-space light size for penumbra control
uniform sampler2D u_shadow_map;  // Color texture containing depth values (Chrome workaround)

out vec4 frag_color;

// Mathematical constants
const float TWO_PI = 6.283185307179586;
const float GOLDEN_ANGLE = 2.399963229728653;

// Vogel disk sampling for better distribution than grid or circular patterns
vec2 vogel_disk_sample( int sample_index, int num_samples, float angle_offset )
{
  float r = sqrt( float( sample_index ) + 0.5 ) / sqrt( float( num_samples ) );
  float theta = float( sample_index ) * GOLDEN_ANGLE + angle_offset;
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
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip if outside shadow map
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    // Sample depth from color texture (stored in R channel)
    float shadow_depth = texture( u_shadow_map, sample_uv ).r;

    // Only consider depths that would block (closer than receiver)
    // Add small epsilon for floating-point precision tolerance
    const float blocker_epsilon = 0.00001;
    if ( shadow_depth < receiver_depth - blocker_epsilon )
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

  // Clamp bias to prevent extreme values
  bias = clamp( bias, 0.00005, 0.005 );

  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  for ( int i = 0; i < num_samples; ++i )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip samples outside shadow map (treat as fully lit)
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    // Sample depth from color texture (stored in R channel)
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

  // Early exit for back-facing surfaces (surface facing away from light)
  vec3 normal = normalize( v_normal );
  float n_dot_l = dot( normal, light_dir_to_use );
  if ( n_dot_l < 0.0 )
  {
    return 0.0; // Back-facing, no shadow
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

  // Note: XY bounds checking is done per-sample in helper functions
  // This allows soft shadow edges to extend beyond shadow map boundaries

  float receiver_depth = proj_coords.z;

  // === Step 1: Blocker Search ===
  // Use uniform light size for scene-dependent shadow softness control
  float light_world_size = u_light_size;

  float search_radius;
  if ( u_is_orthographic > 0.0 )
  {
    // Orthographic: constant search radius
    search_radius = light_world_size * 15.0;
  }
  else
  {
    // Perspective: scale search radius with depth for better results
    // Closer objects get tighter search, farther objects get wider search
    float depth_scale = mix( 1.0, 2.5, clamp( receiver_depth, 0.0, 1.0 ) );
    search_radius = light_world_size * 30.0 * depth_scale;
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
  if ( avg_blocker_depth < 0.0 )
  {
    return 0.0;
  }

  // === Step 2: Penumbra Size Estimation ===
  float penumbra_width;

  if ( u_is_orthographic > 0.0 )
  {
    // For orthographic: penumbra still varies with blocker-receiver distance
    // This produces physically-correct soft shadows for directional lights
    float blocker_distance = receiver_depth - avg_blocker_depth;
    penumbra_width = blocker_distance * light_world_size * 25.0;
  }
  else
  {
    // For perspective: standard PCSS formula
    float blocker_distance = receiver_depth - avg_blocker_depth;
    penumbra_width = blocker_distance * search_radius / max( avg_blocker_depth, 0.01 );
  }

  penumbra_width = clamp( penumbra_width, 2.0, 150.0 );

  // === Step 3: Percentage Closer Filtering ===
  const int pcf_samples = 64;

  float shadow = pcss_shadow_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    pcf_samples,
    light_dir_to_use
  );

  // === Step 4: Penumbra-based Shadow Intensity ===
  // Make shadows darker when sharp (contact) and lighter when blurry (edges)
  // This correlates intensity with softness for more natural appearance

  // Normalize penumbra width to 0-1 range based on our clamp values (2.0 to 150.0)
  float penumbra_norm = clamp( ( penumbra_width - 2.0 ) / ( 150.0 - 2.0 ), 0.0, 1.0 );

  // Apply power curve for smooth, natural transition
  // Lower exponent = keeps shadows darker longer before lightening
  penumbra_norm = pow( penumbra_norm, 0.6 );

  // Map penumbra softness to intensity
  // Sharp shadows (penumbra_norm ≈ 0) -> dark (intensity ≈ 1.0)
  // Soft/blurry shadows (penumbra_norm ≈ 1) -> light (intensity ≈ 0.3)
  float intensity_factor = mix( 1.0, 0.3, penumbra_norm );

  // Apply intensity modulation to shadow
  shadow *= intensity_factor;

  return shadow;
}

float get_shadow( vec4 light_space_pos )
{
  // Perspective divide to get NDC coordinates [-1, 1]
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;

  float closest_depth = texture( u_shadow_map, proj_coords.xy ).r;
  float current_depth = proj_coords.z;

  float is_outside = float( proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 );

  float shadow = float( current_depth > closest_depth ) + is_outside;
  shadow *= float( !( proj_coords.z > 1.0 ) );

  // vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  return shadow;
}

void main()
{
  // Calculate shadow value: 0 = lit, 1 = shadowed
  // float shadow = calculate_shadow( v_light_space_pos );
  float shadow = get_shadow( v_light_space_pos );
  frag_color = vec4( shadow, 0.0, 0.0, 1.0 );
}
