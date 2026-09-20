use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    Unknown,
    Revealed(u8),
    SolverFlagged,
}

impl CellState {
    pub fn is_unknown(&self) -> bool {
        matches!(self, CellState::Unknown)
    }

    pub fn is_revealed(&self) -> bool {
        matches!(self, CellState::Revealed(_))
    }

    pub fn is_flagged(&self) -> bool {
        matches!(self, CellState::SolverFlagged)
    }

    pub fn number(&self) -> Option<u8> {
        match self {
            CellState::Revealed(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub total_mines: usize,
    pub cells: Vec<CellState>,
}

impl Board {
    pub fn new(width: usize, height: usize, total_mines: usize) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("Width and height must be positive".to_string());
        }
        if total_mines > width * height {
            return Err("Total mines cannot exceed total cells".to_string());
        }
        Ok(Self {
            width,
            height,
            total_mines,
            cells: vec![CellState::Unknown; width * height],
        })
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn coords(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    pub fn get(&self, index: usize) -> Option<CellState> {
        self.cells.get(index).copied()
    }

    pub fn set(&mut self, index: usize, state: CellState) -> Result<(), String> {
        if index >= self.cells.len() {
            return Err("Index out of bounds".to_string());
        }
        self.cells[index] = state;
        Ok(())
    }

    pub fn neighbors(&self, index: usize) -> Vec<usize> {
        let (x, y) = self.coords(index);
        let mut result = Vec::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    result.push(self.index(nx as usize, ny as usize));
                }
            }
        }
        result
    }

    pub fn is_unknown(&self, index: usize) -> bool {
        self.get(index).map(|c| c.is_unknown()).unwrap_or(false)
    }

    pub fn is_revealed(&self, index: usize) -> bool {
        self.get(index).map(|c| c.is_revealed()).unwrap_or(false)
    }

    pub fn is_flagged(&self, index: usize) -> bool {
        self.get(index).map(|c| c.is_flagged()).unwrap_or(false)
    }

    pub fn count_unknown_neighbors(&self, index: usize) -> usize {
        self.neighbors(index).iter().filter(|&&i| self.is_unknown(i)).count()
    }

    pub fn count_flagged_neighbors(&self, index: usize) -> usize {
        self.neighbors(index).iter().filter(|&&i| self.is_flagged(i)).count()
    }

    pub fn revealed_number(&self, index: usize) -> Option<u8> {
        self.get(index).and_then(|c| c.number())
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }

    pub fn unknown_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_unknown()).count()
    }

    pub fn flagged_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_flagged()).count()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.total_mines > self.total_cells() {
            return Err("Total mines exceeds total cells".to_string());
        }
        for (i, cell) in self.cells.iter().enumerate() {
            if let CellState::Revealed(n) = cell {
                if *n > 8 {
                    return Err(format!("Cell {} has invalid number {}", i, n));
                }
                let flagged = self.count_flagged_neighbors(i);
                let unknown = self.count_unknown_neighbors(i);
                if flagged > *n as usize {
                    return Err(format!(
                        "Cell {} has {} flagged neighbors but number is {}",
                        i, flagged, n
                    ));
                }
                if flagged + unknown < *n as usize {
                    return Err(format!(
                        "Cell {} cannot satisfy {} mines with {} flagged + {} unknown",
                        i, n, flagged, unknown
                    ));
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let ch = match self.cells[idx] {
                    CellState::Unknown => '.',
                    CellState::SolverFlagged => 'F',
                    CellState::Revealed(0) => '0',
                    CellState::Revealed(n) => char::from_digit(n as u32, 10).unwrap_or('?'),
                };
                write!(f, "{} ", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}