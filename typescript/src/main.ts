import { BrowserManager } from './browser/browser_manager';
import { GameAdapter } from './adapter/game_adapter';
import { RustClient } from './rust/rust_client';
import { Statistics } from './stats/statistics';
import { AutonomousLoop } from './loop/autonomous_loop';

function parseArgs(): {
  url?: string;
  solverPath?: string;
  headless: boolean;
  maxSteps: number;
} {
  const args = process.argv.slice(2);
  let url: string | undefined;
  let solverPath: string | undefined;
  let headless = false;
  let maxSteps = 200;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--url' && i + 1 < args.length) {
      url = args[++i];
    } else if (arg === '--solver-path' && i + 1 < args.length) {
      solverPath = args[++i];
    } else if (arg === '--headless') {
      headless = true;
    } else if (arg === '--max-steps' && i + 1 < args.length) {
      const val = parseInt(args[++i], 10);
      if (!isNaN(val)) maxSteps = val;
    }
  }

  return { url, solverPath, headless, maxSteps };
}

async function main(): Promise<void> {
  const { url, solverPath, headless, maxSteps } = parseArgs();

  if (!url || !solverPath) {
    console.error('Usage: node dist/main.js --url <url> --solver-path <path> [--headless] [--max-steps N]');
    process.exit(1);
  }

  const browserManager = new BrowserManager(headless);
  await browserManager.launch();

  try {
    await browserManager.page.goto(url, { waitUntil: 'domcontentloaded' });
    await browserManager.page.waitForSelector('#game div.square', { timeout: 10000 }).catch(() => {});
    await browserManager.page.waitForTimeout(500);

    const adapter = new GameAdapter(browserManager.page);
    const rustClient = new RustClient(solverPath);
    const stats = new Statistics();

    const loop = new AutonomousLoop(browserManager, adapter, rustClient, stats, { maxSteps });
    await loop.run();

    console.log(stats.report());

    await rustClient.close();
    await browserManager.close();
  } catch (err) {
    console.error('Bot failed:', err);
    await browserManager.close();
    process.exit(1);
  }
}

main();
