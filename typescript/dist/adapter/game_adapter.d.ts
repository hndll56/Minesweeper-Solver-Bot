import { Page } from 'playwright';
import { BoardState } from '../types';
export declare class GameAdapter {
    private page;
    private gameType;
    private controller;
    constructor(page: Page);
    detectBoard(): Promise<boolean>;
    readBoard(): Promise<BoardState>;
    clickCell(index: number): Promise<void>;
    flagCell(index: number): Promise<void>;
    restart(): Promise<void>;
    getGameStatus(): Promise<BoardState['gameStatus']>;
}
//# sourceMappingURL=game_adapter.d.ts.map