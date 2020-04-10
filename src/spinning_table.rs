use std::num::NonZeroUsize;

use structopt::StructOpt;

/// Produces the solution to the spinning table puzzle.
///
/// ## Problem statement
///
/// Exactly one of your seven investors sits in the correct place,
/// and the other six arrange themselves randomly.
/// Find an arrangement the other six investors could make
/// such that there is no rotation
/// that puts at least two of the investors in the correct seat.
pub fn spinning_table(parameters: Parameters) -> Vec<Vec<usize>> {
    // Let the first investor (1)
    // sit in the correct place (1, or index 0 in the vector).
    // Try all permutations of the remaining seats.

    let mut solutions = vec![];

    // Build a vector with investor numbers 2 up to and including n,
    // where n is the number of seats.
    // This is the vector in which permutations will occur.
    let mut seats_after_first: Vec<_> = (2..=parameters.number_of_seats.get()).collect();

    // Allocate another vector that will be used
    // in the closure to store all investor numbers, including 1.
    // (We could allocate one on each call of the closure,
    // but that would be more costly.)
    let mut seats = vec![0; parameters.number_of_seats.get()];

    permutohedron::heap_recursive(&mut seats_after_first, |seats_after_first| {
        // Initialize the full sequence of seats.
        // The first seat is always occupied by investor number 1.
        // (`is_valid_solution` rotates `seats` in place
        // and may leave investor number 1 in another seat.)
        seats[0] = 1;
        seats[1..].copy_from_slice(seats_after_first);

        if is_valid_solution(&mut seats) {
            solutions.push(seats.clone());

            if parameters.include_redundant_solutions {
                // The redundant solutions
                // are simply the distinct rotations of the initial solution.
                for _ in 1..seats.len() {
                    seats.rotate_right(1);
                    solutions.push(seats.clone());
                }
            }
        }
    });

    solutions
}

/// Determines whether the given arrangement of investors is a valid solution.
///
/// An arrangement is valid if,
/// for all rotations of the arrangement,
/// there are not two or more investors in the correct seat.
fn is_valid_solution(seats: &mut [usize]) -> bool {
    for _ in 0..seats.len() {
        if number_of_correctly_seated_investors(seats) >= 2 {
            return false;
        }

        // Rotate the seats by 1 for the next iteration.
        // It is important to rotate on the last iteration as well,
        // because if this is a valid solution,
        // we should return the seats to their initial arrangement.
        seats.rotate_right(1);
    }

    true
}

/// Returns the number of correctly seated investors
/// in the given arrangement of investors.
fn number_of_correctly_seated_investors(seats: &[usize]) -> usize {
    // Generate the sequence of seat numbers
    (1usize..)
        // Zip it with the sequence of investor numbers
        .zip(seats.iter().cloned())
        // Select only the items where the two numbers are equal,
        // which indicates a correctly seated investor
        .filter(|(seat_number, investor_number)| seat_number == investor_number)
        // Count them
        .count()
}

/// Parameters for solving variants of the spinning table puzzle.
#[derive(StructOpt)]
pub struct Parameters {
    /// The number of seats at the table.
    #[structopt(short = "n", long, default_value = "7")]
    number_of_seats: NonZeroUsize,

    /// If set, redundant solutions are included in the result.
    #[structopt(long)]
    include_redundant_solutions: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_redundant_solutions() {
        let result = super::spinning_table(Parameters {
            number_of_seats: NonZeroUsize::new(7).unwrap(),
            include_redundant_solutions: false,
        });

        assert_eq!(
            result,
            [
                vec![1, 4, 7, 5, 3, 2, 6],
                vec![1, 5, 4, 2, 7, 3, 6],
                vec![1, 3, 5, 7, 2, 4, 6],
                vec![1, 6, 5, 2, 4, 7, 3],
                vec![1, 6, 2, 5, 7, 4, 3],
                vec![1, 4, 7, 2, 6, 5, 3],
                vec![1, 6, 4, 2, 7, 5, 3],
                vec![1, 7, 4, 6, 2, 5, 3],
                vec![1, 7, 5, 3, 6, 2, 4],
                vec![1, 3, 6, 2, 7, 5, 4],
                vec![1, 5, 2, 6, 3, 7, 4],
                vec![1, 4, 6, 3, 2, 7, 5],
                vec![1, 4, 2, 7, 6, 3, 5],
                vec![1, 4, 7, 3, 6, 2, 5],
                vec![1, 6, 4, 3, 7, 2, 5],
                vec![1, 3, 7, 6, 4, 2, 5],
                vec![1, 6, 4, 7, 3, 5, 2],
                vec![1, 7, 6, 5, 4, 3, 2],
                vec![1, 5, 7, 3, 6, 4, 2],
            ],
        );
    }

    #[test]
    fn with_redundant_solutions() {
        let result = super::spinning_table(Parameters {
            number_of_seats: NonZeroUsize::new(7).unwrap(),
            include_redundant_solutions: true,
        });

        assert_eq!(result.len(), 133);
    }

    #[test]
    fn with_even_number_of_seats() {
        let result = super::spinning_table(Parameters {
            number_of_seats: NonZeroUsize::new(8).unwrap(),
            include_redundant_solutions: false,
        });

        // There are no solutions for an even number of seats.
        assert!(result.is_empty());
    }
}
