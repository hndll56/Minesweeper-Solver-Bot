"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GameController = void 0;
class GameController {
    page;
    gameType;
    constructor(page, gameType) {
        this.page = page;
        this.gameType = gameType;
    }
    async clickCell(index, board) {
        const width = board.width || 9;
        const col = (index % width) + 1;
        const row = Math.floor(index / width) + 1;
        const id = `${row}_${col}`;
        const loc = this.page.locator(`[id="${id}"]`).first();
        if (await loc.count() > 0) {
            await loc.click({ force: true }).catch(() => { });
        }
        await this.page.evaluate((cellId) => {
            const el = document.getElementById(cellId);
            if (el) {
                el.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, which: 1, button: 0 }));
                el.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, cancelable: true, which: 1, button: 0 }));
            }
        }, id).catch(() => { });
    }
    async flagCell(index, board) {
        const width = board.width || 9;
        const col = (index % width) + 1;
        const row = Math.floor(index / width) + 1;
        const id = `${row}_${col}`;
        const loc = this.page.locator(`[id="${id}"]`).first();
        if (await loc.count() > 0) {
            await loc.click({ button: 'right', force: true }).catch(() => { });
        }
        await this.page.evaluate((cellId) => {
            const el = document.getElementById(cellId);
            if (el) {
                el.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, which: 3, button: 2 }));
                el.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, cancelable: true, which: 3, button: 2 }));
                el.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true }));
            }
        }, id).catch(() => { });
    }
    async chordCell(index, board) {
        const { x, y } = await this.cellCoords(index, board);
        // Middle-click or double-click depending on site
        await this.page.mouse.click(x, y, { button: 'middle' });
    }
    async restart() {
        if (this.gameType === 'minesweeperonline') {
            await this.page.click('#face').catch(() => { });
        }
        else if (this.gameType === 'google') {
            // Google: click the restart/play-again button
            const btn = await this.page.$('[aria-label*="play again" i], [aria-label*="restart" i], [data-action="restart"]');
            if (btn)
                await btn.click();
        }
    }
    async cellCoords(index, board) {
        const col = index % board.width;
        const row = Math.floor(index / board.width);
        if (this.gameType === 'minesweeperonline') {
            const r = row + 1;
            const c = col + 1;
            const cellId = index + 1;
            const loc = this.page.locator(`div[id="${r}_${c}"], div[id="${cellId}"], td[id="${cellId}"], #game .square >> nth=${index}`).first();
            const box = await loc.boundingBox().catch(() => null);
            if (box)
                return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
        }
        // Generic: get all cell elements and pick by index
        const cells = await this.page.$$('[aria-label*="mine" i], [aria-label*="blank" i], [aria-label*="flag" i], td.blank, td[class^="open"], td.bombflagged');
        if (cells[index]) {
            const box = await cells[index].boundingBox();
            if (box)
                return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
        }
        // Fallback: derive from board bounding box
        const boardEl = await this.page.$('#game, .minesweeper-board, [data-difficulty]');
        if (boardEl) {
            const box = await boardEl.boundingBox();
            if (box) {
                const cellW = box.width / board.width;
                const cellH = box.height / board.height;
                return {
                    x: box.x + col * cellW + cellW / 2,
                    y: box.y + row * cellH + cellH / 2,
                };
            }
        }
        throw new Error(`Cannot locate cell ${index} on board`);
    }
}
exports.GameController = GameController;
//# sourceMappingURL=game_controller.js.map