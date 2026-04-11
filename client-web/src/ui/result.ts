import { GameWebSocket, ServerMatchEnded } from '../network/ws';

export class ResultUI {
  private overlay: HTMLElement;
  private titleEl: HTMLElement;
  private scoresEl: HTMLElement;
  private btnPlayAgain: HTMLElement;

  private onPlayAgain: (() => void) | null = null;

  constructor(ws: GameWebSocket) {
    this.overlay = document.getElementById('result-overlay')!;
    this.titleEl = document.getElementById('result-title')!;
    this.scoresEl = document.getElementById('result-scores')!;
    this.btnPlayAgain = document.getElementById('btn-play-again')!;

    this.btnPlayAgain.addEventListener('click', () => {
      this.hide();
      if (this.onPlayAgain) {
        this.onPlayAgain();
      }
    });

    ws.on('MatchEnded', (msg: ServerMatchEnded) => {
      this.showResult(msg);
    });
  }

  public setOnPlayAgain(callback: () => void): void {
    this.onPlayAgain = callback;
  }

  public show(): void {
    this.overlay.style.display = 'flex';
  }

  public hide(): void {
    this.overlay.style.display = 'none';
  }

  private showResult(msg: ServerMatchEnded): void {
    const blueScore = msg.scores['0'] ?? 0;
    const redScore = msg.scores['1'] ?? 0;

    let titleText: string;
    if (msg.result === 'draw' || msg.result === 'Draw') {
      titleText = "IT'S A DRAW!";
    } else {
      titleText = `${msg.result.toUpperCase()} WINS!`;
    }

    // Add silly flavor
    const sillyMessages = [
      'What a game!',
      'The penguins are exhausted!',
      'Absolute 병맛!',
      'The ice has been broken!',
      'Waddle waddle!',
    ];
    const silly = sillyMessages[Math.floor(Math.random() * sillyMessages.length)];

    this.titleEl.textContent = titleText;
    this.scoresEl.innerHTML = `
      <span class="team-blue">BLUE ${blueScore}</span>
      <span class="vs"> - </span>
      <span class="team-red">RED ${redScore}</span>
      <br><br>
      <span style="font-size:1.2rem; color:#ffff00;">${silly}</span>
    `;

    this.show();
  }
}
