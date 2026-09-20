use minesweeper_solver::board::{Board, CellState};
use minesweeper_solver::constraint::{ConstraintSet, Constraint};
use minesweeper_solver::component::{decompose_components, extract_frontier};
use minesweeper_solver::enumerate::{solve_component, SolverConfig, ComponentResult};
use minesweeper_solver::probability::{compute_global_probability, extract_certainties, GlobalProbability};
use minesweeper_solver::select::{SolveResult, SolveStatus};

#[test]
fn seeded_board_generation() {
    let mut board = Board::new(3, 3, 3).unwrap();
    let revealed_cells = [
        (1, 0, 2),
        (0, 1, 1),
        (2, 1, 1),
    ];
    for (x, y, num) in &revealed_cells {
        let idx = board.index(*x, *y);
        board.set(idx, CellState::Revealed(*num)).unwrap();
    }
    assert!(board.validate().is_ok());
}

#[test]
fn seeded_board_probability_sum() {
    let mut board = Board::new(4, 4, 6).unwrap();
    board.set(board.index(1, 1), CellState::Revealed(2)).unwrap();
    board.set(board.index(2, 1), CellState::Revealed(1)).unwrap();
    board.set(board.index(1, 2), CellState::Revealed(1)).unwrap();

    let constraints = ConstraintSet::from_board(&board).unwrap();
    let (frontier, non_frontier) = extract_frontier(&board, &constraints);
    let components = decompose_components(&board, &constraints, &frontier);

    let config = SolverConfig::default();
    let comp_results: Vec<ComponentResult> = components
        .iter()
        .map(|comp| {
            let comp_constraints: Vec<_> = comp.constraints.iter().map(|&ci| &constraints.constraints[ci]).collect();
            solve_component(&board, comp, &comp_constraints, &config)
        })
        .collect();

    let prob = compute_global_probability(&board, &components, &comp_results, &non_frontier);

    // Basic sanity checks on probabilities
    for (_, p) in prob.cell_prob.iter() {
        assert!(0.0 <= *p && *p <= 1.0, "Probability {} is out of [0, 1]", p);
    }
}

#[test]
fn seeded_board_known_mines_probability_one() {
    let mut board = Board::new(3, 3, 3).unwrap();
    // For a 3x3 corner, cell (0,0) has 3 neighbors (1, 3, 4). If Revealed(3) and total_mines=3, all 3 must be mines!
    board.set(0, CellState::Revealed(3)).unwrap();

    let constraints = ConstraintSet::from_board(&board).unwrap();
    let (frontier, non_frontier) = extract_frontier(&board, &constraints);
    let components = decompose_components(&board, &constraints, &frontier);

    let config = SolverConfig::default();
    let comp_results: Vec<ComponentResult> = components
        .iter()
        .map(|comp| {
            let comp_constraints: Vec<_> = comp.constraints.iter().map(|&ci| &constraints.constraints[ci]).collect();
            solve_component(&board, comp, &comp_constraints, &config)
        })
        .collect();

    let prob = compute_global_probability(&board, &components, &comp_results, &non_frontier);
    let (_safe, mines) = extract_certainties(&prob);

    // Neighbors of (0,0) are 1, 3, 4 — all must be mines!
    let neighbors = board.neighbors(0);
    for nb in neighbors {
        assert!(mines.contains(&nb), "Cell {} should be identified as a mine", nb);
    }
}

#[test]
fn seeded_board_known_safes_probability_zero() {
    let mut board = Board::new(3, 3, 0).unwrap();
    // In a 0-mine board, all cells are safe
    let constraints = ConstraintSet::new();
    let (frontier, non_frontier) = extract_frontier(&board, &constraints);
    let components = decompose_components(&board, &constraints, &frontier);

    let comp_results: Vec<ComponentResult> = vec![];
    let prob = compute_global_probability(&board, &components, &comp_results, &non_frontier);
    let (safe, _mines) = extract_certainties(&prob);

    // All unknown cells must be safe
    assert_eq!(safe.len(), board.unknown_count());
}

#[test]
fn seeded_board_constraint_propagation() {
    let mut board = Board::new(3, 3, 3).unwrap();
    // Revealed(3) at corner (0,0) which has exactly 3 neighbors -> all 3 neighbors are forced mines!
    board.set(0, CellState::Revealed(3)).unwrap();

    let mut constraints = ConstraintSet::from_board(&board).unwrap();
    let result = minesweeper_solver::constraint::propagate(&mut constraints, &board).unwrap();

    assert!(result.changed);
    assert_eq!(result.mine_cells().len(), 3);
}

#[test]
fn seeded_board_subset_reasoning() {
    let board = Board::new(3, 3, 0).unwrap();
    let mut constraints = ConstraintSet::new();
    constraints.add(Constraint::new(vec![0, 1, 2], 1));
    constraints.add(Constraint::new(vec![0, 1, 2, 3], 2));

    let _result = minesweeper_solver::constraint::subset_reasoning(&mut constraints, &board).unwrap();
    assert!(constraints.len() > 0);
}

#[test]
fn seeded_board_solver_pipeline() {
    let mut board = Board::new(5, 5, 8).unwrap();
    board.set(board.index(2, 2), CellState::Revealed(3)).unwrap();
    board.set(board.index(3, 2), CellState::Revealed(2)).unwrap();
    board.set(board.index(2, 3), CellState::Revealed(2)).unwrap();

    let constraints = ConstraintSet::from_board(&board).unwrap();
    let (frontier, non_frontier) = extract_frontier(&board, &constraints);
    let components = decompose_components(&board, &constraints, &frontier);

    let config = SolverConfig::default();
    let comp_results: Vec<ComponentResult> = components
        .iter()
        .map(|comp| {
            let comp_constraints: Vec<_> = comp.constraints.iter().map(|&ci| &constraints.constraints[ci]).collect();
            solve_component(&board, comp, &comp_constraints, &config)
        })
        .collect();

    let prob = compute_global_probability(&board, &components, &comp_results, &non_frontier);
    for (_, p) in prob.cell_prob.iter() {
        assert!(0.0 <= *p && *p <= 1.0);
    }
}

#[test]
fn seeded_board_simulator_basic() {
    let mut board = Board::new(4, 4, 4).unwrap();
    let center = board.index(2, 2);
    board.set(center, CellState::Revealed(8)).unwrap();

    let neighbors = board.neighbors(center);
    for &nb in &neighbors {
        board.set(nb, CellState::SolverFlagged).unwrap();
    }

    assert!(board.validate().is_ok());
}

#[test]
fn seeded_board_probability_edge_cases() {
    let board = Board::new(3, 3, 0).unwrap();
    let constraints = ConstraintSet::new();
    let (_frontier, _non_frontier) = extract_frontier(&board, &constraints);

    let prob = GlobalProbability::new();
    let (safe, mines) = extract_certainties(&prob);

    assert!(safe.is_empty());
    assert!(mines.is_empty());
}

#[test]
fn seeded_board_component_decomposition() {
    let board = Board::new(5, 5, 0).unwrap();
    let mut constraints = ConstraintSet::new();
    constraints.add(Constraint::new(vec![0, 1], 1));
    constraints.add(Constraint::new(vec![10, 11, 12], 2));

    let (frontier, _non_frontier) = extract_frontier(&board, &constraints);
    let components = decompose_components(&board, &constraints, &frontier);

    assert!(components.len() >= 1);
}

#[test]
fn seeded_board_solver_result_types() {
    let result = SolveResult::ok();
    assert_eq!(result.status, SolveStatus::Ok);
    assert!(result.actions.is_empty());

    let error_result = SolveResult::error(SolveStatus::NoSolution);
    assert_eq!(error_result.status, SolveStatus::NoSolution);
    assert!(error_result.actions.is_empty());
}
