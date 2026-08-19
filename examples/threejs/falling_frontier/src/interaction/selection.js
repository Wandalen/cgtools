import * as THREE from 'three';
import { selectUnit } from '../ui/unitPanel.js';
import { attachGizmo, detachGizmo, gizmoState } from './transform.js';

// Pointer has to stay within this many pixels between down/up for it to
// count as a selection click rather than a camera orbit/pan drag.
const CLICK_DRAG_THRESHOLD = 6;

// Click-to-select raycasting against ships, the station and asteroids,
// ignoring UI clicks. Selecting an object also attaches the transform gizmo
// to it; clicking empty space detaches it. Uses pointerdown/pointerup (not
// the native 'click' event) so orbiting/panning the camera - which also
// fires 'click' on release - doesn't reset the current selection.
export function setupSelection(world, gizmo) {
    const { scene, camera, groups } = world;
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();
    let downPos = null;

    window.addEventListener('pointerdown', (e) => {
        downPos = { x: e.clientX, y: e.clientY };
    });

    window.addEventListener('pointerup', (e) => {
        if (!downPos) return;
        const dx = e.clientX - downPos.x;
        const dy = e.clientY - downPos.y;
        downPos = null;
        if (dx * dx + dy * dy > CLICK_DRAG_THRESHOLD * CLICK_DRAG_THRESHOLD) return;

        if (gizmoState.wasDragging) return;
        if (e.target.closest('.ui-panel') || e.target.closest('button')) return;

        mouse.x = (e.clientX / window.innerWidth) * 2 - 1;
        mouse.y = -(e.clientY / window.innerHeight) * 2 + 1;

        raycaster.setFromCamera(mouse, camera);
        const intersects = raycaster.intersectObjects(
            [...groups.ships.children, ...groups.station.children, ...groups.asteroids.children],
            true
        );

        if (intersects.length > 0) {
            let obj = intersects[0].object;
            while (obj.parent && !obj.userData.type && obj.parent !== scene) {
                obj = obj.parent;
            }
            if (obj.userData.name) {
                selectUnit(obj);
                attachGizmo(gizmo, obj);
            }
        } else {
            detachGizmo(gizmo);
        }
    });
}
