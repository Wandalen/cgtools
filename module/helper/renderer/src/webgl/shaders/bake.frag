#version 300 es
precision highp float;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;

uniform vec3            u_light_dir;
uniform vec3            u_light_position;
uniform bool            u_is_orthographic;
uniform float           u_light_size;
uniform float           u_near;
uniform float           u_far;
#ifdef GL_FRAGMENT_PRECISION_HIGH
  uniform highp sampler2D u_shadow_map;
#else
  uniform mediump sampler2D u_shadow_map;
#endif

out float frag_color;

const int BLOCKER_SAMPLES = 128;
const int PCF_SAMPLES = 128;

const float TWO_PI = 6.2831853;
const float GOLDEN_ANGLE = 2.3999632;

float linearize_depth( float depth )
{
  if ( u_is_orthographic )
  {
    // Orthographic: depth is already linear in [0,1], just remap to [near, far]
    return depth * ( u_far - u_near ) + u_near;
  }
  else
  {
    // Perspective: apply inverse perspective transformation
    float z = depth * 2.0 - 1.0; // [0,1] → [-1,1] NDC
    return ( 2.0 * u_near * u_far ) / ( u_far + u_near - z * ( u_far - u_near ) );
  }
}

// Creates a spiral pattern using golden angle for optimal sample distribution.
vec2 vogel_disk_sample( int sample_index, int num_samples, float angle_offset )
{
  float r = sqrt( float( sample_index ) + 0.5 ) / sqrt( float( num_samples ) );
  float theta = float( sample_index ) * GOLDEN_ANGLE + angle_offset;
  return vec2( r * cos( theta ), r * sin( theta ) );
}

// High-quality screen-space noise to rotate sampling patterns per-pixel.
// Breaks up banding artifacts much better than simple hash functions.
float interleaved_gradient_noise( vec2 position )
{
  vec3 magic = vec3( 0.06711056, 0.00583715, 52.9829189 );
  return fract( magic.z * fract( dot( position, magic.xy ) ) );
}

// Searches for shadow-casting objects by sampling the shadow map.
// Returns average depth of blockers, or -1.0 if fully lit (no blockers).
//
// This distance determines penumbra width (shadow softness).
float find_blocker_distance( vec2 proj_coords, float receiver_depth, float search_radius, int num_samples )
{
  float blocker_sum = 0.0;
  float blocker_count = 0.0;

  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Rotate sampling pattern per-pixel to reduce banding
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  float depth_range = u_far - u_near;
  float blocker_epsilon = depth_range * 0.0002; // 0.02% of depth range - small but sufficient
  blocker_epsilon = 0.0;

  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = proj_coords + offset * texel_size;

    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    float shadow_depth = linearize_depth( texture( u_shadow_map, sample_uv ).r );

    if ( shadow_depth < receiver_depth - blocker_epsilon )
    {
      blocker_sum += shadow_depth;
      blocker_count += 1.0;
    }
  }

  // No blockers = fully lit
  if ( blocker_count < 0.5 )
  {
    return -1.0;
  }

  return blocker_sum / blocker_count;
}

// Percentage-Closer Filtering: samples shadow map multiple times with computed
// blur radius to create smooth shadow gradients.
//
// Returns: 0.0 = fully lit, 1.0 = fully shadowed
float pcf_filter( vec2 proj_coords, float receiver_depth, float filter_radius, int num_samples )
{
  float shadow_sum = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Rotate pattern per-pixel
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  // Sample in a disk pattern
  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = proj_coords + offset * texel_size;

    // Out of bounds = treat as lit
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    float shadow_depth = linearize_depth( texture( u_shadow_map, sample_uv ).r );
    shadow_sum += float( ( receiver_depth ) > shadow_depth );
  }

  return shadow_sum / float( num_samples );
}

// Complete PCSS with contact-hardening shadows:
//   1. Search for blockers → find average occluder depth
//   2. Estimate penumbra → calculate blur radius from geometry
//   3. Filter shadow map → sample with variable-size kernel
float pcss( vec4 light_space_pos )
{
  vec3 light_dir_to_use = u_is_orthographic ? -u_light_dir : normalize( u_light_position - v_world_pos );

  // Back-facing surfaces can't receive shadows
  vec3 normal = normalize( v_normal );
  float n_dot_l = dot( normal, light_dir_to_use );
  if ( n_dot_l < 0.0 )
  {
    return 0.0;
  }

  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;

  // Outside shadow map frustum = no shadow
  if ( proj_coords.x > 1.0 || proj_coords.x < 0.0
    || proj_coords.y > 1.0 || proj_coords.y < 0.0
    || proj_coords.z > 1.0 || proj_coords.z < 0.0 )
  {
    return 0.0;
  }

  float receiver_depth = linearize_depth( proj_coords.z );

  float light_world_size = u_light_size;

  float search_radius;
  if ( u_is_orthographic )
  {
    search_radius = light_world_size * 100.0;
  }
  else
  {
    float normalized_depth = ( receiver_depth - u_near ) / ( u_far - u_near );
    float depth_scale = mix( 3.0, 1.0, normalized_depth );
    search_radius = light_world_size * 200.0 * depth_scale;
  }

  float avg_blocker_depth = find_blocker_distance
  (
    proj_coords.xy,
    receiver_depth,
    search_radius,
    BLOCKER_SAMPLES
  );

  float blocker_distance = receiver_depth - avg_blocker_depth;
  float min_blocker_dist = ( u_far - u_near ) * 0.002; // 0.2% of depth range
  blocker_distance = max( blocker_distance, min_blocker_dist );

  float penumbra_width;

  if ( u_is_orthographic )
  {
    // Directional light: penumbra proportional to blocker distance
    // Multiplier adjusted for linearized depth (world-space units)
    penumbra_width = blocker_distance * light_world_size * 300.0;
  }
  else
  {
    // Point/spot light: standard PCSS formula
    // penumbra = (receiver - blocker) * lightSize / blocker
    float perspective_scale = 200.0;
    penumbra_width = ( blocker_distance * light_world_size * perspective_scale ) / max( avg_blocker_depth, 0.01 );
  }

  // Clamp to reasonable range (in texels)
  float min_penumbra = 1.0;
  float max_penumbra = 300.0;

  penumbra_width = clamp( penumbra_width, min_penumbra, max_penumbra );

  float shadow = pcf_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    PCF_SAMPLES
  );

  // Normalize penumbra to [0, 1]
  float penumbra_norm = ( penumbra_width - min_penumbra ) / ( max_penumbra - min_penumbra );

  // Adaptive falloff based on light size
  // Larger lights = faster fade (more light scattering), smaller lights = slower fade
  // Exponent < 1.0 = aggressive fade (lighter sooner), > 1.0 = slower fade (darker longer)
  float light_size_normalized = clamp( light_world_size / 0.3, 0.0, 1.0 );
  float fade_exponent = mix( 1.5, 0.5, light_size_normalized );
  penumbra_norm = pow( penumbra_norm, fade_exponent );

  // Contact shadows = dark (1.0), Distant/blurry shadows = lighter (min_shadow_intensity)
  float intensity_factor = mix( 1.0, 0.0, penumbra_norm );
  shadow *= intensity_factor;

  return shadow;
}

void main()
{
  float shadow = pcss( v_light_space_pos );

  // (0 = lit, 1 = shadowed)
  frag_color = shadow;
}
