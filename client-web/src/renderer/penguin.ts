import * as THREE from 'three';

const TEAM_COLORS: Record<number, number> = {
  0: 0x4488ff, // Blue team
  1: 0xff4444, // Red team
};

const LERP_SPEED = 0.15;

export class PenguinMesh {
  public group: THREE.Group;
  public playerId: number;
  public team: number;

  private targetX = 0;
  private targetY = 0;
  private wobbleTime = 0;
  private body: THREE.Mesh;
  private head: THREE.Mesh;

  constructor(playerId: number, team: number) {
    this.playerId = playerId;
    this.team = team;
    this.group = new THREE.Group();

    const color = TEAM_COLORS[team] ?? 0xffffff;

    // Body - cylinder (silly penguin)
    const bodyGeo = new THREE.CylinderGeometry(0.8, 1.0, 2.0, 8);
    const bodyMat = new THREE.MeshStandardMaterial({
      color: 0x111111,
      roughness: 0.6,
    });
    this.body = new THREE.Mesh(bodyGeo, bodyMat);
    this.body.rotation.x = Math.PI / 2; // Stand upright in our XY plane
    this.body.position.z = 1.0;
    this.body.castShadow = true;
    this.group.add(this.body);

    // Belly patch (team color)
    const bellyGeo = new THREE.CylinderGeometry(0.5, 0.65, 1.6, 8);
    const bellyMat = new THREE.MeshStandardMaterial({
      color: color,
      roughness: 0.5,
      emissive: color,
      emissiveIntensity: 0.15,
    });
    const belly = new THREE.Mesh(bellyGeo, bellyMat);
    belly.rotation.x = Math.PI / 2;
    belly.position.set(0, -0.15, 1.0);
    this.group.add(belly);

    // Head - sphere
    const headGeo = new THREE.SphereGeometry(0.7, 8, 8);
    const headMat = new THREE.MeshStandardMaterial({
      color: 0x111111,
      roughness: 0.6,
    });
    this.head = new THREE.Mesh(headGeo, headMat);
    this.head.position.z = 2.3;
    this.head.castShadow = true;
    this.group.add(this.head);

    // Eyes (googly style for 병맛)
    const eyeGeo = new THREE.SphereGeometry(0.2, 8, 8);
    const eyeMat = new THREE.MeshBasicMaterial({ color: 0xffffff });
    const pupilGeo = new THREE.SphereGeometry(0.12, 8, 8);
    const pupilMat = new THREE.MeshBasicMaterial({ color: 0x000000 });

    const leftEye = new THREE.Mesh(eyeGeo, eyeMat);
    leftEye.position.set(-0.25, -0.55, 2.45);
    this.group.add(leftEye);

    const leftPupil = new THREE.Mesh(pupilGeo, pupilMat);
    leftPupil.position.set(-0.25, -0.7, 2.5);
    this.group.add(leftPupil);

    const rightEye = new THREE.Mesh(eyeGeo, eyeMat);
    rightEye.position.set(0.25, -0.55, 2.45);
    this.group.add(rightEye);

    const rightPupil = new THREE.Mesh(pupilGeo, pupilMat);
    rightPupil.position.set(0.25, -0.7, 2.5);
    this.group.add(rightPupil);

    // Beak
    const beakGeo = new THREE.ConeGeometry(0.2, 0.5, 4);
    const beakMat = new THREE.MeshStandardMaterial({ color: 0xff8800 });
    const beak = new THREE.Mesh(beakGeo, beakMat);
    beak.rotation.x = -Math.PI / 2;
    beak.position.set(0, -0.9, 2.3);
    this.group.add(beak);

    // Player ID label - neon text sprite
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 64;
    const ctx = canvas.getContext('2d')!;
    ctx.fillStyle = 'transparent';
    ctx.fillRect(0, 0, 128, 64);
    ctx.font = 'bold 40px Comic Sans MS, cursive';
    ctx.textAlign = 'center';
    ctx.fillStyle = team === 0 ? '#4488ff' : '#ff4444';
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 3;
    ctx.strokeText(`P${playerId}`, 64, 44);
    ctx.fillText(`P${playerId}`, 64, 44);

    const texture = new THREE.CanvasTexture(canvas);
    const spriteMat = new THREE.SpriteMaterial({ map: texture, transparent: true });
    const sprite = new THREE.Sprite(spriteMat);
    sprite.scale.set(2, 1, 1);
    sprite.position.z = 3.5;
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

    // Wobble animation (silly penguin waddle)
    this.wobbleTime += deltaTime * 5;
    const wobble = Math.sin(this.wobbleTime) * 0.08;
    this.group.rotation.z = wobble;
    this.head.position.x = Math.sin(this.wobbleTime * 1.3) * 0.05;
  }

  /** Snap directly to position (no lerp) */
  public setPositionImmediate(worldX: number, worldY: number): void {
    this.targetX = worldX;
    this.targetY = worldY;
    this.group.position.x = worldX;
    this.group.position.y = worldY;
  }
}
