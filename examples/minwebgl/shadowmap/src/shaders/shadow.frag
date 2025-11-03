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
uniform mat4 u_light_view_projection_inv;
uniform sampler2D u_shadow_map;  // Color texture containing depth values (Chrome workaround)

out vec4 frag_color;

const float PI = 3.1415926;
const float TWO_PI = 6.2831853;
const float GOLDEN_ANGLE = 2.3999632;

const float NEAR = 1.0;
const float FAR = 30.0;

float linearize_depth( float depth )
{
  float z = depth * 2.0 - 1.0; // Back to NDC
  return ( 2.0 * NEAR * FAR ) / ( FAR + NEAR - z * ( FAR - NEAR ) );
}

// ═══════════════════════════════════════════════════════════════════════════
// VOGEL DISK SAMPLING
// ═══════════════════════════════════════════════════════════════════════════
// Creates a spiral pattern using golden angle for optimal sample distribution.
// Better than grid patterns (aliasing) or pure random (needs more samples).
//
vec2 vogel_disk_sample( int sample_index, int num_samples, float angle_offset )
{
  float r = sqrt( float( sample_index ) + 0.5 ) / sqrt( float( num_samples ) );
  float theta = float( sample_index ) * GOLDEN_ANGLE + angle_offset;
  return vec2( r * cos( theta ), r * sin( theta ) );
}

// ═══════════════════════════════════════════════════════════════════════════
// INTERLEAVED GRADIENT NOISE
// ═══════════════════════════════════════════════════════════════════════════
// High-quality screen-space noise to rotate sampling patterns per-pixel.
// Breaks up banding artifacts much better than simple hash functions.
//
float interleaved_gradient_noise( vec2 position )
{
  vec3 magic = vec3( 0.06711056, 0.00583715, 52.9829189 );
  return fract( magic.z * fract( dot( position, magic.xy ) ) );
}

// ═══════════════════════════════════════════════════════════════════════════
// BLOCKER SEARCH (PCSS Phase 1)
// ═══════════════════════════════════════════════════════════════════════════
// Searches for shadow-casting objects by sampling the shadow map.
// Returns average depth of blockers, or -1.0 if fully lit (no blockers).
//
// This distance determines penumbra width (shadow softness).
//
float find_blocker_distance( vec2 shadow_uv, float receiver_depth, float search_radius, int num_samples )
{
  float blocker_sum = 0.0;
  float blocker_count = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Rotate sampling pattern per-pixel to reduce banding
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  // Very small epsilon for blocker detection - prevents false positives
  const float BLOCKER_EPSILON = 0.0001;

  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip out-of-bounds samples
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    float shadow_depth = linearize_depth( texture( u_shadow_map, sample_uv ).r );

    // Is this sample closer to light than receiver? Then it's blocking!
    if ( shadow_depth < receiver_depth - BLOCKER_EPSILON )
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

// ═══════════════════════════════════════════════════════════════════════════
// PCF FILTERING (PCSS Phase 3)
// ═══════════════════════════════════════════════════════════════════════════
// Percentage-Closer Filtering: samples shadow map multiple times with computed
// blur radius to create smooth shadow gradients.
//
// Returns: 0.0 = fully lit, 1.0 = fully shadowed
//
float pcf_filter( vec2 shadow_uv, float receiver_depth, float filter_radius, int num_samples, vec3 light_dir_to_use )
{
  float shadow_sum = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // ───────────────────────────────────────────────────────────────────────
  // Adaptive Bias: Prevents shadow acne
  // ───────────────────────────────────────────────────────────────────────
  // Surfaces at steep angles need more bias than flat surfaces
  //
  vec3 normal = normalize( v_normal );
  float cos_theta = max( dot( normal, light_dir_to_use ), 0.0 );


  // float bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);
  // Smaller biases for better contact shadows
  float base_bias = mix(0.0005, 0.0001, u_is_orthographic);
  float slope_bias = mix(0.002, 0.0005, u_is_orthographic);
  float bias = mix(slope_bias, base_bias, cos_theta);

  // Clamp to safe range
  bias = clamp(bias, 0.00005, 0.003);
  bias = 0.0;

  // Rotate pattern per-pixel
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  // Sample in a disk pattern
  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Out of bounds = treat as lit
    if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0)
    {
      continue;
    }

    float shadow_depth = linearize_depth( texture( u_shadow_map, sample_uv ).r );
    shadow_sum += float( (receiver_depth - bias) > shadow_depth );
  }

  return shadow_sum / float(num_samples);
}

// ═══════════════════════════════════════════════════════════════════════════
// PCSS MAIN ALGORITHM
// ═══════════════════════════════════════════════════════════════════════════
// Complete PCSS with contact-hardening shadows:
//   1. Search for blockers → find average occluder depth
//   2. Estimate penumbra → calculate blur radius from geometry
//   3. Filter shadow map → sample with variable-size kernel
//
// Result: Physically-based soft shadows (sharp near contact, soft farther away)
//
float pcss(vec4 light_space_pos)
{
  // ───────────────────────────────────────────────────────────────────────
  // Setup: Determine light direction and validate surface
  // ───────────────────────────────────────────────────────────────────────
  vec3 light_dir_to_use;
  if (u_is_orthographic > 0.5)
  {
    light_dir_to_use = -u_light_dir;
  }
  else
  {
    light_dir_to_use = normalize(u_light_position - v_world_pos);
  }

  // Back-facing surfaces can't receive shadows
  vec3 normal = normalize(v_normal);
  float n_dot_l = dot(normal, light_dir_to_use);
  if (n_dot_l < 0.0)
  {
    return 0.0;
  }

  // Transform to shadow map space [0, 1]
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;

  // Outside shadow map frustum = no shadow
  if (proj_coords.z > 1.0 || proj_coords.z < 0.0)
  {
    return 0.0;
  }

  float receiver_depth = linearize_depth( proj_coords.z );

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 1: BLOCKER SEARCH                                             ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  float light_world_size = u_light_size;

  // Calculate search radius based on projection type
  float search_radius;
  if (u_is_orthographic > 0.5)
  {
    // Directional light: constant search radius
    search_radius = light_world_size * 20.0;
  }
  else
  {
    // Point/spot light: scale with depth
    float normalized_depth = (receiver_depth - NEAR) / (FAR - NEAR);
    float depth_scale = mix(1.0, 3.0, clamp(normalized_depth, 0.0, 1.0));
    search_radius = light_world_size * 40.0 * depth_scale;
  }

  // Quality: 64 samples for blocker search (good balance)
  const int BLOCKER_SAMPLES = 128;

  float avg_blocker_depth = find_blocker_distance
  (
    proj_coords.xy,
    receiver_depth,
    search_radius,
    BLOCKER_SAMPLES
  );

  // No blockers = fully lit (early exit)
  if (avg_blocker_depth < 0.0)
  {
    return 0.0;
  }

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 2: PENUMBRA ESTIMATION                                        ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  // Distance from blocker to receiver
  float blocker_distance = receiver_depth - avg_blocker_depth;
  float penumbra_width;

  if (u_is_orthographic > 0.5)
  {
    // Directional light: penumbra proportional to blocker distance
    penumbra_width = blocker_distance * light_world_size * 30.0;
  }
  else
  {
    // Point/spot light: standard PCSS formula
    // penumbra = (receiver - blocker) * lightSize / blocker
    penumbra_width = ( blocker_distance * search_radius ) / max( avg_blocker_depth, 0.001 );
  }

  // Clamp to reasonable range (in texels)
  penumbra_width = clamp(penumbra_width, 1.0, 250.0);

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 3: PERCENTAGE CLOSER FILTERING                                ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  // Quality: 128 samples for PCF (very smooth shadows)
  const int PCF_SAMPLES = 128;

  float shadow = pcf_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    PCF_SAMPLES,
    light_dir_to_use
  );

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ OPTIONAL: INTENSITY MODULATION                                      ║
  // ╚══════════════════════════════════════════════════════════════════════╝
  // Softer/distant shadows are lighter (artistic & physical approximation)

  // Normalize penumbra to [0, 1]
  float penumbra_norm = clamp( ( penumbra_width - 1.0 ) / ( 250.0 - 1.0 ), 0.0, 1.0 );

  // Smooth falloff curve
  penumbra_norm = pow( penumbra_norm, 0.7 );

  // Sharp shadows = dark (1.0), Soft shadows = lighter (0.35)
  float intensity_factor = mix( 1.0, 0.0, penumbra_norm );
  shadow *= intensity_factor;

  return shadow;
}

void main()
{
  // Calculate PCSS shadow with contact-hardening
  float shadow = pcss( v_light_space_pos );

  // Output: R channel = shadow (0 = lit, 1 = shadowed)
  frag_color = vec4( shadow, 0.0, 0.0, 1.0 );
}
