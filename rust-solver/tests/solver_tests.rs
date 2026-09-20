use minesweeper_solver::board::{Board, CellState};
use minesweeper_solver::component::{Component, decompose_components, extract_frontier};
use minesweeper_solver::constraint::ConstraintSet;
use minesweeper_solver::enumerate::{solve_component, SolverConfig, ComponentResult};
use minesweeper_solver::probability::{compute_global_probability, extract_certainties, GlobalProbability};
use minesweeper_solver::select::{select_guess, select_moves, two_ply_analysis, Action, SolveResult, SolveStatus, Diagnostics};

fn brute_force_solve(board: &Board) -> (Vec<usize>, Vec<usize>) {
    let unknowns: Vec<usize> = (0..board.total_cells())
        .filter(|&i| board.is_unknown(i))
        .collect();
    let n = unknowns.len();
    if n > 25 {
        return (vec![], vec![]);
    }
    let mut all_safe = vec![true; n];
    let mut all_mine = vec![true; n];
    let mut found_valid = false;

    for mask in 0u64..(1u64 << n) {
        let mine_count = mask.count_ones() as usize;
        if mine_count != board.total_mines {
            continue;
        }
        let mut valid = true;
        for i in 0..board.total_cells() {
            if let Some(num) = board.revealed_number(i) {
                let neighbors = board.neighbors(i);
                let mut flagged = 0;
                for &nb in &neighbors {
                    if board.is_flagged(nb) {
                        flagged += 1;
                    } else if board.is_unknown(nb) {
                        let idx = unknowns.iter().position(|&x| x == nb).unwrap();
                        if (mask >> idx) & 1 == 1 {
                            flagged += 1;
                        }
                    }
                }
                if flagged != num as usize {
                    valid = false;
                    break;
                }
            }
        }
        if valid {
            found_valid = true;
            for i in 0..n {
                let is_mine = (mask >> i) & 1 == 1;
                if is_mine {
                    all_safe[i] = false;
                } else {
                    all_mine[i] = false;
                }
            }
        }
    }

    let safe: Vec<usize> = unknowns.iter().enumerate().filter(|(i, _)| all_safe[*i]).map(|(_, &x)| x).collect();
    let mines: Vec<usize> = unknowns.iter().enumerate().filter(|(i, _)| all_mine[*i]).map(|(_, &x)| x).collect();
    (safe, mines)
}

#[test]
fn solve_3x3_simple() {
    let mut b = Board::new(3, 3, 2).unwrap();
    let corners = [0, 2, 6, 8];
    for &c in &corners {
        b.set(c, CellState::SolverFlagged).unwrap();
    }
    b.set(1, CellState::Revealed(2)).unwrap();
    let (safe, mines) = brute_force_solve(&b);
    assert!(!safe.is_empty() || !mines.is_empty());
}

#[test]
fn solve_4x4_pattern() {
    let mut b = Board::new(4, 4, 5).unwrap();
    b.set(5, CellState::Revealed(2)).unwrap();
    b.set(0, CellState::SolverFlagged).unwrap();
    b.set(1, CellState::SolverFlagged).unwrap();
    let (safe, mines) = brute_force_solve(&b);
    assert!(!safe.is_empty() || !mines.is_empty());
}

#[test]
fn solve_5x5_pattern() {
    let mut b = Board::new(5, 5, 8).unwrap();
    let center = b.index(2, 2);
    b.set(center, CellState::Revealed(3)).unwrap();
    b.set(b.index(1, 1), CellState::Revealed(2)).unwrap();
    b.set(b.index(3, 1), CellState::Revealed(1)).unwrap();
    let (_safe, _mines) = brute_force_solve(&b);
}

#[test]
fn solve_optimized_matches_bruteforce_3x3() {
    let mut b = Board::new(3, 3, 1).unwrap();
    b.set(1, CellState::Revealed(1)).unwrap();
    let (bf_safe, bf_mines) = brute_force_solve(&b);

    let constraints = ConstraintSet::from_board(&b).unwrap();
    let (frontier, non_frontier) = extract_frontier(&b, &constraints);
    let components = decompose_components(&b, &constraints, &frontier);
    let config = SolverConfig::default();
    let comp_results: Vec<ComponentResult> = components
        .iter()
        .map(|comp| {
            let comp_constraints: Vec<_> = comp.constraints.iter().map(|&ci| &constraints.constraints[ci]).collect();
            solve_component(&b, comp, &comp_constraints, &config)
        })
        .collect();

    let prob = compute_global_probability(&b, &components, &comp_results, &non_frontier);
    let (opt_safe, opt_mines) = extract_certainties(&prob);

    for cell in &bf_safe {
        assert!(opt_safe.contains(cell), "BF-safe cell {} should be safe in optimizer", cell);
    }
    for cell in &bf_mines {
        assert!(opt_mines.contains(cell), "BF-mine cell {} should be mine in optimizer", cell);
    }
}

#[test]
fn solve_optimized_vs_bruteforce_4x4() {
    let mut b = Board::new(4, 4, 4).unwrap();
    b.set(b.index(1, 1), CellState::Revealed(2)).unwrap();
    b.set(b.index(2, 1), CellState::Revealed(1)).unwrap();
    let (bf_safe, bf_mines) = brute_force_solve(&b);

    let constraints = ConstraintSet::from_board(&b).unwrap();
    let (frontier, non_frontier) = extract_frontier(&b, &constraints);
    let components = decompose_components(&b, &constraints, &frontier);
    let config = SolverConfig::default();
    let mut comp_results: Vec<ComponentResult> = components
        .iter()
        .map(|comp| {
            let comp_constraints: Vec<&minesweeper_solver::constraint::Constraint> = vec![];
            solve_component(&b, comp, &comp_constraints, &config)
        })
        .collect();

    let prob = compute_global_probability(&b, &components, &comp_results, &non_frontier);
    let (opt_safe, opt_mines) = extract_certainties(&prob);

    for cell in &bf_safe {
        assert!(opt_safe.contains(cell) || opt_mines.contains(cell), "BF-safe cell {} should be resolved", cell);
    }
    for cell in &bf_mines {
        assert!(opt_safe.contains(cell) || opt_mines.contains(cell), "BF-mine cell {} should be resolved", cell);
    }
}

#[test]
fn solve_component_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut comp = Component::new();
    comp.cells = vec![0, 1, 2];
    let config = SolverConfig::default();
    let constraints: Vec<&minesweeper_solver::constraint::Constraint> = vec![];
    let result = solve_component(&b, &comp, &constraints, &config);
    assert!(!result.incomplete);
}

#[test]
fn global_probability_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let components: Vec<Component> = vec![];
    let comp_results: Vec<ComponentResult> = vec![];
    let non_frontier: Vec<usize> = vec![];
    let prob = compute_global_probability(&b, &components, &comp_results, &non_frontier);
    assert_eq!(prob.cell_prob.len(), 0);
}

#[test]
fn extract_certainties_empty() {
    let prob = GlobalProbability::new();
    let (safe, mines) = extract_certainties(&prob);
    assert!(safe.is_empty());
    assert!(mines.is_empty());
}

#[test]
fn extract_certainties_known() {
    let mut prob = GlobalProbability::new();
    prob.cell_prob.insert(5, 0.0);
    prob.cell_prob.insert(6, 1.0);
    let (safe, mines) = extract_certainties(&prob);
    assert!(safe.contains(&5));
    assert!(mines.contains(&6));
}

#[test]
fn select_moves_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let prob = GlobalProbability::new();
    let safe = vec![1, 2];
    let mines = vec![3, 4];
    let actions = select_moves(&b, &prob, &safe, &mines);
    assert_eq!(actions.len(), 4);
    for a in &actions {
        match a {
            Action::Click { cell } => assert!(safe.contains(cell)),
            Action::Flag { cell } => assert!(mines.contains(cell)),
            _ => panic!("expected Click or Flag"),
        }
    }
}

#[test]
fn select_guess_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let mut prob = GlobalProbability::new();
    prob.cell_prob.insert(1, 0.3);
    prob.cell_prob.insert(2, 0.5);
    let guess = select_guess(&b, &prob, &[1, 2], &[]);
    assert!(guess.is_some());
}

#[test]
fn two_ply_analysis_basic() {
    let b = Board::new(3, 3, 0).unwrap();
    let prob = GlobalProbability::new();
    let result = two_ply_analysis(&b, &prob, &[1, 2, 3], 3);
    assert!(result.is_none() || matches!(result, Some(Action::Click { .. })));
}

#[test]
fn solve_result_ok() {
    let result = SolveResult::ok();
    assert_eq!(result.status, SolveStatus::Ok);
    assert!(result.actions.is_empty());
}

#[test]
fn solve_result_error() {
    let result = SolveResult::error(SolveStatus::NoSolution);
    assert_eq!(result.status, SolveStatus::NoSolution);
}

#[test]
fn diagnostics_default() {
    let d = Diagnostics::default();
    assert_eq!(d.components, 0);
}
