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

  // For rendering the drag indicator
  private indicatorLine: THREE.Line | null = null;
  private indicatorArrow: THREE.Mesh | null = null;
  private scene: THREE.Scene;

  // The penguin position we're dragging from (screen coords)
  private penguinScreenX = 0;
  private penguinScreenY = 0;
  private penguinWorldPos: THREE.Vector3 = new THREE.Vector3();

  constructor(
    canvas: HTMLCanvasElement,
    camera: THREE.PerspectiveCamera,
    scene: THREE.Scene
  ) {
    this.canvas = canvas;
    this.camera = camera;
    this.scene = scene;

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
    this.clearIndicator();

    const dx = this.startX - this.currentX;
    const dy = this.startY - this.currentY;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < MIN_DRAG_DISTANCE) return;

    const power = Math.min(dist / MAX_DRAG_DISTANCE, 1.0);

    // Direction arrow: show where the penguin will go (slingshot direction)
    const len = Math.sqrt(dx * dx + dy * dy);
    const ndx = dx / len;
    const ndy = -dy / len;

    // Draw arrow from penguin position in world space
    const arrowLength = 3 + power * 8;
    const endPoint = new THREE.Vector3(
      this.penguinWorldPos.x + ndx * arrowLength,
      this.penguinWorldPos.y + ndy * arrowLength,
      1.0
    );

    // Color: green -> yellow -> red based on power
    const r = Math.min(power * 2, 1.0);
    const g = Math.min((1 - power) * 2, 1.0);
    const color = new THREE.Color(r, g, 0);

    // Line
    const points = [
      new THREE.Vector3(this.penguinWorldPos.x, this.penguinWorldPos.y, 1.0),
      endPoint,
    ];
    const lineGeo = new THREE.BufferGeometry().setFromPoints(points);
    const lineMat = new THREE.LineBasicMaterial({
      color: color,
      linewidth: 2,
    });
    this.indicatorLine = new THREE.Line(lineGeo, lineMat);
    this.scene.add(this.indicatorLine);

    // Arrow head
    const arrowGeo = new THREE.ConeGeometry(0.3 + power * 0.5, 1.0 + power * 0.5, 6);
    const arrowMat = new THREE.MeshBasicMaterial({ color: color });
    this.indicatorArrow = new THREE.Mesh(arrowGeo, arrowMat);
    this.indicatorArrow.position.copy(endPoint);

    // Rotate arrow to point in direction
    const angle = Math.atan2(ndy, ndx);
    this.indicatorArrow.rotation.z = angle - Math.PI / 2;
    this.indicatorArrow.rotation.x = Math.PI / 2;
    this.scene.add(this.indicatorArrow);
  }

  private clearIndicator(): void {
    if (this.indicatorLine) {
      this.scene.remove(this.indicatorLine);
      this.indicatorLine.geometry.dispose();
      (this.indicatorLine.material as THREE.Material).dispose();
      this.indicatorLine = null;
    }
    if (this.indicatorArrow) {
      this.scene.remove(this.indicatorArrow);
      this.indicatorArrow.geometry.dispose();
      (this.indicatorArrow.material as THREE.Material).dispose();
      this.indicatorArrow = null;
    }
  }
}
