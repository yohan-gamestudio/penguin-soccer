import * as THREE from 'three';

const TEAM_COLORS: Record<number, number> = {
  0: 0x3388ff, // Blue team
  1: 0xff3333, // Red team
};

const LERP_SPEED = 0.15;
const PENGUIN_SCALE = 2.5; // Scale factor so penguins look proportional on 80x40 field

export class PenguinMesh {
  public group: THREE.Group;
  public playerId: number;
  public team: number;

  private targetX = 0;
  private targetY = 0;
  private wobbleTime = 0;
  private penguinGroup: THREE.Group;

  constructor(playerId: number, team: number) {
    this.playerId = playerId;
    this.team = team;
    this.group = new THREE.Group();

    const teamColor = TEAM_COLORS[team] ?? 0xffffff;
    const darkNavy = 0x1a1a2e;

    // Inner group for the penguin body parts (scaled)
    this.penguinGroup = new THREE.Group();
    this.penguinGroup.scale.setScalar(PENGUIN_SCALE);

    // --- Body ---
    const bodyGeo = new THREE.SphereGeometry(0.5, 16, 16);
    const bodyMat = new THREE.MeshStandardMaterial({
      color: darkNavy,
      roughness: 0.6,
    });
    const body = new THREE.Mesh(bodyGeo, bodyMat);
    body.scale.set(1, 1.4, 0.9);
    body.position.z = 0.7;
    body.castShadow = true;
    this.penguinGroup.add(body);

    // --- White belly ---
    const bellyGeo = new THREE.SphereGeometry(0.42, 16, 16);
    const bellyMat = new THREE.MeshStandardMaterial({
      color: 0xffffff,
      roughness: 0.5,
    });
    const belly = new THREE.Mesh(bellyGeo, bellyMat);
    belly.scale.set(0.85, 1.2, 0.7);
    belly.position.set(0, -0.12, 0.7);
    this.penguinGroup.add(belly);

    // --- Head ---
    const headGeo = new THREE.SphereGeometry(0.32, 16, 16);
    const headMat = new THREE.MeshStandardMaterial({
      color: darkNavy,
      roughness: 0.6,
    });
    const head = new THREE.Mesh(headGeo, headMat);
    head.position.z = 1.25;
    head.castShadow = true;
    this.penguinGroup.add(head);

    // --- Eyes ---
    const eyeGeo = new THREE.SphereGeometry(0.08, 8, 8);
    const eyeMat = new THREE.MeshStandardMaterial({ color: 0xffffff });
    const pupilGeo = new THREE.SphereGeometry(0.04, 8, 8);
    const pupilMat = new THREE.MeshStandardMaterial({ color: 0x000000 });

    const leftEye = new THREE.Mesh(eyeGeo, eyeMat);
    leftEye.position.set(-0.12, -0.25, 1.3);
    this.penguinGroup.add(leftEye);

    const leftPupil = new THREE.Mesh(pupilGeo, pupilMat);
    leftPupil.position.set(-0.12, -0.32, 1.32);
    this.penguinGroup.add(leftPupil);

    const rightEye = new THREE.Mesh(eyeGeo, eyeMat);
    rightEye.position.set(0.12, -0.25, 1.3);
    this.penguinGroup.add(rightEye);

    const rightPupil = new THREE.Mesh(pupilGeo, pupilMat);
    rightPupil.position.set(0.12, -0.32, 1.32);
    this.penguinGroup.add(rightPupil);

    // --- Beak ---
    const beakGeo = new THREE.ConeGeometry(0.08, 0.2, 4);
    const beakMat = new THREE.MeshStandardMaterial({ color: 0xff8800 });
    const beak = new THREE.Mesh(beakGeo, beakMat);
    beak.rotation.x = -Math.PI / 2;
    beak.position.set(0, -0.4, 1.22);
    this.penguinGroup.add(beak);

    // --- Feet ---
    const footGeo = new THREE.BoxGeometry(0.15, 0.25, 0.05);
    const footMat = new THREE.MeshStandardMaterial({ color: 0xff8800 });

    const leftFoot = new THREE.Mesh(footGeo, footMat);
    leftFoot.position.set(-0.15, -0.05, 0.025);
    this.penguinGroup.add(leftFoot);

    const rightFoot = new THREE.Mesh(footGeo, footMat);
    rightFoot.position.set(0.15, -0.05, 0.025);
    this.penguinGroup.add(rightFoot);

    // --- Wings ---
    const wingGeo = new THREE.BoxGeometry(0.15, 0.3, 0.6);
    const wingMat = new THREE.MeshStandardMaterial({
      color: darkNavy,
      roughness: 0.6,
    });

    const leftWing = new THREE.Mesh(wingGeo, wingMat);
    leftWing.position.set(-0.45, 0, 0.7);
    leftWing.rotation.y = 0.2;
    this.penguinGroup.add(leftWing);

    const rightWing = new THREE.Mesh(wingGeo, wingMat);
    rightWing.position.set(0.45, 0, 0.7);
    rightWing.rotation.y = -0.2;
    this.penguinGroup.add(rightWing);

    // --- Scarf (team color) ---
    const scarfGeo = new THREE.TorusGeometry(0.3, 0.06, 8, 16);
    const scarfMat = new THREE.MeshStandardMaterial({
      color: teamColor,
      emissive: teamColor,
      emissiveIntensity: 0.2,
      roughness: 0.4,
    });
    const scarf = new THREE.Mesh(scarfGeo, scarfMat);
    scarf.position.z = 1.0;
    this.penguinGroup.add(scarf);

    this.group.add(this.penguinGroup);

    // --- Player ID label sprite ---
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 64;
    const ctx = canvas.getContext('2d')!;
    ctx.clearRect(0, 0, 128, 64);
    ctx.font = 'bold 36px Segoe UI, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillStyle = team === 0 ? '#3388ff' : '#ff3333';
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 3;
    ctx.strokeText(`P${playerId}`, 64, 44);
    ctx.fillText(`P${playerId}`, 64, 44);

    const texture = new THREE.CanvasTexture(canvas);
    const spriteMat = new THREE.SpriteMaterial({ map: texture, transparent: true });
    const sprite = new THREE.Sprite(spriteMat);
    sprite.scale.set(1.5, 0.75, 1);
    sprite.position.z = 1.25 * PENGUIN_SCALE + 1.0; // Above head
    this.group.add(sprite);
  }

  /** Set target position in Three.js world coords */
  public updatePosition(worldX: number, worldY: number): void {
    this.targetX = worldX;
    this.targetY = worldY;
  }

  /** Call each frame for lerp + wobble */
  public animate(deltaTime: number): void {
    // Lerp to target
    this.group.position.x += (this.targetX - this.group.position.x) * LERP_SPEED;
    this.group.position.y += (this.targetY - this.group.position.y) * LERP_SPEED;

    // Idle wobble
    this.wobbleTime += deltaTime * 2;
    this.penguinGroup.rotation.z = Math.sin(this.wobbleTime) * 0.05;
  }

  /** Snap directly to position (no lerp) */
  public setPositionImmediate(worldX: number, worldY: number): void {
    this.targetX = worldX;
    this.targetY = worldY;
    this.group.position.x = worldX;
    this.group.position.y = worldY;
  }
}
