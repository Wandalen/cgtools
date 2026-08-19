import * as THREE from 'three';
import { COLORS } from '../config/colors.js';

// Asteroids sit at the same altitude as the ships (matches TDF RELIANT) so
// the whole belt reads as one flat tactical plane instead of floating at
// random heights.
const ASTEROID_Y = 12;

// deformToRock() below bulges vertices up to ~17.5% past the nominal
// dodecahedron radius, so the view-zone ribbon (which wraps at the nominal
// radius) can end up visually clipping into the jagged rock. Padding the
// blocking radius past the max bulge keeps the ribbon clear of the mesh.
const ASTEROID_BLOCK_PADDING = 1.3;

// Fixed default layout (not randomized) so positions stay stable across
// reloads while being blocked out with the transform gizmo.
export const ASTEROID_SPECS = [
    { name: 'Asteroid #101', radius: 9, position: [-44.76, ASTEROID_Y, -73.56] },
    { name: 'Asteroid #102', radius: 13, position: [43.93, ASTEROID_Y, 110.36] },
    { name: 'Asteroid #103', radius: 6, position: [-28.67, ASTEROID_Y, -137.09] },
    { name: 'Asteroid #104', radius: 15, position: [-150.55, ASTEROID_Y, 63.03] },
    { name: 'Asteroid #105', radius: 8, position: [47.08, ASTEROID_Y, -60.33] },
    { name: 'Asteroid #106', radius: 11, position: [158.46, ASTEROID_Y, -173.57] },
    { name: 'Asteroid #107', radius: 7, position: [104.19, ASTEROID_Y, -9.05] },
    { name: 'Asteroid #108', radius: 12, position: [153.53, ASTEROID_Y, 93.93] },
];

// Deform a dodecahedron's vertices per-axis to fake an irregular rock shape.
// Purely cosmetic noise, so it's fine for this to stay randomized per load.
function deformToRock(geometry) {
    const pos = geometry.attributes.position;
    for (let j = 0; j < pos.count; j++) {
        const vx = pos.getX(j);
        const vy = pos.getY(j);
        const vz = pos.getZ(j);
        const factor = 1 + (Math.random() - 0.5) * 0.35;
        pos.setXYZ(j, vx * factor, vy * factor, vz * factor);
    }
    geometry.computeVertexNormals();
}

export function createAsteroidBelt(asteroidsGroup) {
    const asteroidMaterial = new THREE.MeshStandardMaterial({
        color: COLORS.asteroid,
        roughness: 0.9,
        metalness: 0.2,
        flatShading: true,
    });

    for (const spec of ASTEROID_SPECS) {
        const geometry = new THREE.DodecahedronGeometry(spec.radius, 1);
        deformToRock(geometry);

        const mesh = new THREE.Mesh(geometry, asteroidMaterial);
        mesh.position.set(...spec.position);
        mesh.rotation.set(Math.random() * Math.PI, Math.random() * Math.PI, Math.random() * Math.PI);

        mesh.castShadow = true;
        mesh.receiveShadow = true;
        mesh.userData = {
            type: 'Asteroid',
            name: spec.name,
            rotSpeed: (Math.random() - 0.5) * 0.005,
            // Radius used for blocking the grid's view-zone ribbon - padded
            // wider than the nominal mesh radius so the ribbon wraps outside
            // the deformed rock instead of touching/penetrating it.
            blockRadius: spec.radius * ASTEROID_BLOCK_PADDING,
        };

        asteroidsGroup.add(mesh);
    }
}

// An asteroid attached to the transform gizmo is skipped so rotating it by
// hand isn't fought by the idle tumble.
export function spinAsteroids(asteroidsGroup, excludeMesh = null) {
    asteroidsGroup.children.forEach((asteroid) => {
        if (asteroid === excludeMesh) return;
        if (asteroid.userData.rotSpeed) {
            asteroid.rotation.x += asteroid.userData.rotSpeed;
            asteroid.rotation.y += asteroid.userData.rotSpeed * 0.8;
        }
    });
}
