import * as THREE from 'three';

const LERP_SPEED = 0.2;

export class BallMesh {
  public mesh: THREE.Group;

  private targetX = 0;
  private targetY = 0;
  private spinTime = 0;
  private sphere: THREE.Mesh;

  constructor() {
    this.mesh = new THREE.Group();

    // Main ball - white sphere with standard material
    const geo = new THREE.SphereGeometry(1.0, 16, 16);
    const mat = new THREE.MeshStandardMaterial({
      color: 0xffffff,
      roughness: 0.3,
      metalness: 0.1,
    });
    this.sphere = new THREE.Mesh(geo, mat);
    this.sphere.position.z = 1.0;
    this.sphere.castShadow = true;
    this.mesh.add(this.sphere);

    // Subtle glow point light attached to ball
    const glow = new THREE.PointLight(0xffffff, 0.3, 5);
    glow.position.z = 1.0;
    this.mesh.add(glow);
  }

  /** Set target position in Three.js world coords */
  public updatePosition(worldX: number, worldY: number): void {
    this.targetX = worldX;
    this.targetY = worldY;
  }

  /** Call each frame for lerp + spin */
  public animate(deltaTime: number): void {
    const dx = this.targetX - this.mesh.position.x;
    const dy = this.targetY - this.mesh.position.y;

    this.mesh.position.x += dx * LERP_SPEED;
    this.mesh.position.y += dy * LERP_SPEED;

    // Spin based on movement
    const speed = Math.sqrt(dx * dx + dy * dy);
    this.spinTime += deltaTime * speed * 2;
    this.sphere.rotation.x = this.spinTime;
    this.sphere.rotation.y = this.spinTime * 0.7;
  }

  /** Snap directly to position */
  public setPositionImmediate(worldX: number, worldY: number): void {
    this.targetX = worldX;
    this.targetY = worldY;
    this.mesh.position.x = worldX;
    this.mesh.position.y = worldY;
  }
}
