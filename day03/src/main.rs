use itertools::Itertools;
use std::{error::Error, fs};

fn priority(item: char) -> u32 {
    if item.is_ascii_lowercase() {
        item as u32 - 96
    } else {
        item as u32 - 38
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("input")?;

    let part1 = input
        .lines()
        .map(|line| line.split_at(line.len() / 2))
        .filter_map(|(left, right)| left.chars().find(|c| right.contains(*c)))
        .map(priority)
        .sum::<u32>();

    println!("Part 1: {}", part1);

    let part2 = input
        .lines()
        .tuples::<(_, _, _)>()
        .filter_map(|(elf1, elf2, elf3)| {
            elf1.chars()
                .find(|c| elf2.contains(*c) && elf3.contains(*c))
        })
        .map(priority)
        .sum::<u32>();

    println!("Part 2: {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn priority() {
        assert_eq!(super::priority('a'), 1);
        assert_eq!(super::priority('z'), 26);
        assert_eq!(super::priority('A'), 27);
        assert_eq!(super::priority('Z'), 52);
    }
}
