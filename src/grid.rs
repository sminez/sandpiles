use crate::Cell;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    collections::BTreeMap,
    fmt,
    fs::{self, File},
    io::Write,
    path::Path,
    time::SystemTime,
};

#[derive(Serialize, Deserialize)]
pub struct RenderedGrid {
    pub iterations: usize,
    pub grid_size: u32,
    pub grid: Vec<Vec<u8>>,
}

impl fmt::Display for RenderedGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .grid
            .iter()
            .map(|line| line.iter().map(|n| format!("{n},")).collect())
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{lines}")
    }
}

pub struct Grid {
    pub grid: BTreeMap<Cell, u32>,
    pub sand_power: u32,
    pub max_per_cell: u32,
    pub topple_cells: Vec<Cell>,
    pub max_dim: i16,
    pub pattern: String,
}

impl Grid {
    pub fn new(sand_power: u32, pattern: String, topple_cells: Vec<Cell>) -> Grid {
        let grid = BTreeMap::new();
        let max_per_cell = topple_cells.len() as u32;
        let max_dim = 1;

        Grid {
            grid,
            max_per_cell,
            sand_power,
            topple_cells,
            max_dim,
            pattern,
        }
    }

    pub fn topple(&mut self) -> usize {
        let starting_sand = 2_u32.pow(self.sand_power);
        self.grid.insert((0, 0), starting_sand);

        let mut cell_max = starting_sand + 1;
        let mut iterations = 0;
        let start = SystemTime::now();

        while cell_max >= self.max_per_cell {
            cell_max = 0;

            let new_sand: Vec<((i16, i16), u32)> = self
                .grid
                .par_iter_mut()
                .flat_map(|(&(row, col), sand)| {
                    if *sand < self.max_per_cell {
                        return None;
                    }
                    let per_cell = *sand / self.max_per_cell;
                    *sand %= self.max_per_cell;

                    Some(
                        self.topple_cells
                            .par_iter()
                            .map(move |&(dx, dy)| ((row + dx, col + dy), per_cell)),
                    )
                })
                .flatten()
                .collect();

            // TODO: see if this can be parallelised as well.
            //       current blocker is needing to mutate the grid cells (dashmap might help?)
            for (cell, sand) in new_sand.iter() {
                let total = self.grid.entry(*cell).or_insert(0);
                *total += sand;
                cell_max = max(cell_max, *total);
                self.max_dim = max(self.max_dim, cell.0.abs());
                self.max_dim = max(self.max_dim, cell.1.abs());
            }

            iterations += 1;

            if iterations % 10 == 0 {
                eprint!(".");
            }

            if iterations % 500 == 0 {
                let duration = match start.elapsed() {
                    Ok(elapsed) => format!("{}", elapsed.as_secs()),
                    Err(_) => String::from("Error in getting run-time"),
                };

                println!(
                    "\n* current run duration: {}s\n* {} iterations\n* max height: {} ({})\n* {} cells created",
                    duration,
                    iterations,
                    cell_max,
                    self.max_per_cell,
                    self.grid.len(),
                );
            }
        }

        iterations
    }

    pub fn render(&self, iterations: usize) -> RenderedGrid {
        let offset = self.max_dim;
        let grid_size = (offset * 2 + 1) as u32;

        let mut grid: Vec<Vec<u8>> = vec![vec![0; grid_size as usize]; grid_size as usize];
        for (&(row, col), &sand) in self.grid.iter() {
            let x = row + offset;
            let y = col + offset;
            grid[x as usize][y as usize] = sand as u8;
        }

        RenderedGrid {
            iterations,
            grid_size,
            grid,
        }
    }

    pub fn write_result(&self, res: RenderedGrid) -> anyhow::Result<()> {
        let dir_name = format!("json/{}", self.pattern);
        if !Path::new(dir_name.as_str()).exists() {
            fs::create_dir(dir_name)?;
        }

        let mut file = File::create(format!(
            "json/{}/2_{}_{}.json",
            self.pattern, self.sand_power, self.pattern
        ))?;

        write!(file, "{}", serde_json::to_string(&res).unwrap())?;

        Ok(())
    }
}
