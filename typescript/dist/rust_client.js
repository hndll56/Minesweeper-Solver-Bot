"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RustClient = void 0;
const child_process_1 = require("child_process");
class RustClient {
    path;
    proc = null;
    pending = new Map();
    constructor(path) {
        this.path = path;
    }
    async start() {
        this.proc = (0, child_process_1.spawn)(this.path, [], { stdio: ['pipe', 'pipe', 'inherit'] });
        this.proc.stdout.on('data', (buf) => {
            const lines = buf.toString().split('\n').filter((l) => l.trim());
            for (const line of lines) {
                try {
                    const res = JSON.parse(line);
                    const cb = this.pending.get(res.request_id);
                    if (cb) {
                        this.pending.delete(res.request_id);
                        cb(res);
                    }
                }
                catch { }
            }
        });
    }
    async solve(board) {
        const id = Date.now().toString(36) + Math.random().toString(36).slice(2);
        const req = { version: 1, request_id: id, type: 'solve', board };
        return new Promise((resolve) => {
            this.pending.set(id, resolve);
            this.proc.stdin.write(JSON.stringify(req) + '\n');
            setTimeout(() => {
                if (this.pending.has(id)) {
                    this.pending.delete(id);
                    resolve({ status: 'TIMEOUT' });
                }
            }, 10000);
        });
    }
    async close() {
        this.proc?.kill();
    }
}
exports.RustClient = RustClient;
//# sourceMappingURL=rust_client.js.map