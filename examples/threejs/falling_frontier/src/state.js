// Mutable playback state shared between the UI controls and the animation loop.
export const playbackState = {
    // Off by default while the static layout is being blocked out with the
    // transform gizmo - flip this on (or the "Animate Ships Motion" button)
    // once positions are locked in.
    isAnimating: false,
    shipSpeedMultiplier: 1,
};
