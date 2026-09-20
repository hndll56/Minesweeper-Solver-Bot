import { ChildProcess, spawn } from 'child_process';
import { randomUUID } from 'crypto';
import { BoardState, SolveResponse } from '../types';
import * as readline from 'readline';

export class RustClient {
  private proc: ChildProcess | null = null;
  private rl: readline.Interface | null = null;
  private pending = new Map<string, { resolve: (r: SolveResponse) => void; reject: (e: Error) => void }>();

  constructor(private solverPath: string) {}

  private ensure(): void {
    if (this.proc) return;
    this.proc = spawn(this.solverPath, [], { stdio: ['pipe', 'pipe', 'pipe'] });

    this.proc.stderr?.on('data', (d: Buffer) => {
      process.stderr.write(`[solver] ${d}`);
    });

    this.rl = readline.createInterface({ input: this.proc.stdout! });
    this.rl.on('line', (line: string) => {
      try {
        const resp = JSON.parse(line) as SolveResponse;
        const cb = this.pending.get(resp.request_id);
        if (cb) {
          this.pending.delete(resp.request_id);
          cb.resolve(resp);
        }
      } catch {
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

  async solve(board: BoardState): Promise<SolveResponse> {
    this.ensure();

    const request_id = randomUUID();
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
      this.proc!.stdin!.write(JSON.stringify(payload) + '\n', (err) => {
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

  async close(): Promise<void> {
    this.rl?.close();
    this.proc?.stdin?.end();
    this.proc?.kill();
    this.proc = null;
    this.rl = null;
  }
}
