//! A Rust implementation of the HashMap based algorithm for computing
//! sandpile fractals.
use anyhow::bail;
use clap::{Parser, Subcommand};
use sandpiles::{
    grid::{Grid, RenderedGrid},
    patterns::patterns,
};
use std::convert::TryFrom;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a new sandpile fractal using the given pattern and 2^power starting sand
    Run {
        /// Pattern to use
        pattern: String,
        /// Starting sand: 2^power
        power: u32,
        /// Render the resulting fractal as a png after computing
        #[clap(long, short, default_value = "true")]
        render: bool,
    },

    /// Render an existing data file
    Render {
        /// Path to the datafile to render
        path: String,
    },

    /// Double the sand of an existing sandpile and re-topple
    Double {
        /// Path to the datafile to render
        path: String,
        /// Render the resulting fractal as a png after computing
        #[clap(long, short, default_value = "true")]
        render: bool,
    },

    /// Double the sand of an existing sandpile and re-topple
    Combine {
        /// Path to the datafile to use as the seed
        path_1: String,
        /// Path to the datafile to layer on top
        path_2: String,
        /// Render the resulting fractal as a png after computing
        #[clap(long, short, default_value = "true")]
        render: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Run {
            pattern,
            power,
            render,
        } => run(pattern, power, render),

        Command::Render { path } => render(path),

        Command::Double { path, render } => double(path, render),

        Command::Combine {
            path_1,
            path_2,
            render,
        } => combine(path_1, path_2, render),
    }
}

fn run(pattern: String, power: u32, render: bool) -> anyhow::Result<()> {
    let topple_cells = match patterns().remove(pattern.as_str()) {
        Some(topple_cells) => topple_cells,
        None => {
            eprintln!("Invalid pattern: `{}`", pattern);
            bail!("Valid patterns are:\n{:?}", patterns().keys());
        }
    };

    println!("Starting sand: 2^{}", power);
    println!("Pattern:       {}", pattern);

    let mut grid = Grid::new(power, pattern, topple_cells);
    let starting_sand = 2_u32.pow(power);
    grid.inner.insert((0, 0), starting_sand);

    grid.topple();

    let r: RenderedGrid = grid.into();
    r.write_single_pattern()?;

    if render {
        r.render_png()?;
    }

    Ok(())
}

fn render(path: String) -> anyhow::Result<()> {
    let r = RenderedGrid::read(&path)?;
    r.render_png()
}

fn double(path: String, render: bool) -> anyhow::Result<()> {
    let r = RenderedGrid::read(&path)?;
    let mut grid = Grid::try_from(r)?;

    grid.inner.values_mut().for_each(|s| *s *= 2);
    grid.topple();

    let r: RenderedGrid = grid.into();
    r.write(&format!("{}-{}", r.pattern, r.power + 1))?;

    if render {
        r.render_png()?;
    }

    Ok(())
}

fn combine(path_1: String, path_2: String, render: bool) -> anyhow::Result<()> {
    let r = RenderedGrid::read(&path_1)?;
    let mut grid = Grid::try_from(r)?;

    let r_2 = RenderedGrid::read(&path_2)?;
    let Grid {
        inner,
        power: power_2,
        pattern: pattern_2,
        ..
    } = Grid::try_from(r_2)?;

    for (cell, sand) in inner.into_iter() {
        grid.inner
            .entry(cell)
            .and_modify(|s| *s += sand)
            .or_insert(sand);
    }

    grid.topple();
    let r: RenderedGrid = grid.into();
    r.write(&format!(
        "{}-{}_{}-{}",
        r.pattern, r.power, pattern_2, power_2
    ))?;

    if render {
        r.render_png()?;
    }

    Ok(())
}
