use std::{error::Error, fs};

fn parse_range(input: &str) -> Result<(u32, u32), Box<dyn Error>> {
    let (left, right) = input.split_once('-').ok_or("range could not be split")?;

    Ok((left.parse()?, right.parse()?))
}

fn check_fully_contained(left_start: u32, left_end: u32, right_start: u32, right_end: u32) -> bool {
    if left_start < right_start || (right_start == left_start && right_end < left_end) {
        right_end <= left_end
    } else {
        left_end <= right_end
    }
}

fn check_overlap(left_start: u32, left_end: u32, right_start: u32, right_end: u32) -> bool {
    if left_start <= right_start {
        right_start <= left_end
    } else {
        left_start <= right_end
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("input/4")?;

    let mut part1 = 0;
    let mut part2 = 0;
    for line in input.lines() {
        let (left, right) = line.split_once(',').unwrap();
        let (left_start, left_end) = parse_range(left)?;
        let (right_start, right_end) = parse_range(right)?;

        if check_fully_contained(left_start, left_end, right_start, right_end) {
            part1 += 1;
        }
        if check_overlap(left_start, left_end, right_start, right_end) {
            part2 += 1;
        }
    }
    println!("Part 1: {}", part1);
    println!("Part 2: {}", part2);

    Ok(())
}
