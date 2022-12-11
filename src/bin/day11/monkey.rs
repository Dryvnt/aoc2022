use std::str::{FromStr, Lines};

use anyhow::{bail, Context, Error};

#[derive(Debug)]
pub struct Monkey {
    brain: LogicContainer,
    items: Vec<u64>,
}

#[derive(Debug)]
struct LogicContainer {
    worry_operation: Operation,
    value_1: Option<u64>,
    value_2: Option<u64>,
    divisor: u64,
    if_true: usize,
    if_false: usize,
}

#[derive(Debug)]
enum Operation {
    Add,
    Multiply,
}

impl Monkey {
    pub fn yeet(&mut self, relief: impl Fn(u64) -> u64) -> Option<(usize, u64)> {
        self.items.pop().map(|item| {
            let item = self.brain.increase_worry(item);
            let item = relief(item);
            let throw_to = self.brain.throw_to(item);
            (throw_to, item)
        })
    }

    pub fn yoink(&mut self, item: u64) {
        self.items.push(item);
    }

    pub fn shared_mod(monkeys: &[Self]) -> u64 {
        monkeys.iter().map(|m| m.brain.divisor).product()
    }
}

impl LogicContainer {
    fn increase_worry(&self, item: u64) -> u64 {
        let v1 = self.value_1.unwrap_or(item);
        let v2 = self.value_2.unwrap_or(item);
        match self.worry_operation {
            Operation::Add => v1 + v2,
            Operation::Multiply => v1 * v2,
        }
    }

    fn throw_to(&self, item: u64) -> usize {
        if item % self.divisor == 0 {
            self.if_true
        } else {
            self.if_false
        }
    }
}

impl FromStr for Monkey {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn parse_items(line: &str) -> Result<Vec<u64>, Error> {
            let items = line
                .trim_start()
                .strip_prefix("Starting items: ")
                .with_context(|| format!("line '{}' did not fit starting items pattern", line))?;
            let mut items = items
                .split(", ")
                .map(|item| item.parse::<u64>())
                .collect::<Result<Vec<_>, _>>()?;

            items.reverse();

            Ok(items)
        }

        let mut lines = input.lines();

        // Header
        assert!(lines
            .next()
            .map(|l| l.starts_with("Monkey "))
            .unwrap_or_default());

        let items = lines
            .next()
            .context("no more lines")
            .and_then(parse_items)?;

        let brain = lines.try_into()?;

        Ok(Monkey { brain, items })
    }
}

impl<'a> TryFrom<Lines<'a>> for LogicContainer {
    type Error = Error;

    fn try_from(mut lines: Lines<'a>) -> Result<Self, Self::Error> {
        fn parse_operation(
            lines: &mut Lines,
        ) -> Result<(Operation, Option<u64>, Option<u64>), Error> {
            // Operation
            let line = lines.next().context("no more lines")?;
            let line = line
                .trim_start()
                .strip_prefix("Operation: new = ")
                .with_context(|| format!("line '{}' did not fit operation pattern", line))?;
            let mut parts = line.split_ascii_whitespace();
            let value_1 = parts
                .next()
                .context("could not extract value_1")?
                .parse()
                .ok();
            let op = parts.next().context("could not extract kind")?;
            let value_2 = parts
                .next()
                .context("could not extract value_2")?
                .parse()
                .ok();

            let worry_operation = match op {
                "+" => Operation::Add,
                "*" => Operation::Multiply,
                err => bail!("cannot parse '{}' as an operator kind", err),
            };

            Ok((worry_operation, value_1, value_2))
        }

        fn parse_logic(lines: &mut Lines) -> Result<(u64, usize, usize), Error> {
            // Logic
            let line = lines.next().context("no more lines")?;
            let divisor = line
                .trim_start()
                .strip_prefix("Test: divisible by ")
                .with_context(|| format!("line '{}' did not fit divisible_by pattern", line))?
                .parse()?;

            let line = lines.next().context("no more lines")?;
            let if_true = line
                .trim_start()
                .strip_prefix("If true: throw to monkey ")
                .with_context(|| format!("line '{}' did not fit if_true pattern", line))?
                .parse()?;

            let line = lines.next().context("no more lines")?;
            let if_false = line
                .trim_start()
                .strip_prefix("If false: throw to monkey ")
                .with_context(|| format!("line '{}' did not fit if_false pattern", line))?
                .parse()?;

            Ok((divisor, if_true, if_false))
        }

        let (worry_operation, value_1, value_2) = parse_operation(&mut lines)?;

        let (divisor, if_true, if_false) = parse_logic(&mut lines)?;

        let monkey = LogicContainer {
            worry_operation,
            value_1,
            value_2,
            divisor,
            if_true,
            if_false,
        };

        Ok(monkey)
    }
}
