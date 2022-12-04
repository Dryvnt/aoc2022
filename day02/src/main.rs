use std::{error::Error, fs};

enum Move {
    Rock,
    Paper,
    Scissors,
}

const fn score_of(opponent: Move, player: Move) -> u64 {
    const LOSE_SCORE: u64 = 0;
    const DRAW_SCORE: u64 = 3;
    const WIN_SCORE: u64 = 6;

    const ROCK_SCORE: u64 = 1;
    const PAPER_SCORE: u64 = 2;
    const SCISSORS_SCORE: u64 = 3;

    match (opponent, player) {
        (Move::Rock, Move::Rock) => DRAW_SCORE + ROCK_SCORE,
        (Move::Rock, Move::Paper) => WIN_SCORE + PAPER_SCORE,
        (Move::Rock, Move::Scissors) => LOSE_SCORE + SCISSORS_SCORE,
        (Move::Paper, Move::Rock) => LOSE_SCORE + ROCK_SCORE,
        (Move::Paper, Move::Paper) => DRAW_SCORE + PAPER_SCORE,
        (Move::Paper, Move::Scissors) => WIN_SCORE + SCISSORS_SCORE,
        (Move::Scissors, Move::Rock) => WIN_SCORE + ROCK_SCORE,
        (Move::Scissors, Move::Paper) => LOSE_SCORE + PAPER_SCORE,
        (Move::Scissors, Move::Scissors) => DRAW_SCORE + SCISSORS_SCORE,
    }
}

fn part1_round_score(line: &str) -> u64 {
    match line {
        "A X" => score_of(Move::Rock, Move::Rock),
        "A Y" => score_of(Move::Rock, Move::Paper),
        "A Z" => score_of(Move::Rock, Move::Scissors),
        "B X" => score_of(Move::Paper, Move::Rock),
        "B Y" => score_of(Move::Paper, Move::Paper),
        "B Z" => score_of(Move::Paper, Move::Scissors),
        "C X" => score_of(Move::Scissors, Move::Rock),
        "C Y" => score_of(Move::Scissors, Move::Paper),
        "C Z" => score_of(Move::Scissors, Move::Scissors),
        _ => 0,
    }
}

fn part2_round_score(line: &str) -> u64 {
    match line {
        "A X" => score_of(Move::Rock, Move::Scissors),
        "A Y" => score_of(Move::Rock, Move::Rock),
        "A Z" => score_of(Move::Rock, Move::Paper),
        "B X" => score_of(Move::Paper, Move::Rock),
        "B Y" => score_of(Move::Paper, Move::Paper),
        "B Z" => score_of(Move::Paper, Move::Scissors),
        "C X" => score_of(Move::Scissors, Move::Paper),
        "C Y" => score_of(Move::Scissors, Move::Scissors),
        "C Z" => score_of(Move::Scissors, Move::Rock),
        _ => 0,
    }
}

fn calculate_part(input: impl AsRef<str>, func: impl Fn(&str) -> u64) -> u64 {
    input.as_ref().lines().map(func).sum::<u64>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("input")?;

    println!("Part 1: {}", calculate_part(&input, part1_round_score));
    println!("Part 2: {}", calculate_part(&input, part2_round_score));

    Ok(())
}

#[cfg(test)]
mod tests {
    static TEST_INPUT: &str = "A Y\nB X\nC Z\n";
    #[test]
    fn part1() {
        assert_eq!(
            super::calculate_part(TEST_INPUT, super::part1_round_score),
            15
        )
    }

    #[test]
    fn part2() {
        assert_eq!(
            super::calculate_part(TEST_INPUT, super::part2_round_score),
            12
        )
    }
}
