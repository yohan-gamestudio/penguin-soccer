import * as THREE from 'three';
import { GameScene } from './renderer/scene';
import { createField, engineToWorld } from './renderer/field';
import { PenguinMesh } from './renderer/penguin';
import { BallMesh } from './renderer/ball';
import { DragInput } from './input/drag';
import { GameWebSocket, ServerSimulationFrame, PlayerInfo } from './network/ws';
import { LobbyUI } from './ui/lobby';
import { HudUI } from './ui/hud';
import { ResultUI } from './ui/result';

// ---- Game State Machine ----

type GameState = 'lobby' | 'planning' | 'simulation' | 'result';

class Game {
  // Core systems
  private scene: GameScene;
  private ws: GameWebSocket;
  private drag: DragInput;

  // UI
  private lobbyUI: LobbyUI;
  private hudUI: HudUI;
  private resultUI: ResultUI;
  private actionButtons: HTMLElement;
  private btnConfirm: HTMLButtonElement;
  private btnCancel: HTMLButtonElement;

  // Game objects
  private penguins: Map<number, PenguinMesh> = new Map();
  private ball: BallMesh;
  private players: PlayerInfo[] = [];
  private myId: number = -1;

  // State
  private state: GameState = 'lobby';
  private clock = new THREE.Clock();

  constructor() {
    const container = document.getElementById('game-container')!;

    // Initialize renderer
    this.scene = new GameScene(container);
    createField(this.scene.scene);

    // Initialize ball
    this.ball = new BallMesh();
    this.scene.scene.add(this.ball.mesh);
    const [bx, by] = engineToWorld(400, 200);
    this.ball.setPositionImmediate(bx, by);

    // Initialize network
    this.ws = new GameWebSocket();

    // Initialize drag input
    this.drag = new DragInput(
      this.scene.getDomElement(),
      this.scene.camera,
      this.scene.scene
    );
    this.drag.setEnabled(false);

    this.drag.onDragComplete((result) => {
      if (this.state === 'planning') {
        this.ws.send({
          type: 'SubmitAction',
          direction: result.direction,
          power: result.power,
        });
      }
    });

    // Action confirm/cancel buttons
    this.actionButtons = document.getElementById('action-buttons')!;
    this.btnConfirm = document.getElementById('btn-confirm-action') as HTMLButtonElement;
    this.btnCancel = document.getElementById('btn-cancel-action') as HTMLButtonElement;

    this.drag.onDragPending(() => {
      this.actionButtons.style.display = 'flex';
    });

    this.btnConfirm.addEventListener('click', () => {
      this.drag.confirmAction();
      this.actionButtons.style.display = 'none';
    });

    this.btnCancel.addEventListener('click', () => {
      this.drag.cancelAction();
      this.actionButtons.style.display = 'none';
    });

    // Initialize UI
    this.lobbyUI = new LobbyUI(this.ws);
    this.hudUI = new HudUI(this.ws);
    this.resultUI = new ResultUI(this.ws);

    this.lobbyUI.setOnGameStart(() => {
      this.transitionTo('planning');
    });

    this.resultUI.setOnPlayAgain(() => {
      this.transitionTo('lobby');
      this.lobbyUI.show();
    });

    // Network event handlers
    this.setupNetworkHandlers();

    // Connect
    this.ws.connect();

    // Start render loop
    this.animate();
  }

  private setupNetworkHandlers(): void {
    this.ws.on('RoomState', (msg) => {
      this.myId = msg.you;
      this.players = msg.players;
    });

    this.ws.on('PhaseChanged', (msg) => {
      switch (msg.phase) {
        case 'planning':
          this.transitionTo('planning');
          break;
        case 'simulating':
          this.transitionTo('simulation');
          break;
      }
    });

    this.ws.on('PlanningStart', () => {
      this.transitionTo('planning');
    });

    this.ws.on('SimulationFrame', (msg: ServerSimulationFrame) => {
      this.handleSimulationFrame(msg);
    });

    this.ws.on('MatchEnded', () => {
      this.transitionTo('result');
    });
  }

  private transitionTo(newState: GameState): void {
    const oldState = this.state;
    this.state = newState;

    console.log(`[Game] State: ${oldState} -> ${newState}`);

    switch (newState) {
      case 'lobby':
        this.hudUI.hide();
        this.resultUI.hide();
        this.drag.setEnabled(false);
        this.actionButtons.style.display = 'none';
        this.clearPenguins();
        break;

      case 'planning':
        this.lobbyUI.hide();
        this.hudUI.show();
        this.resultUI.hide();
        this.drag.setEnabled(true);
        this.actionButtons.style.display = 'none';
        this.ensurePenguinsExist();
        this.updateDragPenguinPosition();
        break;

      case 'simulation':
        this.drag.setEnabled(false);
        this.actionButtons.style.display = 'none';
        this.hudUI.show();
        break;

      case 'result':
        this.drag.setEnabled(false);
        this.actionButtons.style.display = 'none';
        // ResultUI shows itself via MatchEnded handler
        break;
    }
  }

  private handleSimulationFrame(msg: ServerSimulationFrame): void {
    // Update penguins
    for (let i = 0; i < msg.penguins.length; i++) {
      const state = msg.penguins[i];
      const [wx, wy] = engineToWorld(state.x, state.y);

      // Map array index to player ID using the players array order
      const playerId = i < this.players.length ? this.players[i].id : i;

      // Get or create penguin
      let penguin = this.penguins.get(playerId);
      if (!penguin) {
        const team = this.getTeamForPenguinIndex(i);
        penguin = new PenguinMesh(playerId, team);
        penguin.setPositionImmediate(wx, wy);
        this.penguins.set(playerId, penguin);
        this.scene.scene.add(penguin.group);
      }
      penguin.updatePosition(wx, wy);
    }

    // Update ball
    const [bx, by] = engineToWorld(msg.ball.x, msg.ball.y);
    this.ball.updatePosition(bx, by);
  }

  private getTeamForPenguinIndex(index: number): number {
    if (index < this.players.length) {
      return this.players[index].team;
    }
    return index % 2;
  }

  private ensurePenguinsExist(): void {
    // Create penguins for all players if they don't exist yet
    for (let i = 0; i < this.players.length; i++) {
      const player = this.players[i];
      if (!this.penguins.has(player.id)) {
        const penguin = new PenguinMesh(player.id, player.team);

        // Default positions: spread across the field
        const side = player.team === 0 ? 200 : 600;
        const yOffset = (i % 3 - 1) * 100 + 200;
        const [wx, wy] = engineToWorld(side, yOffset);
        penguin.setPositionImmediate(wx, wy);

        this.penguins.set(player.id, penguin);
        this.scene.scene.add(penguin.group);
      }
    }
  }

  private updateDragPenguinPosition(): void {
    const myPenguin = this.penguins.get(this.myId);
    if (myPenguin) {
      this.drag.setPenguinWorldPosition(myPenguin.group.position);
    }
  }

  private clearPenguins(): void {
    for (const [, penguin] of this.penguins) {
      this.scene.scene.remove(penguin.group);
    }
    this.penguins.clear();
  }

  private animate(): void {
    requestAnimationFrame(() => this.animate());

    const delta = this.clock.getDelta();

    // Animate penguins
    for (const [, penguin] of this.penguins) {
      penguin.animate(delta);
    }

    // Animate ball
    this.ball.animate(delta);

    // Update drag origin if in planning
    if (this.state === 'planning') {
      this.updateDragPenguinPosition();
    }

    // Render
    this.scene.render();
  }
}

// ---- Start ----
window.addEventListener('DOMContentLoaded', () => {
  new Game();
});
