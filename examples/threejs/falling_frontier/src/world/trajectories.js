import * as THREE from 'three';
import { COLORS } from '../config/colors.js';

// Dashed vertical drop-line from a point down to the tactical grid plane,
// plus a crosshair where it lands - the "falling" signature visual.
export function createHeightGuideLine(pos, trajectoryGroup, gridGroup) {
    const lineGeo = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(pos.x, pos.y, pos.z),
        new THREE.Vector3(pos.x, 0, pos.z),
    ]);
    const lineMat = new THREE.LineDashedMaterial({
        color: COLORS.gridCyan,
        dashSize: 1.5,
        gapSize: 1,
        opacity: 0.5,
        transparent: true,
    });
    const guideLine = new THREE.Line(lineGeo, lineMat);
    guideLine.computeLineDistances();
    trajectoryGroup.add(guideLine);

    const crossGeo = new THREE.RingGeometry(0.5, 1.0, 4);
    const crossMat = new THREE.MeshBasicMaterial({ color: COLORS.gridCyan, side: THREE.DoubleSide, opacity: 0.6, transparent: true });
    const cross = new THREE.Mesh(crossGeo, crossMat);
    cross.rotation.x = Math.PI / 2;
    cross.position.set(pos.x, 0, pos.z);
    gridGroup.add(cross);
}

// Curved flight path line with waypoint ring markers and drop-lines at each point.
export function createTrajectoryRibbon(points, colorHex, trajectoryGroup, gridGroup) {
    const curve = new THREE.CatmullRomCurve3(points);
    const curvePoints = curve.getPoints(80);

    const geometry = new THREE.BufferGeometry().setFromPoints(curvePoints);
    const material = new THREE.LineBasicMaterial({
        color: colorHex,
        linewidth: 2,
        transparent: true,
        opacity: 0.85,
    });
    trajectoryGroup.add(new THREE.Line(geometry, material));

    points.forEach((pt) => {
        const ringGeo = new THREE.RingGeometry(1.2, 1.8, 12);
        const ringMat = new THREE.MeshBasicMaterial({ color: colorHex, side: THREE.DoubleSide });
        const ring = new THREE.Mesh(ringGeo, ringMat);
        ring.rotation.x = Math.PI / 2;
        ring.position.copy(pt);
        trajectoryGroup.add(ring);

        createHeightGuideLine(pt, trajectoryGroup, gridGroup);
    });
}

// Dashed circle around a unit's position representing sensor/weapon range.
export function createSensorRing(centerPos, radius, sensorRingGroup) {
    const circleGeo = new THREE.BufferGeometry();
    const points = [];
    const segments = 64;

    for (let i = 0; i <= segments; i++) {
        const theta = (i / segments) * Math.PI * 2;
        points.push(new THREE.Vector3(
            centerPos.x + Math.cos(theta) * radius,
            0.2, // Slightly above grid
            centerPos.z + Math.sin(theta) * radius
        ));
    }

    circleGeo.setFromPoints(points);
    const ringMat = new THREE.LineDashedMaterial({
        color: COLORS.sensorRing,
        dashSize: 3,
        gapSize: 2,
        transparent: true,
        opacity: 0.45,
    });

    const ringLine = new THREE.Line(circleGeo, ringMat);
    ringLine.computeLineDistances();
    sensorRingGroup.add(ringLine);
}
