import { Page } from 'playwright';
import { BoardState } from '../types';
import { GameType, detectGame } from '../browser/game_detector';
import { readBoard } from '../browser/board_reader';
import { GameController } from '../browser/game_controller';

export class GameAdapter {
  private gameType: GameType = 'unknown';
  private controller: GameController | null = null;

  constructor(private page: Page) {}

  async detectBoard(): Promise<boolean> {
    this.gameType = await detectGame(this.page);
    this.controller = new GameController(this.page, this.gameType);
    return this.gameType !== 'unknown';
  }

  async readBoard(): Promise<BoardState> {
    return readBoard(this.page, this.gameType);
  }

  async clickCell(index: number): Promise<void> {
    const board = await this.readBoard();
    await this.controller!.clickCell(index, board);
  }

  async flagCell(index: number): Promise<void> {
    const board = await this.readBoard();
    await this.controller!.flagCell(index, board);
  }

  async restart(): Promise<void> {
    await this.controller!.restart();
  }

  async getGameStatus(): Promise<BoardState['gameStatus']> {
    const board = await this.readBoard();
    return board.gameStatus;
  }
}
