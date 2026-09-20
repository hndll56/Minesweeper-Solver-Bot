"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GameAdapter = void 0;
const game_detector_1 = require("../browser/game_detector");
const board_reader_1 = require("../browser/board_reader");
const game_controller_1 = require("../browser/game_controller");
class GameAdapter {
    page;
    gameType = 'unknown';
    controller = null;
    constructor(page) {
        this.page = page;
    }
    async detectBoard() {
        this.gameType = await (0, game_detector_1.detectGame)(this.page);
        this.controller = new game_controller_1.GameController(this.page, this.gameType);
        return this.gameType !== 'unknown';
    }
    async readBoard() {
        return (0, board_reader_1.readBoard)(this.page, this.gameType);
    }
    async clickCell(index) {
        const board = await this.readBoard();
        await this.controller.clickCell(index, board);
    }
    async flagCell(index) {
        const board = await this.readBoard();
        await this.controller.flagCell(index, board);
    }
    async restart() {
        await this.controller.restart();
    }
    async getGameStatus() {
        const board = await this.readBoard();
        return board.gameStatus;
    }
}
exports.GameAdapter = GameAdapter;
//# sourceMappingURL=game_adapter.js.map