use anyhow::Error;
use std::fs;

use monkey::Monkey;

mod monkey;

pub fn parse_input(input: &str) -> Result<Vec<Monkey>, Error> {
    input
        .split("\n\n")
        .map(|m| m.parse())
        .collect::<Result<Vec<_>, _>>()
}

pub fn simulate_rounds(
    monkeys: &mut Vec<Monkey>,
    rounds: u64,
    relief: impl Fn(u64) -> u64,
) -> Vec<u64> {
    let mut inspections = vec![0; monkeys.len()];

    for _ in 0..rounds {
        for i in 0..monkeys.len() {
            while let Some((throw_to, item)) = monkeys[i].yeet(&relief) {
                inspections[i] += 1;
                assert_ne!(i, throw_to, "monkey tried to throw to itself");
                monkeys[throw_to].yoink(item);
            }
        }
    }

    inspections
}

pub fn calculate_business(inspections: &[u64]) -> u64 {
    // Weird but efficient way of "find top 2 items"
    // Redeeming myself after day 1
    let max2 = inspections.iter().fold((0, 0), |(a, b), &c| {
        if c > b {
            if c > a {
                (c, a)
            } else {
                (a, c)
            }
        } else {
            (a, b)
        }
    });

    max2.0 * max2.1
}

fn part1(input: &str) -> Result<u64, Error> {
    let mut monkeys = parse_input(input)?;

    let inspections = simulate_rounds(&mut monkeys, 20, |worry| worry / 3);
    Ok(calculate_business(&inspections))
}

fn part2(input: &str) -> Result<u64, Error> {
    let mut monkeys = parse_input(input)?;
    let shared_mod: u64 = Monkey::shared_mod(&monkeys);

    let inspections = simulate_rounds(&mut monkeys, 10000, |worry| worry % shared_mod);
    Ok(calculate_business(&inspections))
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/11")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{monkey::Monkey, parse_input, part1, part2, simulate_rounds};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 10605);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 2713310158);
    }

    #[rstest]
    #[case(1, [2,4,3,6])]
    #[case(20, [99,97,8,103])]
    #[case(1000, [5204, 4792, 199, 5192])]
    #[case(2000, [10419, 9577, 392, 10391])]
    #[case(3000, [15638 , 14358, 587, 15593])]
    #[case(4000, [20858, 19138, 780, 20797])]
    #[case(5000, [26075, 23921, 974, 26000])]
    #[case(6000, [31294, 28702, 1165, 31204])]
    #[case(7000, [36508, 33488, 1360, 36400])]
    #[case(8000, [41728, 38268, 1553, 41606])]
    #[case(9000, [46945, 43051, 1746, 46807])]
    #[case(10000, [52166, 47830, 1938, 52013])]
    fn part2_inspection_examples(#[case] rounds: u64, #[case] expected_inspections: [u64; 4]) {
        let mut monkeys = parse_input(TEST_INPUT).unwrap();
        let shared_mod: u64 = Monkey::shared_mod(&monkeys);
        let inspections = simulate_rounds(&mut monkeys, rounds, |worry| worry % shared_mod);

        assert_eq!(inspections, expected_inspections);
    }

    static TEST_INPUT: &str = "Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1
";
}
