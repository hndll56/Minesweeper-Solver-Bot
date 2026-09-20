"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.RustClient = void 0;
const child_process_1 = require("child_process");
const crypto_1 = require("crypto");
const readline = __importStar(require("readline"));
class RustClient {
    solverPath;
    proc = null;
    rl = null;
    pending = new Map();
    constructor(solverPath) {
        this.solverPath = solverPath;
    }
    ensure() {
        if (this.proc)
            return;
        this.proc = (0, child_process_1.spawn)(this.solverPath, [], { stdio: ['pipe', 'pipe', 'pipe'] });
        this.proc.stderr?.on('data', (d) => {
            process.stderr.write(`[solver] ${d}`);
        });
        this.rl = readline.createInterface({ input: this.proc.stdout });
        this.rl.on('line', (line) => {
            try {
                const resp = JSON.parse(line);
                const cb = this.pending.get(resp.request_id);
                if (cb) {
                    this.pending.delete(resp.request_id);
                    cb.resolve(resp);
                }
            }
            catch {
                // non-JSON line (e.g. debug output) — ignore
            }
        });
        this.proc.on('exit', (code) => {
            const err = new Error(`solver exited with code ${code}`);
            this.pending.forEach(cb => cb.reject(err));
            this.pending.clear();
            this.proc = null;
            this.rl = null;
        });
    }
    async solve(board) {
        this.ensure();
        const request_id = (0, crypto_1.randomUUID)();
        const payload = {
            version: 1,
            request_id,
            type: 'solve',
            board: {
                width: board.width,
                height: board.height,
                total_mines: board.totalMines,
                cells: board.cells.map(c => ({
                    state: c.state,
                    number: c.number ?? null,
                })),
            },
        };
        return new Promise((resolve, reject) => {
            this.pending.set(request_id, { resolve, reject });
            this.proc.stdin.write(JSON.stringify(payload) + '\n', (err) => {
                if (err) {
                    this.pending.delete(request_id);
                    reject(err);
                }
            });
            // 10s timeout per solve
            setTimeout(() => {
                if (this.pending.has(request_id)) {
                    this.pending.delete(request_id);
                    reject(new Error(`solver timeout for request ${request_id}`));
                }
            }, 10_000);
        });
    }
    async close() {
        this.rl?.close();
        this.proc?.stdin?.end();
        this.proc?.kill();
        this.proc = null;
        this.rl = null;
    }
}
exports.RustClient = RustClient;
//# sourceMappingURL=rust_client.js.map