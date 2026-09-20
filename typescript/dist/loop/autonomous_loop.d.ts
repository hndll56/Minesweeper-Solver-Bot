import { BrowserManager } from '../browser/browser_manager';
import { GameAdapter } from '../adapter/game_adapter';
import { RustClient } from '../rust/rust_client';
import { Statistics } from '../stats/statistics';
interface LoopConfig {
    maxSteps: number;
    maxTotalTime: number;
    initialDelayMs: number;
    noChangeLimit: number;
}
export declare class AutonomousLoop {
    private browserManager;
    private gameAdapter;
    private solver;
    private stats;
    private config;
    private boardHistory;
    private noChangeCount;
    private startTime;
    private moveCount;
    private gameStatus;
    private firstMove;
    private actionsThisGame;
    private guessesThisGame;
    private currentBoard?;
    constructor(browserManager: BrowserManager, gameAdapter: GameAdapter, solver: RustClient, stats: Statistics, config?: Partial<LoopConfig>);
    run(): Promise<void>;
    private normalizeBoard;
    private hashBoard;
    private elapsed;
    private sleep;
}
export {};
//# sourceMappingURL=autonomous_loop.d.ts.map