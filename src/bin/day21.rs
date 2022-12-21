use std::{collections::HashMap, fs};

use anyhow::{anyhow, bail, Error};
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, one_of},
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};

#[derive(Clone, Copy)]
enum Monkey<'a> {
    Const(u64),
    Operation {
        a: &'a str,
        b: &'a str,
        op: Operation,
    },
}

impl<'a> Monkey<'a> {
    fn left_right(&self) -> Result<(&'a str, &'a str), Error> {
        Ok(match self {
            Monkey::Const(_) => bail!("root is const"),
            Monkey::Operation { a, b, op: _ } => (a, b),
        })
    }
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

fn parse_monkey(s: &str) -> Result<(&str, Monkey), Error> {
    fn parse(s: &str) -> IResult<&str, (&str, Monkey)> {
        let (s, name) = alpha1(s)?;
        let (s, _) = tag(": ")(s)?;
        let (s, opt_const) = opt(map_res(digit1, |s: &str| s.parse::<u64>()))(s)?;

        if let Some(c) = opt_const {
            return Ok((s, (name, Monkey::Const(c))));
        }

        let (s, (a, _, op, _, b)) = tuple((alpha1, tag(" "), one_of("+-*/"), tag(" "), alpha1))(s)?;

        let op = match op {
            '+' => Operation::Add,
            '-' => Operation::Sub,
            '*' => Operation::Mul,
            '/' => Operation::Div,
            _ => unreachable!(),
        };

        Ok((s, (name, Monkey::Operation { a, b, op })))
    }

    let (_, m) = parse(s).map_err(|e| anyhow!("could not parse monkey: {:?}", e))?;

    Ok(m)
}

struct MonkeyContext<'a> {
    monkeys: HashMap<&'a str, Monkey<'a>>,
    value_cache: HashMap<&'a str, u64>,
    has_human_cache: HashMap<&'a str, bool>,
}

impl<'a> MonkeyContext<'a> {
    fn parse_input(input: &'a str) -> Result<Self, Error> {
        let monkeys = input
            .lines()
            .map(parse_monkey)
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(MonkeyContext {
            monkeys,
            value_cache: HashMap::new(),
            has_human_cache: HashMap::new(),
        })
    }

    fn get_value(&mut self, which: &'a str) -> u64 {
        if self.value_cache.contains_key(which) {
            return self.value_cache[which];
        }
        let m = self.monkeys[which];
        let v = match m {
            Monkey::Const(c) => c,
            Monkey::Operation { a, b, op } => {
                let a = self.get_value(a);
                let b = self.get_value(b);
                match op {
                    Operation::Add => a + b,
                    Operation::Sub => a - b,
                    Operation::Mul => a * b,
                    Operation::Div => a / b,
                }
            }
        };
        self.value_cache.insert(which, v);
        v
    }

    fn has_human(&mut self, which: &'a str) -> bool {
        if which == "humn" {
            return true;
        }

        if self.has_human_cache.contains_key(which) {
            return self.has_human_cache[which];
        }
        let v = match self.monkeys[which] {
            Monkey::Operation { a, b, op: _ } => self.has_human(a) || self.has_human(b),
            _ => false,
        };
        self.has_human_cache.insert(which, v);
        v
    }

    fn human_value_to_make_eq(&mut self, which: &'a str, value: u64) -> Result<u64, Error> {
        if which == "humn" {
            return Ok(value);
        }

        if !self.has_human(which) {
            bail!("wrong subtree!");
        }

        let m = self.monkeys[which];
        if let Monkey::Operation {
            a: left,
            b: right,
            op,
        } = m
        {
            if self.has_human(left) {
                debug_assert!(self.has_human(left));
                let right = self.get_value(right);
                let to_match = match op {
                    Operation::Add => value - right, // value = left + right, left = value - right
                    Operation::Sub => value + right, // value = left - right, left = value + right
                    Operation::Mul => value / right, // value = left * right, left = value / right
                    Operation::Div => value * right, // value = left / right, left = value * right
                };

                self.human_value_to_make_eq(left, to_match)
            } else {
                debug_assert!(self.has_human(right));
                let left = self.get_value(left);
                let to_match = match op {
                    Operation::Add => value - left, // value = left + right, right = value - left
                    Operation::Sub => left - value, // value = left - right, right = - (value - left)
                    Operation::Mul => value / left, // value = left * right, right = value / left
                    Operation::Div => left / value, // value = left / right, right = left / value
                };

                self.human_value_to_make_eq(right, to_match)
            }
        } else {
            bail!("monkey has no children")
        }
    }
}

fn part1(input: &str) -> Result<u64, Error> {
    let mut ctx = MonkeyContext::parse_input(input)?;

    Ok(ctx.get_value("root"))
}

fn part2(input: &str) -> Result<u64, Error> {
    let mut ctx = MonkeyContext::parse_input(input)?;

    let root = ctx.monkeys["root"];
    let (left, right) = root.left_right()?;

    let (to_solve, value) = if ctx.has_human(left) {
        (left, right)
    } else {
        (right, left)
    };

    let value = ctx.get_value(value);

    let v = ctx.human_value_to_make_eq(to_solve, value)?;

    Ok(v)
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
