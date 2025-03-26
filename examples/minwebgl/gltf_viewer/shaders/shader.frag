precision mediump float;

struct PhysicalMaterial {
  vec3 diffuseColor;
  float roughness;
  vec3 specularColor;
  float specularF90;
  float dispersion;
};

struct IncidentLight {
  vec3 color;
  vec3 direction;
  bool visible;
};

struct ReflectedLight {
  vec3 directDiffuse;
  vec3 directSpecular;
  vec3 indirectDiffuse;
  vec3 indirectSpecular;
};

uniform vec3 diffuse;
uniform vec3 emissive;
uniform float roughness;
uniform float metalness;
uniform float opacity;

vec3 F_Schlick( const in vec3 f0, const in float f90, const in float dotVH ) 
{
  float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
  return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
}

vec3 Schlick_to_F0( const in vec3 f, const in float f90, const in float dotVH )
 {
  float x = clamp( 1.0 - dotVH, 0.0, 1.0 );
  float x2 = x * x;
  float x5 = clamp( x * x2 * x2, 0.0, 0.9999 );
  return ( f - vec3( f90 ) * x5 ) / ( 1.0 - x5 );
}

float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) 
{
  float a2 = pow2( alpha );
  float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
  float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
  return 0.5 / max( gv + gl, 1e-6 );
}
float D_GGX( const in float alpha, const in float dotNH ) 
{
  float a2 = pow2( alpha );
  float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
  return 0.3183098861837907 * a2 / pow2( denom );
}

vec3 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
  vec3 f0 = material.specularColor;
  float f90 = material.specularF90;

  float roughness = material.roughness;
  float alpha = pow2( roughness );
  vec3 halfDir = normalize( lightDir + viewDir );

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );

  vec3 F = F_Schlick( f0, f90, dotVH );
  float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  float D = D_GGX( alpha, dotNH );
  return F * ( V * D );
}

void main()
{
  // Ambient
  #ifdef AMBIENT_COLOR
  vec3 ambient_color = ambient;
  #else
  vec3 ambient_color = vec3( 0.1 );
  #endif
  #ifdef AMBIENT_TEXTURE
  ambient_color *= texture( ambient_texture, vUv ).rgb;
  #endif
  // Gamma correct to linear space
  ambient_color = pow( ambient_color, vec3( 2.2 ) );
  ///////////////////////

  // Diffuse
  #ifdef DIFFUSE_COLOR
    vec3 albedo = diffuse;
  #else
    vec3 albedo = vec3( 1.0 );
  #endif
  #ifdef DIFFUSE_TEXTURE
    albedo *= texture( diffuse_texture, vUv ).rgb;
  #endif
  // Gamma correct to linear space
  albedo = pow( albedo, vec3( 2.2 ) );
  ///////////////////////


  // Specular
  #ifdef SPECULAR_COLOR
  vec3 F0 = specular;
  #else
  vec3 F0 = vec3( 0.04 ); // Glass
  #endif
  #ifdef SPECULAR_TEXTURE
  F0 *= texture( specular_texture, vUv ).rgb;
  #endif
  ///////////////////////

  // Shineness
  // In obj format, shininess is a value from 0.0 to 1000.0
  // It is used as an exponent in Phong model
  // Since I use pbr, I just approximate roughness by normalizing the value
  #ifdef SHININESS
  float shiny = shininess;
  #else
  float shiny = 500.0;
  #endif
  #ifdef SHININESS_TEXTURE
  shiny *= texture( shininess_texture, vUv ).r;
  #endif
  float roughness = max( 1.0 - shiny / 1000.0, 0.0 );
  ///////////////////////

  // Dissolve
  #ifdef DISSOLVE
  float alpha = dissolve;
  #else
  float alpha = 1.0;
  #endif
  #ifdef DISSOLVE_TEXTURE
  alpha *= texture( dissolve_texture, vUv ).r;
  #endif

  vec3 normal = normalize( vNormal );
  // Normal mapping
  // https://learnopengl.com/Advanced-Lighting/Normal-Mapping
  #ifdef NORMAL_TEXTURE
  vec3 n = normalize( texture( normal_texture, vUv ).rgb * 2.0 - 1.0 );
  vec3 dE1 = dFdx( vWorldPos );
  vec3 dE2 = dFdy( vWorldPos );
  vec2 dUv1 = dFdx( vUv );
  vec2 dUv2 = dFdy( vUv );
  float denom = 1.0 / ( dUv1.x * dUv2.y - dUv1.y * dUv2.x );
  vec3 T;
  T.x = dUv2.y * dE1.x - dUv1.y * dE2.x;
  T.y = dUv2.y * dE1.y - dUv1.y * dE2.y;
  T.z = dUv2.y * dE1.z - dUv1.y * dE2.z;
  T = normalize( T * denom );
  vec3 B;
  B.x = -dUv2.x * dE1.x + dUv1.x * dE2.x;
  B.y = -dUv2.x * dE1.y + dUv1.x * dE2.y;
  B.z = -dUv2.x * dE1.z + dUv1.x * dE2.z;
  B = normalize( B * denom );
  mat3x3 TBN = mat3x3(T, B, normal);
  normal = normalize( TBN * n );
  #endif

  float lightIntensity = 2.0;
  vec3 viewDir = normalize( cameraPosition - vWorldPos );
  vec3 lightDirs[] = vec3[]
  (
    vec3( 1.0, 0.0, 0.0 ),
    vec3( 0.0, 1.0, 0.0 ),
    vec3( 0.0, 0.0, 1.0 ),
    vec3( -1.0, 0.0, 0.0 ),
    vec3( 0.0, -1.0, 0.0 ),
    vec3( 0.0, 0.0, -1.0 )
  );

  vec3 color = vec3( 0.0 );

  // If normal were not provided with the model, then their value is going to be 0
  if( length( normal ) > 0.0 )
  {
    vec3 Lo = vec3( 0.0 );
    for( int i = 0; i < 6; i++ )
    {
      Lo += BRDF( albedo, lightDirs[ i ], viewDir, normal, roughness, F0 ) * lightIntensity;
    }
    color += ambient_color * albedo + Lo;
  }
  else
  {
    color = ambient_color * albedo + albedo;
  }

  // Gamma correciton
  color = pow( color, vec3( 1.0 / 2.2 ) );
  
  frag_color = vec4( color, alpha );
}
