#version 300 es
precision mediump float;

out vec4 frag_color;

in vec3 v_world_position;
in vec3 v_world_normal;
in vec2 v_texcoord;

uniform vec3 u_points[ 4 ];
uniform float u_light_intensity;
uniform vec3 u_light_color;
uniform bool u_two_sided;
uniform vec3 u_light_position;

uniform sampler2D u_base_color;
uniform sampler2D u_metallic_roughness;

uniform vec3 u_view_position;
uniform sampler2D u_LTC1; // for inverse M
uniform sampler2D u_LTC2; // GGX norm, fresnel, 0(unused), sphere

const float LUT_SIZE  = 64.0; // ltc_texture size
const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
const float LUT_BIAS  = 0.5 / LUT_SIZE;

// Vector form without project to the plane (dot with the normal)
// Use for proxy sphere clipping
vec3 integrate_edge_vec( vec3 v1, vec3 v2 )
{
  // Using built-in acos() function will result flaws
  // Using fitting result for calculating acos()
  float x = dot( v1, v2 );
  float y = abs( x );

  float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
  float b = 3.4175940 + ( 4.1616724 + y ) * y;
  float v = a / b;

  float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;

  return cross( v1, v2 ) * theta_sintheta;
}

// P is fragPos in world space (LTC distribution)
vec3 ltc_evaluate( vec3 N, vec3 V, vec3 P, mat3 m_inv, vec3 points[ 4 ], bool two_sided )
{
  // construct orthonormal basis around N
  vec3 T1 = normalize( V - N * dot( V, N ) );
  vec3 T2 = cross( N, T1 );

  // rotate area light in (T1, T2, N) basis
  m_inv = m_inv * transpose( mat3( T1, T2, N ) );

  // polygon (allocate 4 vertices for clipping)
  vec3 L[ 4 ];
  // transform polygon from LTC back to origin Do (cosine weighted)
  L[ 0 ] = m_inv * ( points[ 0 ] - P );
  L[ 1 ] = m_inv * ( points[ 1 ] - P );
  L[ 2 ] = m_inv * ( points[ 2 ] - P );
  L[ 3 ] = m_inv * ( points[ 3 ] - P );

  // use tabulated horizon-clipped sphere
  // check if the shading point is behind the light
  vec3 dir = points[ 0 ] - P; // LTC space
  vec3 light_normal = cross( points[ 1 ] - points[ 0 ], points[ 3 ] - points[ 0 ] );
  bool behind = ( dot( dir, light_normal ) < 0.0 );

  // cos weighted space
  L[ 0 ] = normalize( L[ 0 ] );
  L[ 1 ] = normalize( L[ 1 ] );
  L[ 2 ] = normalize( L[ 2 ] );
  L[ 3 ] = normalize( L[ 3 ] );

  // integrate
  vec3 vsum = vec3( 0.0 );
  vsum += integrate_edge_vec( L[ 0 ], L[ 1 ] );
  vsum += integrate_edge_vec( L[ 1 ], L[ 2 ] );
  vsum += integrate_edge_vec( L[ 2 ], L[ 3 ] );
  vsum += integrate_edge_vec( L[ 3 ], L[ 0 ] );

  // form factor of the polygon in direction vsum
  float len = length( vsum );

  float z = vsum.z / len;
  if ( behind )
  {
    z = -z;
  }

  vec2 texcoord = vec2( z * 0.5f + 0.5f, len ); // range [0, 1]
  texcoord = texcoord * LUT_SCALE + LUT_BIAS;

  // Fetch the form factor for horizon clipping
  float scale = texture( u_LTC2, texcoord ).w;

  float sum = len * scale;

  if ( !behind && !two_sided )
  {
    sum = 0.0;
  }

  // Outgoing radiance (solid angle) for the entire polygon
  vec3 Lo_i = vec3( sum, sum, sum );
  return Lo_i;
}

// PBR-maps for roughness (and metallic) are usually stored in non-linear
// color space (sRGB), so we use these functions to convert into linear RGB.
vec3 pow_vec3( vec3 v, float p )
{
  return pow( v, vec3( p ) );
}

const float gamma = 2.2;

vec3 to_linear( vec3 v ) { return pow_vec3( v, gamma ); }

vec3 to_srgb( vec3 v ) { return pow_vec3( v, 1.0 / gamma ); }

vec3 F_Schlick( float cosTheta, vec3 F0 )
{
  return F0 + ( 1.0 - F0 ) * pow( 1.0 - cosTheta, 5.0 );
}

void main()
{
  // gamma correction
  vec3 albedo = texture( u_base_color, v_texcoord ).rgb; // * vec3(0.7f, 0.8f, 0.96f);
  albedo = to_linear( albedo );
  vec4 metallic_roughness = texture( u_metallic_roughness, v_texcoord );
  float metallic = metallic_roughness.r;
  float roughness = metallic_roughness.g;
  vec3 vspecular = to_linear( vec3( 0.23f, 0.23f, 0.23f ) ); // mDiffuse
  // Calculate diffuse and specular based on metalness
  // Dielectric F0 is typically 0.04 (4% reflectance)

  vec3 dielectricF0 = vec3( 0.04 );

  // For metals: no diffuse, specular uses albedo color
  // For dielectrics: full diffuse, specular is ~4%
  vec3 mDiffuse = albedo * ( 1.0 - metallic );
  vec3 mSpecular = mix( dielectricF0, albedo, metallic );

  vec3 result = vec3( 0.0f );

  vec3 N = normalize( v_world_normal );
  vec3 V = normalize( u_view_position - v_world_position );
  vec3 P = v_world_position;
  float dot_NV = clamp( dot( N, V ), 0.0f, 1.0f );

  vec3 F0 = mix( vec3( 0.04 ), albedo, metallic );
  vec3 fresnel = F_Schlick( dot_NV, F0 );

  // use roughness and sqrt(1-cos_theta) to sample M_texture
  vec2 texcoord = vec2( roughness, sqrt( 1.0f - dot_NV ) );
  texcoord = texcoord * LUT_SCALE + LUT_BIAS;

  // get 4 parameters for inverse_M
  vec4 t1 = texture( u_LTC1, texcoord );
  // Get 2 parameters for Fresnel calculation
  vec4 t2 = texture( u_LTC2, texcoord );

  mat3 m_inv = mat3
  (
    vec3( t1.x, 0.0, t1.y ),
    vec3(  0.0, 1.0,  0.0 ),
    vec3( t1.z, 0.0, t1.w )
  );

  // translate light source for testing
  vec3 translated_points[ 4 ];
  translated_points[ 0 ] = u_points[ 0 ] + u_light_position;
  translated_points[ 1 ] = u_points[ 1 ] + u_light_position;
  translated_points[ 2 ] = u_points[ 2 ] + u_light_position;
  translated_points[ 3 ] = u_points[ 3 ] + u_light_position;

  // Evaluate LTC shading
  vec3 diffuse = ltc_evaluate( N, V, P, mat3( 1.0 ), translated_points, u_two_sided );
  vec3 specular = ltc_evaluate( N, V, P, m_inv, translated_points, u_two_sided );

  // GGX BRDF shadowing and Fresnel
  // t2.x: shadowedF90 (F90 normally it should be 1.0)
  // t2.y: Smith function for Geometric Attenuation Term, it is dot(V or L, H).
  specular *= mSpecular * t2.x + ( 1.0f - mSpecular ) * t2.y;
  specular *= fresnel;
  result = u_light_color * u_light_intensity * ( specular + mDiffuse * diffuse );

  // frag_color = vec4( to_srgb( result ), 1.0 );
  // vec3 c = texture( u_base_color, v_texcoord ).rgb;
  frag_color = vec4( to_srgb( result ) , 1.0 );
}
