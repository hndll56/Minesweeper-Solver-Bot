import { BoardState, SolveResponse } from './types';
export declare class RustClient {
    private path;
    private proc;
    private pending;
    constructor(path: string);
    start(): Promise<void>;
    solve(board: BoardState): Promise<SolveResponse>;
    close(): Promise<void>;
}
//# sourceMappingURL=rust_client.d.ts.map