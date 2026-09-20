import { Page } from 'playwright';
import { BoardState } from '../types';
import { GameType } from './game_detector';
export declare class GameController {
    private page;
    private gameType;
    constructor(page: Page, gameType: GameType);
    clickCell(index: number, board: BoardState): Promise<void>;
    flagCell(index: number, board: BoardState): Promise<void>;
    chordCell(index: number, board: BoardState): Promise<void>;
    restart(): Promise<void>;
    private cellCoords;
}
//# sourceMappingURL=game_controller.d.ts.map