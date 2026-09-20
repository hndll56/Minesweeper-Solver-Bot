use minesweeper_solver::board::{Board, CellState};
use minesweeper_solver::component::{Component, decompose_components, extract_frontier};
use minesweeper_solver::constraint::ConstraintSet;

#[test]
fn component_new() {
    let c = Component::new();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}

#[test]
fn extract_frontier_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let set = ConstraintSet::new();
    let (frontier, non_frontier) = extract_frontier(&b, &set);
    assert!(frontier.is_empty());
    assert_eq!(non_frontier.len(), b.unknown_count());
}

#[test]
fn extract_frontier_with_unknown_neighbors() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut set = ConstraintSet::new();
    set.add(minesweeper_solver::constraint::Constraint::new(vec![0, 1, 2], 1));
    let (frontier, non_frontier) = extract_frontier(&b, &set);
    assert!(frontier.contains(&0));
    assert!(frontier.contains(&1));
    assert!(frontier.contains(&2));
    // 0,1,2 are frontier; 3-8 are non-frontier unknowns
    assert_eq!(non_frontier.len(), 6); // cells 3-8
}

#[test]
fn decompose_components_single() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut set = ConstraintSet::new();
    set.add(minesweeper_solver::constraint::Constraint::new(vec![0, 1, 2], 1));
    let (frontier, _non_frontier) = extract_frontier(&b, &set);
    let components = decompose_components(&b, &set, &frontier);
    assert_eq!(components.len(), 1);
}

#[test]
fn decompose_components_disconnected() {
    let b = Board::new(5, 1, 0).unwrap(); // 1x5 board
    let mut set = ConstraintSet::new();
    // Two constraints that don't share unknowns
    set.add(minesweeper_solver::constraint::Constraint::new(vec![0, 1], 1));
    set.add(minesweeper_solver::constraint::Constraint::new(vec![3, 4], 1));
    let (frontier, _non_frontier) = extract_frontier(&b, &set);
    let components = decompose_components(&b, &set, &frontier);
    // cells 0,1 share constraint → one component; cells 3,4 share constraint → another
    // but constraint [0,1] has 0,1; constraint [3,4] has 3,4 — no overlap → 2 components
    assert_eq!(components.len(), 2);
}

#[test]
fn component_cells_sorted() {
    let mut comp = Component::new();
    comp.cells = vec![4, 1, 3];
    comp.cells.sort_unstable();
    assert_eq!(comp.cells, vec![1, 3, 4]);
}

#[test]
fn component_len() {
    let mut comp = Component::new();
    comp.cells = vec![1, 2, 3];
    assert_eq!(comp.len(), 3);
}
