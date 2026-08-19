// Fade curve shapes shared by the camera-distance fade and the inside/ribbon
// fade. Index is what the shader's uFadeMode/uInsideFadeMode uniforms expect.
export const FADE_CURVES = [
    { index: 0, label: 'Linear' },
    { index: 1, label: 'Smoothstep' },
    { index: 2, label: 'Exponential' },
    { index: 3, label: 'Exponential^2' },
];

// Live-tunable defaults for the tactical grid, shared between the tuning
// panel (src/debug/gridTuningPanel.js) and the render loop. viewRadiusOverride
// is null by default, meaning the view-zone circle follows whatever radius
// the selected unit itself defines (userData.viewRadius) - set it to a number
// to preview a different radius regardless of which unit is selected.
export const gridTuning = {
    ribbonGap: 0.3,
    // The inner ribbon (closer to the focus point) is slightly wider than
    // the outer one.
    ribbonWidthOuter: 0.8,
    ribbonWidthInner: 1.1,
    ribbonOpacity: 1.0,
    // Sampled from the reference screenshot - the ribbon isn't a flat
    // color, it has a brighter core that fades into a more saturated edge.
    ribbonColorCore: '#baecfe',
    ribbonColorEdge: '#00d8f6',

    dimAlpha: 0.15,
    brightAlpha: 0.4,

    // Inside/outside fade near the view-zone ribbon: width is in grid
    // squares (how many cells before the ribbon the fade starts), curve
    // picks the falloff shape, gamma reshapes it further (gamma < 1 fades
    // out earlier/faster, > 1 holds bright longer then drops fast).
    insideFadeWidth: 1.1,
    insideFadeCurve: 2,
    insideFadeGamma: 0.95,

    // Grid lines fade out with distance from the camera (not just from the
    // focus point), so the "infinite" plane dissolves into haze at the
    // edges instead of rendering crisply forever.
    cameraFadeStart: 0,
    cameraFadeEnd: 950,
    cameraFadeCurve: 2,
    cameraFadeGamma: 0.95,

    viewRadiusOverride: null,

    // Always-on grid brightening right around every asteroid (not tied to
    // selection), fading back to the normal dim opacity a couple of grid
    // squares out.
    asteroidGlowAlpha: 1.0,
    asteroidGlowWidth: 4.2,
    asteroidGlowCurve: 1,
    asteroidGlowGamma: 1.05,
};
