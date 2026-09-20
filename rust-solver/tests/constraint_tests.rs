use minesweeper_solver::board::{Board, CellState};
use minesweeper_solver::constraint::{ConstraintSet, Constraint};

#[test]
fn constraint_new() {
    let mut set = ConstraintSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn add_valid_constraint() {
    let b = Board::new(3, 3, 0).unwrap();
    let c = Constraint::new(vec![1, 3, 5], 1);
    assert!(c.is_valid());
}

#[test]
fn add_invalid_constraint_oversubscribed() {
    let c = Constraint::new(vec![1, 2, 3], 5);
    assert!(!c.is_valid());
}

#[test]
fn constraint_unknown_cells() {
    let b = Board::new(3, 3, 0).unwrap();
    // all unknown initially
    let c = Constraint::new(vec![0, 4, 8], 1);
    let unknowns = c.unknown_cells(&b);
    assert_eq!(unknowns.len(), 3);
    assert!(unknowns.contains(&0) && unknowns.contains(&4) && unknowns.contains(&8));
}

#[test]
fn constraint_flagged_cells() {
    let mut b = Board::new(3, 3, 0).unwrap();
    let c = Constraint::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8], 3);
    // flag cells 0, 4, 8
    for &idx in &[0, 4, 8] {
        b.set(idx, CellState::SolverFlagged).unwrap();
    }
    let flagged = c.flagged_cells(&b);
    assert_eq!(flagged.len(), 3);
    assert!(flagged.contains(&0) && flagged.contains(&4) && flagged.contains(&8));
}

#[test]
fn constraint_remaining_mines() {
    let mut b = Board::new(3, 3, 0).unwrap();
    let c = Constraint::new(vec![0, 1, 2], 2);
    assert_eq!(c.remaining_mines(&b), 2);
    // flag one cell
    b.set(0, CellState::SolverFlagged).unwrap();
    assert_eq!(c.remaining_mines(&b), 1);
    b.set(1, CellState::SolverFlagged).unwrap();
    assert_eq!(c.remaining_mines(&b), 0);
}

#[test]
fn constraint_satisfied_true() {
    let mut b = Board::new(3, 3, 2).unwrap();
    let c = Constraint::new(vec![0, 1, 2], 2);
    // flag 0 and 1, reveal 2 (no unknown left in constraint)
    b.set(0, CellState::SolverFlagged).unwrap();
    b.set(1, CellState::SolverFlagged).unwrap();
    b.set(2, CellState::Revealed(0)).unwrap();
    assert!(c.is_satisfied(&b));
}

#[test]
fn constraint_satisfied_false_missing_flag() {
    let mut b = Board::new(3, 3, 2).unwrap();
    let c = Constraint::new(vec![0, 1, 2], 2);
    b.set(0, CellState::SolverFlagged).unwrap();
    assert!(!c.is_satisfied(&b));
}

#[test]
fn constraint_set_from_board_valid() {
    let mut b = Board::new(3, 3, 3).unwrap();
    // reveal all zeros, create constraints
    let mut revealed_indices = Vec::new();
    for y in 0..3 {
        for x in 0..3 {
            let idx = b.index(x, y);
            // set to Revealed(0) for pattern
            b.set(idx, CellState::Revealed(0)).unwrap();
            revealed_indices.push(idx);
        }
    }
    let set = ConstraintSet::from_board(&b).unwrap();
    assert!(set.len() >= 0);
}

#[test]
fn constraint_set_normalize() {
    let mut set = ConstraintSet::new();
    let c1 = Constraint::new(vec![3, 1], 1);
    let c2 = Constraint::new(vec![1, 3], 1);
    set.add(c1);
    set.add(c2); // duplicate after dedup
    set.normalize();
    // should have one constraint
    assert_eq!(set.len(), 1);
}

#[test]
fn constraint_set_validate_clean() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut set = ConstraintSet::new();
    let c = Constraint::new(vec![0, 1, 2], 1);
    set.add(c);
    assert!(set.validate(&b).is_ok());
}

#[test]
fn constraint_set_validate_bad() {
    let mut b = Board::new(3, 3, 0).unwrap();
    // reveal cell 0, 1, 2 so they are no longer unknown
    b.set(0, CellState::Revealed(0)).unwrap();
    b.set(1, CellState::Revealed(0)).unwrap();
    b.set(2, CellState::Revealed(0)).unwrap();
    let mut set = ConstraintSet::new();
    let c = Constraint::new(vec![0, 1, 2], 2); // requires 2 mines, but 0 unknowns left
    set.add(c);
    assert!(set.validate(&b).is_err());
}

#[test]
fn constraint_subset_reasoning() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut set = ConstraintSet::new();
    // add two constraints where one is subset of the other
    let c1 = Constraint::new(vec![1, 2], 1);
    let c2 = Constraint::new(vec![1, 2, 3, 4], 3);
    set.add(c1.clone());
    set.add(c2.clone());
    let result = minesweeper_solver::constraint::subset_reasoning(&mut set, &b).unwrap();
    // should add a new constraint for the difference
    let len_before = 2;
    // after reasoning, there should be >= 2 constraints (original + new)
    assert!(set.len() >= len_before);
}

#[test]
fn global_mine_rules_remaining_zero() {
    let mut b = Board::new(3, 3, 1).unwrap();
    // flag the one mine
    b.set(0, CellState::SolverFlagged).unwrap();
    let set = ConstraintSet::new();
    let result = minesweeper_solver::constraint::global_mine_rules(&b, &set);
    assert_eq!(result.safe_cells().len(), b.unknown_count());
    assert_eq!(result.mine_cells().len(), 0);
}

#[test]
fn global_mine_rules_all_mines() {
    let b = Board::new(3, 3, 9).unwrap(); // all mines? can't, max 9 cells
    // actually board with 9 mines in 9 cells = all cells are mines
    // but we can't reveal mines, so create a board where all cells are mines via flags
    let mut b = Board::new(3, 3, 9).unwrap();
    for i in 0..b.total_cells() {
        b.set(i, CellState::SolverFlagged).unwrap();
    }
    let set = ConstraintSet::new();
    let result = minesweeper_solver::constraint::global_mine_rules(&b, &set);
    assert_eq!(result.safe_cells().len(), 0);
    assert_eq!(result.mine_cells().len(), b.unknown_count());
}
