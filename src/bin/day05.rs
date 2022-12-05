use std::fs;

use anyhow::bail;
use anyhow::Context;
use anyhow::Error;
use itertools::Itertools;

struct Instruction {
    from: usize,
    to: usize,
    amount: usize,
}

impl Instruction {
    fn parse(line: &str) -> Result<Self, Error> {
        if let Some(("move", amount_str, "from", from_str, "to", to_str)) =
            line.split(' ').next_tuple::<(_, _, _, _, _, _)>()
        {
            let amount = amount_str.parse()?;
            let from = from_str.parse::<usize>()?;
            let to = to_str.parse::<usize>()?;
            Ok(Instruction { from, to, amount })
        } else {
            bail!("invalid move instruction {}", line)
        }
    }
}

struct CargoState {
    stacks: Vec<String>,
}

impl CargoState {
    fn parse(input: &str) -> Result<Self, Error> {
        let last_line = input.lines().last().context("no last line?")?;
        let crates = last_line
            .char_indices()
            .filter(|(_, c)| c.is_ascii_digit())
            .map(|(x, _)| {
                input
                    .lines()
                    .rev()
                    .skip(1)
                    .filter_map(|line| line.chars().nth(x))
                    .filter(|c| !c.is_ascii_whitespace())
                    .collect()
            })
            .collect();

        Ok(CargoState { stacks: crates })
    }

    fn apply_instruction_part1(&mut self, inst: &Instruction) {
        for _ in 0..inst.amount {
            let c = self.stacks[inst.from - 1].pop().expect("no crate to pick");
            self.stacks[inst.to - 1].push(c);
        }
    }

    fn apply_instruction_part2(&mut self, inst: &Instruction) {
        let from = inst.from - 1;
        let to = inst.to - 1;
        let drain_range = (self.stacks[from].len() - inst.amount)..self.stacks[from].len();

        // Have to collect this into Vec to convince borrow checker that everything is fine
        let yoink = self.stacks[from].drain(drain_range).collect_vec();
        self.stacks[to].extend(yoink);
    }

    fn get_code(&self) -> String {
        self.stacks
            .iter()
            .map(|stack| stack.chars().last().unwrap())
            .collect()
    }
}

fn parse_input(input: &str) -> Result<(CargoState, Vec<Instruction>), Error> {
    let split_idx = input.find("\n\n").unwrap();
    let (setup, instructions) = input.split_at(split_idx + 1);

    let cargo = CargoState::parse(setup)?;
    let instructions = instructions
        .lines()
        .map(Instruction::parse)
        .filter_map(Result::ok)
        .collect_vec();

    Ok((cargo, instructions))
}

fn run(
    input: &str,
    apply_instruction: impl Fn(&mut CargoState, &Instruction),
) -> Result<String, Error> {
    let (mut cargo, instructions) = parse_input(input)?;

    for inst in instructions {
        apply_instruction(&mut cargo, &inst);
    }

    Ok(cargo.get_code())
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/5")?;

    println!(
        "Part 1: {}",
        run(&input, CargoState::apply_instruction_part1)?
    );
    println!(
        "Part 2: {}",
        run(&input, CargoState::apply_instruction_part2)?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_instruction() {
        let inst = super::Instruction::parse("move 3 from 1 to 3").unwrap();
        assert_eq!(inst.amount, 3);
        assert_eq!(inst.from, 1);
        assert_eq!(inst.to, 3);
    }

    static TEST_INPUT: &str = "    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
";

    #[test]
    fn example_part1() {
        assert_eq!(
            super::run(TEST_INPUT, super::CargoState::apply_instruction_part1).unwrap(),
            "CMZ"
        );
    }

    #[test]
    fn example_part2() {
        assert_eq!(
            super::run(TEST_INPUT, super::CargoState::apply_instruction_part2).unwrap(),
            "MCD"
        );
    }
}
