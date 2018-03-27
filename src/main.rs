//! A Rust implementation of the HashMap based algorithm for computing
//! sandpile fractals.
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::time::SystemTime;

// Helper macro for making map literals
//
// >>> let my_map = map!{ "this" => "that", "foo" => "bar" };
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut _map = ::std::collections::HashMap::new();
            $(_map.insert($key, $value);)+
            _map
        }
     };
);

// Alias for our cell coordinates
type Cell = (i32, i32);

// A result struct for storing results as JSON using serde
#[derive(Serialize, Deserialize)]
struct SandResult {
    iterations: i64,
    topples:    i64,
    grid_size:  i32,
    grid:       Vec<Vec<i32>>,
}

// Rather than bring in lazy static, I'm just building the pattern list when we
// start
fn get_patterns() -> HashMap<&'static str, Vec<Cell>> {
    map!{
        "+" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0)
        ],
        "x" => vec![
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o+" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "oo" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (-1, -2), (-1, 2), (1, -2), (1, 2),
            (-2, -1), (-2, 1), (2, -1), (2, 1),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (2, 2), (2, -2), (-2, 2), (-2, -2)
        ],
        "ox" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "++" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0)
        ],
        "+++" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0)
        ],
        "+_+" => vec![
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 3), (0, -3), (3, 0), (-3, 0)
        ],
        "o++" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o+++" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 3), (0, -3), (3, 0), (-3, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o_+" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 3), (0, -3), (3, 0), (-3, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o-+" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o-+x" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "o=+" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "+o" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (-1, -2), (-1, 2), (1, -2), (1, 2),
            (-2, -1), (-2, 1), (2, -1), (2, 1),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (2, 2), (2, -2), (-2, 2), (-2, -2)
        ],
        "xo" => vec![
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (-1, -2), (-1, 2), (1, -2), (1, 2),
            (-2, -1), (-2, 1), (2, -1), (2, 1),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (2, 2), (2, -2), (-2, 2), (-2, -2)
        ],
        "+x" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (2, 2), (2, -2), (-2, 2), (-2, -2)
        ],
        "x+" => vec![
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (0, 2), (0, -2), (2, 0), (-2, 0)
        ],
        "::" => vec![
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 2), (1, -2), (2, 2), (2, -2),
            (-1, 2), (-1, -2), (-2, 2), (-2, -2)
        ],
        ";;" => vec![
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 2), (1, -2), (2, 1), (2, -1),
            (-1, 2), (-1, -2), (-2, 1), (-2, -1)
        ],
        "Y" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2)
        ],
        "A" => vec![
            (0, 2), (0, -2), (2, 0), (-2, 0),
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ],
        "H" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 2), (1, -2), (2, 2), (2, -2),
            (-1, 2), (-1, -2), (-2, 2), (-2, -2)
        ],
        "sh" => vec![
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 2), (1, -2), (2, 1), (2, -1),
            (-1, 2), (-1, -2), (-2, 1), (-2, -1)
        ]
    }
}

// Control structure for managing topples
struct Grid<'a> {
    grid:         HashMap<Cell, i32>,
    sand_power:   u32,
    max_per_cell: i32,
    topple_cells: &'a Vec<Cell>,
    max_dim:      i32,
    pattern:      String,
}

impl<'a> Grid<'a> {
    fn new(sand_power: u32, pattern: String, topple_cells: &'a Vec<Cell>) -> Grid<'a> {
        let grid = HashMap::new();
        let max_per_cell = topple_cells.len() as i32;
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

    // Topple the grid using the hashmap
    fn topple(&mut self) {
        // Set the starting sand.
        let base: i32 = 2;
        let starting_sand = base.pow(self.sand_power);
        self.grid.insert((0, 0), starting_sand);

        let mut cell_max = starting_sand + 1;
        let mut iterations = 0;
        let mut topples = 0;
        let start = SystemTime::now();

        // Topple until we reach steady state
        loop {
            // If nothing is going to topple then we are done.
            if cell_max < self.max_per_cell {
                let dim = self.max_dim * 2 + 1;
                eprintln!(
                    "\nToppling took {} iterations during which there were {} cell topples.",
                    iterations, topples
                );
                eprintln!("The final grid size is {}x{}.", dim, dim);
                self.render_grid(iterations, topples);
                return;
            }

            // Reset the max cell value and run the next iteration
            cell_max = 0;
            let mut new_sand = HashMap::new();

            // Find toppling cells and redistribute its excess sand to
            // temporary holding cells before then combining at the end
            // once all cells have toppled. This prevents the random key
            // iteration order from disrupting the pattern of topples.
            for (&(row, col), sand) in self.grid.iter_mut() {
                if *sand >= self.max_per_cell {
                    let per_cell = *sand / self.max_per_cell;
                    *sand %= self.max_per_cell;

                    for &(dx, dy) in self.topple_cells.iter() {
                        let loc = (row + dx, col + dy);

                        // Keep track of the dimensions of the grid
                        if loc.0 > self.max_dim {
                            self.max_dim = loc.0
                        }
                        if loc.1 > self.max_dim {
                            self.max_dim = loc.1
                        }

                        let new_sand = new_sand.entry(loc).or_insert(0);
                        *new_sand += per_cell;
                    }
                    topples += 1;
                }
            }

            // Now add in the toppled sand
            for (cell, sand) in new_sand.iter() {
                let c = cell.clone();
                let total = self.grid.entry(c).or_insert(0);
                *total += sand;
                if *total > cell_max {
                    cell_max = *total;
                }
            }
            iterations += 1;
            if iterations % 10 == 0 {
                eprint!(".");
            }
            if iterations % 500 == 0 {
                // Display some stats about the current run
                let duration = match start.elapsed() {
                    Ok(elapsed) => format!("{}", elapsed.as_secs()),
                    Err(_) => String::from("Error in getting run-time"),
                };
                eprintln!(
                    "\n* current run duration: {}s\n* {} iterations\n* {} topples\n* {} cells created",
                    duration,
                    iterations,
                    topples,
                    self.grid.len(),
                );
            }
        }
    }

    // Convert the internal grid to csv format
    fn render_grid(&self, iterations: i64, topples: i64) {
        // Map the max coordinates (centred on 0,0) to array size
        let offset = self.max_dim;
        let grid_size = offset * 2 + 1;

        let mut grid: Vec<Vec<i32>> = vec![vec![0; grid_size as usize]; grid_size as usize];
        for (&(row, col), sand) in self.grid.iter() {
            let x = row + offset;
            let y = col + offset;
            grid[x as usize][y as usize] = *sand;
        }

        // Write to a file if we can make one else dump to stdout
        let dir_name = format!("json/{}", self.pattern);
        if !Path::new(dir_name.as_str()).exists() {
            fs::create_dir(dir_name).unwrap();
        }

        match File::create(format!(
            "json/{}/2_{}_{}.json",
            self.pattern, self.sand_power, self.pattern
        )) {
            Err(_) => {
                eprintln!("Unable to create output file");
            }
            Ok(mut file) => {
                let res = SandResult {
                    iterations,
                    topples,
                    grid_size,
                    grid,
                };
                write!(file, "{}", serde_json::to_string(&res).unwrap()).unwrap();
            }
        }
    }
}

// Topple some sand!
fn main() {
    let args: Vec<String> = env::args().collect();
    // Will error if this can't be parsed as a string
    let sand_power = args[1].parse::<u32>().unwrap();
    let pattern = args[2].clone();

    let patterns = get_patterns();

    match patterns.get(pattern.as_str()) {
        Some(topple_cells) => {
            eprintln!("Starting sand: 2^{}", sand_power);
            eprintln!("Pattern:       {}", pattern);
            let mut grid = Grid::new(sand_power, pattern, topple_cells);
            grid.topple();
        }
        None => {
            eprintln!("Invalid pattern: `{}`", pattern);
            eprintln!("Valid patterns are:\n{:?}", get_patterns().keys());
            process::exit(1);
        }
    }
}
