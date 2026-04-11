import { GameWebSocket, PlayerInfo } from '../network/ws';

export class LobbyUI {
  private overlay: HTMLElement;
  private menuSection: HTMLElement;
  private roomSection: HTMLElement;
  private btnCreate: HTMLElement;
  private btnJoin: HTMLElement;
  private btnStart: HTMLElement;
  private inputRoomCode: HTMLInputElement;
  private roomCodeDisplay: HTMLElement;
  private playerList: HTMLElement;
  private ws: GameWebSocket;

  private myId: number = -1;
  private players: PlayerInfo[] = [];
  private onGameStart: (() => void) | null = null;
  private currentRoomId: string = '';

  constructor(ws: GameWebSocket) {
    this.ws = ws;

    this.overlay = document.getElementById('lobby-overlay')!;
    this.menuSection = document.getElementById('lobby-menu')!;
    this.roomSection = document.getElementById('lobby-room')!;
    this.btnCreate = document.getElementById('btn-create')!;
    this.btnJoin = document.getElementById('btn-join')!;
    this.btnStart = document.getElementById('btn-start')!;
    this.inputRoomCode = document.getElementById('input-room-code') as HTMLInputElement;
    this.roomCodeDisplay = document.getElementById('room-code-display')!;
    this.playerList = document.getElementById('player-list')!;

    this.setupEvents();
    this.setupNetworkHandlers();
  }

  public setOnGameStart(callback: () => void): void {
    this.onGameStart = callback;
  }

  public show(): void {
    this.overlay.style.display = 'flex';
    this.menuSection.style.display = 'block';
    this.roomSection.style.display = 'none';
  }

  public hide(): void {
    this.overlay.style.display = 'none';
  }

  public getMyId(): number {
    return this.myId;
  }

  public getPlayers(): PlayerInfo[] {
    return this.players;
  }

  private setupEvents(): void {
    this.btnCreate.addEventListener('click', () => {
      this.ws.send({ type: 'CreateRoom' });
    });

    this.btnJoin.addEventListener('click', () => {
      const code = this.inputRoomCode.value.trim();
      if (code) {
        this.currentRoomId = code;
        this.ws.send({ type: 'JoinRoom', room_id: code });
      }
    });

    this.inputRoomCode.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        this.btnJoin.click();
      }
    });

    this.btnStart.addEventListener('click', () => {
      this.ws.send({ type: 'StartGame' });
    });
  }

  private setupNetworkHandlers(): void {
    this.ws.on('RoomCreated', (msg) => {
      this.roomCodeDisplay.textContent = msg.room_id;
      this.menuSection.style.display = 'none';
      this.roomSection.style.display = 'block';
    });

    this.ws.on('RoomState', (msg) => {
      this.myId = msg.you;
      this.players = msg.players;

      // If we just joined (menu was showing), switch to room view
      if (this.menuSection.style.display !== 'none') {
        this.menuSection.style.display = 'none';
        this.roomSection.style.display = 'block';
        this.roomCodeDisplay.textContent = this.currentRoomId;
      }

      this.renderPlayerList();

      // Show start button if 2+ players
      this.btnStart.style.display = msg.players.length >= 2 ? 'block' : 'none';
    });

    this.ws.on('PhaseChanged', (msg) => {
      if (msg.phase === 'planning' || msg.phase === 'simulating') {
        this.hide();
        if (this.onGameStart) {
          this.onGameStart();
        }
      }
    });

    this.ws.on('Error', (msg) => {
      console.error('[Lobby] Error:', msg.message);
    });
  }

  private renderPlayerList(): void {
    this.playerList.innerHTML = '';

    for (const player of this.players) {
      const entry = document.createElement('div');
      entry.className = 'player-entry';

      const nameSpan = document.createElement('span');
      nameSpan.className = 'player-name';
      nameSpan.textContent = player.name + (player.id === this.myId ? ' (you)' : '');
      nameSpan.style.color = player.team === 0 ? '#4488ff' : '#ff4444';

      const teamBtn = document.createElement('button');
      teamBtn.className = `team-btn team-${player.team}`;
      teamBtn.textContent = player.team === 0 ? 'BLUE' : 'RED';

      // Only allow changing own team
      if (player.id === this.myId) {
        teamBtn.addEventListener('click', () => {
          const newTeam = player.team === 0 ? 1 : 0;
          this.ws.send({ type: 'ChangeTeam', team: newTeam });
        });
      } else {
        teamBtn.style.opacity = '0.5';
        teamBtn.style.cursor = 'default';
      }

      entry.appendChild(nameSpan);
      entry.appendChild(teamBtn);
      this.playerList.appendChild(entry);
    }
  }
}
