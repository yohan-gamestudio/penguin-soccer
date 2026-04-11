import * as THREE from 'three';

// Engine coords: 720x360, scale 0.1 => Three.js: 72x36, centered at origin
// Engine (0,0) maps to Three.js (-36, -18)
// Engine (360, 180) maps to Three.js (0, 0)

export const ENGINE_SCALE = 0.1;
export const FIELD_WIDTH = 72;  // 720 * 0.1
export const FIELD_HEIGHT = 36; // 360 * 0.1

/** Convert engine coordinates to Three.js world coordinates */
export function engineToWorld(ex: number, ey: number): [number, number] {
  return [
    ex * ENGINE_SCALE - FIELD_WIDTH / 2,
    ey * ENGINE_SCALE - FIELD_HEIGHT / 2,
  ];
}

export function createField(scene: THREE.Scene): void {
  // Large dark floor plane below field
  const floorGeo = new THREE.PlaneGeometry(200, 200);
  const floorMat = new THREE.MeshStandardMaterial({
    color: 0x001133,
    roughness: 0.3,
    metalness: 0.5,
  });
  const floor = new THREE.Mesh(floorGeo, floorMat);
  floor.position.set(0, 0, -5);
  floor.receiveShadow = true;
  scene.add(floor);

  // Field surface - glacial blue ice (rounded rectangle)
  const cornerRadius = 3; // world units
  const hw = FIELD_WIDTH / 2;
  const hh = FIELD_HEIGHT / 2;

  const shape = new THREE.Shape();
  shape.moveTo(-hw + cornerRadius, -hh);
  shape.lineTo(hw - cornerRadius, -hh);
  shape.quadraticCurveTo(hw, -hh, hw, -hh + cornerRadius);
  shape.lineTo(hw, hh - cornerRadius);
  shape.quadraticCurveTo(hw, hh, hw - cornerRadius, hh);
  shape.lineTo(-hw + cornerRadius, hh);
  shape.quadraticCurveTo(-hw, hh, -hw, hh - cornerRadius);
  shape.lineTo(-hw, -hh + cornerRadius);
  shape.quadraticCurveTo(-hw, -hh, -hw + cornerRadius, -hh);

  const fieldGeo = new THREE.ShapeGeometry(shape);
  const fieldMat = new THREE.MeshStandardMaterial({
    color: 0xaaddff,
    roughness: 0.1,
    metalness: 0.2,
    transparent: true,
    opacity: 0.85,
  });
  const field = new THREE.Mesh(fieldGeo, fieldMat);
  field.position.set(0, 0, -0.1);
  field.receiveShadow = true;
  scene.add(field);

  // Field markings (white lines, subtle opacity)
  const lineMat = new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 2, transparent: true, opacity: 0.4 });

  // Rounded outline
  const outlineCurve = new THREE.Path();
  outlineCurve.moveTo(-hw + cornerRadius, -hh);
  outlineCurve.lineTo(hw - cornerRadius, -hh);
  outlineCurve.quadraticCurveTo(hw, -hh, hw, -hh + cornerRadius);
  outlineCurve.lineTo(hw, hh - cornerRadius);
  outlineCurve.quadraticCurveTo(hw, hh, hw - cornerRadius, hh);
  outlineCurve.lineTo(-hw + cornerRadius, hh);
  outlineCurve.quadraticCurveTo(-hw, hh, -hw, hh - cornerRadius);
  outlineCurve.lineTo(-hw, -hh + cornerRadius);
  outlineCurve.quadraticCurveTo(-hw, -hh, -hw + cornerRadius, -hh);
  const outlinePoints = outlineCurve.getPoints(64);
  const outlineVectors = outlinePoints.map(p => new THREE.Vector3(p.x, p.y, 0));
  const outlineGeo = new THREE.BufferGeometry().setFromPoints(outlineVectors);
  const outline = new THREE.LineLoop(outlineGeo, lineMat);
  outline.position.z = 0.01;
  scene.add(outline);

  // Center line
  const centerLinePoints = [
    new THREE.Vector3(0, -FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(0, FIELD_HEIGHT / 2, 0),
  ];
  const centerLineGeo = new THREE.BufferGeometry().setFromPoints(centerLinePoints);
  const centerLine = new THREE.Line(centerLineGeo, lineMat);
  centerLine.position.z = 0.01;
  scene.add(centerLine);

  // Center circle
  const circleGeo = new THREE.RingGeometry(5.8, 6, 32);
  const circleMat = new THREE.MeshBasicMaterial({
    color: 0xffffff,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: 0.4,
  });
  const centerCircle = new THREE.Mesh(circleGeo, circleMat);
  centerCircle.position.z = 0.01;
  scene.add(centerCircle);

  // Goal areas - semi-transparent glow rectangles
  const goalDepth = 3;
  const goalWidth = 12;

  // Left goal (team 0 - blue)
  const leftGoalGeo = new THREE.PlaneGeometry(goalDepth, goalWidth);
  const leftGoalMat = new THREE.MeshStandardMaterial({
    color: 0x4488ff,
    emissive: 0x4488ff,
    emissiveIntensity: 0.3,
    transparent: true,
    opacity: 0.4,
  });
  const leftGoal = new THREE.Mesh(leftGoalGeo, leftGoalMat);
  leftGoal.position.set(-FIELD_WIDTH / 2 - goalDepth / 2, 0, 0);
  scene.add(leftGoal);

  // Left goal frame
  const leftFramePoints = [
    new THREE.Vector3(-FIELD_WIDTH / 2, -goalWidth / 2, 0),
    new THREE.Vector3(-FIELD_WIDTH / 2 - goalDepth, -goalWidth / 2, 0),
    new THREE.Vector3(-FIELD_WIDTH / 2 - goalDepth, goalWidth / 2, 0),
    new THREE.Vector3(-FIELD_WIDTH / 2, goalWidth / 2, 0),
  ];
  const leftFrameGeo = new THREE.BufferGeometry().setFromPoints(leftFramePoints);
  const leftFrame = new THREE.Line(leftFrameGeo, new THREE.LineBasicMaterial({ color: 0x4488ff, linewidth: 2, transparent: true, opacity: 0.6 }));
  leftFrame.position.z = 0.02;
  scene.add(leftFrame);

  // Right goal (team 1 - red)
  const rightGoalGeo = new THREE.PlaneGeometry(goalDepth, goalWidth);
  const rightGoalMat = new THREE.MeshStandardMaterial({
    color: 0xff3333,
    emissive: 0xff3333,
    emissiveIntensity: 0.3,
    transparent: true,
    opacity: 0.4,
  });
  const rightGoal = new THREE.Mesh(rightGoalGeo, rightGoalMat);
  rightGoal.position.set(FIELD_WIDTH / 2 + goalDepth / 2, 0, 0);
  scene.add(rightGoal);

  // Right goal frame
  const rightFramePoints = [
    new THREE.Vector3(FIELD_WIDTH / 2, -goalWidth / 2, 0),
    new THREE.Vector3(FIELD_WIDTH / 2 + goalDepth, -goalWidth / 2, 0),
    new THREE.Vector3(FIELD_WIDTH / 2 + goalDepth, goalWidth / 2, 0),
    new THREE.Vector3(FIELD_WIDTH / 2, goalWidth / 2, 0),
  ];
  const rightFrameGeo = new THREE.BufferGeometry().setFromPoints(rightFramePoints);
  const rightFrame = new THREE.Line(rightFrameGeo, new THREE.LineBasicMaterial({ color: 0xff3333, linewidth: 2, transparent: true, opacity: 0.6 }));
  rightFrame.position.z = 0.02;
  scene.add(rightFrame);

  // Walls - dark subtle color
  const wallHeight = 1.5;
  const wallThickness = 0.3;
  const wallMat = new THREE.MeshStandardMaterial({
    color: 0x334466,
    transparent: true,
    opacity: 0.7,
    roughness: 0.5,
    metalness: 0.3,
  });

  // Top wall (shortened to accommodate rounded corners)
  const topWallGeo = new THREE.BoxGeometry(FIELD_WIDTH - cornerRadius * 2 + wallThickness * 2, wallThickness, wallHeight);
  const topWall = new THREE.Mesh(topWallGeo, wallMat);
  topWall.position.set(0, FIELD_HEIGHT / 2 + wallThickness / 2, wallHeight / 2);
  scene.add(topWall);

  // Bottom wall
  const bottomWall = new THREE.Mesh(topWallGeo, wallMat);
  bottomWall.position.set(0, -FIELD_HEIGHT / 2 - wallThickness / 2, wallHeight / 2);
  scene.add(bottomWall);

  // Left wall (top portion above goal)
  const sideWallLength = (FIELD_HEIGHT - goalWidth) / 2;
  const sideWallGeo = new THREE.BoxGeometry(wallThickness, sideWallLength, wallHeight);

  const leftWallTop = new THREE.Mesh(sideWallGeo, wallMat);
  leftWallTop.position.set(
    -FIELD_WIDTH / 2 - wallThickness / 2,
    FIELD_HEIGHT / 2 - sideWallLength / 2,
    wallHeight / 2
  );
  scene.add(leftWallTop);

  const leftWallBottom = new THREE.Mesh(sideWallGeo, wallMat);
  leftWallBottom.position.set(
    -FIELD_WIDTH / 2 - wallThickness / 2,
    -FIELD_HEIGHT / 2 + sideWallLength / 2,
    wallHeight / 2
  );
  scene.add(leftWallBottom);

  // Right wall
  const rightWallTop = new THREE.Mesh(sideWallGeo, wallMat);
  rightWallTop.position.set(
    FIELD_WIDTH / 2 + wallThickness / 2,
    FIELD_HEIGHT / 2 - sideWallLength / 2,
    wallHeight / 2
  );
  scene.add(rightWallTop);

  const rightWallBottom = new THREE.Mesh(sideWallGeo, wallMat);
  rightWallBottom.position.set(
    FIELD_WIDTH / 2 + wallThickness / 2,
    -FIELD_HEIGHT / 2 + sideWallLength / 2,
    wallHeight / 2
  );
  scene.add(rightWallBottom);
}
