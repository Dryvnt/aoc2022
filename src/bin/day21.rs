use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    ops::Index,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{anyhow, bail, Context, Error};
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, one_of},
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};

enum Monkey<'a> {
    Const(u64),
    Operation {
        left: &'a str,
        right: &'a str,
        op: Operation,
    },
}

enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

impl Operation {
    fn solve_for_left(&self, value: u64, right: u64) -> u64 {
        match self {
            Operation::Add => value - right,
            Operation::Sub => value + right,
            Operation::Mul => value / right,
            Operation::Div => value * right,
            Operation::Eq => right,
        }
    }

    fn solve_for_right(&self, value: u64, left: u64) -> u64 {
        match self {
            Operation::Add => value - left,
            Operation::Sub => left - value,
            Operation::Mul => value / left,
            Operation::Div => left / value,
            Operation::Eq => left,
        }
    }

    fn eval(&self, left: u64, right: u64) -> u64 {
        match self {
            Operation::Add => left + right,
            Operation::Sub => left - right,
            Operation::Mul => left * right,
            Operation::Div => left / right,
            Operation::Eq => unimplemented!("cannot eval eq operation"),
        }
    }
}

fn parse_monkey(s: &str) -> Result<(&str, Monkey), Error> {
    fn parse(s: &str) -> IResult<&str, (&str, Monkey)> {
        let (s, name) = alpha1(s)?;
        let (s, _) = tag(": ")(s)?;
        let (s, opt_const) = opt(map_res(digit1, |s: &str| s.parse::<u64>()))(s)?;

        if let Some(c) = opt_const {
            return Ok((s, (name, Monkey::Const(c))));
        }

        let (s, (left, _, op, _, right)) =
            tuple((alpha1, tag(" "), one_of("+-*/"), tag(" "), alpha1))(s)?;

        let op = match op {
            '+' => Operation::Add,
            '-' => Operation::Sub,
            '*' => Operation::Mul,
            '/' => Operation::Div,
            _ => unreachable!(),
        };

        Ok((s, (name, Monkey::Operation { left, right, op })))
    }

    let (_, m) = parse(s).map_err(|e| anyhow!("could not parse monkey: {:?}", e))?;

    Ok(m)
}

struct MonkeyCollection<'a> {
    monkeys: HashMap<&'a str, Monkey<'a>>,
}

impl<'a> MonkeyCollection<'a> {
    fn parse_input(input: &'a str) -> Result<Self, Error> {
        let monkeys = input
            .lines()
            .map(parse_monkey)
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(MonkeyCollection { monkeys })
    }

    fn get(&self, which: &str) -> Result<&Monkey, Error> {
        self.monkeys.get(which).context("could not find monkey")
    }

    // Will be called once on each monkey, literally no gain from memoizing
    fn get_value(&self, which: &'a str) -> Result<u64, Error> {
        match self.get(which)? {
            Monkey::Const(c) => Ok(*c),
            Monkey::Operation { left, right, op } => {
                let left = self.get_value(left)?;
                let right = self.get_value(right)?;
                Ok(op.eval(left, right))
            }
        }
    }

    // Will be called multiple times on the same monkey, but not worth memoizing
    fn has_human(&self, which: &'a str) -> Result<bool, Error> {
        if which == "humn" {
            return Ok(true);
        }

        Ok(match self.get(which)? {
            Monkey::Operation { left, right, op: _ } => {
                self.has_human(left)? || self.has_human(right)?
            }
            _ => false,
        })
    }

    fn solve_for_human(&self, which: &'a str, value: u64) -> Result<u64, Error> {
        if which == "humn" {
            return Ok(value);
        }

        if let Monkey::Operation { left, right, op } = &self.monkeys[which] {
            let op = if which == "root" { &Operation::Eq } else { op };
            let left_has_human = self.has_human(left)?;
            debug_assert_eq!(self.has_human(right)?, !left_has_human);

            if left_has_human {
                let right = self.get_value(right)?;
                let target_value = op.solve_for_left(value, right);
                self.solve_for_human(left, target_value)
            } else {
                let left = self.get_value(left)?;
                let target_value = op.solve_for_right(value, left);
                self.solve_for_human(right, target_value)
            }
        } else {
            bail!("monkey has no subtree")
        }
    }
}

fn part1(input: &str) -> Result<u64, Error> {
    let monkeys = MonkeyCollection::parse_input(input)?;

    monkeys.get_value("root")
}

fn part2(input: &str) -> Result<u64, Error> {
    let monkeys = MonkeyCollection::parse_input(input)?;

    let root_value = monkeys.get_value("root")?;

    monkeys.solve_for_human("root", root_value)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/21")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 152);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 301);
    }

    static TEST_INPUT: &str = "root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32
";
}
