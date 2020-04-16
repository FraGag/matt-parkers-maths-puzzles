use std::{
    convert::{TryFrom, TryInto},
    num::ParseIntError,
    str::FromStr,
};

use quick_error::quick_error;

use structopt::StructOpt;

/// Produces a solution to the card puzzle.
///
/// ## Problem statement
///
/// Submit an optimal set of flips
/// (a solution that uses the minimum number of flips)
/// that guarantees all four cards
/// will eventually be face down
/// given any starting position.
pub fn card(parameters: Parameters) -> Box<[u8]> {
    // The general idea is to generate a sequence of flips
    // that will visit every possible state of the set of cards.
    //
    // Expressed numerically,
    // we must visit all numbers from 0 to 2**n - 1 exactly once.
    // To do this, we start at 0,
    // then flip a bit such that we get a number we haven't seen before.
    //
    // Expressed geometrically,
    // consider an n-cube comprised of 2 smaller n-cubes in each dimension
    // (which gives 2**n smaller n-cubes).
    // We must visit each of the smaller n-cubes exactly once.
    // To do this, we start at an arbitrary position,
    // then move to one of the adjacent n-cubes
    // that we haven't visited yet.
    // This way, we gradually explore all n-cubes
    // on one particular m-dimensional plane (depending on where we started)
    // for all dimensions m from 1 to n.

    // The optimal number of flips for n cards is 2**n - 1.
    let capacity = parameters.number_of_cards.number_of_card_states - 1;
    let mut solution = vec![0; capacity].into_boxed_slice();
    let mut solution_len = 0;

    // The solution for n cards is
    // the solution for n-1 cards,
    // followed by a flip of card n,
    // followed by the solution for n-1 cards again.
    //
    // Symbolically (where || is the concatenation operator):
    //
    //   solution(0) = {}
    //   solution(n) = solution(n - 1) || {n} || solution(n - 1)
    //
    // By doing this,
    // we explore all possible states for the n-1 cards before card n,
    // then we explore them again with the opposite state for card n.
    // This ensures that we don't visit the same state twice.
    //
    // Incrementally solve for one more card at a time.
    for m in 1..=parameters.number_of_cards.number_of_cards {
        let partial_solution_len = solution_len;

        // Insert a flip of the nth card.
        solution[solution_len] = m;
        solution_len += 1;

        // Insert the flips for n-1 cards again.
        let (head, tail) = solution.split_at_mut(solution_len);
        tail[0..partial_solution_len].copy_from_slice(&head[0..partial_solution_len]);
        solution_len += partial_solution_len;
    }

    assert_eq!(solution.len(), solution_len);

    solution
}

/// Parameters for solving variants of the card puzzle.
#[derive(StructOpt)]
pub struct Parameters {
    /// The number of cards to play with.
    #[structopt(short = "n", long, default_value = "4")]
    number_of_cards: NumberOfCards,
}

/// A validated number of cards parameter.
pub struct NumberOfCards {
    /// The number of cards to play with.
    number_of_cards: u8,

    /// The number of possible states for the number of cards.
    number_of_card_states: usize,
}

impl TryFrom<u8> for NumberOfCards {
    type Error = NumberOfCardsError;

    fn try_from(number_of_cards: u8) -> Result<Self, Self::Error> {
        if let Some(number_of_card_states) = 1usize.checked_shl(number_of_cards as u32) {
            Ok(Self {
                number_of_cards,
                number_of_card_states,
            })
        } else {
            Err(NumberOfCardsError::NumberOfStatesOverflow(number_of_cards))
        }
    }
}

impl FromStr for NumberOfCards {
    type Err = NumberOfCardsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u8>()?.try_into()
    }
}

quick_error! {
    /// An error that can be returned when parsing the number of cards to play with.
    #[derive(Debug)]
    pub enum NumberOfCardsError {
        /// The number of cards doesn't fit in a `u8`.
        ParseIntError(err: ParseIntError) {
            cause(err)
            display("{}", err)
            from()
        }

        /// The number of states (and thus the number of flips)
        /// for the number of cards is too large.
        NumberOfStatesOverflow(number_of_cards: u8) {
            display("the number of flips required for {} cards is too large!", number_of_cards)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve() {
        let solution = card(Parameters {
            number_of_cards: 4.try_into().unwrap(),
        });

        assert_eq!(
            solution,
            vec![1, 2, 1, 3, 1, 2, 1, 4, 1, 2, 1, 3, 1, 2, 1].into_boxed_slice()
        );
    }

    #[test]
    fn verify() {
        let number_of_cards = 4;
        let solution = card(Parameters {
            number_of_cards: number_of_cards.try_into().unwrap(),
        });

        for card_state in 0..(1u64 << number_of_cards) {
            assert!(
                check_state(&solution, card_state),
                "the solution doesn't verify for card state {:01$b}",
                card_state,
                number_of_cards as usize
            );
        }
    }

    fn check_state(solution: &[u8], mut card_state: u64) -> bool {
        let mut solution_iter = solution.iter().cloned();
        loop {
            if card_state == 0 {
                return true;
            }

            match solution_iter.next() {
                Some(card_number) => {
                    let bit_number = card_number - 1;
                    card_state ^= 1 << bit_number;
                }
                None => {
                    return false;
                }
            }
        }
    }
}
