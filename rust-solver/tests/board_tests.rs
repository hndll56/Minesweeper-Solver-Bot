use minesweeper_solver::board::{Board, CellState};

#[test]
fn new_board_all_unknown() {
    let b = Board::new(3, 3, 2).unwrap();
    assert_eq!(b.total_cells(), 9);
    assert_eq!(b.unknown_count(), 9);
    assert_eq!(b.flagged_count(), 0);
}

#[test]
fn new_board_zero_dim_errors() {
    assert!(Board::new(0, 3, 0).is_err());
    assert!(Board::new(3, 0, 0).is_err());
}

#[test]
fn new_board_too_many_mines_errors() {
    assert!(Board::new(2, 2, 5).is_err());
}

#[test]
fn index_roundtrip() {
    let b = Board::new(5, 4, 0).unwrap();
    for y in 0..4 {
        for x in 0..5 {
            let idx = b.index(x, y);
            assert_eq!(b.coords(idx), (x, y));
        }
    }
}

#[test]
fn set_and_get() {
    let mut b = Board::new(3, 3, 1).unwrap();
    let idx = b.index(1, 1);
    b.set(idx, CellState::Revealed(2)).unwrap();
    assert_eq!(b.get(idx), Some(CellState::Revealed(2)));
    assert!(b.is_revealed(idx));
    assert!(!b.is_unknown(idx));
}

#[test]
fn set_out_of_bounds_errors() {
    let mut b = Board::new(2, 2, 0).unwrap();
    assert!(b.set(99, CellState::Unknown).is_err());
}

#[test]
fn flag_cell() {
    let mut b = Board::new(3, 3, 1).unwrap();
    let idx = b.index(0, 0);
    b.set(idx, CellState::SolverFlagged).unwrap();
    assert!(b.is_flagged(idx));
    assert_eq!(b.flagged_count(), 1);
}

#[test]
fn neighbors_corner() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut n = b.neighbors(b.index(0, 0));
    n.sort_unstable();
    // corner (0,0) has neighbors (1,0), (0,1), (1,1) → indices 1, 3, 4
    assert_eq!(n, vec![1, 3, 4]);
}

#[test]
fn neighbors_center() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut n = b.neighbors(b.index(1, 1));
    n.sort_unstable();
    assert_eq!(n.len(), 8);
}

#[test]
fn neighbors_edge() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut n = b.neighbors(b.index(1, 0)); // top edge
    n.sort_unstable();
    assert_eq!(n.len(), 5);
}

#[test]
fn validate_clean_board() {
    let b = Board::new(3, 3, 1).unwrap();
    assert!(b.validate().is_ok());
}

#[test]
fn validate_detects_over_flagged() {
    let mut b = Board::new(3, 3, 1).unwrap();
    // reveal center as 0, flag a neighbor — flagged > number
    let center = b.index(1, 1);
    b.set(center, CellState::Revealed(0)).unwrap();
    let n0 = b.neighbors(center)[0];
    b.set(n0, CellState::SolverFlagged).unwrap();
    assert!(b.validate().is_err());
}

#[test]
fn validate_detects_impossible_constraint() {
    // Cell revealed(3) but only 1 unknown neighbor + 0 flagged → unsatisfiable
    let mut b = Board::new(3, 3, 3).unwrap();
    let center = b.index(1, 1);
    // Reveal all neighbors as revealed(0) except one, making center unsatisfiable
    let neighbors = b.neighbors(center);
    for &n in &neighbors[1..] {
        b.set(n, CellState::Revealed(0)).unwrap();
    }
    // center needs 3 mines but only 1 unknown neighbor
    b.set(center, CellState::Revealed(3)).unwrap();
    assert!(b.validate().is_err());
}

#[test]
fn cell_state_helpers() {
    assert!(CellState::Unknown.is_unknown());
    assert!(!CellState::Unknown.is_revealed());
    assert!(CellState::Revealed(3).is_revealed());
    assert_eq!(CellState::Revealed(3).number(), Some(3));
    assert_eq!(CellState::Unknown.number(), None);
    assert!(CellState::SolverFlagged.is_flagged());
}

#[test]
fn count_flagged_and_unknown_neighbors() {
    let mut b = Board::new(3, 3, 2).unwrap();
    let center = b.index(1, 1);
    let neighbors = b.neighbors(center);
    b.set(neighbors[0], CellState::SolverFlagged).unwrap();
    b.set(neighbors[1], CellState::Revealed(0)).unwrap();
    assert_eq!(b.count_flagged_neighbors(center), 1);
    assert_eq!(b.count_unknown_neighbors(center), 6); // 8 neighbors - 1 flagged - 1 revealed
}
