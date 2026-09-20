"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AutonomousLoop = void 0;
class AutonomousLoop {
    browserManager;
    gameAdapter;
    solver;
    stats;
    config = {
        maxSteps: 200,
        maxTotalTime: 120_000,
        initialDelayMs: 300,
        noChangeLimit: 5,
    };
    boardHistory = [];
    noChangeCount = 0;
    startTime = 0;
    moveCount = 0;
    gameStatus = 'idle';
    firstMove = true;
    actionsThisGame = 0;
    guessesThisGame = 0;
    currentBoard;
    constructor(browserManager, gameAdapter, solver, stats, config = {}) {
        this.browserManager = browserManager;
        this.gameAdapter = gameAdapter;
        this.solver = solver;
        this.stats = stats;
        this.config = { ...this.config, ...config };
    }
    async run() {
        this.startTime = Date.now();
        while (this.moveCount < this.config.maxSteps && this.elapsed() < this.config.maxTotalTime) {
            try {
                // 1. Detect game (only once)
                if (this.moveCount === 0) {
                    const detected = await this.gameAdapter.detectBoard();
                    if (!detected) {
                        throw new Error('No supported game detected');
                    }
                }
                // 2. Read board with retry/wait for board to render
                let board = await this.gameAdapter.readBoard();
                let waitRetries = 0;
                while ((board.width === 0 || board.height === 0 || board.cells.length === 0) && waitRetries < 20) {
                    await this.sleep(500);
                    board = await this.gameAdapter.readBoard();
                    waitRetries++;
                }
                if (board.width === 0 || board.height === 0 || board.cells.length === 0) {
                    console.warn('Board not rendered yet, retrying...');
                    await this.sleep(1000);
                    continue;
                }
                // 4. If first move: click center or (0,0)
                if (this.firstMove) {
                    const centerIndex = Math.floor(board.width / 2) + Math.floor(board.height / 2) * board.width;
                    await this.gameAdapter.clickCell(centerIndex);
                    this.actionsThisGame++;
                    await this.sleep(this.config.initialDelayMs);
                    this.firstMove = false;
                    this.moveCount++;
                    continue;
                }
                // 5. Send to Rust solver
                const boardForSolve = this.normalizeBoard(board);
                const response = await this.solver.solve(boardForSolve);
                // Track guesses
                if (response.is_guess)
                    this.guessesThisGame++;
                // 6. Execute returned actions
                for (const action of response.actions) {
                    if (action.type === 'click') {
                        await this.gameAdapter.clickCell(action.index);
                    }
                    else if (action.type === 'flag') {
                        await this.gameAdapter.flagCell(action.index);
                    }
                    this.actionsThisGame++;
                }
                // 7. Small delay
                await this.sleep(this.config.initialDelayMs);
                // 8. Read board again
                const newBoard = await this.gameAdapter.readBoard();
                this.currentBoard = newBoard;
                this.gameStatus = newBoard.gameStatus;
                // 9. Check win/loss
                if (this.gameStatus === 'won') {
                    this.stats.recordGame(true, this.actionsThisGame, this.guessesThisGame);
                    break;
                }
                if (this.gameStatus === 'lost') {
                    this.stats.recordGame(false, this.actionsThisGame, this.guessesThisGame);
                    break;
                }
                // 10. Board hash check (safety)
                const hash = this.hashBoard(newBoard);
                if (this.boardHistory.length > 0 && this.boardHistory[this.boardHistory.length - 1] === hash) {
                    this.noChangeCount++;
                    if (this.noChangeCount >= this.config.noChangeLimit) {
                        console.warn('No change detected for', this.noChangeCount, 'steps — stopping');
                        break;
                    }
                }
                else {
                    this.noChangeCount = 0;
                }
                this.boardHistory.push(hash);
                if (this.boardHistory.length > 20)
                    this.boardHistory.shift();
                this.moveCount++;
            }
            catch (err) {
                console.error('Loop error:', err);
                break;
            }
        }
    }
    normalizeBoard(board) {
        // Normalize cells for solver: ensure every cell has a number property
        return {
            ...board,
            cells: board.cells.map(c => ({
                state: c.state,
                number: c.number ?? 0,
            })),
        };
    }
    hashBoard(board) {
        return board.cells.map(c => `${c.state}:${c.number ?? 0}`).join('|');
    }
    elapsed() {
        return Date.now() - this.startTime;
    }
    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}
exports.AutonomousLoop = AutonomousLoop;
//# sourceMappingURL=autonomous_loop.js.map