#version 300 es
precision mediump float;

out vec4 frag_color;

in vec3 v_world_position;
in vec3 v_world_normal;
in vec2 v_texcoord;

uniform vec3 u_points[ 4 ]; // pretransformed
uniform float u_light_intensity;
uniform vec3 u_light_color;
uniform bool u_two_sided;

uniform sampler2D u_base_color;
uniform sampler2D u_arm; // ao, roughness, metalness

uniform vec3 u_view_position;
uniform sampler2D u_LTC1; // for inverse M
uniform sampler2D u_LTC2; // GGX norm, fresnel, 0(unused), sphere

const float LUT_SIZE  = 64.0;
const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
const float LUT_BIAS  = 0.5 / LUT_SIZE;

vec3 integrate_edge_vec( vec3 v1, vec3 v2 )
{
  float x = dot( v1, v2 );
  float y = abs( x );

  float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
  float b = 3.4175940 + ( 4.1616724 + y ) * y;
  float v = a / b;

  float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;

  return cross( v1, v2 ) * theta_sintheta;
}

vec3 ltc_evaluate( vec3 N, vec3 V, vec3 P, mat3 m_inv, vec3 points[ 4 ], bool two_sided )
{
  vec3 T1 = normalize( V - N * dot( V, N ) );
  vec3 T2 = cross( N, T1 );

  m_inv = m_inv * transpose( mat3( T1, T2, N ) );

  vec3 L[ 4 ] = vec3[]
  (
    m_inv * ( points[ 0 ] - P ),
    m_inv * ( points[ 1 ] - P ),
    m_inv * ( points[ 2 ] - P ),
    m_inv * ( points[ 3 ] - P )
  );

  vec3 dir = P - points[ 0 ];
  vec3 light_normal = cross( points[ 1 ] - points[ 0 ], points[ 2 ] - points[ 0 ] );
  bool is_illuminated = dot( dir, light_normal ) > 0.0;

  if ( !is_illuminated && !two_sided )
  {
    return vec3( 0.0 );
  }

  vec3 vsum = vec3( 0.0 );
  L[ 0 ] = normalize( L[ 0 ] );
  L[ 1 ] = normalize( L[ 1 ] );
  L[ 2 ] = normalize( L[ 2 ] );
  L[ 3 ] = normalize( L[ 3 ] );

  vsum += integrate_edge_vec( L[ 1 ], L[ 0 ] );
  vsum += integrate_edge_vec( L[ 0 ], L[ 2 ] );
  vsum += integrate_edge_vec( L[ 2 ], L[ 3 ] );
  vsum += integrate_edge_vec( L[ 3 ], L[ 1 ] );

  float len = length( vsum );
  float z = vsum.z / len;

  // if is two sided but not illuminated then change sign of z
  z = !is_illuminated ? abs( z ) : z;

  vec2 texcoord = vec2( z * 0.5f + 0.5f, len );
  texcoord = texcoord * LUT_SCALE + LUT_BIAS;

  float scale = texture( u_LTC2, texcoord ).w;

  float sum = len * scale;

  return vec3( sum );
}

vec3 to_linear( vec3 v ) { return pow( v, vec3( 2.2 ) ); }

vec3 to_srgb( vec3 v ) { return pow( v, vec3( 1.0 / 2.2 ) ); }

vec3 f_schlick( float cos_theta, vec3 F0 )
{
  return F0 + ( 1.0 - F0 ) * pow( 1.0 - cos_theta, 5.0 );
}

void main()
{
  float ambient = 0.02;
  vec3 albedo = to_linear( texture( u_base_color, v_texcoord ).rgb );
  vec4 arm = texture( u_arm, v_texcoord );

  float ao = arm.r;
  float roughness = arm.g;
  float metallic = arm.b;

  vec3 N = normalize( v_world_normal );
  vec3 V = normalize( u_view_position - v_world_position );
  vec3 P = v_world_position;
  float dot_NV = max( dot( N, V ), 0.0 );

  if ( dot_NV < 0.001 ) // self-shadowing
  {
    frag_color = vec4( 0.0, 0.0, 0.0, 1.0 );
    return;
  }

  // Determine PBR diffuse and specular colors based on metallic workflow
  vec3 dielectric_F0 = vec3( 0.04 );
  vec3 F0 = mix( dielectric_F0, albedo, metallic );
  vec3 kd = albedo * ( 1.0 - metallic );

  vec2 texcoord = vec2( roughness, sqrt( 1.0 - dot_NV ) );
  texcoord = texcoord * LUT_SCALE + LUT_BIAS;
  vec4 t1 = texture( u_LTC1, texcoord );
  vec4 t2 = texture( u_LTC2, texcoord ); // t2.x = GGX norm, t2.y = Fresnel term

  // Construct the inverse transformation matrix for the specular component
  mat3 m_inv = mat3
  (
    vec3( t1.x, 0.0, t1.y ),
    vec3(  0.0, 1.0,  0.0 ),
    vec3( t1.z, 0.0, t1.w )
  );

  // Evaluate LTC integrals
  // The diffuse integral uses an identity matrix
  vec3 diffuse = ltc_evaluate( N, V, P, mat3( 1.0 ), u_points, u_two_sided );
  // The specular integral uses the fitted inverse matrix
  vec3 specular = ltc_evaluate( N, V, P, m_inv, u_points, u_two_sided );

  // Correctly combine the results for the final shading
  // t2 contains the amplitude and an average Fresnel term.
  // We combine this with our material's F0.
  vec3 specular_color = t2.x * f_schlick( 1.0, F0 ) + t2.y;

  vec3 result = ( diffuse + ambient ) * kd + specular * specular_color;
  result *= ao;
  result *= u_light_color * u_light_intensity;
  result = result / ( result + vec3( 1.0 ) ); // simple reinhard tonemapper to handle bright highlights gracefully
  result = to_srgb( result );

  frag_color = vec4( result, 1.0 );
}
