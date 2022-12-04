use std::cmp::Reverse;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("input")?;

    let mut elves = Vec::new();
    let lines = input.lines();
    let mut elf = 0u64;
    for l in lines {
        if l.is_empty() {
            elves.push(elf);
            elf = 0;
        } else {
            elf += l.parse::<u64>()?;
        }
    }

    elves.sort_unstable_by_key(|e| Reverse(*e));
    println!("Part 1: {}", elves.first().unwrap());
    println!("Part 2: {}", elves.iter().take(3).sum::<u64>());

    Ok(())
}
