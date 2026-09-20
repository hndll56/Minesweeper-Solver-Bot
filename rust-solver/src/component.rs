use crate::board::Board;
use crate::constraint::ConstraintSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub cells: Vec<usize>,
    pub constraints: Vec<usize>,
}

impl Component {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl Default for Component {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_frontier(board: &Board, constraints: &ConstraintSet) -> (Vec<usize>, Vec<usize>) {
    let mut frontier_set = HashSet::new();
    for c in constraints.constraints() {
        for cell in c.unknown_cells(board) {
            frontier_set.insert(cell);
        }
    }
    let mut frontier: Vec<_> = frontier_set.into_iter().collect();
    frontier.sort_unstable();

    let mut non_frontier = Vec::new();
    for i in 0..board.total_cells() {
        if board.is_unknown(i) && !frontier.contains(&i) {
            non_frontier.push(i);
        }
    }

    (frontier, non_frontier)
}

pub fn decompose_components(
    board: &Board,
    constraints: &ConstraintSet,
    frontier: &[usize],
) -> Vec<Component> {
    let mut cell_to_constraints: HashMap<usize, Vec<usize>> = HashMap::new();
    for (ci, c) in constraints.constraints().iter().enumerate() {
        for cell in c.unknown_cells(board) {
            cell_to_constraints.entry(cell).or_default().push(ci);
        }
    }

    let mut visited = HashSet::new();
    let mut components = Vec::new();

    for &start_cell in frontier {
        if visited.contains(&start_cell) {
            continue;
        }
        let mut comp = Component::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_cell);
        visited.insert(start_cell);

        while let Some(cell) = queue.pop_front() {
            comp.cells.push(cell);
            if let Some(constraint_indices) = cell_to_constraints.get(&cell) {
                for &ci in constraint_indices {
                    comp.constraints.push(ci);
                    let constraint = &constraints.constraints()[ci];
                    for neighbor in constraint.unknown_cells(board) {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        comp.constraints.sort_unstable();
        comp.constraints.dedup();
        comp.cells.sort_unstable();
        if !comp.is_empty() {
            components.push(comp);
        }
    }

    components
}

pub fn component_constraints<'a>(
    component: &Component,
    constraints: &'a ConstraintSet,
    board: &Board,
) -> Vec<&'a crate::constraint::Constraint> {
    component
        .constraints
        .iter()
        .map(|&ci| &constraints.constraints()[ci])
        .collect()
}