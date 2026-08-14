import * as THREE from 'three';
import { COLORS } from '../config/colors.js';

function buildCruiser(ship, hullMat, darkMat, glowMat) {
    const mainBody = new THREE.Mesh(new THREE.BoxGeometry(8, 6, 28), hullMat);
    ship.add(mainBody);

    const bow = new THREE.Mesh(new THREE.ConeGeometry(4, 10, 4), hullMat);
    bow.rotation.x = -Math.PI / 2;
    bow.position.set(0, 0, -18);
    ship.add(bow);

    // Command Bridge Tower
    const bridge = new THREE.Mesh(new THREE.BoxGeometry(4, 4, 6), darkMat);
    bridge.position.set(0, 4, 2);
    ship.add(bridge);

    // Engine Array
    for (const x of [-2.5, 0, 2.5]) {
        const engine = new THREE.Mesh(new THREE.CylinderGeometry(1.2, 1.5, 4, 12), darkMat);
        engine.rotation.x = Math.PI / 2;
        engine.position.set(x, -0.5, 15);
        ship.add(engine);

        const glow = new THREE.Mesh(new THREE.ConeGeometry(1.1, 5, 12), glowMat);
        glow.rotation.x = -Math.PI / 2;
        glow.position.set(x, -0.5, 18);
        ship.add(glow);
    }
}

const HULL_LENGTH = { frigate: 18, corvette: 12, scout: 8 };

function buildFrigateOrCorvette(ship, type, hullMat, darkMat, glowMat) {
    const length = HULL_LENGTH[type] ?? 12;
    const wingSpan = type === 'scout' ? 5 : 8;
    const mainBody = new THREE.Mesh(new THREE.BoxGeometry(4, 3, length), hullMat);
    ship.add(mainBody);

    const nose = new THREE.Mesh(new THREE.BoxGeometry(2.5, 2, 6), darkMat);
    nose.position.set(0, -0.5, -length / 2 - 2);
    ship.add(nose);

    // Side Wings / Armor Plating
    const wings = new THREE.Mesh(new THREE.BoxGeometry(wingSpan, 0.8, 6), hullMat);
    wings.position.set(0, 0, 2);
    ship.add(wings);

    const engine = new THREE.Mesh(new THREE.CylinderGeometry(1, 1.2, 3, 12), darkMat);
    engine.rotation.x = Math.PI / 2;
    engine.position.set(0, 0, length / 2 + 1);
    ship.add(engine);

    const glow = new THREE.Mesh(new THREE.ConeGeometry(0.9, 4, 12), glowMat);
    glow.rotation.x = -Math.PI / 2;
    glow.position.set(0, 0, length / 2 + 3);
    ship.add(glow);
}

// type: 'frigate' | 'corvette' | 'scout' | 'cruiser'
export function createShipMesh(type, name, commander) {
    const ship = new THREE.Group();

    const hullMat = new THREE.MeshStandardMaterial({ color: COLORS.shipHull, roughness: 0.4, metalness: 0.8 });
    const darkMat = new THREE.MeshStandardMaterial({ color: COLORS.shipDark, roughness: 0.6, metalness: 0.7 });
    const glowMat = new THREE.MeshBasicMaterial({ color: COLORS.engineGlow });

    if (type === 'cruiser') {
        buildCruiser(ship, hullMat, darkMat, glowMat);
    } else {
        buildFrigateOrCorvette(ship, type, hullMat, darkMat, glowMat);
    }

    ship.castShadow = true;
    ship.receiveShadow = true;
    ship.userData = { type: type.toUpperCase(), name, commander };

    return ship;
}
