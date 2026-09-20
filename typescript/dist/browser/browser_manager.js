"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BrowserManager = void 0;
const playwright_1 = require("playwright");
class BrowserManager {
    headless;
    browser = null;
    _page = null;
    constructor(headless = false) {
        this.headless = headless;
    }
    async launch() {
        this.browser = await playwright_1.chromium.launch({
            executablePath: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
            channel: 'chrome',
            headless: this.headless,
            args: ['--no-sandbox'],
        });
        this._page = await this.browser.newPage();
    }
    get page() {
        if (!this._page)
            throw new Error('Browser not launched');
        return this._page;
    }
    async close() {
        await this.browser?.close();
        this.browser = null;
        this._page = null;
    }
}
exports.BrowserManager = BrowserManager;
//# sourceMappingURL=browser_manager.js.map