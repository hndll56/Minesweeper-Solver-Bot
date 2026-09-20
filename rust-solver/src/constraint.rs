use crate::board::Board;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Constraint {
    pub cells: Vec<usize>,
    pub required_mines: usize,
}

impl Constraint {
    pub fn new(mut cells: Vec<usize>, required_mines: usize) -> Self {
        cells.sort_unstable();
        cells.dedup();
        Self {
            cells,
            required_mines,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.required_mines <= self.cells.len()
    }

    pub fn is_satisfied(&self, board: &Board) -> bool {
        let flagged = self.cells.iter().filter(|&&i| board.is_flagged(i)).count();
        let unknown = self.cells.iter().filter(|&&i| board.is_unknown(i)).count();
        flagged == self.required_mines && unknown == 0
    }

    pub fn remaining_mines(&self, board: &Board) -> usize {
        let flagged = self.cells.iter().filter(|&&i| board.is_flagged(i)).count();
        self.required_mines.saturating_sub(flagged)
    }

    pub fn unknown_cells(&self, board: &Board) -> Vec<usize> {
        self.cells
            .iter()
            .copied()
            .filter(|&i| board.is_unknown(i))
            .collect()
    }

    pub fn flagged_cells(&self, board: &Board) -> Vec<usize> {
        self.cells
            .iter()
            .copied()
            .filter(|&i| board.is_flagged(i))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSet {
    pub constraints: Vec<Constraint>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    pub fn from_board(board: &Board) -> Result<Self, String> {
        let mut set = Self::new();
        for i in 0..board.total_cells() {
            if let Some(num) = board.revealed_number(i) {
                let flagged = board.count_flagged_neighbors(i);
                let unknowns: Vec<usize> = board
                    .neighbors(i)
                    .into_iter()
                    .filter(|&n| board.is_unknown(n))
                    .collect();
                let remaining = num as usize;
                if remaining > flagged {
                    let req = remaining - flagged;
                    if req > unknowns.len() {
                        return Err(format!(
                            "Cell {}: required {} mines but only {} unknown neighbors",
                            i, req, unknowns.len()
                        ));
                    }
                    if !unknowns.is_empty() {
                        set.add(Constraint::new(unknowns, req));
                    } else if req > 0 {
                        return Err(format!("Cell {}: required {} mines but no unknown neighbors", i, req));
                    }
                }
            }
        }
        set.normalize();
        Ok(set)
    }

    pub fn add(&mut self, constraint: Constraint) {
        if constraint.is_valid() {
            self.constraints.push(constraint);
        }
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    pub fn normalize(&mut self) {
        self.constraints.sort_by(|a, b| {
            a.cells
                .iter()
                .cmp(b.cells.iter())
                .then_with(|| a.required_mines.cmp(&b.required_mines))
        });
        self.constraints.dedup();
    }

    pub fn validate(&self, board: &Board) -> Result<(), String> {
        for c in &self.constraints {
            if !c.is_valid() {
                return Err(format!("Invalid constraint: required_mines > cells.len()"));
            }
            let unknowns = c.unknown_cells(board);
            if c.remaining_mines(board) > unknowns.len() {
                return Err(format!(
                    "Constraint impossible: requires {} mines but only {} unknowns",
                    c.required_mines,
                    unknowns.len()
                ));
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Constraint> {
        self.constraints.iter()
    }
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellInference {
    Safe,
    Mine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub inferences: Vec<(usize, CellInference)>,
    pub changed: bool,
}

impl InferenceResult {
    pub fn new() -> Self {
        Self {
            inferences: Vec::new(),
            changed: false,
        }
    }

    pub fn add(&mut self, cell: usize, inference: CellInference) {
        self.inferences.push((cell, inference));
        self.changed = true;
    }

    pub fn safe_cells(&self) -> Vec<usize> {
        self.inferences
            .iter()
            .filter(|(_, inf)| *inf == CellInference::Safe)
            .map(|(c, _)| *c)
            .collect()
    }

    pub fn mine_cells(&self) -> Vec<usize> {
        self.inferences
            .iter()
            .filter(|(_, inf)| *inf == CellInference::Mine)
            .map(|(c, _)| *c)
            .collect()
    }
}

pub fn propagate(constraints: &mut ConstraintSet, board: &Board) -> Result<InferenceResult, String> {
    let mut result = InferenceResult::new();
    let mut queue: VecDeque<usize> = (0..constraints.len()).collect();
    let mut in_queue = vec![true; constraints.len()];

    while let Some(idx) = queue.pop_front() {
        in_queue[idx] = false;
        let constraint = &constraints.constraints[idx];
        let remaining = constraint.remaining_mines(board);
        let unknowns = constraint.unknown_cells(board);

        if remaining == 0 {
            for &cell in &unknowns {
                result.add(cell, CellInference::Safe);
            }
        } else if remaining == unknowns.len() {
            for &cell in &unknowns {
                result.add(cell, CellInference::Mine);
            }
        }
    }

    if result.changed {
        let mut new_constraints = ConstraintSet::new();
        for c in &constraints.constraints {
            let unknowns = c.unknown_cells(board);
            let remaining = c.remaining_mines(board);
            if !unknowns.is_empty() && remaining > 0 {
                new_constraints.add(Constraint::new(unknowns, remaining));
            }
        }
        *constraints = new_constraints;
    }

    Ok(result)
}

pub fn subset_reasoning(constraints: &mut ConstraintSet, board: &Board) -> Result<InferenceResult, String> {
    let mut result = InferenceResult::new();
    let mut new_constraints = Vec::new();

    for i in 0..constraints.len() {
        for j in 0..constraints.len() {
            if i == j {
                continue;
            }
            let c1 = &constraints.constraints[i];
            let c2 = &constraints.constraints[j];

            let u1: HashSet<_> = c1.unknown_cells(board).into_iter().collect();
            let u2: HashSet<_> = c2.unknown_cells(board).into_iter().collect();

            if u1.is_empty() || u2.is_empty() {
                continue;
            }

            if u1.is_subset(&u2) {
                let diff: Vec<_> = u2.difference(&u1).copied().collect();
                if !diff.is_empty() {
                    let rem1 = c1.remaining_mines(board);
                    let rem2 = c2.remaining_mines(board);
                    if rem2 >= rem1 {
                        let new_req = rem2 - rem1;
                        new_constraints.push(Constraint::new(diff, new_req));
                    }
                }
            }
        }
    }

    for c in new_constraints {
        if c.is_valid() && c.required_mines > 0 && c.required_mines < c.cells.len() {
            constraints.add(c);
        }
    }

    constraints.normalize();
    Ok(result)
}

pub fn global_mine_rules(board: &Board, constraints: &ConstraintSet) -> InferenceResult {
    let mut result = InferenceResult::new();
    let known_mines = board.flagged_count();
    let remaining_global = board.total_mines.saturating_sub(known_mines);
    let unknown_total = board.unknown_count();

    if remaining_global == 0 {
        for i in 0..board.total_cells() {
            if board.is_unknown(i) {
                result.add(i, CellInference::Safe);
            }
        }
    } else if remaining_global == unknown_total {
        for i in 0..board.total_cells() {
            if board.is_unknown(i) {
                result.add(i, CellInference::Mine);
            }
        }
    }
    result
}