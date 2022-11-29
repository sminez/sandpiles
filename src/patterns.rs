use crate::Cell;
use std::collections::HashMap;

// Helper macro for making a map literal
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut _map = ::std::collections::HashMap::new();
            $(_map.insert($key, $value);)+
            _map
        }
     };
);

// Convert a human readable toppling pattern into a vector of cell offsets
macro_rules! pattern(
    [$($row:tt),+] => {
        {
            let mut vec = Vec::new();
            let mut _rix = 0;
            $({
                let offset = ($row.len() / 2) as i16;
                for (cix, cell) in $row.chars().enumerate() {
                    if cell != '.' {
                        // This will panic if non-numeric characters are given.
                        let count = cell.to_digit(10).unwrap();
                        for _ in 0..count {
                            vec.push((offset - _rix as i16, offset - cix as i16));
                        }
                    };
                };
                _rix += 1;
            })+
            vec
        }
    };
);

// Rather than bring in lazy static, I'm just building the pattern list when we
// start
pub fn patterns() -> HashMap<&'static str, Vec<Cell>> {
    map! {
        "X++" => pattern![
            "..1..",
            ".313.",
            "11.11",
            ".313.",
            "..1.."
        ],
        "ivy" => pattern![
            "121",
            "222",
            "121"
        ],
        "+" => pattern![
            ".1.",
            "1.1",
            ".1."
        ],
        "x" => pattern![
            "1.1",
            "...",
            "1.1"
        ],
        "o" => pattern![
            "111",
            "1.1",
            "111"
        ],
        "O" => pattern![
            "11111",
            "1...1",
            "1...1",
            "1...1",
            "11111"
        ],
        "xO" => pattern![
            "11111",
            "11.11",
            "1...1",
            "11.11",
            "11111"
        ],
        "o+" => pattern![
            "121",
            "2.2",
            "121"
        ],
        "oo" => pattern![
            "11211",
            "11111",
            "21.12",
            "11111",
            "11211"
        ],
        "ox" => pattern![
            "212",
            "1.1",
            "212"
        ],
        "++" => pattern![
            "..1..",
            "..1..",
            "11.11",
            "..1..",
            "..1.."
        ],
        "+++" => pattern![
            "..2..",
            "..1..",
            "21.12",
            "..1..",
            "..2.."
        ],
        "+_+" => pattern![
            "...1...",
            "...1...",
            ".......",
            "11...11",
            ".......",
            "...1...",
            "...1..."
        ],
        "o++" => pattern![
            "..1..",
            ".111.",
            "11.11",
            ".111.",
            "..1.."
        ],
        "o+++" => pattern![
            "...1...",
            "...1...",
            "..111..",
            "111.111",
            "..111..",
            "...1...",
            "...1..."
        ],
        "o_+" => pattern![
            "...1...",
            ".......",
            "..111..",
            "1.1.1.1",
            "..111..",
            ".......",
            "...1..."
        ],
        "o-+" => pattern![
            "..1..",
            ".121.",
            "12.21",
            ".121.",
            "..1.."
        ],
        "o-+x" => pattern![
            "..1..",
            ".222.",
            "12.21",
            ".222.",
            "..1.."
        ],
        "o=+" => pattern![
            "..2..",
            ".111.",
            "21.12",
            ".111.",
            "..2.."
        ],
        "+o" => pattern![
            "11211",
            "1.1.1",
            "21.12",
            "1.1.1",
            "11211"
        ],
        "xo" => pattern![
            "11211",
            "11.11",
            "2...2",
            "11.11",
            "11211"
        ],
        "+x" => pattern![
            "1...1",
            "..1..",
            ".1.1.",
            "..1..",
            "1...1"
        ],
        "x+" => pattern![
            "..1..",
            ".1.1.",
            "1...1",
            ".1.1.",
            "..1.."
        ],
        "::" => pattern![
            "11.11",
            ".1.1.",
            ".....",
            ".1.1.",
            "11.11"
        ],
        ";;" => pattern![
            ".1.1.",
            "11.11",
            ".....",
            "11.11",
            ".1.1."
        ],
        "Y" => pattern![
            ".111.",
            "1.1.1",
            "11.11",
            "1.1.1",
            ".111."
        ],
        "Y+" => pattern![
            ".121.",
            "1.1.1",
            "21.12",
            "1.1.1",
            ".121."
        ],
        "H" => pattern![
            ".1.1.",
            "11211",
            ".2.2.",
            "11211",
            ".1.1."
        ],
        "sh" => pattern![
            ".1.1.",
            "11111",
            ".1.1.",
            "11111",
            ".1.1."
        ]
    }
}
