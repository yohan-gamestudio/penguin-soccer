import * as THREE from 'three';

export class GameScene {
  public scene: THREE.Scene;
  public camera: THREE.PerspectiveCamera;
  public renderer: THREE.WebGLRenderer;

  constructor(container: HTMLElement) {
    // Scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0a0a1a);

    // Camera - angled ~45 degrees looking down for pseudo-3D view
    this.camera = new THREE.PerspectiveCamera(
      50,
      window.innerWidth / window.innerHeight,
      0.1,
      200
    );
    this.camera.position.set(0, -30, 50);
    this.camera.lookAt(0, 5, 0);

    // Renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    container.appendChild(this.renderer.domElement);

    // Lights
    const ambient = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambient);

    const directional = new THREE.DirectionalLight(0xffffff, 0.8);
    directional.position.set(20, -10, 40);
    directional.castShadow = true;
    directional.shadow.mapSize.width = 2048;
    directional.shadow.mapSize.height = 2048;
    directional.shadow.camera.near = 0.5;
    directional.shadow.camera.far = 100;
    directional.shadow.camera.left = -50;
    directional.shadow.camera.right = 50;
    directional.shadow.camera.top = 30;
    directional.shadow.camera.bottom = -30;
    this.scene.add(directional);

    // Fun colored point lights for 병맛 vibes
    const pinkLight = new THREE.PointLight(0xff00ff, 0.3, 80);
    pinkLight.position.set(-30, 0, 20);
    this.scene.add(pinkLight);

    const cyanLight = new THREE.PointLight(0x00ffff, 0.3, 80);
    cyanLight.position.set(30, 0, 20);
    this.scene.add(cyanLight);

    // Resize handler
    window.addEventListener('resize', this.onResize.bind(this));
  }

  private onResize(): void {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  }

  public render(): void {
    this.renderer.render(this.scene, this.camera);
  }

  public getDomElement(): HTMLCanvasElement {
    return this.renderer.domElement;
  }
}
