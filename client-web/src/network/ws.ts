// ---- Protocol Types (matching server/src/protocol.rs) ----

export interface ClientCreateRoom {
  type: 'CreateRoom';
}

export interface ClientJoinRoom {
  type: 'JoinRoom';
  room_id: string;
}

export interface ClientChangeTeam {
  type: 'ChangeTeam';
  team: number;
}

export interface ClientStartGame {
  type: 'StartGame';
}

export interface ClientSubmitAction {
  type: 'SubmitAction';
  direction: [number, number];
  power: number;
}

export type ClientMessage =
  | ClientCreateRoom
  | ClientJoinRoom
  | ClientChangeTeam
  | ClientStartGame
  | ClientSubmitAction;

// Server -> Client

export interface PlayerInfo {
  id: number;
  team: number;
  name: string;
}

export interface EntityState {
  x: number;
  y: number;
}

export interface ServerRoomCreated {
  type: 'RoomCreated';
  room_id: string;
}

export interface ServerRoomState {
  type: 'RoomState';
  players: PlayerInfo[];
  you: number;
}

export interface ServerError {
  type: 'Error';
  message: string;
}

export interface ServerPhaseChanged {
  type: 'PhaseChanged';
  phase: string;
}

export interface ServerPlanningStart {
  type: 'PlanningStart';
  deadline: number;
}

export interface ServerActionConfirmed {
  type: 'ActionConfirmed';
}

export interface ServerSimulationFrame {
  type: 'SimulationFrame';
  penguins: EntityState[];
  ball: EntityState;
  match_timer: number;
}

export interface ServerGoalScored {
  type: 'GoalScored';
  scoring_team: number;
  scores: Record<string, number>;
}

export interface ServerMatchEnded {
  type: 'MatchEnded';
  result: string;
  scores: Record<string, number>;
}

export type ServerMessage =
  | ServerRoomCreated
  | ServerRoomState
  | ServerError
  | ServerPhaseChanged
  | ServerPlanningStart
  | ServerActionConfirmed
  | ServerSimulationFrame
  | ServerGoalScored
  | ServerMatchEnded;

// ---- Event Emitter ----

type MessageHandler<T extends ServerMessage = ServerMessage> = (msg: T) => void;

type EventMap = {
  [K in ServerMessage['type']]: Extract<ServerMessage, { type: K }>;
};

// ---- WebSocket Client ----

export class GameWebSocket {
  private ws: WebSocket | null = null;
  private url: string;
  private handlers: Map<string, Set<MessageHandler<never>>> = new Map();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 1000;
  private maxReconnectDelay = 10000;
  private shouldReconnect = true;
  private connectionHandlers: {
    onOpen: Array<() => void>;
    onClose: Array<() => void>;
  } = { onOpen: [], onClose: [] };

  constructor(url: string = 'ws://localhost:3001/ws') {
    this.url = url;
  }

  public connect(): void {
    this.shouldReconnect = true;

    try {
      this.ws = new WebSocket(this.url);
    } catch (err) {
      console.error('[WS] Failed to create WebSocket:', err);
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      console.log('[WS] Connected to', this.url);
      this.reconnectDelay = 1000;
      this.connectionHandlers.onOpen.forEach((h) => h());
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as ServerMessage;
        this.dispatch(msg);
      } catch (err) {
        console.error('[WS] Failed to parse message:', event.data, err);
      }
    };

    this.ws.onclose = (event) => {
      console.log('[WS] Disconnected:', event.code, event.reason);
      this.connectionHandlers.onClose.forEach((h) => h());
      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = (err) => {
      console.error('[WS] Error:', err);
    };
  }

  public disconnect(): void {
    this.shouldReconnect = false;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  public send(msg: ClientMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[WS] Cannot send, not connected');
      return;
    }
    this.ws.send(JSON.stringify(msg));
  }

  /** Register handler for a specific server message type */
  public on<K extends keyof EventMap>(
    type: K,
    handler: MessageHandler<EventMap[K]>
  ): void {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler as MessageHandler<never>);
  }

  /** Remove a handler */
  public off<K extends keyof EventMap>(
    type: K,
    handler: MessageHandler<EventMap[K]>
  ): void {
    this.handlers.get(type)?.delete(handler as MessageHandler<never>);
  }

  public onConnect(handler: () => void): void {
    this.connectionHandlers.onOpen.push(handler);
  }

  public onDisconnect(handler: () => void): void {
    this.connectionHandlers.onClose.push(handler);
  }

  public isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private dispatch(msg: ServerMessage): void {
    const handlers = this.handlers.get(msg.type);
    if (handlers) {
      handlers.forEach((h) => (h as MessageHandler<ServerMessage>)(msg));
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer) return;

    console.log(`[WS] Reconnecting in ${this.reconnectDelay}ms...`);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.reconnectDelay = Math.min(
        this.reconnectDelay * 1.5,
        this.maxReconnectDelay
      );
      this.connect();
    }, this.reconnectDelay);
  }
}
