use crate::Cell;
use fnv::FnvHashMap;
use rayon::{
    iter::{once, Either},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
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

pub struct Grid {
    pub grid: FnvHashMap<Cell, u32>,
    pub sand_power: u32,
    pub max_per_cell: u32,
    pub topple_cells: Vec<Cell>,
    pub max_dim: i16,
    pub pattern: String,
}

impl Grid {
    pub fn new(sand_power: u32, pattern: String, topple_cells: Vec<Cell>) -> Grid {
        let grid = FnvHashMap::default();
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
        let mut grid = FnvHashMap::default();
        grid.insert((0, 0), starting_sand);

        let mut cell_max = starting_sand + 1;
        let mut iterations = 0;
        let start = SystemTime::now();

        while cell_max >= self.max_per_cell {
            let mut new_sand: FnvHashMap<(i16, i16), u32> = grid
                .par_iter_mut()
                .flat_map(|(&(row, col), sand)| {
                    if *sand < self.max_per_cell {
                        Either::Left(once(((row, col), 0)))
                    } else {
                        let per_cell = *sand / self.max_per_cell;
                        *sand %= self.max_per_cell;

                        Either::Right(
                            self.topple_cells
                                .par_iter()
                                .map(move |&(dx, dy)| ((row + dx, col + dy), per_cell))
                                .chain(once(((row, col), 0))),
                        )
                    }
                })
                .fold(FnvHashMap::default, |mut m, (cell, sand)| {
                    m.entry(cell).and_modify(|s| *s += sand).or_insert(sand);
                    m
                })
                .reduce(FnvHashMap::default, |mut m, child| {
                    child.into_iter().for_each(|(cell, sand)| {
                        m.entry(cell).and_modify(|s| *s += sand).or_insert(sand);
                    });

                    m
                });

            cell_max = new_sand
                .par_iter_mut()
                .map(|(cell, sand)| {
                    let total = grid.get(cell).unwrap_or(&0);
                    *sand += *total;

                    *sand
                })
                .max()
                .unwrap();

            grid = new_sand;
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
                    grid.len(),
                );
            }
        }

        self.grid = grid;
        self.max_dim = self
            .grid
            .keys()
            .map(|(x, y)| max(x.abs(), y.abs()))
            .max()
            .unwrap();

        let dim = self.max_dim * 2 + 1;
        let duration = match start.elapsed() {
            Ok(elapsed) => format!("{}", elapsed.as_secs()),
            Err(_) => String::from("Error in getting run-time"),
        };
        println!("\nToppling took {iterations} iterations.");
        println!("The final grid size is {dim}x{dim}.");
        println!("Final run duration: {duration}s");

        iterations
    }

    pub fn render(&self, iterations: usize) -> RenderedGrid {
        let offset = self.max_dim;
        let grid_size = (offset * 2 + 1) as u32;

        let mut grid: Vec<Vec<u8>> = vec![vec![0; grid_size as usize]; grid_size as usize];
        for (&(row, col), &sand) in self.grid.iter() {
            let x = row + offset;
            let y = col + offset;
            grid[y as usize][x as usize] = sand as u8;
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
