use crate::board::Board;
use crate::component::Component;
use crate::constraint::Constraint;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub max_solutions: usize,
    pub max_time_ms: u64,
    pub max_component_cells: usize,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_solutions: 1_000_000,
            max_time_ms: 5000,
            max_component_cells: 25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentSolution {
    pub assignments: Vec<(usize, bool)>,
    pub mine_count: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentResult {
    pub ways: Vec<f64>,
    pub mine_ways: HashMap<usize, Vec<f64>>,
    pub total_solutions: f64,
    pub incomplete: bool,
}

impl ComponentResult {
    pub fn new(num_cells: usize, max_mines: usize) -> Self {
        Self {
            ways: vec![0.0; max_mines + 1],
            mine_ways: HashMap::new(),
            total_solutions: 0.0,
            incomplete: false,
        }
    }

    pub fn add_solution(&mut self, assignment: &[(usize, bool)]) {
        let mine_count = assignment.iter().filter(|(_, is_mine)| *is_mine).count();
        if mine_count < self.ways.len() {
            self.ways[mine_count] += 1.0;
            for &(cell, is_mine) in assignment {
                if is_mine {
                    self.mine_ways
                        .entry(cell)
                        .or_insert_with(|| vec![0.0; self.ways.len()])[mine_count] += 1.0;
                }
            }
            self.total_solutions += 1.0;
        }
    }

    pub fn cell_mine_probability(&self, cell: usize, k: usize) -> f64 {
        if let Some(ways) = self.mine_ways.get(&cell) {
            if k < ways.len() && self.ways[k] > 0.0 {
                return ways[k] / self.ways[k];
            }
        }
        0.0
    }
}

pub fn solve_component(
    board: &Board,
    component: &Component,
    constraints: &[&Constraint],
    config: &SolverConfig,
) -> ComponentResult {
    let cells = &component.cells;
    let max_mines = cells.len().min(board.total_mines);
    let mut result = ComponentResult::new(cells.len(), max_mines);

    if cells.len() > config.max_component_cells {
        result.incomplete = true;
        return result;
    }

    let cell_to_idx: HashMap<usize, usize> = cells.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    let mut assignment = vec![None; cells.len()];
    let mut constraint_states: Vec<(usize, usize)> = constraints
        .iter()
        .map(|c| {
            let unknown: Vec<_> = c.cells.iter().copied().filter(|cell| cell_to_idx.contains_key(cell)).collect();
            (c.required_mines, unknown.len())
        })
        .collect();

    let start_time = std::time::Instant::now();

    fn backtrack(
        idx: usize,
        assignment: &mut [Option<bool>],
        constraint_states: &mut [(usize, usize)],
        constraints: &[&Constraint],
        cells: &[usize],
        cell_to_idx: &HashMap<usize, usize>,
        result: &mut ComponentResult,
        config: &SolverConfig,
        start_time: &std::time::Instant,
    ) -> bool {
        if result.total_solutions >= config.max_solutions as f64 {
            return true;
        }
        if start_time.elapsed().as_millis() > config.max_time_ms as u128 {
            result.incomplete = true;
            return true;
        }

        if idx == cells.len() {
            let sol: Vec<_> = cells.iter().enumerate().filter_map(|(i, &c)| assignment[i].map(|v| (c, v))).collect();
            result.add_solution(&sol);
            return false;
        }

        let cell = cells[idx];
        let mut tried_mine = false;
        let mut tried_safe = false;

        for is_mine in [true, false] {
            if is_mine {
                tried_mine = true;
            } else {
                tried_safe = true;
            }

            let mut ok = true;
            let mut changes = Vec::new();

            for (ci, constraint) in constraints.iter().enumerate() {
                if !constraint.cells.contains(&cell) {
                    continue;
                }
                let (req_mines, unassigned) = &mut constraint_states[ci];
                if is_mine {
                    if *req_mines == 0 {
                        ok = false;
                        break;
                    }
                    *req_mines -= 1;
                }
                *unassigned -= 1;
                changes.push(ci);

                if *req_mines > *unassigned {
                    ok = false;
                    break;
                }
            }

            if ok {
                assignment[idx] = Some(is_mine);
                if backtrack(
                    idx + 1,
                    assignment,
                    constraint_states,
                    constraints,
                    cells,
                    cell_to_idx,
                    result,
                    config,
                    start_time,
                ) {
                    return true;
                }
                assignment[idx] = None;
            }

            for &ci in &changes {
                constraint_states[ci].1 += 1;
                if is_mine {
                    constraint_states[ci].0 += 1;
                }
            }

            if !ok {
                break;
            }
        }

        false
    }

    let _ = backtrack(
        0,
        &mut assignment,
        &mut constraint_states,
        constraints,
        cells,
        &cell_to_idx,
        &mut result,
        config,
        &start_time,
    );

    result
}