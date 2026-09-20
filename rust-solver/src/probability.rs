use crate::board::Board;
use crate::component::Component;
use crate::enumerate::ComponentResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProbability {
    pub cell_prob: HashMap<usize, f64>,
    pub non_frontier_prob: f64,
    pub frontier_probs: HashMap<usize, f64>,
}

impl GlobalProbability {
    pub fn new() -> Self {
        Self {
            cell_prob: HashMap::new(),
            non_frontier_prob: 0.0,
            frontier_probs: HashMap::new(),
        }
    }
}

impl Default for GlobalProbability {
    fn default() -> Self {
        Self::new()
    }
}

fn binomial(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    if k == 0 || k == n {
        return 1.0;
    }
    let k = k.min(n - k);
    let mut result = 1.0;
    for i in 1..=k {
        result = result * (n - k + i) as f64 / i as f64;
    }
    result
}

pub fn compute_global_probability(
    board: &Board,
    components: &[Component],
    component_results: &[ComponentResult],
    non_frontier: &[usize],
) -> GlobalProbability {
    let mut prob = GlobalProbability::new();

    let R = board.total_mines.saturating_sub(board.flagged_count()) as i32;
    let N = non_frontier.len();

    if components.is_empty() {
        if N > 0 && R >= 0 {
            prob.non_frontier_prob = R as f64 / N as f64;
            for &cell in non_frontier {
                prob.cell_prob.insert(cell, prob.non_frontier_prob);
            }
        }
        return prob;
    }

    let num_components = components.len();
    let max_mines_per_comp: Vec<usize> = component_results.iter().map(|r| r.ways.len().saturating_sub(1)).collect();

    let mut prefix = vec![vec![0.0; R as usize + 1]; num_components + 1];
    prefix[0][0] = 1.0;

    for i in 0..num_components {
        let ways = &component_results[i].ways;
        for k in 0..=R as usize {
            if prefix[i][k] == 0.0 {
                continue;
            }
            for m in 0..ways.len() {
                if k + m <= R as usize {
                    prefix[i + 1][k + m] += prefix[i][k] * ways[m];
                }
            }
        }
    }

    let mut suffix = vec![vec![0.0; R as usize + 1]; num_components + 1];
    suffix[num_components][0] = 1.0;

    for i in (0..num_components).rev() {
        let ways = &component_results[i].ways;
        for k in 0..=R as usize {
            if suffix[i + 1][k] == 0.0 {
                continue;
            }
            for m in 0..ways.len() {
                if k + m <= R as usize {
                    suffix[i][k + m] += suffix[i + 1][k] * ways[m];
                }
            }
        }
    }

    let total_weight = prefix[num_components].iter().sum::<f64>();

    for (comp_idx, comp) in components.iter().enumerate() {
        let comp_result = &component_results[comp_idx];
        for &cell in &comp.cells {
            let mut cell_weight = 0.0;
            let mine_ways = &comp_result.mine_ways;

            for m in 0..comp_result.ways.len() {
                if comp_result.ways[m] == 0.0 {
                    continue;
                }
                let other_mines = R as usize - m;
                let mut other_weight = 0.0;

                for k in 0..=other_mines {
                    let pre = prefix[comp_idx][k];
                    let suf = if comp_idx + 1 < suffix.len() {
                        suffix[comp_idx + 1][other_mines - k]
                    } else {
                        0.0
                    };
                    other_weight += pre * suf;
                }

                let non_front_weight = if N > 0 && other_mines <= N {
                    binomial(N, other_mines)
                } else {
                    0.0
                };

                let weight = comp_result.ways[m] * other_weight * non_front_weight;
                if let Some(mw) = mine_ways.get(&cell) {
                    if m < mw.len() {
                        cell_weight += mw[m] * other_weight * non_front_weight;
                    }
                }
            }

            if total_weight > 0.0 {
                let p = cell_weight / total_weight;
                prob.cell_prob.insert(cell, p);
                prob.frontier_probs.insert(cell, p);
            }
        }
    }

    if N > 0 && R >= 0 {
        let mut expected_nf_mines = 0.0;
        let total_weight = prefix[num_components].iter().sum::<f64>();

        for k in 0..=R as usize {
            if prefix[num_components][k] == 0.0 {
                continue;
            }
            let remaining = R as usize - k;
            if remaining <= N {
                let nf_weight = binomial(N, remaining);
                expected_nf_mines += remaining as f64 * prefix[num_components][k] * nf_weight;
            }
        }

        if total_weight > 0.0 {
            prob.non_frontier_prob = expected_nf_mines / total_weight / N as f64;
            for &cell in non_frontier {
                prob.cell_prob.insert(cell, prob.non_frontier_prob);
            }
        }
    }

    prob
}

pub fn extract_certainties(prob: &GlobalProbability) -> (Vec<usize>, Vec<usize>) {
    let mut safe = Vec::new();
    let mut mines = Vec::new();
    for (&cell, &p) in &prob.cell_prob {
        if p <= 1e-10 {
            safe.push(cell);
        } else if p >= 1.0 - 1e-10 {
            mines.push(cell);
        }
    }
    safe.sort_unstable();
    mines.sort_unstable();
    (safe, mines)
}