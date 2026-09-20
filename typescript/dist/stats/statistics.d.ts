export declare class Statistics {
    gamesPlayed: number;
    wins: number;
    losses: number;
    totalMoves: number;
    totalGuesses: number;
    recordGame(won: boolean, moves: number, guesses: number): void;
    get winRate(): number;
    get avgMovesPerGame(): number;
    report(): string;
}
//# sourceMappingURL=statistics.d.ts.map