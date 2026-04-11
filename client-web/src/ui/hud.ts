import { GameWebSocket } from '../network/ws';

export class HudUI {
  private container: HTMLElement;
  private scoreDisplay: HTMLElement;
  private phaseDisplay: HTMLElement;
  private timerDisplay: HTMLElement;
  private matchTimerDisplay: HTMLElement;

  private scores: Record<string, number> = { '0': 0, '1': 0 };
  private timerInterval: ReturnType<typeof setInterval> | null = null;

  constructor(ws: GameWebSocket) {
    this.container = document.getElementById('hud')!;
    this.scoreDisplay = document.getElementById('score-display')!;
    this.phaseDisplay = document.getElementById('phase-display')!;
    this.timerDisplay = document.getElementById('timer-display')!;
    this.matchTimerDisplay = document.getElementById('match-timer')!;

    this.setupNetworkHandlers(ws);
  }

  public show(): void {
    this.container.style.display = 'flex';
  }

  public hide(): void {
    this.container.style.display = 'none';
    this.stopTimer();
  }

  public updateScores(newScores: Record<string, number>): void {
    this.scores = newScores;
    this.renderScores();
  }

  private renderScores(): void {
    const blueScore = this.scores['0'] ?? 0;
    const redScore = this.scores['1'] ?? 0;
    this.scoreDisplay.innerHTML = `
      <span class="team-blue">BLUE ${blueScore}</span>
      <span class="vs">vs</span>
      <span class="team-red">RED ${redScore}</span>
    `;
  }

  private setPhase(text: string): void {
    this.phaseDisplay.textContent = text;
  }

  private startPlanningTimer(deadlineSeconds: number): void {
    this.stopTimer();
    const endTime = Date.now() + deadlineSeconds * 1000;

    this.timerInterval = setInterval(() => {
      const remaining = Math.max(0, Math.ceil((endTime - Date.now()) / 1000));
      this.timerDisplay.textContent = `⏱ ${remaining}s`;

      if (remaining <= 5) {
        this.timerDisplay.style.color = '#ff4444';
      } else {
        this.timerDisplay.style.color = '#ffffff';
      }

      if (remaining <= 0) {
        this.stopTimer();
      }
    }, 200);
  }

  public updateMatchTimer(seconds: number): void {
    const totalSeconds = Math.max(0, Math.floor(seconds));
    const minutes = Math.floor(totalSeconds / 60);
    const secs = totalSeconds % 60;
    this.matchTimerDisplay.textContent = `${minutes}:${secs.toString().padStart(2, '0')}`;
  }

  private stopTimer(): void {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = null;
    }
    this.timerDisplay.textContent = '';
  }

  private setupNetworkHandlers(ws: GameWebSocket): void {
    ws.on('PhaseChanged', (msg) => {
      switch (msg.phase) {
        case 'planning':
          this.setPhase('Choose your move!');
          break;
        case 'simulating':
          this.setPhase('Simulating...');
          this.stopTimer();
          break;
        default:
          this.setPhase(msg.phase);
      }
    });

    ws.on('PlanningStart', (msg) => {
      this.setPhase('Choose your move!');
      this.startPlanningTimer(msg.deadline);
    });

    ws.on('ActionConfirmed', () => {
      this.setPhase('Waiting for others...');
    });

    ws.on('GoalScored', (msg) => {
      this.updateScores(msg.scores);
      const teamName = msg.scoring_team === 0 ? 'BLUE' : 'RED';
      this.setPhase(`GOAL by ${teamName}!!!`);
    });

    ws.on('SimulationFrame', (msg) => {
      this.updateMatchTimer(msg.match_timer);
    });
  }
}
