# Minesweeper Solver

A high-performance Minesweeper solver with a Rust core and TypeScript + Playwright browser layer.

```
TypeScript + Playwright
        |
        | JSONL over persistent stdin/stdout
        |
        v
Rust Solver
```

## Architecture

### Rust Solver (`rust-solver/`)

The entire solving logic lives in Rust. No AI, no OCR, no computer vision — pure constraint mathematics.

**Solver pipeline:**

1. **Constraint generation** — from revealed numbers, compute `sum(unknown_neighbors) = number - known_mines`
2. **Deterministic propagation** — if `required = 0` → all safe; if `required = count` → all mines
3. **Subset reasoning** — if `S1 ⊆ S2` then `S2 - S1` is a new constraint
4. **Global mine count** — if `remaining_mines == 0` → all unknown safe; if `== unknown_count` → all mines
5. **Frontier detection** — unknown cells that appear in constraints vs. those that don't
6. **Connected components** — decompose frontier into independent subproblems
7. **Exact enumeration** — backtracking with forward checking per component
8. **Solution counting** — `ways[k]` and `mine_ways[cell][k]` per component
9. **Global probability** — prefix/suffix convolution across components + binomial(N, R-K) for non-frontier
10. **Certainty extraction** — P=0 → safe, P=1 → mine
11. **Guess selection** — minimum probability candidate
12. **2-ply analysis** — lookahead scoring for guess quality

### TypeScript Layer (`typescript/`)

Handles browser interaction only. No solver logic here.

- `BrowserManager` — Playwright browser lifecycle
- `GameDetector` — detect which Minesweeper variant is running
- `BoardReader` — read board state from DOM
- `GameController` — click/flag cells
- `GameAdapter` — unified interface over game variants
- `RustClient` — JSONL communication with Rust process
- `AutonomousLoop` — the main play loop
- `Statistics` — win/loss tracking

### Simulator (`simulator/`)

Standalone seeded Minesweeper board generator. No external dependencies — uses xoshiro256** PRNG initialized with SplitMix64.

```
minesweeper-sim --width 9 --height 9 --mines 10 --seed 123456
minesweeper-sim --test   # runs 100 test games
```

## Building

### Requirements

- Rust 1.70+
- Node.js 18+
- npm

### Rust Solver

```bash
cd rust-solver
cargo build --release
cargo test
```

### Simulator

```bash
cd simulator
cargo build --release
```

### TypeScript

```bash
cd typescript
npm install
npm run build
```

## Running

### Solver (JSONL server)

```bash
./rust-solver/target/release/minesweeper-solver
```

Send JSON lines to stdin:

```json
{"version":1,"request_id":"abc123","type":"solve","board":{"width":9,"height":9,"total_mines":10,"cells":[...]}}
```

Receive JSON lines from stdout:

```json
{"version":1,"request_id":"abc123","type":"result","status":"OK","actions":[{"type":"click","cell":12}],"probabilities":{"cell_prob":{...},"non_frontier_prob":0.123},"diagnostics":{...}}
```

Cell states in request:
- `{"state":"unknown"}` — unrevealed
- `{"state":"revealed","number":2}` — revealed with number
- `{"state":"flagged"}` — solver-flagged

Status codes in response:
- `OK` — actions available
- `NO_SOLUTION` — no valid configuration found (board state inconsistent)
- `INVALID_STATE` — board data is malformed
- `INCOMPLETE` — component too large, partial result
- `TIME_LIMIT` — solver timeout
- `INTERNAL_ERROR` — unexpected error

### TypeScript Bot

```bash
cd typescript
node dist/main.js --url "https://minesweeperonline.com" --solver-path "../rust-solver/target/release/minesweeper-solver.exe"
```

Options:
- `--url` — URL of the Minesweeper game
- `--solver-path` — path to the Rust solver binary
- `--headless` — run Chrome headlessly
- `--max-steps` — maximum moves per game (default: 200)

## Design Decisions

### Flag Policy

External flags from the game UI are ignored by default. The solver tracks its own mine certainties internally (`TRUST_SOLVER_FLAGS_ONLY`). This avoids corrupted state if the user or game places flags the solver didn't decide.

### Solver Does Not Need to Flag to Win

Clicking all safe cells is sufficient to win Minesweeper. Flagging is optional.

### Numeric Stability

Solution counts can be astronomically large (exponential in board size). The solver uses `f64` with normalization and log-space arithmetic where needed. Binomial coefficients use Pascal's triangle for small `n` and log-gamma for large `n`.

### No Brute-Force as Main Solver

Full-board brute-force is only used as a reference implementation in tests for correctness comparison on boards up to 5×5.

### 2-Ply Guessing

When no deterministic move exists, the solver evaluates the top 5 candidates with 1-step lookahead:

```
score(C) = P(C is mine) + (1 - P(C is mine)) * E[next_guess_probability]
```

Choose minimum score.

## Test Strategy

- **Unit tests** — board operations, constraint generation, propagation
- **Correctness tests** — optimized solver vs. brute-force reference on seeded 3×3–5×5 boards
- **Property tests** — thousands of random small boards (proptest)
- **Probability tests** — sum of probabilities, P=0/P=1 consistency

## Performance Targets

Per solve call on a standard 9×9 board:

| Metric | Target |
|--------|--------|
| Deterministic move | < 1ms |
| Enumeration (small component) | < 10ms |
| Probability computation | < 5ms |
| 2-ply analysis | < 50ms |

## Status Codes Reference

| Code | Meaning |
|------|---------|
| `OK` | At least one action returned |
| `NO_SOLUTION` | Board has 0 valid mine configurations |
| `INVALID_STATE` | Board data fails validation |
| `INCOMPLETE` | Component exceeded size/time limit |
| `TIME_LIMIT` | Solver exceeded allowed time |
| `MEMORY_LIMIT` | Too many solutions to enumerate |
| `UNSUPPORTED` | Unknown request type or version |
| `INTERNAL_ERROR` | Unexpected internal failure |
