"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.readBoard = readBoard;
exports.clickCell = clickCell;
exports.restart = restart;
// ponytail: hardcoded selectors for minesweeperonline.com, add game_adapter layer when supporting 3+ sites
async function readBoard(page) {
    const cells = await page.locator('#game .square').all();
    const data = [];
    for (const cell of cells) {
        const cls = await cell.getAttribute('class') || '';
        if (cls.includes('blank')) {
            data.push({ state: 'unknown' });
        }
        else if (cls.includes('bombflagged')) {
            data.push({ state: 'flagged' });
        }
        else if (cls.includes('open')) {
            const num = cls.match(/open(\d)/)?.[1];
            data.push({ state: 'revealed', number: num ? parseInt(num) : 0 });
        }
        else {
            data.push({ state: 'unknown' });
        }
    }
    const face = await page.locator('#face').getAttribute('class') || '';
    const status = face.includes('facedead') ? 'lost' : face.includes('facewin') ? 'won' : 'playing';
    return {
        width: 9,
        height: 9,
        totalMines: 10,
        cells: data,
        gameStatus: status
    };
}
async function clickCell(page, index) {
    const cells = await page.locator('#game .square').all();
    if (cells[index])
        await cells[index].click();
}
async function restart(page) {
    await page.locator('#face').click();
    await page.waitForTimeout(500);
}
//# sourceMappingURL=minesweeper_online.js.map