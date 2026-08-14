import * as THREE from 'three';
import { createShipMesh } from './ships.js';
import { createTrajectoryRibbon, createSensorRing } from './trajectories.js';

// All ships fly at the same altitude (12) - previously the cruiser/corvette
// spawned higher/lower than the rest for no real reason.
const FLEET_Y = 12;

// All ships share the same tactical view-zone radius for now.
const FLEET_VIEW_RADIUS = 160;

// Ship roster: mesh spec, starting position/rotation and the flight path it patrols.
export const FLEET = [
    {
        type: 'frigate',
        name: 'TDF RELIANT',
        commander: 'NOLAN ADAMS',
        position: [-41.55, FLEET_Y, 57.63],
        rotationY: -1.56,
        path: [
            [-41.55, FLEET_Y, 57.63],
            [-61.55, FLEET_Y, 32.63],
            [-86.55, FLEET_Y, 2.63],
            [-71.55, FLEET_Y, -42.37],
            [-31.55, FLEET_Y, -67.37],
        ],
        trajectoryColor: 0x00f0ff,
        speed: 0.0008,
        sensorRadius: 45,
        viewRadius: FLEET_VIEW_RADIUS,
    },
    {
        type: 'cruiser',
        name: 'TDF PINNACLE',
        commander: 'CAPTAIN STERLING',
        position: [-58.26, FLEET_Y, -145.5],
        rotationY: -0.85,
        path: [
            [-58.26, FLEET_Y, -145.5],
            [-38.26, FLEET_Y, -195.5],
            [-8.26, FLEET_Y, -245.5],
        ],
        trajectoryColor: 0x00e5ff,
        speed: 0.0005,
        sensorRadius: 65,
        viewRadius: FLEET_VIEW_RADIUS,
    },
    {
        type: 'corvette',
        name: 'TDF VALIANT',
        commander: 'LT. CHEN',
        position: [25.06, FLEET_Y, -15.2],
        rotationY: -1.02,
        path: [
            [25.06, FLEET_Y, -15.2],
            [5.06, FLEET_Y, -45.2],
            [35.06, FLEET_Y, -75.2],
        ],
        trajectoryColor: 0x00f0ff,
        speed: 0.0012,
        sensorRadius: null,
        viewRadius: FLEET_VIEW_RADIUS,
    },
    {
        type: 'scout',
        name: 'TDF OSPREY',
        commander: 'ENS. REYES',
        position: [87.85, FLEET_Y, -34.42],
        rotationY: -1.08,
        path: [
            [87.85, FLEET_Y, -34.42],
            [107.85, FLEET_Y, -64.42],
            [67.85, FLEET_Y, -84.42],
        ],
        trajectoryColor: 0x00f0ff,
        speed: 0.001,
        sensorRadius: null,
        viewRadius: FLEET_VIEW_RADIUS,
    },
];

// Builds every ship, its trajectory ribbon and sensor ring, and returns the
// list of { mesh, path, progress, speed } entries the animation loop drives.
export function createFleetAndTrajectories(groups) {
    const { ships: shipsGroup, trajectory: trajectoryGroup, sensorRing: sensorRingGroup, grid: gridGroup } = groups;
    const animatedShips = [];

    for (const spec of FLEET) {
        const ship = createShipMesh(spec.type, spec.name, spec.commander);
        ship.position.set(...spec.position);
        if (spec.rotationY !== undefined) ship.rotation.y = spec.rotationY;
        ship.userData.viewRadius = spec.viewRadius;
        shipsGroup.add(ship);

        const pathPoints = spec.path.map(([x, y, z]) => new THREE.Vector3(x, y, z));
        createTrajectoryRibbon(pathPoints, spec.trajectoryColor, trajectoryGroup, gridGroup);
        animatedShips.push({ mesh: ship, path: pathPoints, progress: 0, speed: spec.speed });

        if (spec.sensorRadius) {
            createSensorRing(ship.position, spec.sensorRadius, sensorRingGroup);
        }
    }

    return animatedShips;
}

// Advances each ship along its Catmull-Rom path and orients it along the tangent.
// A ship currently attached to the transform gizmo is skipped so dragging it
// isn't fought by the path animation.
export function updateFleetMotion(animatedShips, speedMultiplier, excludeMesh = null) {
    animatedShips.forEach((item) => {
        if (item.mesh === excludeMesh) return;

        item.progress += item.speed * speedMultiplier;
        if (item.progress > 1) item.progress = 0;

        const curve = new THREE.CatmullRomCurve3(item.path);
        const pos = curve.getPointAt(item.progress);
        const tangent = curve.getTangentAt(item.progress);

        item.mesh.position.copy(pos);
        item.mesh.lookAt(pos.clone().add(tangent));
    });
}
