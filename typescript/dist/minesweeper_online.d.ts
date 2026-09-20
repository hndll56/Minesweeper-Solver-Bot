import { Page } from 'playwright';
import { BoardState } from './types';
export declare function readBoard(page: Page): Promise<BoardState>;
export declare function clickCell(page: Page, index: number): Promise<void>;
export declare function restart(page: Page): Promise<void>;
//# sourceMappingURL=minesweeper_online.d.ts.map