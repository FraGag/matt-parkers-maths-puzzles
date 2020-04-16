#![warn(clippy::all)]

use structopt::StructOpt;

/// [Puzzle 1 - Spinning table](http://www.think-maths.co.uk/table-puzzle)
mod spinning_table;

/// [Puzzle 3 - Scrabble®](http://www.think-maths.co.uk/scrabble-puzzle)
mod scrabble;

/// [Puzzle 4 - Card](http://www.think-maths.co.uk/card-puzzle)
mod card;

/// Entry point.
fn main() {
    match Puzzle::from_args() {
        Puzzle::SpinningTable(parameters) => {
            for solution in spinning_table::spinning_table(parameters) {
                println!("{:?}", solution);
            }
        }

        Puzzle::Scrabble(parameters) => {
            println!("{}", scrabble::scrabble(parameters));
        }

        Puzzle::Card(parameters) => {
            println!("{:?}", card::card(parameters));
        }
    }
}

/// Command-line arguments.
#[derive(StructOpt)]
#[structopt(about("Solutions to Matt Parker's Math Puzzles in Rust"))]
enum Puzzle {
    /// Puzzle 1 - Spinning table <http://www.think-maths.co.uk/table-puzzle>
    SpinningTable(spinning_table::Parameters),

    /// Puzzle 3 - Scrabble® <http://www.think-maths.co.uk/scrabble-puzzle>
    Scrabble(scrabble::Parameters),

    /// Puzzle 4 - Card <http://www.think-maths.co.uk/card-puzzle>
    Card(card::Parameters),
}
