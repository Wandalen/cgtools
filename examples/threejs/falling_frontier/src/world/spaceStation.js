import * as THREE from 'three';
import { COLORS } from '../config/colors.js';
import { createHeightGuideLine } from './trajectories.js';

export const STATION_SPEC = {
    name: 'OUTPOST ALPHA-7',
    commander: 'ADMIRAL VANCE',
    // Negative Y sits it below the tactical grid plane (y=0).
    position: [-89.6, -15, 17.32],
    viewRadius: 160,
};

export function createSpaceStation(stationGroup, trajectoryGroup, gridGroup) {
    const station = new THREE.Group();
    station.position.set(...STATION_SPEC.position);

    const darkMat = new THREE.MeshStandardMaterial({ color: COLORS.shipDark, roughness: 0.5, metalness: 0.8 });
    const metallicMat = new THREE.MeshStandardMaterial({ color: COLORS.shipHull, roughness: 0.3, metalness: 0.9 });
    const glowMat = new THREE.MeshBasicMaterial({ color: COLORS.engineGlow });

    // Central Axis Core Cylinder
    const core = new THREE.Mesh(new THREE.CylinderGeometry(4, 4, 30, 12), darkMat);
    station.add(core);

    // Rotating Ring Structure
    const ring = new THREE.Mesh(new THREE.TorusGeometry(18, 1.8, 8, 24), metallicMat);
    ring.rotation.x = Math.PI / 2;
    station.add(ring);

    // Connecting Spoke Arms
    for (let i = 0; i < 4; i++) {
        const spoke = new THREE.Mesh(new THREE.BoxGeometry(32, 1, 1.5), darkMat);
        spoke.rotation.y = (Math.PI / 4) * i;
        station.add(spoke);
    }

    // Docking Modules
    for (let i = 0; i < 6; i++) {
        const mod = new THREE.Mesh(new THREE.CylinderGeometry(2, 2.5, 6, 8), metallicMat);
        const angle = (Math.PI / 3) * i;
        mod.position.set(Math.cos(angle) * 18, 0, Math.sin(angle) * 18);
        station.add(mod);
    }

    // Beacon Glow Light
    const lightMesh = new THREE.Mesh(new THREE.SphereGeometry(0.8, 8, 8), glowMat);
    lightMesh.position.set(0, 16, 0);
    station.add(lightMesh);

    station.castShadow = true;
    station.receiveShadow = true;
    station.userData = {
        type: 'Station',
        name: STATION_SPEC.name,
        commander: STATION_SPEC.commander,
        viewRadius: STATION_SPEC.viewRadius,
    };

    stationGroup.add(station);

    // Vertical guide line to tactical grid
    createHeightGuideLine(station.position, trajectoryGroup, gridGroup);

    return station;
}

// A station attached to the transform gizmo is skipped so rotating it by
// hand isn't fought by the idle spin.
export function spinStations(stationGroup, excludeMesh = null) {
    stationGroup.children.forEach((station) => {
        if (station === excludeMesh) return;
        station.rotation.y += 0.002;
    });
}
