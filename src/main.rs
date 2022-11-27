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
            let iterations = grid.topple();
            let dim = grid.max_dim * 2 + 1;

            println!("\nToppling took {iterations} iterations.");
            println!("The final grid size is {dim}x{dim}.");

            let r = grid.render(iterations);
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

fn render_png(r: RenderedGrid) -> anyhow::Result<()> {
    let root_drawing_area = BitMapBackend::new("example.png", (1000, 1000)).into_drawing_area();
    let d = r.grid_size as usize;
    let max_sand = *r.grid.iter().flatten().max().unwrap() as f64;
    let child_drawing_areas = root_drawing_area.split_evenly((d, d));

    // See https://docs.rs/colorgrad/latest/colorgrad/index.html#functions
    // for more palette options
    let palette = colorgrad::rd_yl_bu();

    for (index, area) in child_drawing_areas.into_iter().enumerate() {
        let col = index % d;
        let row = (index - col) / d;
        let sand = r.grid[row][col] as f64;
        let raw = palette.at(sand / max_sand).to_rgba8();

        area.fill(&RGBColor(raw[0], raw[1], raw[2]))?;
    }

    root_drawing_area.present()?;

    Ok(())
}
