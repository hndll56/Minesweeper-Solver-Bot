import { Browser, Page, chromium } from 'playwright';

export class BrowserManager {
  private browser: Browser | null = null;
  private _page: Page | null = null;

  constructor(private headless: boolean = false) {}

  async launch(): Promise<void> {
    this.browser = await chromium.launch({
      executablePath: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
      channel: 'chrome',
      headless: this.headless,
      args: ['--no-sandbox'],
    });
    this._page = await this.browser.newPage();
  }

  get page(): Page {
    if (!this._page) throw new Error('Browser not launched');
    return this._page;
  }

  async close(): Promise<void> {
    await this.browser?.close();
    this.browser = null;
    this._page = null;
  }
}
