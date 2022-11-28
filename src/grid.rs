use crate::{patterns::patterns, Cell};
use fnv::FnvHashMap;
use rayon::{
    iter::{once, Either},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    convert::TryFrom,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    time::SystemTime,
};

#[derive(Serialize, Deserialize)]
pub struct RenderedGrid {
    pub pattern: String,
    pub power: u32,
    pub grid: Vec<Vec<u8>>,
}

impl RenderedGrid {
    pub fn write(&self) -> anyhow::Result<()> {
        let dir_name = format!("data/{}", self.pattern);
        if !Path::new(dir_name.as_str()).exists() {
            fs::create_dir(dir_name)?;
        }

        let bytes = bincode::serialize(&self)?;

        let mut file = File::create(format!("data/{}-{}.dat", self.pattern, self.power))?;
        file.write_all(&bytes)?;

        Ok(())
    }

    pub fn read(path: &str) -> anyhow::Result<Self> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(bincode::deserialize(&bytes)?)
    }
}

impl From<Grid> for RenderedGrid {
    fn from(
        Grid {
            inner,
            power,
            max_dim,
            pattern,
            ..
        }: Grid,
    ) -> Self {
        let offset = max_dim;
        let grid_size = (offset * 2 + 1) as u32;

        let mut grid: Vec<Vec<u8>> = vec![vec![0; grid_size as usize]; grid_size as usize];
        for (&(row, col), &sand) in inner.iter() {
            let x = row + offset;
            let y = col + offset;
            grid[y as usize][x as usize] = sand as u8;
        }

        RenderedGrid {
            pattern,
            power,
            grid,
        }
    }
}

pub struct Grid {
    pub inner: FnvHashMap<Cell, u32>,
    pub power: u32,
    pub max_per_cell: u32,
    pub topple_cells: Vec<Cell>,
    pub max_dim: i16,
    pub pattern: String,
}

impl Grid {
    pub fn new(power: u32, pattern: String, topple_cells: Vec<Cell>) -> Grid {
        let max_per_cell = topple_cells.len() as u32;
        let max_dim = 1;

        Grid {
            inner: Default::default(),
            max_per_cell,
            power,
            topple_cells,
            max_dim,
            pattern,
        }
    }

    pub fn topple(&mut self) {
        let starting_sand = 2_u32.pow(self.power);
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

        self.inner = grid;
        self.max_dim = self
            .inner
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
    }
}

impl TryFrom<RenderedGrid> for Grid {
    type Error = String;

    fn try_from(
        RenderedGrid {
            pattern,
            power,
            grid: cells,
        }: RenderedGrid,
    ) -> Result<Self, Self::Error> {
        let topple_cells = patterns()
            .remove(&pattern.as_ref())
            .ok_or_else(|| format!("unknown pattern: '{pattern}'"))?;

        let mut grid = Self::new(power, pattern, topple_cells);
        let offset = ((cells.len() - 1) / 2) as i16;

        for (i, row) in cells.into_iter().enumerate() {
            for (j, sand) in row.into_iter().enumerate() {
                let cell = (i as i16 - offset, j as i16 - offset);
                grid.inner.insert(cell, sand as u32);
            }
        }

        Ok(grid)
    }
}
