precision mediump float;

const float PI = 3.1415926535897932384626433;
const float EPSILON = 1e-4;
// Reflectance of a diamond at light incidence of theta = 0
// Max distance to the surface in cubeNormalMap
// This value was calculated during generation of the map
//const float MAX_DISTANCE = 5.7610855;
// const float MAX_DISTANCE = 1.0;
// const int RAY_BOUNCES = 7;
// const float TRANSMISSION = 0.5;
// const vec3 DIAMOND_COLOR = vec3( .98, 0.95, 0.9 );
// const vec3 BOOST_FACTORS = vec3( 0.8920, 0.8920, 0.9860 );

const vec3 ambientColor = vec3(0.7);
const float ambientint = 0.08;

uniform samplerCube envMap;
uniform samplerCube cubeNormalMap;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 inverseWorldMatrix;
uniform mat4x4 offsetMatrix;
uniform mat4x4 inverseOffsetMatrix;

uniform int rayBounces;
uniform vec3 diamondColor;
uniform vec3 boostFactors;

uniform float envMapIntensity;
uniform float rainbowDelta;
uniform float squashFactor;
uniform float radius;
uniform float geometryFactor;
uniform float absorptionFactor;
uniform vec3 colorAbsorption;
uniform vec3 cameraPosition;

uniform float maxDistance;

in vec2 vUvs;
in vec3 vWorldNormal;
in vec3 vWorldPosition;
in vec3 vViewPosition;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

// https://github.com/mrdoob/three.js/blob/dev/src/nodes/functions/BSDF/DFGApprox.js
vec3 EnvBRDFApprox( const in float NdotV, const in vec3 specularColor, const in float roughness )
{
  const vec4 c0 = vec4( -1, -0.0275, -0.572, 0.022 );
  const vec4 c1 = vec4( 1, 0.0425, 1.04, -0.04 );

  vec4 r = roughness * c0 + c1;

  float a004 = min( r.x * r.x, exp2( -9.28 * NdotV ) ) * r.x + r.y;
  vec2 AB = vec2( -1.04, 1.04 ) * a004 + r.zw;

  return specularColor * AB.x + AB.y;
}

// Extract the data from the normal map
vec4 getNormalData( vec3 dir )
{
  vec4 data = texture( cubeNormalMap, dir );
  data.rgb = normalize( data.rgb * 2.0 - 1.0 );
  data.a *= radius;
  return data;
}

// Schlick ver.
vec3 freshel( vec3 viewDir, vec3 halfway, vec3 f0, float criticalAngleCosine )
{
  float VdotH = dot( viewDir, halfway );
  // Case of full reflection
  if( VdotH < criticalAngleCosine )
  {
    return vec3( 1.0 );
  }

  return f0 + ( 1.0 - f0 ) * pow( ( 1.0 - dot( viewDir, halfway ) ), 5.0 );
}

vec2 dirToEquirectUV( vec3 dir )
{
  float phi = atan( -dir.z, dir.x );
  float theta = asin( dir.y );
  vec2 uv = vec2( 0.5 + phi / ( 2.0 * PI ), 0.5 - theta / PI );

  if ( uv.x < 0.0005 || uv.x > 0.9995 )
  {
    uv = vec2( 0.0001, uv.y );
  }

  return uv;
}

vec3 sampleSpecularReflection( vec3 direction )
{
 // vec3 sample_value = texture( envMap, dirToEquirectUV( direction ) ).rgb;
  vec3 sample_value = texture( envMap,  direction, 0.0 ).xyz;
  return envMapIntensity * sample_value;
}

vec3 convertDirLocalToWorld( vec3 direction )
{
  return  mat3x3( inverseOffsetMatrix ) * direction;
}

vec3 sampleEnvFromLocal( vec3 direction )
{
 // vec3 sample_value = texture( envMap, dirToEquirectUV( convertDirLocalToWorld( direction ) ) ).rgb;
  vec3 world_dir = convertDirLocalToWorld( direction );
  world_dir = mat3(viewMatrix) * world_dir;
  world_dir.xz *= -1.0;
  vec3 sample_value = texture( envMap, world_dir, 0.0 ).rgb;
  return 1.0 * sample_value;
}

vec3 sampleEnv( vec3 direction )
{
  direction.xyz *= -1.0;
  vec3 sample_value = texture( envMap, direction, 0.0 ).rgb;
  return sample_value;
}

vec3 SampleSpecularContribution( vec3 direction )
{
  direction = mat3( inverseOffsetMatrix ) * direction;
  direction = mat3( viewMatrix ) * direction;
  direction = normalize( direction );
  direction.x *= -1.;
  direction.z *= -1.;
  float envMapIntencity = 1.0;
  return envMapIntencity * sampleEnv( direction ).rgb;
}

// Finds an intersection points of a given line with a sphere at the origin
// and picks the father of the two possible solutions
vec3 intersectSphere( vec3 origin, vec3 direction )
{
  float sqFactor = 0.98;
  float gmFactor = 0.5;
  direction.y /= sqFactor;

  // Having parametric equation for the line in 'direction'
  // Solve the quadratic equation for 't' using sphere equation
  float A = dot( direction, direction );
  float B = 2.0 * dot( origin, direction );
  float C = dot( origin, origin ) - radius * radius;
  float disc = B * B - 4.0 * A * C;
  if( disc > 0.0 )
  {
      disc = sqrt( disc );
      float x1 = ( -B + disc ) * gmFactor / A;
      float x2 = ( -B - disc ) * gmFactor / A;
      float t = ( x1 > x2 ) ? x1 : x2;
      //t = x1;
      direction.y *= sqFactor;
      return vec3( origin + direction * t );
  }

  return vec3( 0.0 );
}

// Finds an intersection point of a line with a plane
vec3 linePlaneIntersect( in vec3 pointOnLine, in vec3 lineDirection, in vec3 pointOnPlane, in vec3 planeNormal )
{
  float t = dot( planeNormal, pointOnPlane - pointOnLine ) / dot( planeNormal, lineDirection );
  return lineDirection * t + pointOnLine;
}


// Approximates a point of intersection of the diamond with the given ray direction
vec3 intersectDiamond( vec3 rayOrigin, vec3 rayDirection )
{
  // Intersect a sphere at the center
  vec3 sphereHitPoint = intersectSphere( rayOrigin, rayDirection );
  // Direction from the center to the hit point on the sphere
  vec3 directionToSpherePoint = normalize( sphereHitPoint );
  // Sample the normal of the diamond in that direction
  // n.rgb - normal, n.a - distance to the surface
  vec4 normalData = getNormalData( directionToSpherePoint );
  // Flip the normal to point inwards
  vec3 surfaceNormal = normalData.rgb;
  float surfaceDistance = normalData.a;

  // Point on the surface of the diamond
  vec3 pointOnSurface1 = directionToSpherePoint * surfaceDistance;


  vec3 planeHitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface1, -surfaceNormal );
  vec3 directionToPlanePoint = normalize( planeHitPoint );

  normalData = getNormalData( directionToPlanePoint );
  surfaceNormal = normalData.rgb;
  surfaceDistance = normalData.a;

  // Point on the surface of the diamond
  vec3 pointOnSurface2 = directionToPlanePoint * surfaceDistance;

  vec3 hitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface2, -surfaceNormal );
  return hitPoint;
}


// Calculate the color by tracing a ray
vec3 getRefractionColor( vec3 rayHitPoint, vec3 rayDirection, vec3 hitPointNormal, float n2 )
{
  vec3 resultColor = vec3( 0.0 );

  const float _absorptionFactor = 1.0;
  const vec3 _absorptionColor = vec3( 1.0 );
  const vec3 _boostFactors = vec3( 1.0 );
  const vec3 _colorCorrection = vec3( 1.0 );
  const float _rIndexDelta = 0.0120;
  const float refractiveIndex = 2.6;

  // Refractive index of air
  const float n1 = 1.0;

  vec3 f0 = vec3( (n2 - n1) / (n2 + n1) );
  f0 *= f0;
  // vec3 f0 = 1.0 / vec3( 2.407, 2.426, 2.451 );

  n2 = refractiveIndex;

  float iorRatioAtoD = n1 / n2;
  float iorRatioDtoA = n2 / n1;


  // Angle of total refleciton
  float criticalAngleCosine = sqrt( max( 0.0, 1.0 - (iorRatioAtoD * iorRatioAtoD) ) );

  vec3 newRayDirection = refract( rayDirection, hitPointNormal, iorRatioAtoD );
  // Convert data to local space
  newRayDirection = ( vec4( newRayDirection, 0.0 ) ).xyz;
  newRayDirection = normalize( newRayDirection );
  vec3 rayOrigin =  ( offsetMatrix * vec4( rayHitPoint, 1.0 ) ).xyz;

  float totalDistance = 0.0;
  // Overall intensity of the light as it goes through the medium
  vec3 attenuationFactor = vec3( 1.0 );

  vec3 reflectedAmount = EnvBRDFApprox( abs(dot( -rayDirection, hitPointNormal )), f0, 0.0 );
  // Only take into account transmitted part
  attenuationFactor *= ( vec3( 1.0 ) - reflectedAmount );

  int c = 0;
  int v = 5;

  for( int i = 0; i < v; i++ )
  {
    // Intersection point on the diamond surface
    vec3 intersectPos = intersectDiamond( rayOrigin, newRayDirection );
    vec3 dirOriginToIntersect = normalize( intersectPos );

    // Get normal in direction to the intersected position
    vec4 normalData = getNormalData( dirOriginToIntersect );
    vec3 surfaceNormal = normalData.rgb;
    float surfaceDistance = normalData.a;

    // resultColor = dirOriginToIntersect * 0.5 + 0.5;
    // Update the origin position
    vec3 oldOrigin = rayOrigin;
    rayOrigin = dirOriginToIntersect * surfaceDistance;

    float r = length( rayOrigin - oldOrigin ) / radius * _absorptionFactor;
    attenuationFactor *= exp( -r *( vec3(1.0) - _absorptionColor) );


    // Calculate new rays
    vec3 newReflectedDirection = reflect( newRayDirection, -surfaceNormal );
    vec3 newRefractedDirection = refract( newRayDirection, -surfaceNormal, iorRatioDtoA );

    if( dot( newRefractedDirection, newRefractedDirection ) < 1e-4 )
    {
      if ( i == v - 1 )
      {
        vec3 reflectedAmount = EnvBRDFApprox( abs( dot( newRayDirection, surfaceNormal ) ), f0, 0.0 );
        resultColor += SampleSpecularContribution( newRayDirection ) * attenuationFactor * _boostFactors * _colorCorrection * ( vec3( 1.0 ) - reflectedAmount );
      }
    }
    else
    {
      vec3 refractedAmount = vec3( 1.0 ) - EnvBRDFApprox( abs( dot( newRefractedDirection, surfaceNormal ) ), f0, 0.0 );
      vec3 d1 = normalize( newRefractedDirection );

      {
        vec3 d1 = newRefractedDirection;
        vec3 d2 = refract( newRayDirection, -surfaceNormal, ( n2 + _rIndexDelta ) / n1 );
        vec3 d3 = refract( newRayDirection, -surfaceNormal, ( n2 - _rIndexDelta ) / n1 );
        vec3 specColor = vec3
        (
          SampleSpecularContribution( d2 ).r,
          SampleSpecularContribution( d1 ).g,
          SampleSpecularContribution( d3 ).b
        ) * attenuationFactor * refractedAmount * _boostFactors * _colorCorrection;

        resultColor += specColor;
      }

      vec3 reflectedAmount = EnvBRDFApprox( abs( dot( newReflectedDirection, -surfaceNormal ) ), f0, 0.0 );
      attenuationFactor *= reflectedAmount * _boostFactors;
    }

    newRayDirection = newReflectedDirection;
  }

  return resultColor;
}

mat3 rotY( float angle )
{
  float s = sin(angle);
  float c = cos(angle);

  return mat3
  (
    c, 0.0, s,
    0.0, 1.0, 0.0,
    -s, 0.0, c
  );
}

float luminosity( vec3 color )
{
  return 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
}

void main()
{
  const vec3 f0 = vec3( 0.1724 );
  vec3 normal = normalize( vWorldNormal );
  vec3 viewDirection = normalize( vWorldPosition - cameraPosition );
  vec3 reflectedDirection = reflect( viewDirection, normal );

  // An approximation of specular reflection from environment
  vec3 brdfReflected = EnvBRDFApprox( dot( reflectedDirection, normal ), f0, 0.0 );
  // Sample color from an environment map
  vec3 reflectionColor = sampleSpecularReflection( reflectedDirection );
  //reflectionColor = vec3(0.0);
  // The actual diamond calculation
  vec3 refractionColor = getRefractionColor( vWorldPosition, viewDirection, normal, 2.4 );
  // vec3 refractionColor = vec3
  // (
  //   getRefractionColor( vWorldPosition, viewDirection, normal, 2.408 ).r,
  //   getRefractionColor( vWorldPosition, viewDirection, normal, 2.424 ).g,
  //   getRefractionColor( vWorldPosition, viewDirection, normal, 2.432 ).b
  // );

  vec3 diffuseColor = diamondColor;
  vec3 colour = diffuseColor * ( refractionColor * ( vec3( 1.0 - brdfReflected ) ) +  reflectionColor * brdfReflected );
  colour = refractionColor;
  //colour = EnvBRDFApprox(abs(dot(normal, -viewDirection)), f0, 0.0);

  // vec3 p = ( inverseWorldMatrix * vec4( vWorldPosition, 1.0 ) ).xyz;
  // p = rotY(radians(0.0)) * vWorldPosition;

  // colour = texture( cubeNormalMap, vWorldPosition ).rgb * 2.0 - 1.0;
  // colour.r *= -1.0;
  // //colour = normal;
  // colour = vec3( dot( colour, normal ) );

  // Gamma
  //colour = tanh( colour * 8.0 );
  //colour *= 5.0;
  float alpha = 1.0;

  // if (luminosity( colour ) > 15.0 )
  // {
  //   emissive_color = vec4( tanh(colour), alpha );
  // }
  // else
  {
    emissive_color = vec4( 0.0 );
  }

  //colour *= 5.0;
  //colour *= pow( tanh( colour ) + vec3(1.05), vec3(16.0) );
  //colour = pow( colour, vec3( 1.0 / 2.2 ) );
  // vec3 pos = ( offsetMatrix * vec4( vWorldPosition, 1.0 ) ).xyz;
  // colour = getNormalData(pos).rgb;

  float a_weight = alpha * alpha_weight( alpha );
  

  // vec3 emissive = step(  vec3(0.9), colour ) * 1.0;
  // emissive_color = vec4( emissive, alpha );

  trasnparentA = vec4( colour * a_weight, alpha );
  transparentB = a_weight;
  frag_color = vec4( colour, alpha );
}
