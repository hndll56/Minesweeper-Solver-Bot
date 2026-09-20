/// Minesweeper Simulator — seeded, reproducible board generator
/// Usage: minesweeper-sim --width 9 --height 9 --mines 10 --seed 123456
use std::collections::VecDeque;

/// A fully-realized Minesweeper board with mine positions and revealed numbers.
#[derive(Debug, Clone)]
pub struct SimBoard {
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
    pub seed: u64,
    pub mines: Vec<bool>,          // true = mine
    pub numbers: Vec<u8>,          // adjacent mine count
    pub revealed: Vec<bool>,       // has the player revealed this cell?
    pub flagged: Vec<bool>,        // has the player flagged this cell?
}

/// Minimal xoshiro256** PRNG for reproducibility without external deps
struct Rng {
    s: [u64; 4],
}

impl Rng {
    fn from_seed(seed: u64) -> Self {
        // SplitMix64 to initialize state
        let mut z = seed;
        let mut s = [0u64; 4];
        for item in &mut s {
            z = z.wrapping_add(0x9e3779b97f4a7c15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            *item = x ^ (x >> 31);
        }
        Self { s }
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.s[1]
            .wrapping_mul(5)
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform random usize in [0, n)
    fn next_usize(&mut self, n: usize) -> usize {
        ((self.next_u64() >> 33) as usize) % n
    }
}

impl SimBoard {
    /// Create a new board without placing mines yet (before first click).
    pub fn new(width: usize, height: usize, mine_count: usize, seed: u64) -> Self {
        let total = width * height;
        Self {
            width,
            height,
            mine_count,
            seed,
            mines: vec![false; total],
            numbers: vec![0; total],
            revealed: vec![false; total],
            flagged: vec![false; total],
        }
    }

    pub fn total_cells(&self) -> usize {
        self.width * self.height
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }

    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let (x, y) = self.coords(idx);
        let mut result = Vec::with_capacity(8);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    result.push(self.index(nx as usize, ny as usize));
                }
            }
        }
        result
    }

    /// Place mines using Fisher-Yates shuffle, avoiding the safe_cell and its neighbors.
    /// If safe_cell is None, all cells are candidates.
    pub fn place_mines(&mut self, safe_cell: Option<usize>) {
        let total = self.total_cells();
        let mut rng = Rng::from_seed(self.seed);

        // Build candidate list
        let excluded: std::collections::HashSet<usize> = if let Some(sc) = safe_cell {
            let mut ex = self.neighbors(sc);
            ex.push(sc);
            ex.into_iter().collect()
        } else {
            std::collections::HashSet::new()
        };

        let mut candidates: Vec<usize> = (0..total).filter(|i| !excluded.contains(i)).collect();

        // Fisher-Yates partial shuffle to pick mine_count candidates
        let place = self.mine_count.min(candidates.len());
        for i in 0..place {
            let j = i + rng.next_usize(candidates.len() - i);
            candidates.swap(i, j);
            self.mines[candidates[i]] = true;
        }

        // Compute numbers
        for idx in 0..total {
            if self.mines[idx] { continue; }
            let count = self.neighbors(idx).iter().filter(|&&n| self.mines[n]).count();
            self.numbers[idx] = count as u8;
        }
    }

    /// Reveal a cell. Returns false if the cell is a mine (game over).
    /// Performs flood-fill for cells with number=0.
    pub fn reveal(&mut self, idx: usize) -> bool {
        if self.mines[idx] {
            self.revealed[idx] = true;
            return false; // hit a mine
        }
        if self.revealed[idx] || self.flagged[idx] {
            return true;
        }
        // BFS flood fill
        let mut queue = VecDeque::new();
        queue.push_back(idx);
        self.revealed[idx] = true;
        while let Some(cur) = queue.pop_front() {
            if self.numbers[cur] == 0 {
                for nb in self.neighbors(cur) {
                    if !self.revealed[nb] && !self.flagged[nb] && !self.mines[nb] {
                        self.revealed[nb] = true;
                        queue.push_back(nb);
                    }
                }
            }
        }
        true
    }

    /// Toggle a flag on an unrevealed cell.
    pub fn toggle_flag(&mut self, idx: usize) {
        if !self.revealed[idx] {
            self.flagged[idx] = !self.flagged[idx];
        }
    }

    /// Check if the game is won (all non-mine cells revealed).
    pub fn is_won(&self) -> bool {
        (0..self.total_cells())
            .filter(|&i| !self.mines[i])
            .all(|i| self.revealed[i])
    }

    /// Print the board to stdout (for debugging).
    pub fn print(&self) {
        println!("Board {}x{}, {} mines, seed={}", self.width, self.height, self.mine_count, self.seed);
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let ch = if self.revealed[idx] {
                    if self.mines[idx] {
                        '*'
                    } else if self.numbers[idx] == 0 {
                        ' '
                    } else {
                        char::from_digit(self.numbers[idx] as u32, 10).unwrap_or('?')
                    }
                } else if self.flagged[idx] {
                    'F'
                } else {
                    '.'
                };
                print!("{} ", ch);
            }
            println!();
        }
    }

    /// Export board state as JSON (compatible with Rust solver JSONL protocol).
    /// Only reveals what is currently visible.
    pub fn to_solver_board_json(&self) -> String {
        use std::fmt::Write;
        let mut cells = String::from("[");
        for i in 0..self.total_cells() {
            if i > 0 { cells.push(','); }
            if self.revealed[i] && !self.mines[i] {
                write!(cells, "{{\"state\":\"revealed\",\"number\":{}}}", self.numbers[i]).unwrap();
            } else if self.flagged[i] {
                cells.push_str("{\"state\":\"flagged\"}");
            } else {
                cells.push_str("{\"state\":\"unknown\"}");
            }
        }
        cells.push(']');
        format!(
            "{{\"version\":1,\"request_id\":\"sim-{}\",\"type\":\"solve\",\"board\":{{\"width\":{},\"height\":{},\"total_mines\":{},\"cells\":{}}}}}",
            self.seed, self.width, self.height, self.mine_count, cells
        )
    }
}

/// Run a quick automated test game using a simple "reveal any unknown" strategy.
/// Returns (won, steps).
pub fn run_test_game(width: usize, height: usize, mines: usize, seed: u64) -> (bool, usize) {
    let mut board = SimBoard::new(width, height, mines, seed);
    let center = board.index(width / 2, height / 2);
    board.place_mines(Some(center));
    let alive = board.reveal(center);
    if !alive { return (false, 1); }
    let mut steps = 1;
    loop {
        if board.is_won() { return (true, steps); }
        // Find first unrevealed non-flagged cell and reveal it
        let next = (0..board.total_cells()).find(|&i| !board.revealed[i] && !board.flagged[i] && !board.mines[i]);
        match next {
            Some(idx) => {
                let alive = board.reveal(idx);
                steps += 1;
                if !alive { return (false, steps); }
            }
            None => break,
        }
    }
    (board.is_won(), steps)
}

fn parse_args() -> (usize, usize, usize, u64, bool) {
    let args: Vec<String> = std::env::args().collect();
    let mut width = 9;
    let mut height = 9;
    let mut mines = 10;
    let mut seed = 42u64;
    let mut test_mode = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--width" | "-w" => { i += 1; if i < args.len() { width = args[i].parse().unwrap_or(9); } }
            "--height" | "-h" => { i += 1; if i < args.len() { height = args[i].parse().unwrap_or(9); } }
            "--mines" | "-m" => { i += 1; if i < args.len() { mines = args[i].parse().unwrap_or(10); } }
            "--seed" | "-s" => { i += 1; if i < args.len() { seed = args[i].parse().unwrap_or(42); } }
            "--test" | "-t" => { test_mode = true; }
            _ => {}
        }
        i += 1;
    }
    (width, height, mines, seed, test_mode)
}

fn main() {
    let (width, height, mines, seed, test_mode) = parse_args();

    if test_mode {
        println!("Running 100 test games {}x{} with {} mines...", width, height, mines);
        let mut wins = 0;
        for i in 0..100u64 {
            let (won, _steps) = run_test_game(width, height, mines, seed.wrapping_add(i));
            if won { wins += 1; }
        }
        println!("Wins: {}/100 ({:.1}%)", wins, wins as f64);
        return;
    }

    let mut board = SimBoard::new(width, height, mines, seed);
    let center = board.index(width / 2, height / 2);
    board.place_mines(Some(center));
    board.reveal(center);
    board.print();
    println!();
    println!("Solver JSON (first line):");
    println!("{}", board.to_solver_board_json());
}
