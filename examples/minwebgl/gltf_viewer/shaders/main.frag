precision mediump float;

#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6

in vec2 vUv_0;
in vec2 vUv_1;
in vec2 vUv_2;
in vec2 vUv_3;
in vec2 vUv_4;
in vec3 vWorldPos;
in vec3 vViewPos;
in vec3 vNormal;

layout( location = 0 ) out vec4 frag_color;
//layout( location = 1 ) out vec4 emissive_color;

uniform vec3 cameraPosition;

struct PhysicalMaterial
{
  vec3 diffuseColor;
  float metallness;
  float roughness;
  vec3 specularColor;
  float occlusionFactor;
};

struct ReflectedLight
{
  vec3 directDiffuse;
  vec3 directSpecula;
};

#ifdef USE_PBR
  uniform float metallicFactor; // Default: 1
  uniform float roughnessFactor; // Default: 1
  uniform vec4 baseColorFactor; // Default: [1, 1, 1, 1]
  #ifdef USE_MR_TEXTURE
    // Roughness is sampled from the G channel
    // Metalness is sampled from the B channel
    // vMRUv
    uniform sampler2D metallicRoughnessTexture;
  #endif
  #ifdef USE_BASE_COLOR_TEXTURE
    // vBaseColorUv
    uniform sampler2D baseColorTexture;
  #endif
#endif

#ifdef USE_NORMAL_TEXTURE
  // vNormalUv
  uniform sampler2D normalTexture;
  // Scales the normal in X and Y directions
  // ( <sample normalTexture> * 2.0 - 1.0 ) * vec3( normalScale, normalScale, 1.0 )
  uniform float normalScale; // Default: 1
#endif

#ifdef USE_OCCLUSION_TEXTURE
  // vOcclusionUv
  uniform sampler2D occlusionTexture;
  // 1.0 + occlusionStrength * ( <sample occlusionTexture> - 1.0 )
  uniform float occlusionStrength; // Default: 1
#endif


#ifdef USE_EMISSION_TEXTURE
  // vEmissionUv
  uniform sampler2D emissiveTexture;
  uniform float emissiveFactor;
#endif



float pow2( const in float x ) 
{
  return x*x;
}

vec3 pow2( const in vec3 x ) 
{
  return x*x;
}

float pow3( const in float x ) 
{
  return x*x*x;
}

float pow4( const in float x ) 
{
  float x2 = x*x;
  return x2*x2;
}

vec4 SrgbToLinear( const in vec4 color )
{
  vec3 more = pow( color.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color.rgb * 0.0773993808;

  return vec4( mix( more, less, vec3( lessThanEqual( color.rgb, vec3( 0.04045 ) ) ) ), color.a );
}

vec4 LinearToSrgb( const in vec4 color )
{
  vec3 more = pow( color.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color.rgb * 12.92;

  return vec4( mix( more, less, vec3( lessThanEqual( color.rgb, vec3( 0.0031308 ) ) ) ), color.a );
}

vec3 SrgbToLinear( const in vec3 color )
{
  vec3 more = pow( color * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color * 0.0773993808;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.04045 ) ) ) );
}

vec3 LinearToSrgb( const in vec3 color )
{
  vec3 more = pow( color, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color * 12.92;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.0031308 ) ) ) );
}

// Schilck's version of Fresnel equation, with Spherical Gaussian approximation for the power
// https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
vec3 F_Schlick( const in vec3 f0, const in float dotVH ) 
{
  float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
  return f0 + ( vec3( 1.0 ) - f0 ) * fresnel;
}

// https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
// https://inria.hal.science/hal-00942452v1/document
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

vec4 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
  vec3 f0 = material.specularColor;

  float roughness = material.roughness;
  float alpha = pow2( roughness );
  vec3 halfDir = normalize( lightDir + viewDir );

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );

  // Fresnel
  vec3 F = F_Schlick( f0, dotVH );
  // Geometry function
  float G = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  // Normal distribution function
  float D = D_GGX( alpha, dotNH );
  return vec4( F, G * D ) ;
}

#ifdef USE_NORMAL_TEXTURE
  // http://www.thetenthplanet.de/archives/1180
  mat3 getTBN( vec3 surf_normal, vec3 pos, vec2 uv )
  {
    vec3 dE1 = dFdx( pos );
    vec3 dE2 = dFdy( pos );
    vec2 dUv1 = dFdx( uv );
    vec2 dUv2 = dFdy( uv );

    vec3 q1perp = cross( dE2, surf_normal );
		vec3 q0perp = cross( surf_normal, dE1 );
    
    vec3 T = q1perp * dUv1.x + q0perp * dUv2.x;
		vec3 B = q1perp * dUv1.y + q0perp * dUv2.y;

    float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );

		return mat3( T * scale, B * scale, surf_normal );
  }
#endif

void main()
{
  PhysicalMaterial material = PhysicalMaterial
  (
    vec3( 1.0 ),
    1.0,
    1.0,
    vec3( 1.0 ),
    1.0
  );
  
  float alpha = 1.0;
  #ifdef USE_PBR
    #ifdef USE_BASE_COLOR_TEXTURE
      vec4 baseColor = baseColorFactor * SrgbToLinear( texture( baseColorTexture, vBaseColorUv ) );
      material.diffuseColor =  baseColor.rgb;
      alpha = baseColor.a;
    #else
      material.diffuseColor = baseColorFactor.xyz;
      alpha = baseColorFactor.w;
    #endif
    #ifdef USE_MR_TEXTURE
      vec4 mr_sample = texture( metallicRoughnessTexture, vMRUv );
      material.metallness = metallicFactor * mr_sample.b;
      material.roughness = roughnessFactor * mr_sample.g;
    #else
    material.metallness = metallicFactor;
    material.roughness = roughnessFactor;
    #endif
  #else
    material.metallness = 0.0;
    material.roughness = 1.0;
  #endif

  material.specularColor = mix( vec3( 0.04 ), material.diffuseColor, material.metallness );

  vec3 normal = normalize( vNormal );
  #ifdef USE_NORMAL_TEXTURE
    vec3 normalSample = texture( normalTexture, vNormalUv ).xyz * 2.0 - 1.0;
    normalSample.xy *= vec2( normalScale );
    normal = getTBN( normal, vWorldPos, vNormalUv ) * normalSample;
    normal = normalize( normal );
  #endif

  // Works only with indirect light
  #ifdef USE_OCCLUSION_TEXTURE
    float occlusion = texture( occlusionTexture, vOcclusionUv ).r;
    material.occlusionFactor = 1.0 + occlusionStrength * ( occlusion - 1.0 );
  #else
    material.occlusionFactor = 1.0;
  #endif


  vec3 color = vec3( 0.0 );
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

  const vec3 lightColor = vec3( 1.0 );
  vec3 ambientColor = 0.1 * material.diffuseColor * material.occlusionFactor;
  for( int i = 0; i < 6; i++ )
  {
    vec4 brdf = BRDF_GGX( lightDirs[ i ], viewDir, normal, material );
    vec3 F = brdf.xyz;
    float DG = brdf.w;
    float dotNL = clamp( dot( normal, lightDirs[ i ] ), 0.0, 1.0 );
    float dotVN = clamp( dot( viewDir, normal ), 0.0, 1.0 );

    vec3 light_diffuse = ( vec3( 1.0 ) - F ) * material.diffuseColor * RECIPROCAL_PI;
    vec3 light_specular = F * DG ;// max( 4.0 * dotVN * dotNL, 0.0001 );
    color += ( light_diffuse + light_specular ) * lightColor * dotNL;
  }
  color += ambientColor;

  // #ifdef USE_EMISSION
  //   emissive_color = vec4( 1.0 );
  //   emissive_color.xyz *= emissiveFactor;
  //   #ifdef USE_EMISSION_TEXTURE
  //     emissive_color.xyz *= texture( emissiveTexture, {EMISSION_UV} )
  //   #endif
  // #endif

  color = LinearToSrgb( color );

  
  frag_color = vec4( color * alpha, alpha );
  //frag_color = vec4( 1.0 );
}
