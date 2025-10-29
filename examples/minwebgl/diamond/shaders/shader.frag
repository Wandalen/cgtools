#version 300 es

precision mediump float;

const float EPSILON = 1e-4;
// Reflectance of a diamond at light incidence of theta = 0
//const vec3 F0 = vec3( 0.1724 );
const vec3 F0 = 1.0 / vec3( 2.407, 2.426, 2.451 );
// Max distance to the surface in cubeNormalMap
// This value was calculated during generation of the map
//const float MAX_DISTANCE = 5.7610855;
const float MAX_DISTANCE = 1.0;
const int RAY_BOUNCES = 5;
const float TRANSMISSION = 0.5;
const vec3 BOOST_FACTORS = vec3( 0.8920, 0.8920, 0.9860 );

uniform samplerCube envMap;
uniform samplerCube cubeNormalMap;

uniform mat4x4 modelMatrix;
uniform mat4x4 inverseModelMatrix;

uniform float envMapIntensity;
uniform float rainbowDelta;
uniform float squashFactor;
uniform float radius;
uniform float geometryFactor;
uniform float absorptionFactor;
uniform vec3 colorAbsorption;
uniform vec3 cameraPosition;

in vec2 vUvs;
in vec3 vWorldNormal;
in vec3 vWorldPosition;
in vec3 vViewPosition;

out vec4 frag_color;

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
  data.a *= MAX_DISTANCE;
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

vec3 sampleSpecularReflection( vec3 direction ) 
{
  vec3 sample_value = texture( envMap, direction ).rgb;
  return envMapIntensity * pow( sample_value, vec3( 2.2 ) );
}

vec3 convertDirLocalToWorld( vec3 direction ) 
{
  return  mat3x3( modelMatrix ) * direction;
}

vec3 sampleEnvFromLocal( vec3 direction ) 
{
  vec3 sample_value = texture( envMap, convertDirLocalToWorld( direction ) ).rgb;
  return envMapIntensity * pow( sample_value, vec3( 2.2 ) );
}

vec3 SampleSpecularContribution( vec3 direction ) 
{
  direction = normalize( direction );
  direction.x *= -1.;
  direction.z *= -1.;
  //return vec3( 1.0, 0.0, 1.0 );
  return sampleEnvFromLocal( direction ) * 5.0;
}

// Finds an intersection points of a given line with a sphere at the origin
// and picks the father of the two possible solutions
vec3 intersectSphere( vec3 origin, vec3 direction ) 
{
  direction.y /= squashFactor;

  // Having parametric equation for the line in 'direction'
  // Solve the quadratic equation for 't' using sphere equation
  float A = dot( direction, direction );
  float B = 2.0 * dot( origin, direction );
  float C = dot( origin, origin ) - radius * radius;
  float disc = B * B - 4.0 * A * C;
  if( disc > 0.0 ) 
  {
      disc = sqrt( disc );
      float x1 = ( -B + disc ) * geometryFactor / A;
      float x2 = ( -B - disc ) * geometryFactor / A;
      float t = ( x1 > x2 ) ? x1 : x2;
      direction.y *= squashFactor;
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
  vec3 surfaceNormal = -normalData.rgb;
  float surfaceDistance = normalData.a * radius;

  // Point on the surface of the diamond
  vec3 pointOnSurface1 = directionToSpherePoint * surfaceDistance;

  vec3 planeHitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface1, surfaceNormal );
  vec3 directionToPlanePoint = normalize( planeHitPoint );

  normalData = getNormalData( directionToPlanePoint );
  surfaceNormal = -normalData.rgb;
  surfaceDistance = normalData.a * radius;

  // Point on the surface of the diamond
  vec3 pointOnSurface2 = directionToPlanePoint * surfaceDistance;

  vec3 hitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface2, surfaceNormal );
  return hitPoint;
}

// Approximates a point of intersection of the diamond with the given ray direction
// vec3 intersectDiamond( vec3 rayOrigin, vec3 rayDirection ) 
// {
//   // Intersect a sphere at the center
//   vec3 sphereHitPoint = intersectSphere( rayOrigin, rayDirection );
//   // Direction from the center to the hit point on the sphere
//   vec3 directionToSpherePoint = normalize( sphereHitPoint );
//   // Sample the normal of the diamond in that direction
//   // n.rgb - normal, n.a - distance to the surface
//   vec4 normalData = getNormalData( directionToSpherePoint );
//   // Flip the normal to point inwards
//   vec3 surfaceNormal = -normalData.rgb;
//   float surfaceDistance = normalData.a;

//   // Point on the surface of the diamond
//   vec3 pointOnSurface1 = directionToSpherePoint * surfaceDistance;

//   vec3 planeHitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface1, surfaceNormal );
//   vec3 directionToPlanePoint = normalize( planeHitPoint );

//   normalData = getNormalData( directionToPlanePoint );
//   surfaceNormal = -normalData.rgb;
//   surfaceDistance = normalData.a;

//   // Point on the surface of the diamond
//   vec3 pointOnSurface2 = directionToPlanePoint * surfaceDistance;

//   vec3 hitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface2, surfaceNormal );
//   return hitPoint;
// }

// vec3 getSurfaceNormal(vec4 surfaceInfos) {
//     vec3 surfaceNormal = surfaceInfos.rgb;
//     surfaceNormal = surfaceNormal*2.-1.;
//     return-normalize(surfaceNormal);
// }

// vec3 intersect(vec3 rayOrigin, vec3 rayDirection) {
//     vec3 sphereHitPoint = intersectSphere(rayOrigin, rayDirection);
//     vec3 direction1 = normalize(sphereHitPoint);
//     vec4 normalDistanceData1 = getNormalData(direction1);
//     float distance1 = normalDistanceData1.a*radius;
//     vec3 pointOnPlane1 = direction1*distance1;
//     vec3 planeNormal1 = getSurfaceNormal(normalDistanceData1);
//     vec3 hitPoint1 = linePlaneIntersect(rayOrigin, rayDirection, pointOnPlane1, planeNormal1);
//     vec3 direction2 = normalize(hitPoint1);
//     vec4 normalDistanceData2 = getNormalData(direction2);
//     float distance2 = normalDistanceData2.a*radius;
//     vec3 pointOnPlane2 = direction2*distance2;
//     vec3 hitPoint = hitPoint1;
//     vec3 planeNormal2 = getSurfaceNormal(normalDistanceData2);
//     hitPoint = linePlaneIntersect(rayOrigin, rayDirection, pointOnPlane2, planeNormal2);
//     return hitPoint;
// }

// Calculate the color by tracing a ray
vec3 getRefractionColor( vec3 rayHitPoint, vec3 rayDirection, vec3 hitPointNormal ) 
{
  vec3 resultColor = vec3( 0.0 );

  // Refractive index of air
  const float n1 = 1.0;
  // Refractive index of a diamond
  const float n2 = 2.42;

  vec3 f0 = vec3( (n2 - n1) / (n2 + n1));
  f0 *= f0;

  float iorRatioAtoD = n1 / n2;
  float iorRatioDtoA = n2 / n1;

  vec3 lightAbsorption = vec3( 0.8 );

  //resultColor = max(0.0, dot(normalize(vec3(1.0)), hitPointNormal)) * vec3( 1.0 );

  // Angle of total refleciton
  float criticalAngleCosine = sqrt( max( 0.0, 1.0 - (iorRatioAtoD * iorRatioAtoD) ) );

  vec3 newRayDirection = refract( rayDirection, hitPointNormal, iorRatioAtoD );
  // Convert data to local space
  newRayDirection = ( inverseModelMatrix * vec4( newRayDirection, 0.0 ) ).xyz;
  newRayDirection = normalize( newRayDirection );
  vec3 rayOrigin =  ( inverseModelMatrix * vec4( rayHitPoint, 1.0 ) ).xyz;

  float totalDistance = 0.0;
  vec3 diffuseColor = vec3( 1.0 );
  // Overall intensity of the light as it goes through the medium
  vec3 attenuationFactor = vec3( 1.0 );

  vec3 reflectedAmount = EnvBRDFApprox( dot( -rayDirection, hitPointNormal ), F0, 0.0 );
  // Only take into account transmitted part
  attenuationFactor *= ( vec3( 1.0 ) - reflectedAmount );

  for( int i = 0; i < RAY_BOUNCES; i++ ) 
  {
    // Intersection point on the diamond surface
    vec3 intersectPos = intersectDiamond( rayOrigin, newRayDirection );
    vec3 dirOriginToIntersect = normalize( intersectPos );

    // Get normal in direction to the intersected position
    vec4 normalData = getNormalData( dirOriginToIntersect );
    vec3 surfaceNormal = normalData.rgb;
    float surfaceDistance = normalData.a;  

    // Update the origin position
    vec3 oldOrigin = rayOrigin;
    rayOrigin = dirOriginToIntersect * surfaceDistance;

    float r = length( rayOrigin - oldOrigin ) / radius * absorptionFactor;
    attenuationFactor *= exp( -r * ( 1.0 - colorAbsorption ) );

    //totalDistance += length( rayOrigin - oldOrigin );

    // Calculate new rays

    vec3 newReflectedDirection = reflect( newRayDirection, -surfaceNormal );
    vec3 newRefractedDirection = refract( newRayDirection, -surfaceNormal, iorRatioDtoA );

    // vec3 FRefracted = freshel( newRefractedDirection, surfaceNormal, F0, 0.0 );
    // vec3 FReflected = freshel( newReflectedDirection, -surfaceNormal, F0, criticalAngleCosine );

    if( dot( newRefractedDirection, newRefractedDirection ) < 1e-2 )
    {
      if ( i == RAY_BOUNCES - 1 ) 
      {
        vec3 reflectedAmount = EnvBRDFApprox( dot( newRayDirection, surfaceNormal ), F0, 0.0 );
        newRayDirection = normalize( newRayDirection );
        float cosT = 1.0 - dot( newRayDirection, rayDirection );

        if( TRANSMISSION > 0.0 && cosT < TRANSMISSION )
        {
          resultColor += vec3( 1.0 ) * 0.1;
        }
        else
        {
          resultColor += SampleSpecularContribution( newRayDirection ) * attenuationFactor * BOOST_FACTORS * ( vec3( 1.0 ) - min( vec3( 1.0 ), reflectedAmount ) );
        }
      }
    }
    else
    {
      vec3 refractedAmount = vec3( 1.0 ) - min( vec3( 1.0 ), EnvBRDFApprox( dot( newRefractedDirection, surfaceNormal ), F0, 0.0 ) );
      vec3 d1 = normalize( newRefractedDirection );
      float cosT = 1.0 - dot( d1, rayDirection );

      if( TRANSMISSION > 0.0 && cosT < TRANSMISSION )
      {
        vec3 specColor = vec3( 1.0 ) * refractedAmount * attenuationFactor;
        resultColor += specColor;
      }
      else
      {
        vec3 d1 = newRefractedDirection;
        vec3 d2 = refract( newRayDirection, -surfaceNormal, ( n2 + rainbowDelta ) / n1 );
        vec3 d3 = refract( newRayDirection, -surfaceNormal, ( n2 - rainbowDelta ) / n1 );
        vec3 specColor = vec3
        (
          SampleSpecularContribution( d2 ).r,
          SampleSpecularContribution( d1 ).g,
          SampleSpecularContribution( d3 ).b
        ) * refractedAmount * attenuationFactor;

        resultColor += specColor;
      }
    }

    vec3 reflectedAmount = EnvBRDFApprox( dot( newReflectedDirection, -surfaceNormal ), F0, 0.0 );
    attenuationFactor *= reflectedAmount;
    newRayDirection = newReflectedDirection;



    // float RaydotN = dot( newRayDirection, surfaceNormal );
    // //Case of total reflection
    // //Needs more work to be done
    // if( RaydotN <= criticalAngleCosine ) 
    // // {
    // //   vec3 brdfEnvReflected = EnvBRDFApprox( dot( newReflectedDirection, -surfaceNormal ), F0, 0.0 );
    // //   resultColor += sampleEnvFromLocal( newReflectedDirection ) * brdfEnvReflected * attenuationFactor;
    // // }
    // // // Light dispersion that causes a rainbow to appear
    // // else if( RaydotN < 0.99 ) 
    // // {
    // //   vec3 dirGreen = newRefractedDirection;
    // //   vec3 dirRed = refract( newRayDirection, -surfaceNormal, ( n2 + rainbowDelta ) / n1 );
    // //   vec3 dirBlue = refract( newRayDirection, -surfaceNormal, ( n2 - rainbowDelta ) / n1 );

    // //   vec3 sampleColor = vec3
    // //   (
    // //     sampleEnvFromLocal( dirRed ).r,
    // //     sampleEnvFromLocal( dirGreen ).g,
    // //     sampleEnvFromLocal( dirBlue ).b
    // //   );

    // //   resultColor += sampleColor * ( vec3( 1.0 ) - FRefracted ) * attenuationFactor;
    // // }
    // // // Incident angle is 0 degress
    // // else 
    // // {
    // //   vec3 brdfEnvRefracted = EnvBRDFApprox( dot( newRefractedDirection, surfaceNormal ), F0, 0.0 );
    // //   vec3 sampleColor =  sampleEnvFromLocal( newRefractedDirection );
    // //   resultColor += sampleColor * ( vec3( 1.0 ) - FRefracted ) * attenuationFactor;
    // //   break;
    // // }

    

    // vec3 envBrdf = EnvBRDFApprox( dot( newReflectedDirection, -surfaceNormal ), F0, 0.0 );
    // attenuationFactor *= envBrdf;
    // newRayDirection = newReflectedDirection;
  }

  return resultColor;
}


void main() 
{
  vec3 normal = normalize( vWorldNormal );
  vec3 viewDirection = normalize( vWorldPosition - cameraPosition );
  vec3 reflectedDirection = reflect( viewDirection, normal );

  // An approximation of specular reflection from environment
  vec3 brdfReflected = EnvBRDFApprox( dot( reflectedDirection, normal ), F0, 0.0 );
  // Sample color from an environment map
  vec3 reflectionColor = sampleSpecularReflection( reflectedDirection ); 
  // The actual diamond calculation
  vec3 refractionColor = getRefractionColor( vWorldPosition, viewDirection, normal );

  vec3 diffuseColor = vec3( 1.0 );
  vec3 colour = diffuseColor * ( refractionColor +  pow( reflectionColor * brdfReflected, vec3( 1.0 / 2.0) ) );
  //colour = refractionColor;

  // Gamma 
  colour = pow( colour, vec3( 1.0 / 2.2 ) );

  frag_color = vec4( colour, 1.0 );
}

