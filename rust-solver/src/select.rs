use crate::board::Board;
use crate::probability::GlobalProbability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    Click { cell: usize },
    Flag { cell: usize },
    Chord { cell: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub status: SolveStatus,
    pub actions: Vec<Action>,
    pub probabilities: Option<GlobalProbability>,
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolveStatus {
    Ok,
    InvalidState,
    NoSolution,
    Incomplete,
    TimeLimit,
    MemoryLimit,
    Unsupported,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Diagnostics {
    pub components: usize,
    pub max_component_size: usize,
    pub total_solutions: f64,
    pub enumeration_time_ms: u64,
    pub probability_time_ms: u64,
    pub two_ply_time_ms: u64,
    pub guess_candidates: usize,
}

impl SolveResult {
    pub fn ok() -> Self {
        Self {
            status: SolveStatus::Ok,
            actions: Vec::new(),
            probabilities: None,
            diagnostics: Diagnostics::default(),
        }
    }

    pub fn error(status: SolveStatus) -> Self {
        Self {
            status,
            actions: Vec::new(),
            probabilities: None,
            diagnostics: Diagnostics::default(),
        }
    }
}

pub fn select_moves(
    board: &Board,
    prob: &GlobalProbability,
    safe_cells: &[usize],
    mine_cells: &[usize],
) -> Vec<Action> {
    let mut actions = Vec::new();

    for &cell in safe_cells {
        actions.push(Action::Click { cell });
    }

    for &cell in mine_cells {
        actions.push(Action::Flag { cell });
    }

    actions
}

pub fn select_guess(
    board: &Board,
    prob: &GlobalProbability,
    frontier: &[usize],
    non_frontier: &[usize],
) -> Option<Action> {
    let mut candidates = Vec::new();

    for &cell in frontier {
        if let Some(&p) = prob.cell_prob.get(&cell) {
            candidates.push((cell, p));
        }
    }

    for &cell in non_frontier {
        if let Some(&p) = prob.cell_prob.get(&cell) {
            candidates.push((cell, p));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    candidates.first().map(|(cell, _)| Action::Click { cell: *cell })
}

pub fn two_ply_analysis(
    board: &Board,
    prob: &GlobalProbability,
    candidates: &[usize],
    _max_candidates: usize,
) -> Option<Action> {
    let mut best_score = f64::INFINITY;
    let mut best_cell = None;

    for &cell in candidates.iter().take(5) {
        let p_mine = prob.cell_prob.get(&cell).copied().unwrap_or(0.5);

        let score = p_mine + (1.0 - p_mine) * p_mine;

        if score < best_score {
            best_score = score;
            best_cell = Some(cell);
        }
    }

    best_cell.map(|cell| Action::Click { cell })
}