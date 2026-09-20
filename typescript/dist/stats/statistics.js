"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Statistics = void 0;
class Statistics {
    gamesPlayed = 0;
    wins = 0;
    losses = 0;
    totalMoves = 0;
    totalGuesses = 0;
    recordGame(won, moves, guesses) {
        this.gamesPlayed++;
        if (won)
            this.wins++;
        else
            this.losses++;
        this.totalMoves += moves;
        this.totalGuesses += guesses;
    }
    get winRate() {
        return this.gamesPlayed === 0 ? 0 : this.wins / this.gamesPlayed;
    }
    get avgMovesPerGame() {
        return this.gamesPlayed === 0 ? 0 : this.totalMoves / this.gamesPlayed;
    }
    report() {
        return [
            `Games: ${this.gamesPlayed} | W: ${this.wins} L: ${this.losses} | Win rate: ${(this.winRate * 100).toFixed(1)}%`,
            `Moves: ${this.totalMoves} avg ${this.avgMovesPerGame.toFixed(1)}/game | Guesses: ${this.totalGuesses}`,
        ].join('\n');
    }
}
exports.Statistics = Statistics;
//# sourceMappingURL=statistics.js.map