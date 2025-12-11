#ifndef USE_ENVMAP
#define USE_ENVMAP
#endif

varying vec3 vWorldPosition;
varying vec3 vWorldNormal;
varying vec3 vViewPosition;
varying vec3 vNormal;
varying vec2 vUv;

#ifdef USE_INSTANCING
uniform mat4 inverseModelMatrix;
uniform mat4 modelOffsetMatrix;
varying mat4 vModelInstanceOffsetMatrix;
varying mat4 vModelInstanceOffsetMatrixInv;
#endif

#include <morphtarget_pars_vertex>

#ifdef USE_MORPHTARGETS
varying vec3 vCenterOffset;
#ifndef USE_INSTANCING
uniform mat4 modelOffsetMatrixInv;
#endif
#endif

void main() {
#ifdef USE_INSTANCING
    vWorldNormal = (modelMatrix * instanceMatrix * vec4(normal, 0.)).xyz;
#else
    vWorldNormal = (modelMatrix * vec4(normal, 0.)).xyz;
#endif

#include <beginnormal_vertex>
#include <defaultnormal_vertex>
#include <normal_vertex>
#include <begin_vertex>

#ifdef USE_MORPHTARGETS
    vCenterOffset = transformed;
#include <morphtarget_vertex>
    vCenterOffset = transformed - vCenterOffset;
#ifndef USE_INSTANCING
    vCenterOffset = (modelOffsetMatrixInv * modelMatrix * vec4(vCenterOffset, 1.)).xyz;
#endif
#endif

#include <project_vertex>

    vViewPosition = -mvPosition.xyz;
    vUv = uv;

#include <worldpos_vertex>

    vWorldPosition = worldPosition.xyz;

#ifdef USE_INSTANCING
    vModelInstanceOffsetMatrix = modelMatrix * instanceMatrix * inverseModelMatrix * modelOffsetMatrix;
    vModelInstanceOffsetMatrixInv = inverse(vModelInstanceOffsetMatrix);
#ifdef USE_MORPHTARGETS
    vCenterOffset = (vModelInstanceOffsetMatrixInv * modelMatrix * vec4(vCenterOffset, 1.)).xyz;
#endif
#endif
}
