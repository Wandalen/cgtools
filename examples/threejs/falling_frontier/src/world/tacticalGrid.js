import * as THREE from 'three';
import { COLORS } from '../config/colors.js';
import { gridTuning } from '../debug/gridTuning.js';

// A huge flat plane so the grid reads as effectively infinite regardless of
// camera distance/orbit, instead of the old fixed 400-unit GridHelper.
const PLANE_SIZE = 3000;
const CELL_SIZE = 10;

// The view-zone boundary is an explicit polyline (not a smooth math curve) -
// a plain circle facets into this many straight segments, and each asteroid
// that blocks the view gets its own local segments wrapping the near side of
// its silhouette. That's what gives the ribbon its faceted, "drawn by hand"
// look instead of a perfect circle, and matches the reference: a taut string
// from the focus point that goes straight to the view radius everywhere
// except where it's pulled in around an obstacle.
const BASE_CIRCLE_SEGMENTS = 32;
const ARC_SUBDIVISIONS = 4;
const TANGENT_EPS = 0.0025;
const MAX_BOUNDARY_BLOCKERS = 12;
export const MAX_BOUNDARY_PTS = 128;

// Asteroids inside the selected unit's view range brighten the grid
// immediately around them, fading back to the normal opacity a couple of
// grid squares out - scoped to whichever asteroids/ground are currently
// inside the view-zone ribbon above, not a global effect.
export const MAX_ASTEROID_GLOW = 16;

const vertexShader = /* glsl */ `
    varying vec3 vWorldPos;
    void main() {
        vec4 worldPos = modelMatrix * vec4(position, 1.0);
        vWorldPos = worldPos.xyz;
        gl_Position = projectionMatrix * viewMatrix * worldPos;
    }
`;

// The grid line color is the SAME everywhere - only its opacity changes
// between the dim (unfocused) and bright (inside the view zone) states, so
// selecting a unit never shifts the grid's hue, just how visible it is.
const fragmentShader = /* glsl */ `
    uniform vec3 uLineColor;
    uniform vec3 uRingColorCore;
    uniform vec3 uRingColorEdge;
    uniform float uDimAlpha;
    uniform float uBrightAlpha;
    uniform vec2 uFocusPoint;
    uniform float uFocusActive;
    uniform float uViewRadius;
    uniform float uCellSize;
    uniform float uRibbonWidthOuter;
    uniform float uRibbonWidthInner;
    uniform float uRibbonGap;
    uniform float uRibbonOpacity;
    uniform float uInsideFadeWidth;
    uniform float uInsideFadeMode;
    uniform float uInsideFadeGamma;
    uniform float uCameraFadeStart;
    uniform float uCameraFadeEnd;
    uniform float uCameraFadeMode;
    uniform float uCameraFadeGamma;
    uniform vec2 uBoundaryPts[${MAX_BOUNDARY_PTS}];
    uniform int uBoundaryCount;
    uniform vec2 uAsteroidPos[${MAX_ASTEROID_GLOW}];
    uniform float uAsteroidRadius[${MAX_ASTEROID_GLOW}];
    uniform int uAsteroidCount;
    uniform float uAsteroidGlowAlpha;
    uniform float uAsteroidGlowWidth;
    uniform float uAsteroidGlowMode;
    uniform float uAsteroidGlowGamma;
    varying vec3 vWorldPos;

    // Euclidean distance from point p to the segment [a, b].
    float distToSegment(vec2 p, vec2 a, vec2 b) {
        vec2 ab = b - a;
        float t = clamp(dot(p - a, ab) / max(dot(ab, ab), 0.0001), 0.0, 1.0);
        return length(p - (a + ab * t));
    }

    // 1 where |signedDist - target| < halfWidth, antialiased falloff outside.
    float ribbonMask(float signedDist, float target, float halfWidth) {
        return 1.0 - smoothstep(halfWidth - 0.6, halfWidth, abs(signedDist - target));
    }

    // Returns 1 for x <= start, 0 for x >= end (roughly - exponential modes
    // are asymptotic so never quite hit exactly 0), shaped by "mode":
    //   0 = linear, 1 = smoothstep (ease), 2 = exponential, 3 = exponential^2
    // gamma reshapes the result afterwards (< 1 fades earlier/faster,
    // > 1 holds near 1 longer then drops off quicker at the end).
    float fadeFactor(float x, float start, float end, float mode, float gamma) {
        float span = max(end - start, 0.0001);
        float t;
        if (mode < 0.5) {
            // Linear
            t = clamp((x - start) / span, 0.0, 1.0);
        } else if (mode < 1.5) {
            // Smoothstep
            t = smoothstep(start, end, x);
        } else {
            // Exponential / exponential^2 decay, tuned so the curve has
            // dropped to ~5% by "end".
            float k = 3.0 / span;
            float d = max(x - start, 0.0);
            if (mode < 2.5) {
                t = 1.0 - exp(-k * d);
            } else {
                float e = k * d;
                t = 1.0 - exp(-e * e);
            }
        }
        return pow(clamp(1.0 - t, 0.0, 1.0), max(gamma, 0.001));
    }

    void main() {
        // Anti-aliased grid lines every uCellSize world units.
        // TODO: add a highlighted point/dot at each grid line intersection
        // (where gridUv.x and gridUv.y are both near 0), like the reference.
        vec2 coord = vWorldPos.xz / uCellSize;
        vec2 gridUv = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
        float lineMask = 1.0 - min(min(gridUv.x, gridUv.y), 1.0);

        // Fades the whole plane out with distance from the camera, so the
        // "infinite" grid dissolves into haze instead of tiling forever.
        float camDist = length(vWorldPos - cameraPosition);
        float camFade = fadeFactor(camDist, uCameraFadeStart, uCameraFadeEnd, uCameraFadeMode, uCameraFadeGamma);

        vec2 fragXZ = vWorldPos.xz;
        float innerHalf = uRibbonWidthInner * 0.5;
        float outerHalf = uRibbonWidthOuter * 0.5;
        float gapHalf = uRibbonGap * 0.5;

        // The view-zone boundary is baked on the CPU each frame into an
        // explicit closed polyline (uBoundaryPts) - a faceted circle around
        // the focus point, replaced locally by segments that wrap the near
        // side of any blocking asteroid. Here we just need the true
        // Euclidean distance from this fragment to the nearest point on that
        // polyline (for the ribbon) and whether the fragment is inside it
        // (for the brightness fade) via a standard crossing-number test -
        // both computed in one pass over the same point list, so the ribbon
        // is always a single continuous, constant-width, constant-gap band
        // with no separate "circle vs. arc" logic to fall out of sync.
        float boundaryDist = 1.0e6;
        bool insideFlag = false;
        float inside = 0.0;
        float ringInner = 0.0;
        float ringOuter = 0.0;
        float ring = 0.0;
        float asteroidGlow = 0.0;

        if (uFocusActive > 0.5 && uBoundaryCount > 0) {
            float dist = length(fragXZ - uFocusPoint);
            float nearZone = uViewRadius + outerHalf + gapHalf + uInsideFadeWidth * uCellSize + 4.0;

            if (dist < nearZone) {
                for (int i = 0; i < ${MAX_BOUNDARY_PTS}; i++) {
                    if (i >= uBoundaryCount) break;
                    int j = (i + 1 < uBoundaryCount) ? (i + 1) : 0;
                    vec2 a = uBoundaryPts[i];
                    vec2 b = uBoundaryPts[j];
                    boundaryDist = min(boundaryDist, distToSegment(fragXZ, a, b));

                    if ((a.y > fragXZ.y) != (b.y > fragXZ.y)) {
                        float xCross = a.x + (b.x - a.x) * (fragXZ.y - a.y) / (b.y - a.y);
                        if (fragXZ.x < xCross) {
                            insideFlag = !insideFlag;
                        }
                    }
                }

                float signedDist = insideFlag ? -boundaryDist : boundaryDist;

                // 1 well inside the boundary, fading down to 0 (dim) over
                // uInsideFadeWidth grid squares as it approaches the ribbon.
                float insideFadeStart = -uInsideFadeWidth * uCellSize;
                inside = uFocusActive * fadeFactor(signedDist, insideFadeStart, 0.0, uInsideFadeMode, uInsideFadeGamma);

                // Both ribbons are constant-offset copies of the SAME
                // boundary distance, so their width and the gap between them
                // stay constant all the way through a bend.
                ringInner = uFocusActive * ribbonMask(signedDist, -(gapHalf + innerHalf), innerHalf);
                ringOuter = uFocusActive * ribbonMask(signedDist, gapHalf + outerHalf, outerHalf);
                ring = clamp(ringInner + ringOuter, 0.0, 1.0);

                // Asteroids only brighten the grid where that ground is
                // actually visible from the focus point - inside the ribbon,
                // never past it - and uAsteroidPos/Count is itself already
                // pre-filtered on the CPU to asteroids within the current
                // view range, so nothing outside the selected unit's view
                // zone glows at all.
                if (insideFlag) {
                    float glowSpan = max(uAsteroidGlowWidth * uCellSize, 0.0001);
                    for (int i = 0; i < ${MAX_ASTEROID_GLOW}; i++) {
                        if (i >= uAsteroidCount) break;
                        float r = uAsteroidRadius[i];
                        if (r <= 0.0) continue;
                        float surfaceDist = length(fragXZ - uAsteroidPos[i]) - r;
                        if (surfaceDist > glowSpan) continue;
                        float g = fadeFactor(surfaceDist, 0.0, glowSpan, uAsteroidGlowMode, uAsteroidGlowGamma);
                        asteroidGlow = max(asteroidGlow, g);
                    }
                }
            }
        }

        // The ribbon isn't a flat color - it has a brighter core that
        // fades into a more saturated edge color, antialiased by reusing
        // each ribbon's own distance-based falloff as the color mix factor.
        float ringCoreMix = clamp(max(ringInner, ringOuter), 0.0, 1.0);
        vec3 ringColor = mix(uRingColorEdge, uRingColorCore, ringCoreMix);

        float combinedMask = max(lineMask, ring);
        if (combinedMask < 0.01) discard;

        vec3 color = mix(uLineColor, ringColor, ring);

        // The ribbon is a tactical HUD marker, not part of the hazy
        // "infinite" grid - it stays at its own opacity regardless of how
        // far the camera is, only the grid lines fade with distance. The
        // asteroid glow is a second, independent brightness source - take
        // whichever of the two (view-zone focus, asteroid proximity) is
        // stronger at this fragment rather than stacking them additively.
        float focusBright = mix(uDimAlpha, uBrightAlpha, inside);
        float glowBright = mix(uDimAlpha, uAsteroidGlowAlpha, asteroidGlow);
        float gridAlpha = max(focusBright, glowBright) * lineMask * camFade;
        float ringAlpha = ring * uRibbonOpacity;
        float alpha = max(gridAlpha, ringAlpha);
        if (alpha < 0.003) discard;

        gl_FragColor = vec4(color, alpha);
    }
`;

// Builds the shader-driven tactical grid plane and returns its material so
// the caller can drive the "view zone" focus circle via setGridFocus() and
// live-tune it via the grid tuning panel.
export function createTacticalGrid(gridGroup) {
    const geometry = new THREE.PlaneGeometry(PLANE_SIZE, PLANE_SIZE);
    geometry.rotateX(-Math.PI / 2);

    const material = new THREE.ShaderMaterial({
        uniforms: {
            uLineColor: { value: new THREE.Color(COLORS.gridCyan) },
            uRingColorCore: { value: new THREE.Color(gridTuning.ribbonColorCore) },
            uRingColorEdge: { value: new THREE.Color(gridTuning.ribbonColorEdge) },
            uDimAlpha: { value: gridTuning.dimAlpha },
            uBrightAlpha: { value: gridTuning.brightAlpha },
            uFocusPoint: { value: new THREE.Vector2(0, 0) },
            uFocusActive: { value: 0 },
            uViewRadius: { value: 40 },
            uCellSize: { value: CELL_SIZE },
            uRibbonWidthOuter: { value: gridTuning.ribbonWidthOuter },
            uRibbonWidthInner: { value: gridTuning.ribbonWidthInner },
            uRibbonGap: { value: gridTuning.ribbonGap },
            uRibbonOpacity: { value: gridTuning.ribbonOpacity },
            uInsideFadeWidth: { value: gridTuning.insideFadeWidth },
            uInsideFadeMode: { value: gridTuning.insideFadeCurve },
            uInsideFadeGamma: { value: gridTuning.insideFadeGamma },
            uCameraFadeStart: { value: gridTuning.cameraFadeStart },
            uCameraFadeEnd: { value: gridTuning.cameraFadeEnd },
            uCameraFadeMode: { value: gridTuning.cameraFadeCurve },
            uCameraFadeGamma: { value: gridTuning.cameraFadeGamma },
            uBoundaryPts: { value: Array.from({ length: MAX_BOUNDARY_PTS }, () => new THREE.Vector2()) },
            uBoundaryCount: { value: 0 },
            uAsteroidPos: { value: Array.from({ length: MAX_ASTEROID_GLOW }, () => new THREE.Vector2()) },
            uAsteroidRadius: { value: new Array(MAX_ASTEROID_GLOW).fill(0) },
            uAsteroidCount: { value: 0 },
            uAsteroidGlowAlpha: { value: gridTuning.asteroidGlowAlpha },
            uAsteroidGlowWidth: { value: gridTuning.asteroidGlowWidth },
            uAsteroidGlowMode: { value: gridTuning.asteroidGlowCurve },
            uAsteroidGlowGamma: { value: gridTuning.asteroidGlowGamma },
        },
        vertexShader,
        fragmentShader,
        transparent: true,
        depthWrite: false,
    });
    material.extensions = { ...material.extensions, derivatives: true };

    const mesh = new THREE.Mesh(geometry, material);
    gridGroup.add(mesh);

    return material;
}

// point: THREE.Vector3-like with x/z, or null/undefined to dim the whole grid.
export function setGridFocus(material, point, radius) {
    if (point) {
        material.uniforms.uFocusActive.value = 1;
        material.uniforms.uFocusPoint.value.set(point.x, point.z);
        material.uniforms.uViewRadius.value = radius;
    } else {
        material.uniforms.uFocusActive.value = 0;
    }
}

// Finds the angle (relative to the focus point) at which the ray from the
// focus first hits a blocker, for every direction where it does. Used both
// to sample the near-side wrap of each blocker and to let overlapping
// blockers correctly shadow each other (always takes the nearest hit).
function sampleBoundaryRadius(dx, dz, focusX, focusZ, viewRadius, activeBlockers) {
    let r = viewRadius;
    for (const b of activeBlockers) {
        const ocx = b.x - focusX;
        const ocz = b.z - focusZ;
        const tCenter = ocx * dx + ocz * dz;
        if (tCenter <= 0) continue;
        const rayX = ocx - dx * tCenter;
        const rayZ = ocz - dz * tCenter;
        const distToRay = Math.hypot(rayX, rayZ);
        if (distToRay >= b.r) continue;
        const halfChord = Math.sqrt(Math.max(b.r * b.r - distToRay * distToRay, 0));
        const tHit = tCenter - halfChord;
        if (tHit > 0 && tHit < r) r = tHit;
    }
    return r;
}

const TWO_PI = Math.PI * 2;
function normalizeAngle(a) {
    return ((a % TWO_PI) + TWO_PI) % TWO_PI;
}

// Builds the closed boundary polyline (in world XZ) that the ribbon wraps:
// a faceted circle of BASE_CIRCLE_SEGMENTS around the focus point, with
// extra angle samples inserted at and around each blocking asteroid's
// tangent points so the wrap snaps in tight (near-vertical "return to
// radius value" corners) rather than being smoothed out by the coarse base
// sampling. Writes into `out` (an array of THREE.Vector2) and returns the
// point count actually used.
function buildBoundaryPolyline(focusX, focusZ, viewRadius, blockers, out) {
    const active = [];
    for (const b of blockers) {
        if (active.length >= MAX_BOUNDARY_BLOCKERS) break;
        const ocx = b.x - focusX;
        const ocz = b.z - focusZ;
        const d = Math.hypot(ocx, ocz);
        if (d <= b.radius) continue; // focus point is inside the asteroid - degenerate, ignore
        const tangentLen = Math.sqrt(Math.max(d * d - b.radius * b.radius, 0));
        if (tangentLen >= viewRadius) continue; // doesn't reach into the view radius - no wrap needed
        const centerAngle = Math.atan2(ocz, ocx);
        const halfAngle = Math.asin(Math.min(b.radius / d, 1));
        active.push({ x: b.x, z: b.z, r: b.radius, aStart: centerAngle - halfAngle, aEnd: centerAngle + halfAngle });
    }

    const angles = [];
    for (let i = 0; i < BASE_CIRCLE_SEGMENTS; i++) {
        angles.push((i / BASE_CIRCLE_SEGMENTS) * TWO_PI);
    }
    for (const w of active) {
        angles.push(w.aStart - TANGENT_EPS, w.aStart, w.aEnd, w.aEnd + TANGENT_EPS);
        for (let k = 1; k < ARC_SUBDIVISIONS; k++) {
            angles.push(w.aStart + (w.aEnd - w.aStart) * (k / ARC_SUBDIVISIONS));
        }
    }

    for (let i = 0; i < angles.length; i++) {
        angles[i] = normalizeAngle(angles[i]);
    }
    angles.sort((a, b) => a - b);

    let count = 0;
    let lastAngle = null;
    const limit = Math.min(angles.length, MAX_BOUNDARY_PTS);
    for (let i = 0; i < limit; i++) {
        const angle = angles[i];
        if (lastAngle !== null && Math.abs(angle - lastAngle) < 1e-6) continue;
        lastAngle = angle;

        const dx = Math.cos(angle);
        const dz = Math.sin(angle);
        const r = sampleBoundaryRadius(dx, dz, focusX, focusZ, viewRadius, active);
        out[count].set(focusX + dx * r, focusZ + dz * r);
        count++;
    }
    return count;
}

// Recomputes the view-zone boundary polyline from the current focus point
// (already set via setGridFocus) and the live asteroid positions/radii, and
// uploads it to the shader. Cheap enough to call every frame so a moved
// focus or a dragged asteroid stays in sync.
export function updateGridBoundary(material, asteroidsGroup) {
    if (!material.uniforms.uFocusActive.value) {
        material.uniforms.uBoundaryCount.value = 0;
        return;
    }

    const focus = material.uniforms.uFocusPoint.value;
    const viewRadius = material.uniforms.uViewRadius.value;
    const blockers = [];
    for (const asteroid of asteroidsGroup.children) {
        const radius = asteroid.userData.blockRadius;
        if (radius > 0) {
            blockers.push({ x: asteroid.position.x, z: asteroid.position.z, radius });
        }
    }

    const count = buildBoundaryPolyline(focus.x, focus.y, viewRadius, blockers, material.uniforms.uBoundaryPts.value);
    material.uniforms.uBoundaryCount.value = count;
}

// Feeds the asteroids that fall within the current focus's view range into
// the shader for the proximity glow - nothing glows when no unit is
// selected, and an asteroid outside the selected unit's view radius doesn't
// glow either, since the shader also clips the effect to the ribbon's
// interior. Cheap enough to call every frame so a moved focus or a dragged
// asteroid stays in sync.
export function updateAsteroidGlow(material, asteroidsGroup) {
    const radii = material.uniforms.uAsteroidRadius.value;

    if (!material.uniforms.uFocusActive.value) {
        material.uniforms.uAsteroidCount.value = 0;
        return;
    }

    const positions = material.uniforms.uAsteroidPos.value;
    const focus = material.uniforms.uFocusPoint.value;
    const viewRadius = material.uniforms.uViewRadius.value;

    let count = 0;
    for (const asteroid of asteroidsGroup.children) {
        if (count >= MAX_ASTEROID_GLOW) break;
        const radius = asteroid.userData.blockRadius;
        if (!(radius > 0)) continue;

        const dx = asteroid.position.x - focus.x;
        const dz = asteroid.position.z - focus.y;
        const distToSurface = Math.hypot(dx, dz) - radius;
        if (distToSurface >= viewRadius) continue; // entirely outside the view range

        positions[count].set(asteroid.position.x, asteroid.position.z);
        radii[count] = radius;
        count++;
    }
    for (let i = count; i < MAX_ASTEROID_GLOW; i++) {
        radii[i] = 0;
    }

    material.uniforms.uAsteroidCount.value = count;
}
