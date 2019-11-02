use rand::*;
use std::collections::HashSet;

pub struct Table {
    pub grid: Vec<u8>,
    pub quadrant_side: usize,
    pub side: usize,
}

impl Table {
    pub fn new(quadrant_side: usize) -> Table {
        let side = quadrant_side * quadrant_side;
        Table {
            grid: vec![0; side * side],
            quadrant_side,
            side,
        }
    }

    pub fn clear(&mut self) {
        for value in &mut self.grid {
            *value = 0;
        }
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.side
    }

    pub fn position(&self, index: usize) -> (usize, usize) {
        (index % self.side, index / self.side)
    }

    pub fn quadrant(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
        let start_x = (x / self.quadrant_side) * self.quadrant_side;
        let start_y = (y / self.quadrant_side) * self.quadrant_side;

        // Get all the indexes of values in this quadrant
        (start_x..start_x + self.quadrant_side).flat_map(move |x| {
            (start_y..start_y + self.quadrant_side).map(move |y| self.index(x, y))
        })
    }

    pub fn column(&self, x: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.side).map(move |y| self.index(x, y))
    }

    pub fn row(&self, y: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.side).map(move |x| self.index(x, y))
    }

    // Aka: column + row + quadrant
    pub fn neighborhood(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
        self.column(x).chain(self.row(y)).chain(self.quadrant(x, y))
    }

    pub fn valid(&self, index: usize) -> HashSet<u8> {
        let (x, y) = self.position(index);

        // Calculate the possible valid cells without
        // considering the value of this cell
        let mut possibles: HashSet<u8> = (1..=self.side as u8).collect();
        let used = self
            .neighborhood(x, y)
            .map(|i| if index == i { 0 } else { self.grid[i] });

        for value in used {
            possibles.remove(&value);
        }

        possibles
    }

    pub fn fill(&mut self, current_cell: usize) -> bool {
        if current_cell >= self.side * self.side {
            return true;
        }

        for n in self.valid(current_cell) {
            self.grid[current_cell] = n;

            // If we are able to complete the sudoku with the current value set to n
            // Then we are done
            // Otherwise we set the current cell to the next value and try again
            if self.fill(current_cell + 1) {
                return true;
            }
        }

        // We have exausted all the possible, values,
        // We need to backtrack
        self.grid[current_cell] = 0;
        false
    }

    pub fn obviuos_step(&mut self, holes: &mut HashSet<usize>) -> bool {
        for to_place in holes.iter() {
            let to_place = *to_place;

            let possibles = self.valid(to_place);

            // If we have no possibilities on a tile than the puzzle is unsolvable
            // Because we only make moves were we cant do anything else
            if possibles.is_empty() {
                return false;
            } else if possibles.len() == 1 {
                // We are obliged to do this move, as it is the only one possible
                // The "hole" is now filled and we can loop again with the new blocks to place
                self.grid[to_place] = *possibles.iter().nth(0).unwrap();
                holes.remove(&to_place);
                return true;
            }
        }

        // We couldn't make a single obvious move
        false
    }

    pub fn obviuos(&mut self, initial: &HashSet<usize>) -> bool {
        let mut holes = initial.clone();

        let solvable = loop {
            let placed_something = self.obviuos_step(&mut holes);

            // There were no values to place
            if holes.is_empty() {
                break true;
            } else if !placed_something {
                break false;
            }
        };

        // Reset the grid to the unsolved state
        for init in initial {
            self.grid[*init] = 0;
        }

        solvable
    }

    pub fn unsolve(&mut self) {
        let mut rng = thread_rng();
        let length = self.side * self.side;

        let mut holes = HashSet::new();
        let start = rng.gen_range(0, length);
        for i in 0..length {
            let i = (i + start) % length;
            let original = self.grid[i];

            self.grid[i] = 0;
            holes.insert(i);

            // If we have made a guessing board
            if !self.obviuos(&holes) {
                self.grid[i] = original;
                holes.remove(&i);
            }
        }
    }
}
