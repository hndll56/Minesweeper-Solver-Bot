"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.detectGame = detectGame;
async function detectGame(page) {
    const url = page.url();
    if (url.includes('google.com'))
        return 'google';
    if (url.includes('minesweeperonline.com'))
        return 'minesweeperonline';
    return 'unknown';
}
//# sourceMappingURL=game_detector.js.map