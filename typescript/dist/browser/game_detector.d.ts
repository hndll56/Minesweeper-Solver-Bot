import { Page } from 'playwright';
export type GameType = 'google' | 'minesweeperonline' | 'unknown';
export declare function detectGame(page: Page): Promise<GameType>;
//# sourceMappingURL=game_detector.d.ts.map