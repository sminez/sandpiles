//! A Rust implementation of the HashMap based algorithm for computing
//! sandpile fractals.
use anyhow::bail;
use plotters::prelude::*;
use sandpiles::{
    grid::{Grid, RenderedGrid},
    patterns::patterns,
};
use std::env;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let sand_power = args[1].parse::<u32>()?;
    let pattern = args[2].clone();

    match patterns().get(pattern.as_str()) {
        Some(topple_cells) => {
            println!("Starting sand: 2^{}", sand_power);
            println!("Pattern:       {}", pattern);

            let mut grid = Grid::new(sand_power, pattern, topple_cells.clone());
            let starting_sand = 2_u32.pow(sand_power);
            grid.inner.insert((0, 0), starting_sand);

            grid.topple();

            let r: RenderedGrid = grid.into();
            render_png(r)?;

            // grid.write_result(res)?;
        }

        None => {
            eprintln!("Invalid pattern: `{}`", pattern);
            bail!("Valid patterns are:\n{:?}", patterns().keys());
        }
    }

    Ok(())
}

fn render_png(RenderedGrid { grid, .. }: RenderedGrid) -> anyhow::Result<()> {
    let desired = 700;
    let grid_size = grid.len();
    // Pad so that our pixel dimensions are a multiple of the grid size
    let dim = desired + grid_size - (desired % grid_size);
    println!("{dim}x{dim}");

    let root_drawing_area =
        BitMapBackend::new("example.png", (dim as u32, dim as u32)).into_drawing_area();
    let grid_size = grid_size as usize;
    let child_drawing_areas = root_drawing_area.split_evenly((grid_size, grid_size));
    let max_sand = *grid.iter().flatten().max().unwrap() as f64;

    // See https://docs.rs/colorgrad/latest/colorgrad/index.html#functions
    // for more palette options
    let palette = colorgrad::rd_yl_bu();

    for (index, area) in child_drawing_areas.into_iter().enumerate() {
        let col = index % grid_size;
        let row = (index - col) / grid_size;
        let sand = grid[row][col] as f64;
        let raw = palette.at(sand / max_sand).to_rgba8();

        area.fill(&RGBColor(raw[0], raw[1], raw[2]))?;
    }

    root_drawing_area.present()?;

    Ok(())
}
