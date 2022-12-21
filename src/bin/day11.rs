use anyhow::{anyhow, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, multispace0},
    combinator::{map_res, value},
    multi::{self, many1, separated_list1},
    sequence::tuple,
    IResult,
};
use std::{cell::RefCell, fs};

struct Monkey {
    items: Vec<u64>,

    arg1: Option<u64>,
    arg2: Option<u64>,
    operation: Operation,

    test: u64,
    if_true: usize,
    if_false: usize,
}

#[derive(Clone)]
enum Operation {
    Add,
    Multiply,
}

impl Operation {
    fn eval(&self, a: u64, b: u64) -> u64 {
        match self {
            Operation::Add => a + b,
            Operation::Multiply => a * b,
        }
    }
}

fn parse_input(input: &str) -> Result<Vec<Monkey>, Error> {
    fn parse_items(s: &str) -> IResult<&str, Vec<u64>> {
        let map_items = map_res(separated_list1(tag(", "), digit1), |items| {
            items
                .into_iter()
                .map(|i: &str| i.parse::<u64>())
                .collect::<Result<Vec<_>, _>>()
        });

        let (s, (_, items)) = tuple((tag("Starting items: "), map_items))(s)?;
        let (s, _) = multispace0(s)?;

        Ok((s, items))
    }
    fn parse_operation(s: &str) -> IResult<&str, (Option<u64>, Operation, Option<u64>)> {
        let (s, _) = tag("Operation: new = ")(s)?;
        let (s, _) = multispace0(s)?;

        let maybe_arg = || {
            alt((
                map_res(digit1, |arg: &str| arg.parse::<u64>().map(Some)),
                value(None, tag("old")),
            ))
        };

        let map_op = alt((
            value(Operation::Add, char('+')),
            value(Operation::Multiply, char('*')),
        ));

        let (s, (arg1, _, operation, _, arg2)) =
            tuple((maybe_arg(), char(' '), map_op, char(' '), maybe_arg()))(s)?;

        let (s, _) = multispace0(s)?;

        Ok((s, (arg1, operation, arg2)))
    }

    fn parse_test(s: &str) -> IResult<&str, (u64, usize, usize)> {
        let (s, (_, test)) = tuple((
            tag("Test: divisible by "),
            map_res(digit1, |test: &str| test.parse::<u64>()),
        ))(s)?;
        let (s, _) = multispace0(s)?;

        let (s, (_, if_true)) = tuple((
            tag("If true: throw to monkey "),
            map_res(digit1, |test: &str| test.parse::<usize>()),
        ))(s)?;
        let (s, _) = multispace0(s)?;

        let (s, (_, if_false)) = tuple((
            tag("If false: throw to monkey "),
            map_res(digit1, |test: &str| test.parse::<usize>()),
        ))(s)?;
        let (s, _) = multispace0(s)?;

        Ok((s, (test, if_true, if_false)))
    }

    fn parse_monkey(s: &str) -> IResult<&str, Monkey> {
        let (s, _) = tuple((tag("Monkey "), digit1, tag(":"), multispace0))(s)?;
        let (s, items) = parse_items(s)?;
        let (s, (arg1, operation, arg2)) = parse_operation(s)?;
        let (s, (test, if_true, if_false)) = parse_test(s)?;
        let (s, _) = multispace0(s)?;

        let m = Monkey {
            items,
            arg1,
            arg2,
            operation,
            test,
            if_true,
            if_false,
        };

        Ok((s, m))
    }

    let (_, monkeys) =
        many1(parse_monkey)(input).map_err(|e| anyhow!("could not parse monkeys: {}", e))?;

    Ok(monkeys)
}

fn simulate_rounds(monkeys: Vec<Monkey>, rounds: usize, relief: impl Fn(u64) -> u64) -> Vec<usize> {
    let monkeys: Vec<_> = monkeys.into_iter().map(RefCell::new).collect();
    let mut inspections = vec![0; monkeys.len()];

    for _ in 0..rounds {
        for i in 0..monkeys.len() {
            let mut m = monkeys[i].borrow_mut();
            let mut t = monkeys[m.if_true].borrow_mut();
            let mut f = monkeys[m.if_false].borrow_mut();

            let arg1 = m.arg1;
            let arg2 = m.arg2;
            let operation = m.operation.clone();
            let test = m.test;

            inspections[i] += m.items.len();
            for item in m.items.drain(..) {
                let a = arg1.unwrap_or(item);
                let b = arg2.unwrap_or(item);
                let item = operation.eval(a, b);
                let item = relief(item);

                if item % test == 0 {
                    t.items.push(item);
                } else {
                    f.items.push(item);
                }
            }
        }
    }

    inspections
}

pub fn calculate_business(inspections: &[usize]) -> usize {
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

fn part1(input: &str) -> Result<usize, Error> {
    let monkeys = parse_input(input)?;

    let inspections = simulate_rounds(monkeys, 20, |worry| worry / 3);
    Ok(calculate_business(&inspections))
}

fn part2(input: &str) -> Result<usize, Error> {
    let monkeys = parse_input(input)?;
    let shared_mod: u64 = monkeys.iter().map(|m| m.test).product();

    let inspections = simulate_rounds(monkeys, 10000, |worry| worry % shared_mod);
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

    use super::{parse_input, part1, part2, simulate_rounds};

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
    fn part2_inspection_examples(#[case] rounds: usize, #[case] expected_inspections: [usize; 4]) {
        let monkeys = parse_input(TEST_INPUT).unwrap();
        let shared_mod: u64 = monkeys.iter().map(|m| m.test).product();
        let inspections = simulate_rounds(monkeys, rounds, |worry| worry % shared_mod);

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
