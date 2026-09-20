mod board;
mod component;
mod constraint;
mod enumerate;
mod probability;
mod select;

use crate::board::Board;
use crate::component::{decompose_components, extract_frontier};
use crate::constraint::{ConstraintSet, global_mine_rules, propagate, subset_reasoning};
use crate::enumerate::{solve_component, SolverConfig};
use crate::probability::{compute_global_probability, extract_certainties};
use crate::select::{select_guess, select_moves, two_ply_analysis, Action, Diagnostics, SolveResult, SolveStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Stdout, Write};
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct SolveRequest {
    version: u32,
    request_id: String,
    #[serde(rename = "type")]
    request_type: String,
    board: BoardData,
}

#[derive(Debug, Deserialize)]
struct BoardData {
    width: usize,
    height: usize,
    total_mines: usize,
    cells: Vec<CellData>,
}

#[derive(Debug, Deserialize)]
struct CellData {
    state: String,
    number: Option<u8>,
}

#[derive(Debug, Serialize)]
struct SolveResponse {
    version: u32,
    request_id: String,
    #[serde(rename = "type")]
    response_type: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<ActionResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    probabilities: Option<ProbabilityData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<DiagnosticsData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct ActionResponse {
    #[serde(rename = "type")]
    action_type: String,
    cell: usize,
}

#[derive(Debug, Serialize)]
struct ProbabilityData {
    cell_prob: std::collections::HashMap<usize, f64>,
    non_frontier_prob: f64,
}

#[derive(Debug, Serialize)]
struct DiagnosticsData {
    components: usize,
    max_component_size: usize,
    total_solutions: f64,
    enumeration_time_ms: u64,
    probability_time_ms: u64,
    two_ply_time_ms: u64,
    guess_candidates: usize,
}

impl TryFrom<BoardData> for Board {
    type Error = String;

    fn try_from(data: BoardData) -> Result<Self, Self::Error> {
        let mut board = Board::new(data.width, data.height, data.total_mines)?;
        for (i, cell) in data.cells.into_iter().enumerate() {
            let state = match cell.state.as_str() {
                "revealed" => crate::board::CellState::Revealed(cell.number.unwrap_or(0)),
                "flagged" => crate::board::CellState::SolverFlagged,
                _ => crate::board::CellState::Unknown,
            };
            board.set(i, state)?;
        }
        Ok(board)
    }
}

impl From<Action> for ActionResponse {
    fn from(action: Action) -> Self {
        match action {
            Action::Click { cell } => Self {
                action_type: "click".to_string(),
                cell,
            },
            Action::Flag { cell } => Self {
                action_type: "flag".to_string(),
                cell,
            },
            Action::Chord { cell } => Self {
                action_type: "chord".to_string(),
                cell,
            },
        }
    }
}

fn solve_board(board: &Board, config: &SolverConfig) -> Result<SolveResult> {
    let _start = Instant::now();

    if let Err(_e) = board.validate() {
        return Ok(SolveResult::error(SolveStatus::InvalidState));
    }

    let mut constraints = ConstraintSet::from_board(board)
        .map_err(|e| anyhow::anyhow!(e))?;
    let mut diag = Diagnostics::default();

    let mut loop_count = 0;
    loop {
        loop_count += 1;
        if loop_count > 100 {
            break;
        }

        let mut changed = false;

        let global_result = global_mine_rules(board, &constraints);
        if global_result.changed {
            changed = true;
            for (cell, inf) in global_result.inferences {
                if inf == crate::constraint::CellInference::Safe {
                    // handled by re-reading board
                } else if inf == crate::constraint::CellInference::Mine {
                    // handled by re-reading board
                }
            }
        }

        let prop_result = propagate(&mut constraints, board)
            .map_err(|e| anyhow::anyhow!(e))?;
        if prop_result.changed {
            changed = true;
        }

        let subset_result = subset_reasoning(&mut constraints, board)
            .map_err(|e| anyhow::anyhow!(e))?;
        if subset_result.changed {
            changed = true;
        }

        if !changed {
            break;
        }
    }

    let (frontier, non_frontier) = extract_frontier(board, &constraints);
    let components = decompose_components(board, &constraints, &frontier);

    diag.components = components.len();
    diag.max_component_size = components.iter().map(|c| c.len()).max().unwrap_or(0);

    let mut component_results = Vec::new();
    let enum_start = Instant::now();

    for comp in &components {
        let comp_constraints: Vec<_> = comp
            .constraints
            .iter()
            .map(|&ci| &constraints.constraints[ci])
            .collect();
        let result = solve_component(board, comp, &comp_constraints, config);
        component_results.push(result);
    }

    diag.enumeration_time_ms = enum_start.elapsed().as_millis() as u64;
    diag.total_solutions = component_results.iter().map(|r| r.total_solutions).sum();

    let prob_start = Instant::now();
    let global_prob = compute_global_probability(board, &components, &component_results, &non_frontier);
    diag.probability_time_ms = prob_start.elapsed().as_millis() as u64;

    let (safe_cells, mine_cells) = extract_certainties(&global_prob);

    if !safe_cells.is_empty() || !mine_cells.is_empty() {
        let actions = select_moves(board, &global_prob, &safe_cells, &mine_cells);
        return Ok(SolveResult {
            status: SolveStatus::Ok,
            actions,
            probabilities: Some(global_prob),
            diagnostics: diag,
        });
    }

    let guess = select_guess(board, &global_prob, &frontier, &non_frontier);

    if let Some(guess_action) = guess {
        let mut actions = vec![guess_action];
        let mut candidates: Vec<_> = global_prob.cell_prob.keys().copied().collect();
        candidates.sort_by(|a, b| {
            global_prob
                .cell_prob
                .get(a)
                .partial_cmp(&global_prob.cell_prob.get(b))
                .unwrap()
        });
        diag.guess_candidates = candidates.len();

        let ply_start = Instant::now();
        if let Some(ply_action) = two_ply_analysis(board, &global_prob, &candidates, 5) {
            actions.clear();
            actions.push(ply_action);
        }
        diag.two_ply_time_ms = ply_start.elapsed().as_millis() as u64;

        return Ok(SolveResult {
            status: SolveStatus::Ok,
            actions,
            probabilities: Some(global_prob),
            diagnostics: diag,
        });
    }

    Ok(SolveResult::error(SolveStatus::NoSolution))
}

fn main() -> Result<()> {
    let config = SolverConfig::default();
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin);
    let mut stdout = std::io::stdout();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: SolveRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("JSON parse error: {}", e);
                continue;
            }
        };

        if request.request_type != "solve" || request.version != 1 {
            let response = SolveResponse {
                version: 1,
                request_id: request.request_id,
                response_type: "error".to_string(),
                status: "UNSUPPORTED".to_string(),
                actions: None,
                probabilities: None,
                diagnostics: None,
                reason: Some("Unsupported request type or version".to_string()),
            };
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
            continue;
        }

        let board: Board = match Board::try_from(request.board) {
            Ok(b) => b,
            Err(e) => {
                let response = SolveResponse {
                    version: 1,
                    request_id: request.request_id,
                    response_type: "error".to_string(),
                    status: "INVALID_STATE".to_string(),
                    actions: None,
                    probabilities: None,
                    diagnostics: None,
                    reason: Some(e),
                };
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let result = match solve_board(&board, &config) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Solver error: {}", e);
                SolveResult::error(SolveStatus::InternalError)
            }
        };

        let response = SolveResponse {
            version: 1,
            request_id: request.request_id,
            response_type: "result".to_string(),
            status: format!("{:?}", result.status).to_uppercase(),
            actions: Some(result.actions.into_iter().map(Into::into).collect()),
            probabilities: result.probabilities.map(|p| ProbabilityData {
                cell_prob: p.cell_prob,
                non_frontier_prob: p.non_frontier_prob,
            }),
            diagnostics: Some(DiagnosticsData {
                components: result.diagnostics.components,
                max_component_size: result.diagnostics.max_component_size,
                total_solutions: result.diagnostics.total_solutions,
                enumeration_time_ms: result.diagnostics.enumeration_time_ms,
                probability_time_ms: result.diagnostics.probability_time_ms,
                two_ply_time_ms: result.diagnostics.two_ply_time_ms,
                guess_candidates: result.diagnostics.guess_candidates,
            }),
            reason: None,
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}