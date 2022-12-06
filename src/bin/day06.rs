use std::fs;

use anyhow::Context;
use anyhow::Error;
use itertools::Itertools;

fn find_header<const N: usize>(input: &[char]) -> Result<usize, Error> {
    input
        .windows(N)
        .enumerate()
        .find(|(_, window)| window.iter().unique().count() == N)
        .map(|(i, _)| i + N)
        .context("could not find packet marker")
}

fn part1(input: &str) -> Result<usize, Error> {
    find_header::<4>(&input.chars().collect_vec())
}

fn part2(input: &str) -> Result<usize, Error> {
    find_header::<14>(&input.chars().collect_vec())
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/6")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};
    use rstest::rstest;

    #[rstest]
    #[case("bvwbjplbgvbhsrlpgdmjqwftvncz", 5)]
    #[case("nppdvjthqldpwncqszvftbrmjlhg", 6)]
    #[case("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 10)]
    #[case("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 11)]
    fn part1_examples(#[case] input: &str, #[case] expected: usize) {
        assert_eq!(part1(input).unwrap(), expected);
    }

    #[rstest]
    #[case("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 19)]
    #[case("bvwbjplbgvbhsrlpgdmjqwftvncz", 23)]
    #[case("nppdvjthqldpwncqszvftbrmjlhg", 23)]
    #[case("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 29)]
    #[case("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 26)]
    fn part2_examples(#[case] input: &str, #[case] expected: usize) {
        assert_eq!(part2(input).unwrap(), expected);
    }
}
