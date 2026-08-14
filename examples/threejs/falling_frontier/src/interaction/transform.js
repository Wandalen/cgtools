import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';

// Which gizmo handles are visible/active per mode - this is what constrains
// dragging to the XZ plane in translate mode and to the Y axis in rotate mode.
const MODE_CONSTRAINTS = {
    translate: { showX: true, showY: false, showZ: true, showXY: false, showYZ: false, showXZ: true, showE: false, showXYZE: false },
    rotate: { showX: false, showY: true, showZ: false, showE: false, showXYZE: false },
};

function applyConstraints(gizmo, mode) {
    for (const [key, value] of Object.entries(MODE_CONSTRAINTS[mode])) {
        gizmo[key] = value;
    }
}

export function setGizmoMode(gizmo, mode) {
    gizmo.setMode(mode);
    applyConstraints(gizmo, mode);
}

export function attachGizmo(gizmo, object) {
    gizmo.attach(object);
}

export function detachGizmo(gizmo) {
    gizmo.detach();
}

// Set right after a gizmo drag (or even a plain click on a handle) finishes,
// so the synthetic 'click' event that follows doesn't fall through to
// selection raycasting and immediately deselect/reselect through the gizmo.
export const gizmoState = { wasDragging: false };

// XZ-move / Y-rotate gizmo for repositioning ships and the station.
// Controls: G = move (XZ plane), R = rotate (Y axis), Escape = deselect.
export function createGizmo(world) {
    const gizmo = new TransformControls(world.camera, world.renderer.domElement);
    setGizmoMode(gizmo, 'translate');

    // Dragging the gizmo shouldn't also orbit the camera, and the click that
    // ends a drag shouldn't be re-interpreted as a selection click.
    gizmo.addEventListener('dragging-changed', (event) => {
        world.controls.enabled = !event.value;
        if (!event.value) {
            gizmoState.wasDragging = true;
            setTimeout(() => { gizmoState.wasDragging = false; }, 0);
        }
    });

    world.scene.add(gizmo.getHelper());

    window.addEventListener('keydown', (e) => {
        if (!gizmo.object) return;
        if (e.key === 'g' || e.key === 'G') setGizmoMode(gizmo, 'translate');
        if (e.key === 'r' || e.key === 'R') setGizmoMode(gizmo, 'rotate');
        if (e.key === 'Escape') detachGizmo(gizmo);
    });

    return gizmo;
}
