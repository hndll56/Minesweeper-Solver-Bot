"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const browser_manager_1 = require("./browser/browser_manager");
const game_adapter_1 = require("./adapter/game_adapter");
const rust_client_1 = require("./rust/rust_client");
const statistics_1 = require("./stats/statistics");
const autonomous_loop_1 = require("./loop/autonomous_loop");
function parseArgs() {
    const args = process.argv.slice(2);
    let url;
    let solverPath;
    let headless = false;
    let maxSteps = 200;
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '--url' && i + 1 < args.length) {
            url = args[++i];
        }
        else if (arg === '--solver-path' && i + 1 < args.length) {
            solverPath = args[++i];
        }
        else if (arg === '--headless') {
            headless = true;
        }
        else if (arg === '--max-steps' && i + 1 < args.length) {
            const val = parseInt(args[++i], 10);
            if (!isNaN(val))
                maxSteps = val;
        }
    }
    return { url, solverPath, headless, maxSteps };
}
async function main() {
    const { url, solverPath, headless, maxSteps } = parseArgs();
    if (!url || !solverPath) {
        console.error('Usage: node dist/main.js --url <url> --solver-path <path> [--headless] [--max-steps N]');
        process.exit(1);
    }
    const browserManager = new browser_manager_1.BrowserManager(headless);
    await browserManager.launch();
    try {
        await browserManager.page.goto(url, { waitUntil: 'domcontentloaded' });
        await browserManager.page.waitForTimeout(1500);
        const adapter = new game_adapter_1.GameAdapter(browserManager.page);
        const rustClient = new rust_client_1.RustClient(solverPath);
        const stats = new statistics_1.Statistics();
        const loop = new autonomous_loop_1.AutonomousLoop(browserManager, adapter, rustClient, stats, { maxSteps });
        await loop.run();
        console.log(stats.report());
        await rustClient.close();
        await browserManager.close();
    }
    catch (err) {
        console.error('Bot failed:', err);
        await browserManager.close();
        process.exit(1);
    }
}
main();
//# sourceMappingURL=main.js.map