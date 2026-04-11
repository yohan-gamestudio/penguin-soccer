import * as THREE from 'three';

// Engine coords: 800x400, scale 0.1 => Three.js: 80x40, centered at origin
// Engine (0,0) maps to Three.js (-40, -20)
// Engine (400, 200) maps to Three.js (0, 0)

export const ENGINE_SCALE = 0.1;
export const FIELD_WIDTH = 80;  // 800 * 0.1
export const FIELD_HEIGHT = 40; // 400 * 0.1

/** Convert engine coordinates to Three.js world coordinates */
export function engineToWorld(ex: number, ey: number): [number, number] {
  return [
    ex * ENGINE_SCALE - FIELD_WIDTH / 2,
    ey * ENGINE_SCALE - FIELD_HEIGHT / 2,
  ];
}

export function createField(scene: THREE.Scene): void {
  // Field surface
  const fieldGeo = new THREE.PlaneGeometry(FIELD_WIDTH, FIELD_HEIGHT);
  const fieldMat = new THREE.MeshStandardMaterial({
    color: 0x1a8c1a,
    roughness: 0.8,
  });
  const field = new THREE.Mesh(fieldGeo, fieldMat);
  field.position.set(0, 0, -0.1);
  field.receiveShadow = true;
  scene.add(field);

  // Field markings (white lines)
  const lineMat = new THREE.MeshBasicMaterial({ color: 0xffffff });

  // Outline
  const outlinePoints = [
    new THREE.Vector3(-FIELD_WIDTH / 2, -FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(FIELD_WIDTH / 2, -FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(FIELD_WIDTH / 2, FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(-FIELD_WIDTH / 2, FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(-FIELD_WIDTH / 2, -FIELD_HEIGHT / 2, 0),
  ];
  const outlineGeo = new THREE.BufferGeometry().setFromPoints(outlinePoints);
  const outline = new THREE.Line(outlineGeo, new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 2 }));
  outline.position.z = 0.01;
  scene.add(outline);

  // Center line
  const centerLinePoints = [
    new THREE.Vector3(0, -FIELD_HEIGHT / 2, 0),
    new THREE.Vector3(0, FIELD_HEIGHT / 2, 0),
  ];
  const centerLineGeo = new THREE.BufferGeometry().setFromPoints(centerLinePoints);
  const centerLine = new THREE.Line(centerLineGeo, new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 2 }));
  centerLine.position.z = 0.01;
  scene.add(centerLine);

  // Center circle
  const circleGeo = new THREE.RingGeometry(5.8, 6, 32);
  const circleMat = new THREE.MeshBasicMaterial({ color: 0xffffff, side: THREE.DoubleSide });
  const centerCircle = new THREE.Mesh(circleGeo, circleMat);
  centerCircle.position.z = 0.01;
  scene.add(centerCircle);

  // Goal areas - colored rectangles at left/right edges
  // Goal width in engine: height * 0.3 = 400 * 0.3 = 120 => 12 Three.js units
  const goalDepth = 3;
  const goalWidth = 12;

  // Left goal (team 0 - blue)
  const leftGoalGeo = new THREE.PlaneGeometry(goalDepth, goalWidth);
  const leftGoalMat = new THREE.MeshStandardMaterial({
    color: 0x4488ff,
    transparent: true,
    opacity: 0.5,
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
  const leftFrame = new THREE.Line(leftFrameGeo, new THREE.LineBasicMaterial({ color: 0x4488ff, linewidth: 2 }));
  leftFrame.position.z = 0.02;
  scene.add(leftFrame);

  // Right goal (team 1 - red)
  const rightGoalGeo = new THREE.PlaneGeometry(goalDepth, goalWidth);
  const rightGoalMat = new THREE.MeshStandardMaterial({
    color: 0xff4444,
    transparent: true,
    opacity: 0.5,
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
  const rightFrame = new THREE.Line(rightFrameGeo, new THREE.LineBasicMaterial({ color: 0xff4444, linewidth: 2 }));
  rightFrame.position.z = 0.02;
  scene.add(rightFrame);

  // Walls - thin box geometries around edges
  const wallHeight = 1.5;
  const wallThickness = 0.3;
  const wallMat = new THREE.MeshStandardMaterial({
    color: 0xff00ff,
    emissive: 0xff00ff,
    emissiveIntensity: 0.3,
    transparent: true,
    opacity: 0.6,
  });

  // Top wall
  const topWallGeo = new THREE.BoxGeometry(FIELD_WIDTH + wallThickness * 2, wallThickness, wallHeight);
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
