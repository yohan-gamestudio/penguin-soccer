import * as THREE from 'three';

export interface DragResult {
  direction: [number, number];
  power: number;
}

export type DragCompleteCallback = (result: DragResult) => void;
export type DragPendingCallback = (result: DragResult) => void;

const MAX_DRAG_DISTANCE = 150; // pixels
const MIN_DRAG_DISTANCE = 10;  // pixels - minimum to register

export class DragInput {
  private canvas: HTMLCanvasElement;
  private camera: THREE.PerspectiveCamera;
  private isDragging = false;
  private startX = 0;
  private startY = 0;
  private currentX = 0;
  private currentY = 0;
  private onComplete: DragCompleteCallback | null = null;
  private onPending: DragPendingCallback | null = null;
  private pendingResult: DragResult | null = null;
  private enabled = false;

  // For rendering the drag indicator — pre-allocated, never recreated
  private indicatorLine: THREE.Line;
  private indicatorArrow: THREE.Mesh;
  private indicatorLineGeo: THREE.BufferGeometry;
  private indicatorLineMat: THREE.LineBasicMaterial;
  private indicatorArrowMat: THREE.MeshBasicMaterial;
  private scene: THREE.Scene;

  // The penguin position we're dragging from (screen coords)
  private penguinScreenX = 0;
  private penguinScreenY = 0;
  private penguinWorldPos: THREE.Vector3 = new THREE.Vector3();

  // Reusable scratch objects — avoid per-frame allocation
  private _color: THREE.Color = new THREE.Color();

  constructor(
    canvas: HTMLCanvasElement,
    camera: THREE.PerspectiveCamera,
    scene: THREE.Scene
  ) {
    this.canvas = canvas;
    this.camera = camera;
    this.scene = scene;

    // --- Pre-create line ---
    this.indicatorLineGeo = new THREE.BufferGeometry();
    // Allocate a Float32Array for 2 points (6 floats) and register it
    const linePositions = new Float32Array(6);
    this.indicatorLineGeo.setAttribute(
      'position',
      new THREE.BufferAttribute(linePositions, 3)
    );
    this.indicatorLineMat = new THREE.LineBasicMaterial({ color: 0x00ff00, linewidth: 2 });
    this.indicatorLine = new THREE.Line(this.indicatorLineGeo, this.indicatorLineMat);
    this.indicatorLine.visible = false;
    this.indicatorLine.frustumCulled = false;
    scene.add(this.indicatorLine);

    // --- Pre-create arrow head ---
    // Fixed-size cone; we'll scale it per-frame instead of recreating geometry
    const arrowGeo = new THREE.ConeGeometry(0.8, 1.5, 6);
    this.indicatorArrowMat = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    this.indicatorArrow = new THREE.Mesh(arrowGeo, this.indicatorArrowMat);
    this.indicatorArrow.visible = false;
    this.indicatorArrow.frustumCulled = false;
    scene.add(this.indicatorArrow);

    // Mouse events
    canvas.addEventListener('mousedown', this.onPointerDown.bind(this));
    canvas.addEventListener('mousemove', this.onPointerMove.bind(this));
    canvas.addEventListener('mouseup', this.onPointerUp.bind(this));

    // Touch events
    canvas.addEventListener('touchstart', this.onTouchStart.bind(this), { passive: false });
    canvas.addEventListener('touchmove', this.onTouchMove.bind(this), { passive: false });
    canvas.addEventListener('touchend', this.onTouchEnd.bind(this));
  }

  public setEnabled(enabled: boolean): void {
    this.enabled = enabled;
    if (!enabled) {
      this.isDragging = false;
      this.pendingResult = null;
      this.clearIndicator();
    }
  }

  public setPenguinWorldPosition(pos: THREE.Vector3): void {
    this.penguinWorldPos.copy(pos);
    // Project to screen
    const projected = pos.clone().project(this.camera);
    this.penguinScreenX = (projected.x * 0.5 + 0.5) * this.canvas.clientWidth;
    this.penguinScreenY = (-projected.y * 0.5 + 0.5) * this.canvas.clientHeight;
  }

  public onDragComplete(callback: DragCompleteCallback): void {
    this.onComplete = callback;
  }

  public onDragPending(callback: DragPendingCallback): void {
    this.onPending = callback;
  }

  public hasPendingAction(): boolean {
    return this.pendingResult !== null;
  }

  public confirmAction(): void {
    if (this.pendingResult && this.onComplete) {
      this.onComplete(this.pendingResult);
    }
    this.pendingResult = null;
    this.clearIndicator();
  }

  public cancelAction(): void {
    this.pendingResult = null;
    this.clearIndicator();
  }

  public isCurrentlyDragging(): boolean {
    return this.isDragging;
  }

  /** Call when the DragInput instance is no longer needed. */
  public dispose(): void {
    this.scene.remove(this.indicatorLine);
    this.scene.remove(this.indicatorArrow);
    this.indicatorLineGeo.dispose();
    this.indicatorLineMat.dispose();
    this.indicatorArrow.geometry.dispose();
    this.indicatorArrowMat.dispose();
  }

  private onTouchStart(e: TouchEvent): void {
    e.preventDefault();
    if (e.touches.length > 0) {
      this.handlePointerDown(e.touches[0].clientX, e.touches[0].clientY);
    }
  }

  private onTouchMove(e: TouchEvent): void {
    e.preventDefault();
    if (e.touches.length > 0) {
      this.handlePointerMove(e.touches[0].clientX, e.touches[0].clientY);
    }
  }

  private onTouchEnd(_e: TouchEvent): void {
    this.handlePointerUp();
  }

  private onPointerDown(e: MouseEvent): void {
    this.handlePointerDown(e.clientX, e.clientY);
  }

  private onPointerMove(e: MouseEvent): void {
    this.handlePointerMove(e.clientX, e.clientY);
  }

  private onPointerUp(_e: MouseEvent): void {
    this.handlePointerUp();
  }

  private handlePointerDown(x: number, y: number): void {
    if (!this.enabled) return;
    this.isDragging = true;
    this.startX = x;
    this.startY = y;
    this.currentX = x;
    this.currentY = y;
  }

  private handlePointerMove(x: number, y: number): void {
    if (!this.isDragging || !this.enabled) return;
    this.currentX = x;
    this.currentY = y;
    this.updateIndicator();
  }

  private handlePointerUp(): void {
    if (!this.isDragging || !this.enabled) return;
    this.isDragging = false;

    const dx = this.startX - this.currentX;
    const dy = this.startY - this.currentY;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < MIN_DRAG_DISTANCE) {
      this.clearIndicator();
      return;
    }

    const power = Math.min(dist / MAX_DRAG_DISTANCE, 1.0);

    // Slingshot: direction is OPPOSITE of drag (drag back to shoot forward)
    // Screen Y is inverted relative to world Y
    const len = Math.sqrt(dx * dx + dy * dy);
    const dirX = dx / len;
    const dirY = -dy / len; // invert Y for world coords

    // Store pending result and keep the indicator visible for confirmation
    this.pendingResult = {
      direction: [dirX, dirY],
      power,
    };

    if (this.onPending) {
      this.onPending(this.pendingResult);
    }
    // Do NOT clear indicator here — it stays until confirm or cancel
  }

  private updateIndicator(): void {
    const dx = this.startX - this.currentX;
    const dy = this.startY - this.currentY;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < MIN_DRAG_DISTANCE) {
      this.clearIndicator();
      return;
    }

    const power = Math.min(dist / MAX_DRAG_DISTANCE, 1.0);

    // Direction arrow: show where the penguin will go (slingshot direction)
    const ndx = dx / dist;
    const ndy = -dy / dist; // invert Y for world coords

    // Draw arrow from penguin position in world space
    const arrowLength = 3 + power * 8;
    const endX = this.penguinWorldPos.x + ndx * arrowLength;
    const endY = this.penguinWorldPos.y + ndy * arrowLength;
    const endZ = 1.0;

    // Color: green -> yellow -> red based on power (no allocation — reuse _color)
    const r = Math.min(power * 2, 1.0);
    const g = Math.min((1 - power) * 2, 1.0);
    this._color.setRGB(r, g, 0);

    // --- Update line geometry in-place (no allocation) ---
    const posAttr = this.indicatorLineGeo.attributes['position'] as THREE.BufferAttribute;
    posAttr.setXYZ(0, this.penguinWorldPos.x, this.penguinWorldPos.y, 1.0);
    posAttr.setXYZ(1, endX, endY, endZ);
    posAttr.needsUpdate = true;
    this.indicatorLineMat.color.copy(this._color);
    this.indicatorLine.visible = true;

    // --- Update arrow head in-place ---
    this.indicatorArrow.position.set(endX, endY, endZ);
    // Scale the fixed ConeGeometry(0.8, 1.5) to mimic power-dependent sizing
    const coneScale = 0.375 + power * 0.625; // maps [0,1] -> [0.375, 1.0]
    this.indicatorArrow.scale.setScalar(coneScale);
    const angle = Math.atan2(ndy, ndx);
    this.indicatorArrow.rotation.set(0, 0, angle - Math.PI / 2);
    this.indicatorArrowMat.color.copy(this._color);
    this.indicatorArrow.visible = true;
  }

  private clearIndicator(): void {
    this.indicatorLine.visible = false;
    this.indicatorArrow.visible = false;
  }
}
