use std::{
    cell::Cell,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    iter::{self, Peekable},
};

use itertools::Itertools;

use structopt::{clap::arg_enum, StructOpt};

/// Produces the solution to the Scrabble® puzzle.
///
/// ## Problem statement
///
/// How many ways, from the 100 standard Scrabble® tiles,
/// can you choose seven which total 46 points?
///
/// Clarification: for this we're asking
/// how many distinct scrabble hands (groups of 7 letters) there are
/// that total exactly 46.
/// So order does not matter
/// and identical letters are indistinguishable.
pub fn scrabble(parameters: Parameters) -> Output {
    match parameters.output {
        OutputFormat::Count => Output::Count(scrabble1::<SolutionCount>(parameters)),
        OutputFormat::List => Output::List(scrabble1::<SolutionList>(parameters)),
    }
}

/// Produces the solution to the Scrabble® puzzle in the specified output format.
fn scrabble1<S>(parameters: Parameters) -> S
where
    S: SolutionAccumulator,
{
    let mut solution_accumulator = S::new();

    // Group tile definitions by their value.
    let tiles_by_value = STANDARD_ENGLISH_SCRABBLE_TILES
        .iter()
        // Accumulate into a BTreeMap so that the order is consistent between runs.
        .fold(BTreeMap::new(), |mut map, counted_tile| {
            let tile_value = map
                .entry(counted_tile.tile.value)
                .or_insert_with(TilesForValue::default);
            tile_value.live_counted_tiles.push(LiveCountedTile {
                counted_tile,
                occurrences_drawn: Default::default(),
            });
            tile_value.number_of_tiles += counted_tile.occurrences;
            map
        });

    draw_abstract(
        &parameters,
        &tiles_by_value,
        &mut solution_accumulator,
        // When we do a recursive call,
        // we must start at the same tile value,
        // not from the start,
        // otherwise we would find duplicate solutions.
        tiles_by_value.values().peekable(),
        0,
    );

    solution_accumulator.finish();

    solution_accumulator
}

/// For each distinct tile value,
/// draw one tile with that value (without specifying which letter),
/// then recursively draw more tiles
/// until the hand size is reached.
/// If the hand is full, add the solutions to `solution_accumulator`.
fn draw_abstract<'a, S>(
    parameters: &Parameters,
    tiles_by_value: &BTreeMap<u32, TilesForValue<'_>>,
    solution_accumulator: &mut S,
    mut tiles_for_value_iter: Peekable<impl Iterator<Item = &'a TilesForValue<'a>> + Clone>,
    tiles_drawn_so_far: u32,
) where
    S: SolutionAccumulator,
{
    // Have we drawn enough tiles yet?
    if tiles_drawn_so_far == parameters.hand_size {
        // Does the cumulative value of the tiles we drew match the target score?
        let hand_score: u32 = tiles_by_value
            .iter()
            .map(|(&tile_value, tiles)| tile_value * tiles.number_of_abstract_tiles_drawn.get())
            .sum();
        if hand_score == parameters.target_score {
            // Enumerate the possible hands
            // for the combination of tile values that was drawn.

            let concrete_tile_combinations_by_tile_value = tiles_by_value
                // For each tile value
                .values()
                // for which we drew at least one tile (for performance),
                .filter(|x| x.number_of_abstract_tiles_drawn.get() > 0)
                // enumerate all unique groups of letters
                // of the size matching the number of tiles drawn
                .map(|tiles_for_value| {
                    let mut concrete_tile_combinations = S::new();
                    draw_concrete(
                        tiles_for_value,
                        &mut concrete_tile_combinations,
                        // When we do a recursive call,
                        // we must start at the same letter,
                        // not from the start,
                        // otherwise we would find duplicate solutions.
                        tiles_for_value.live_counted_tiles.iter().peekable(),
                        0,
                    );
                    concrete_tile_combinations
                })
                .collect();

            // Perform the cartesian product
            // of the possible groups of letters by tile value,
            // and add the result to the solutions.
            solution_accumulator.add_solutions(S::cartesian_product(
                concrete_tile_combinations_by_tile_value,
            ));
        }
    } else {
        // We need to clone the iterator here,
        // because `peek` borrows the iterator
        // and the borrow is still active
        // at the point where we need the cloned iterator.
        let mut tiles_for_value_iter_clone = tiles_for_value_iter.clone();

        while let Some(tiles_for_value) = tiles_for_value_iter.peek() {
            // If there are any tiles of that value left:
            if tiles_for_value.number_of_abstract_tiles_drawn.get()
                < tiles_for_value.number_of_tiles
            {
                // Draw any tile of that value.
                tiles_for_value
                    .number_of_abstract_tiles_drawn
                    .set(tiles_for_value.number_of_abstract_tiles_drawn.get() + 1);

                draw_abstract(
                    parameters,
                    tiles_by_value,
                    solution_accumulator,
                    tiles_for_value_iter_clone,
                    tiles_drawn_so_far + 1,
                );

                // Put the tile back in the bag for the next draw.
                tiles_for_value
                    .number_of_abstract_tiles_drawn
                    .set(tiles_for_value.number_of_abstract_tiles_drawn.get() - 1);
            }

            tiles_for_value_iter.next();
            tiles_for_value_iter_clone = tiles_for_value_iter.clone();
        }
    }
}

/// For a given tile value,
/// for each distinct letter,
/// draw one tile of that letter,
/// then recursively draw more tiles
/// until we've reached the number of tiles
/// that were drawn for the tile value.
/// If we've reached the target number of tiles,
/// add the partial solutions to `concrete_tile_combinations`.
fn draw_concrete<'a, S>(
    tiles_for_value: &TilesForValue<'_>,
    concrete_tile_combinations: &mut S,
    mut live_counted_tiles_iter: Peekable<impl Iterator<Item = &'a LiveCountedTile<'a>> + Clone>,
    tiles_drawn_so_far: u32,
) where
    S: SolutionAccumulator,
{
    if tiles_drawn_so_far == tiles_for_value.number_of_abstract_tiles_drawn.get() {
        concrete_tile_combinations.add_solution(|| {
            tiles_for_value
                .live_counted_tiles
                .iter()
                .flat_map(|live_counted_tile| {
                    iter::repeat(live_counted_tile.counted_tile.tile.letter)
                        .take(live_counted_tile.occurrences_drawn.get() as usize)
                })
                .collect()
        });
    } else {
        // We need to clone the iterator here,
        // because `peek` borrows the iterator
        // and the borrow is still active
        // at the point where we need the cloned iterator.
        let mut live_counted_tiles_iter_clone = live_counted_tiles_iter.clone();

        while let Some(live_counted_tile) = live_counted_tiles_iter.peek() {
            if live_counted_tile.occurrences_drawn.get()
                < live_counted_tile.counted_tile.occurrences
            {
                // Draw a tile of that letter.
                live_counted_tile
                    .occurrences_drawn
                    .set(live_counted_tile.occurrences_drawn.get() + 1);

                draw_concrete(
                    tiles_for_value,
                    concrete_tile_combinations,
                    live_counted_tiles_iter_clone,
                    tiles_drawn_so_far + 1,
                );

                // Put the tile back in the bag for the next draw.
                live_counted_tile
                    .occurrences_drawn
                    .set(live_counted_tile.occurrences_drawn.get() - 1);
            }

            live_counted_tiles_iter.next();
            live_counted_tiles_iter_clone = live_counted_tiles_iter.clone();
        }
    }
}

/// Parameters for solving variants of the Scrabble puzzle.
#[derive(Debug, StructOpt)]
pub struct Parameters {
    /// The number of tiles in a hand.
    #[structopt(short = "h", long, default_value = "7")]
    hand_size: u32,

    /// The target score for a hand.
    #[structopt(short = "s", long, default_value = "46")]
    target_score: u32,

    /// How the solution will be presented.
    #[structopt(long, possible_values = &OutputFormat::variants(), case_insensitive = true, default_value = "count")]
    output: OutputFormat,
}

arg_enum! {
    /// Choices for how the solution should be presented.
    #[derive(Debug)]
    pub enum OutputFormat {
        Count,
        List,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Output {
    /// The number of valid hands.
    Count(SolutionCount),

    /// The full list of valid hands.
    List(SolutionList),
}

impl Display for Output {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count(count) => {
                write!(fmt, "{}", count)?;
            }

            Self::List(list) => {
                let mut iter = list.iter();
                if let Some(item) = iter.next() {
                    write!(fmt, "{}", item)?;
                    for item in iter {
                        writeln!(fmt)?;
                        write!(fmt, "{}", item)?;
                    }
                }
            }
        }

        Ok(())
    }
}

macro_rules! tiles {
    ($($letter:tt x $occurrences:tt, value $value:tt)*) => {
        [
            $(
                CountedTile {
                    tile: Tile {
                        letter: $letter,
                        value: $value,
                    },
                    occurrences: $occurrences,
                },
            )*
        ]
    };
}

/// The distribution of tiles in a standard English edition of Scrabble®.
static STANDARD_ENGLISH_SCRABBLE_TILES: &[CountedTile] = &tiles![
    ' ' x  2, value 0
    'A' x  9, value 1
    'B' x  2, value 3
    'C' x  2, value 3
    'D' x  4, value 2
    'E' x 12, value 1
    'F' x  2, value 4
    'G' x  3, value 2
    'H' x  2, value 4
    'I' x  9, value 1
    'J' x  1, value 8
    'K' x  1, value 5
    'L' x  4, value 1
    'M' x  2, value 3
    'N' x  6, value 1
    'O' x  8, value 1
    'P' x  2, value 3
    'Q' x  1, value 10
    'R' x  6, value 1
    'S' x  4, value 1
    'T' x  6, value 1
    'U' x  4, value 1
    'V' x  2, value 4
    'W' x  2, value 4
    'X' x  1, value 8
    'Y' x  2, value 4
    'Z' x  1, value 10
];

/// A tile from the Scrabble board game.
#[derive(Debug)]
struct Tile {
    /// The letter on the tile (or a space for blank tiles).
    letter: char,

    /// The point value of the tile
    /// in the English edition of Scrabble.
    value: u32,
}

/// A tile along with the number of copies of that tile
/// in the English edition of Scrabble.
#[derive(Debug)]
struct CountedTile {
    /// The attributes of a tile.
    tile: Tile,

    /// The number of occurrences of that tile.
    occurrences: u32,
}

/// Aggregates all the tiles for a particular tile value.
#[derive(Debug, Default)]
struct TilesForValue<'a> {
    /// The list of `LiveCountedTile`s with the tile value.
    live_counted_tiles: Vec<LiveCountedTile<'a>>,

    /// The total number of tiles with the tile value.
    number_of_tiles: u32,

    /// The number of tiles (no matter the letter) that have been drawn
    /// at the current point in the algorithm.
    number_of_abstract_tiles_drawn: Cell<u32>,
}

/// A `CountedTile` paired with how many of them have currently been drawn.
#[derive(Debug)]
struct LiveCountedTile<'a> {
    /// A reference to a `CountedTile`.
    counted_tile: &'a CountedTile,

    /// The number of tiles that have been drawn
    /// at the current point in the algorithm.
    occurrences_drawn: Cell<u32>,
}

/// Implemented for types
/// that can accumulate solutions as they are found
/// and that represent possible output formats for the solution.
trait SolutionAccumulator: Sized {
    /// Initializes a new accumulator.
    fn new() -> Self;

    /// Adds a solution to this accumulator.
    ///
    /// The solution is generated lazily
    /// because some accumulators don't care about the particular value.
    fn add_solution(&mut self, solution_fn: impl Fn() -> String);

    /// Adds all the solutions from another accumulator to this accumulator.
    fn add_solutions(&mut self, other: Self);

    /// Performs the cartesian product
    /// of partial solutions by tile value
    /// to generate final solutions.
    fn cartesian_product(solutions_by_value: Vec<Self>) -> Self;

    /// Transforms the accumulated solutions for presentation.
    fn finish(&mut self) {}
}

/// A `SolutionAccumulator` that simply counts the number of solutions.
///
/// This is more efficient than actually listing the solutions
/// because the `cartesian_product` can be implemented
/// as simply the product of the solution counts.
type SolutionCount = u64;

impl SolutionAccumulator for SolutionCount {
    fn new() -> Self {
        0
    }

    fn add_solution(&mut self, _: impl FnOnce() -> String) {
        *self += 1;
    }

    fn add_solutions(&mut self, other: Self) {
        *self += other;
    }

    fn cartesian_product(solutions_by_value: Vec<Self>) -> Self {
        // We only need to multiply the counts together.
        solutions_by_value.iter().product()
    }
}

/// A `SolutionAccumulator` that list all hands that match the target score.
type SolutionList = Vec<String>;

impl SolutionAccumulator for SolutionList {
    fn new() -> Self {
        vec![]
    }

    fn add_solution(&mut self, solution_fn: impl FnOnce() -> String) {
        self.push(solution_fn());
    }

    fn add_solutions(&mut self, other: Self) {
        self.extend(other);
    }

    fn cartesian_product(solutions_by_value: Vec<Self>) -> Self {
        solutions_by_value
            .into_iter()
            .multi_cartesian_product()
            .map(|v| v.join(""))
            .collect()
    }

    fn finish(&mut self) {
        // Sort the results for easier eyeballing.
        self.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ($($x:expr),*) => {
            vec![
                $(String::from($x),)*
            ]
        };

        ($($x:expr,)*) => {
            vec_of_strings![$($x),*]
        }
    }

    #[test]
    fn solution_count() {
        let result = super::scrabble(Parameters {
            hand_size: 7,
            target_score: 46,
            output: OutputFormat::Count,
        });

        assert_eq!(result, Output::Count(138));
    }

    #[test]
    fn solution_list() {
        let result = super::scrabble(Parameters {
            hand_size: 7,
            target_score: 46,
            output: OutputFormat::List,
        });

        assert_eq!(
            result,
            Output::List(vec_of_strings![
                "AFKJXQZ", "AHKJXQZ", "AVKJXQZ", "AWKJXQZ", "AYKJXQZ", "BBFJXQZ", "BBHJXQZ",
                "BBVJXQZ", "BBWJXQZ", "BBYJXQZ", "BCFJXQZ", "BCHJXQZ", "BCVJXQZ", "BCWJXQZ",
                "BCYJXQZ", "BMFJXQZ", "BMHJXQZ", "BMVJXQZ", "BMWJXQZ", "BMYJXQZ", "BPFJXQZ",
                "BPHJXQZ", "BPVJXQZ", "BPWJXQZ", "BPYJXQZ", "CCFJXQZ", "CCHJXQZ", "CCVJXQZ",
                "CCWJXQZ", "CCYJXQZ", "CMFJXQZ", "CMHJXQZ", "CMVJXQZ", "CMWJXQZ", "CMYJXQZ",
                "CPFJXQZ", "CPHJXQZ", "CPVJXQZ", "CPWJXQZ", "CPYJXQZ", "DBKJXQZ", "DCKJXQZ",
                "DFFJXQZ", "DFHJXQZ", "DFVJXQZ", "DFWJXQZ", "DFYJXQZ", "DHHJXQZ", "DHVJXQZ",
                "DHWJXQZ", "DHYJXQZ", "DMKJXQZ", "DPKJXQZ", "DVVJXQZ", "DVWJXQZ", "DVYJXQZ",
                "DWWJXQZ", "DWYJXQZ", "DYYJXQZ", "EFKJXQZ", "EHKJXQZ", "EVKJXQZ", "EWKJXQZ",
                "EYKJXQZ", "GBKJXQZ", "GCKJXQZ", "GFFJXQZ", "GFHJXQZ", "GFVJXQZ", "GFWJXQZ",
                "GFYJXQZ", "GHHJXQZ", "GHVJXQZ", "GHWJXQZ", "GHYJXQZ", "GMKJXQZ", "GPKJXQZ",
                "GVVJXQZ", "GVWJXQZ", "GVYJXQZ", "GWWJXQZ", "GWYJXQZ", "GYYJXQZ", "IFKJXQZ",
                "IHKJXQZ", "IVKJXQZ", "IWKJXQZ", "IYKJXQZ", "LFKJXQZ", "LHKJXQZ", "LVKJXQZ",
                "LWKJXQZ", "LYKJXQZ", "MMFJXQZ", "MMHJXQZ", "MMVJXQZ", "MMWJXQZ", "MMYJXQZ",
                "MPFJXQZ", "MPHJXQZ", "MPVJXQZ", "MPWJXQZ", "MPYJXQZ", "NFKJXQZ", "NHKJXQZ",
                "NVKJXQZ", "NWKJXQZ", "NYKJXQZ", "OFKJXQZ", "OHKJXQZ", "OVKJXQZ", "OWKJXQZ",
                "OYKJXQZ", "PPFJXQZ", "PPHJXQZ", "PPVJXQZ", "PPWJXQZ", "PPYJXQZ", "RFKJXQZ",
                "RHKJXQZ", "RVKJXQZ", "RWKJXQZ", "RYKJXQZ", "SFKJXQZ", "SHKJXQZ", "SVKJXQZ",
                "SWKJXQZ", "SYKJXQZ", "TFKJXQZ", "THKJXQZ", "TVKJXQZ", "TWKJXQZ", "TYKJXQZ",
                "UFKJXQZ", "UHKJXQZ", "UVKJXQZ", "UWKJXQZ", "UYKJXQZ",
            ])
        );
    }
}
