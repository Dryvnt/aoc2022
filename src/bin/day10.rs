use anyhow::{bail, Error};
use itertools::Itertools;
use std::{fs, iter};

enum Instruction {
    Noop,
    Addx(i64),
}

impl Instruction {
    fn parse(line: &str) -> Result<Self, Error> {
        let parts = line.split(' ').collect_vec();
        let inst = match parts.as_slice() {
            ["noop", ..] => Instruction::Noop,
            ["addx", v] => Instruction::Addx(v.parse()?),
            _ => bail!("cannot parse instruction '{}'", line),
        };

        Ok(inst)
    }

    fn cycles(&self) -> usize {
        match self {
            Instruction::Noop => 1,
            Instruction::Addx(_) => 2,
        }
    }
}

fn simulate_machine(input: &str) -> Result<Vec<i64>, Error> {
    let mut x = 1;
    let mut out = Vec::new();
    for line in input.lines() {
        let instruction = Instruction::parse(line)?;

        out.extend(iter::repeat(x).take(instruction.cycles()));

        match instruction {
            Instruction::Noop => {}
            Instruction::Addx(val) => x += val,
        }
    }

    Ok(out)
}

fn part1(input: &str) -> Result<i64, Error> {
    let x_values = simulate_machine(input)?;

    let sum = [19, 59, 99, 139, 179, 219]
        .map(|i| x_values[i] * (1 + i as i64))
        .iter()
        .sum();

    Ok(sum)
}

fn part2(input: &str) -> Result<String, Error> {
    let x_values = simulate_machine(input)?;
    let mut output = String::new();

    for y in 0..6i64 {
        for x in 0..40i64 {
            let cycle = x + y * 40;
            let x_value = x_values[cycle as usize];
            let c = if x.abs_diff(x_value) <= 1 { '#' } else { '.' };
            output.push(c);
        }
        output.push('\n');
    }

    Ok(output)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/10")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2:\n{}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 13140);
    }

    static PART2_EXPECTED: &str = "##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....
";

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), PART2_EXPECTED);
    }

    static TEST_INPUT: &str = "addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";
}
