use minesweeper_solver::board::Board;
use minesweeper_solver::probability::{extract_certainties, GlobalProbability};

#[test]
fn global_probability_known_mine_p_is_one() {
    let mut prob = GlobalProbability::new();
    prob.cell_prob.insert(0, 1.0);
    prob.cell_prob.insert(1, 1.0);
    let (safe, mines) = extract_certainties(&prob);
    assert!(safe.is_empty());
    assert!(mines.contains(&0));
    assert!(mines.contains(&1));
}

#[test]
fn global_probability_known_safe_p_is_zero() {
    let mut prob = GlobalProbability::new();
    prob.cell_prob.insert(0, 0.0);
    prob.cell_prob.insert(1, 0.0);
    let (safe, mines) = extract_certainties(&prob);
    assert!(safe.contains(&0));
    assert!(safe.contains(&1));
    assert!(mines.is_empty());
}

#[test]
fn global_probability_probabilities_sum_reasonable() {
    let prob = GlobalProbability::new();
    let (safe, mines) = extract_certainties(&prob);
    assert!(safe.is_empty());
    assert!(mines.is_empty());
}
