use anyhow::{Context, Error};
use std::{collections::HashSet, fs};

fn run_simulation<const N: usize>(input: &str) -> Result<HashSet<(i32, i32)>, Error> {
    let mut rope = [(0i32, 0i32); N];
    let mut visited = HashSet::new();
    visited.insert((0, 0));

    for line in input.lines() {
        let (dir, amount) = line.split_once(' ').context("could not split input")?;
        let amount = amount.parse::<i32>()?;

        for _ in 0..amount {
            let head = &mut rope[0];
            move_head(dir, head);
            if simulate_rope(&mut rope) {
                visited.insert(rope[N - 1]);
            }
        }
    }

    Ok(visited)
}

fn move_head(dir: &str, head: &mut (i32, i32)) {
    match dir {
        "L" => head.0 -= 1,
        "R" => head.0 += 1,
        "D" => head.1 -= 1,
        "U" => head.1 += 1,
        _ => {}
    }
}

// returns true if we moved the tail of the rope
fn simulate_rope<const N: usize>(rope: &mut [(i32, i32); N]) -> bool {
    for i in 0..N - 1 {
        let h = rope[i];
        let t = &mut rope[i + 1];

        if t.0.abs_diff(h.0) > 1 || t.1.abs_diff(h.1) > 1 {
            t.0 += (h.0 - t.0).signum();
            t.1 += (h.1 - t.1).signum();
        } else {
            // If we did not change this part of the rope, later parts won't change
            return false;
        }
    }
    true
}

fn part1(input: &str) -> Result<usize, Error> {
    Ok(run_simulation::<2>(input)?.len())
}

fn part2(input: &str) -> Result<usize, Error> {
    Ok(run_simulation::<10>(input)?.len())
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/9")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    static TEST_INPUT_1: &str = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT_1).unwrap(), 13);
    }
    static TEST_INPUT_2: &str = "R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT_2).unwrap(), 36);
    }
}
