precision highp float;

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
#ifdef USE_TANGENTS
  in vec4 vTangent;
#endif
in vec3 vWorldPos;
in vec3 vViewPos;
in vec3 vNormal;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;

uniform vec3 cameraPosition;
uniform float exposure;

#ifdef USE_ALPHA_CUTOFF
  uniform float alphaCutoff;
#endif

struct PhysicalMaterial
{
  vec3 diffuseColor;
  float metallness;
  float roughness;
  vec3 f0;
  vec3 f90;
  float specularFactor;
};

struct ReflectedLight
{
  vec3 indirectDiffuse;
  vec3 indirectSpecular;
  vec3 directDiffuse;
  vec3 directSpecular;
};

struct LightInfo
{
  vec3 direction;
  vec3 color;
  float strength;
};

uniform float luminosityThreshold;
uniform float luminositySmoothWidth;
uniform float metallicFactor; // Default: 1
uniform float roughnessFactor; // Default: 1
uniform vec4 baseColorFactor; // Default: [1, 1, 1, 1]

#ifdef USE_IBL
  uniform samplerCube irradianceTexture;
  uniform samplerCube prefilterEnvMap;
  uniform sampler2D integrateBRDF;
#endif
#ifdef USE_KHR_materials_specular
  uniform float specularFactor;
  uniform vec3 specularColorFactor;
  #ifdef USE_SPECULAR_TEXTURE
    uniform sampler2D specularTexture;
  #endif
  #ifdef USE_SPECULAR_COLOR_TEXTURE
    uniform sampler2D specularColorTexture;
  #endif
#endif
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


// Scales the normal in X and Y directions
// ( <sample normalTexture> * 2.0 - 1.0 ) * vec3( normalScale, normalScale, 1.0 )
uniform float normalScale; // Default: 1
#ifdef USE_NORMAL_TEXTURE
  // vNormalUv
  uniform sampler2D normalTexture;
#endif

// 1.0 + occlusionStrength * ( <sample occlusionTexture> - 1.0 )
uniform float occlusionStrength; // Default: 1
#ifdef USE_OCCLUSION_TEXTURE
  // vOcclusionUv
  uniform sampler2D occlusionTexture;
#endif


// vEmissionUv
#ifdef USE_EMISSION_TEXTURE
  uniform sampler2D emissiveTexture;
#endif
uniform vec3 emissiveFactor;



float max_value( const in vec3 v )
{
  return max( v.x, max( v.y, v.z ) );
}

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

float dot2( const in vec3 v )
{
  return dot( v, v );
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
vec3 F_Schlick( const in vec3 f0, const in vec3 f90, const in float dotVH )
{
  float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
  return f0 + ( f90 - f0 ) * fresnel;
}

vec3 Fd_Barley
(
  const in float alpha,
  const in float dotNV,
  const in float dotNL,
  const in float dotLH
)
{
  vec3 f90 = vec3( 0.5 + 2.0 * alpha * pow2( dotLH ) );
  vec3 lightScatter = F_Schlick( vec3( 1.0 ), f90, dotNL );
  vec3 viewScatter = F_Schlick( vec3( 1.0 ), f90, dotNV );
  return viewScatter * lightScatter * RECIPROCAL_PI;
}

// https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
// https://inria.hal.science/hal-00942452v1/document
// Visibility Geometry function
// V = G / ( 4 * dotNV * dotNL )
// G = G1( L ) * G1( V )
// G1( L ) = 2dotNL / ( dotNL + sqrt( a2 + ( 1 - a2 ) * dotNL2 ) )
// The term ( 4 * dotNV * dotNL ) in BRDF cancels out
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV )
{
  float a2 = pow2( alpha );
  float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
  float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

// Normal distribution function
float D_GGX( const in float alpha, const in float dotNH )
{
  float a2 = pow2( alpha );
  float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
  return 0.3183098861837907 * a2 / pow2( denom );
}

vec4 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material )
{
  float alpha = pow2( material.roughness );
  vec3 halfDir = normalize( lightDir + viewDir );

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );

  // Fresnel
  vec3 F = F_Schlick( material.f0, material.f90, dotVH );
  // Geometry function
  float G = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  // Normal distribution function
  float D = D_GGX( alpha, dotNH );
  return vec4( F, G * D ) ;
}

void computeDirectLight
(
  const in LightInfo lightInfo,
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  inout ReflectedLight reflectedLight
)
{
  float alpha = pow2( material.roughness );
  vec3 halfDir = normalize( lightInfo.direction + viewDir );

  float dotNL = clamp( dot( normal, lightInfo.direction ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );
  float dotLH = clamp( dot( lightInfo.direction, halfDir ), 0.0, 1.0 );

  // Fresnel
  vec3 Fs = F_Schlick( material.f0, material.f90, dotVH );
  //float Fd = max_value( Fs );
  vec3 Fd = Fd_Barley( alpha, dotNV, dotNL, dotLH );
  // Visibility Geometry function
  float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  // Normal distribution function
  float D = D_GGX( alpha, dotNH );

  vec3 irradiance = lightInfo.color * lightInfo.strength * dotNL;
  vec3 diffuseColor =  material.diffuseColor * irradiance;
  vec3 specularColor =  D * V * irradiance;

  // Diffuse BRDF( Lambert )
  reflectedLight.directDiffuse += Fd * diffuseColor;

  // Specular BRDF
  reflectedLight.directSpecular += Fs * specularColor;
}

#ifdef USE_IBL
  void sampleEnvIrradiance( const in vec3 N, const in vec3 V, const in PhysicalMaterial material, inout ReflectedLight reflectedLight )
  {
    float alpha = pow2( material.roughness );
    float dotNV = clamp( dot( N, V ), 0.0, 1.0 );

    const float MAX_LOD = 9.0;
    //if( dotNV > 0.0 )
    {
      vec3 Fs = F_Schlick( material.f0, material.f90, dotNV );
      vec3 R = reflect( -V, N );

      vec3 diffuse = texture( irradianceTexture, N ).xyz * pow( 2.0, exposure );
      vec3 prefilter = texture( prefilterEnvMap, R, material.roughness * MAX_LOD ).xyz * pow( 2.0, exposure );
      vec2 envBrdf = texture( integrateBRDF, vec2( dotNV, material.roughness ) ).xy;

      vec3 diffuseBRDF = diffuse * material.diffuseColor;
      vec3 specularBRDF = prefilter * ( material.f0 * envBrdf.x + envBrdf.y );
      //vec3 specularBRDF = prefilter * ( Fs * envBrdf.x + envBrdf.y );

      reflectedLight.indirectDiffuse += diffuseBRDF;
      reflectedLight.indirectSpecular += specularBRDF;
    }
  }
#endif

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

// float alpha_weight( float a )
// {
//   float z = abs( vViewPos.z );
//   float b = min( 3e3, 10.0 / ( 1e-5 + pow( z / 5.0 , 3.0 ) + pow( z / 2e2, 6.0 ) ) );
//   float c = max( 1e-2, b );
//   return a * c;
// }

// float alpha_weight( float a )
// {
//   float c = max( 1e-2, 3e3 * pow( 1.0 - gl_FragCoord.z, 3.0 ) );
//   return a * c;
// }

#ifndef USE_TANGENTS
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

float adjustRoughnessNormalMap ( const in float roughness, const in vec3 normal )
{
  float nlen2 = dot (normal, normal );
  if( nlen2 < 1.0 )
  {
    float nlen = sqrt( nlen2 );
    float kappa = (3.0 * nlen -  nlen2 * nlen) / (1.0 - nlen2);
    return min(1.0, sqrt(roughness * roughness + 1.0 / kappa));
  }
  return roughness;
}

void main()
{
  PhysicalMaterial material;
  ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );

  float alpha = 1.0;

  material.metallness = metallicFactor;
  material.roughness = roughnessFactor;
  material.diffuseColor = baseColorFactor.rgb;
  alpha *= baseColorFactor.a;
  #ifdef USE_BASE_COLOR_TEXTURE
    vec4 baseColor = texture( baseColorTexture, vBaseColorUv );
    baseColor.rgb = SrgbToLinear( baseColor.rgb );
    material.diffuseColor *= baseColor.rgb;
    alpha *= baseColor.a;
  #endif
  #ifdef USE_MR_TEXTURE
    vec4 mr_sample = texture( metallicRoughnessTexture, vMRUv );
    material.metallness *= mr_sample.b;
    material.roughness *= mr_sample.g;
  #endif

  #ifdef USE_ALPHA_CUTOFF
    alpha = step( alpha, 1.0 - alphaCutoff );
    if( alpha == 0.0 )
    {
      discard;
    }
  #endif

  //Specular part
  // https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md
  // 0.04 - reflectance of the Glass
  material.f0 = vec3( 0.04 );
  material.f90 = vec3( 1.0 );
  material.specularFactor = 1.0;
  #ifdef USE_KHR_materials_specular
    material.specularFactor *= specularFactor;
    material.f0 *= specularColorFactor;
    #ifdef USE_SPECULAR_COLOR_TEXTURE
      material.f0 *= SrgbToLinear( texture( specularColorTexture, vSpecularColorUv ).rgb );
    #endif
    #ifdef USE_SPECULAR_TEXTURE
      material.specularFactor *= texture( specularTexture, vSpecularUv ).a;
    #endif
    material.f0 = min( material.f0 * material.specularFactor, vec3( 1.0 ) );
  #endif
  material.f0 = mix( material.f0, material.diffuseColor, material.metallness );
  material.diffuseColor *= 1.0 - material.metallness;

  vec3 normal = normalize( vNormal );

  #ifdef USE_NORMAL_TEXTURE
    vec3 normalSample = texture( normalTexture, vNormalUv ).xyz * 2.0 - 1.0;
    //material.roughness = adjustRoughnessNormalMap( material.roughness, normalSample );
    normalSample.xy *= vec2( normalScale );

    #ifdef USE_TANGENTS
    {
      vec3 bitangent = cross( normal, vTangent.xyz ) * vTangent.w;
      mat3x3 TBN = mat3x3( vTangent.xyz, bitangent, normal );
      normal = TBN * normalSample;
    }
    #else
      normal = getTBN( normal, vWorldPos, vNormalUv ) * normalSample;
    #endif
    normal = normalize( normal );
  #endif

  float faceDirection = gl_FrontFacing ? 1.0 : -1.0;
  normal *= faceDirection;

  vec3 color = vec3( 0.0 );
  vec3 viewDir = normalize( cameraPosition - vWorldPos );
  // vec3 lightDirs[] = vec3[]
  // (
  //   vec3( 1.0, 0.0, 0.0 ),
  //   vec3( 0.0, 1.0, 0.0 ),
  //   vec3( 0.0, 0.0, 1.0 ),
  //   vec3( -1.0, 0.0, 0.0 ),
  //   vec3( 0.0, -1.0, 0.0 ),
  //   vec3( 0.0, 0.0, -1.0 )
  // );

  #if !defined( USE_IBL )
    vec3 lightDirs[] = vec3[]
    (
      vec3( 1.0, 1.0, 1.0 ),
      vec3( -1.0, 1.0, 1.0 ),
      vec3( 1.0, -1.0, 1.0 ),
      vec3( -1.0, -1.0, 1.0 ),
      vec3( 1.0, 1.0, -1.0 ),
      vec3( 1.0, -1.0, -1.0 ),
      vec3( -1.0, 1.0, -1.0 ),
      vec3( -1.0, -1.0, -1.0 )
    );

    LightInfo lightInfo;
    lightInfo.strength = 2.0;
    lightInfo.color = vec3( 1.0 );
    float dotVN = clamp( dot( viewDir, normal ), 0.0, 1.0 );

    for( int i = 0; i < 8; i++ )
    {
      lightInfo.direction = normalize( lightDirs[ i ] );
      float dotNL = clamp( dot( normal, lightInfo.direction ), 0.0, 1.0 );

      if( dotNL > 0.0 )
      {
        computeDirectLight( lightInfo, viewDir, normal, material, reflectedLight );
      }
    }
  #endif

  // Ambient color
  #if defined( USE_IBL )
    sampleEnvIrradiance( normal, viewDir, material, reflectedLight );
  #else
    reflectedLight.indirectDiffuse += 0.1 * material.diffuseColor;
  #endif

   // Works only with indirect light
  #ifdef USE_OCCLUSION_TEXTURE
    float occlusion = texture( occlusionTexture, vOcclusionUv ).r;
    reflectedLight.indirectDiffuse *= 1.0 + occlusionStrength * ( occlusion - 1.0 );
  #endif

  emissive_color = vec4( emissiveFactor, 1.0 );
  #ifdef USE_EMISSION_TEXTURE
    emissive_color.xyz *= SrgbToLinear( texture( emissiveTexture, vEmissionUv ).rgb );
  #endif


  color = reflectedLight.indirectDiffuse +
  reflectedLight.indirectSpecular +
  reflectedLight.directDiffuse +
  reflectedLight.directSpecular;

  //color = vec3( material.occlusionFactor );
  //color = vec3( material.f0 );
  //color = vec3( alpha );
  //color = material.diffuseColor;
  //float a_weight = alpha * alpha_weight( alpha );
  //alpha = 0.9;
  //color = material.diffuseColor;
  //color = normal;
  float a_weight = alpha * alpha_weight( alpha );
  trasnparentA = vec4( color * a_weight, alpha );
  transparentB = a_weight;
  frag_color = vec4( color, alpha );
}
