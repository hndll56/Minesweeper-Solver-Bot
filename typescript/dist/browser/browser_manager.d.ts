import { Page } from 'playwright';
export declare class BrowserManager {
    private headless;
    private browser;
    private _page;
    constructor(headless?: boolean);
    launch(): Promise<void>;
    get page(): Page;
    close(): Promise<void>;
}
//# sourceMappingURL=browser_manager.d.ts.map