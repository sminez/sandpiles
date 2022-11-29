use crate::{patterns::patterns, Cell};
use anyhow::anyhow;
use fnv::FnvHashMap;
use plotters::prelude::*;
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
    mem::take,
    path::Path,
    time::SystemTime,
};

const DATA_DIR: &str = "data";

#[derive(Serialize, Deserialize)]
pub struct RenderedGrid {
    pub pattern: String,
    pub power: u32,
    pub grid: Vec<Vec<u8>>,
}

impl RenderedGrid {
    pub fn write_single_pattern(&self) -> anyhow::Result<()> {
        self.write(&format!("{}-{}", self.pattern, self.power))
    }

    pub fn write(&self, name: &str) -> anyhow::Result<()> {
        if !Path::new(DATA_DIR).exists() {
            fs::create_dir(DATA_DIR)?;
        }

        let bytes = bincode::serialize(&self)?;
        let mut file = File::create(format!("{DATA_DIR}/{name}.dat"))?;
        file.write_all(&bytes)?;

        Ok(())
    }

    pub fn read(path: &str) -> anyhow::Result<Self> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(bincode::deserialize(&bytes)?)
    }

    pub fn render_png(&self) -> anyhow::Result<()> {
        let desired = 700;
        let grid_size = self.grid.len();
        // Pad so that our pixel dimensions are a multiple of the grid size
        let dim = desired + grid_size - (desired % grid_size);
        // println!("{dim}x{dim}");

        let root_drawing_area =
            BitMapBackend::new("example.png", (dim as u32, dim as u32)).into_drawing_area();
        let grid_size = grid_size as usize;
        let child_drawing_areas = root_drawing_area.split_evenly((grid_size, grid_size));
        let max_sand = *self.grid.iter().flatten().max().unwrap() as f64;

        // See https://docs.rs/colorgrad/latest/colorgrad/index.html#functions
        // for more palette options
        // let palette = colorgrad::yl_gn_bu();
        // let palette = colorgrad::viridis();
        // let palette = colorgrad::sinebow();
        // let palette = colorgrad::rainbow();
        let palette = colorgrad::rd_yl_bu();

        for (index, area) in child_drawing_areas.into_iter().enumerate() {
            let col = index % grid_size;
            let row = (index - col) / grid_size;
            let sand = self.grid[row][col] as f64;
            let raw = palette.at(sand / max_sand).to_rgba8();

            area.fill(&RGBColor(raw[0], raw[1], raw[2]))?;
        }

        root_drawing_area.present()?;

        Ok(())
    }

    fn from_raw(inner: &FnvHashMap<Cell, u32>, power: u32, max_dim: i16, pattern: String) -> Self {
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
        RenderedGrid::from_raw(&inner, power, max_dim, pattern)
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
        let mut cell_max = self.max_per_cell + 1;
        let mut iterations = 0;
        let mut grid = take(&mut self.inner);
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
    type Error = anyhow::Error;

    fn try_from(
        RenderedGrid {
            pattern,
            power,
            grid: cells,
        }: RenderedGrid,
    ) -> Result<Self, Self::Error> {
        let topple_cells = patterns()
            .remove(&pattern.as_ref())
            .ok_or_else(|| anyhow!("unknown pattern: '{pattern}'"))?;

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
