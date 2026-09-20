import { Page } from 'playwright';
import { BoardState } from '../types';
import { GameType } from './game_detector';
export declare function readBoard(page: Page, gameType: GameType): Promise<BoardState>;
//# sourceMappingURL=board_reader.d.ts.map