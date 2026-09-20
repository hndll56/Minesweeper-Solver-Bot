import { BoardState, SolveResponse } from '../types';
export declare class RustClient {
    private solverPath;
    private proc;
    private rl;
    private pending;
    constructor(solverPath: string);
    private ensure;
    solve(board: BoardState): Promise<SolveResponse>;
    close(): Promise<void>;
}
//# sourceMappingURL=rust_client.d.ts.map