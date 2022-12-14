use anyhow::{Context, Error};
use itertools::Itertools;
use std::{collections::HashSet, fs, str::FromStr};

struct Map {
    rocks: HashSet<(usize, usize)>,
    bottom: usize,
}

impl Map {
    fn fill_sand(&self, start: (usize, usize), has_floor: bool) -> HashSet<(usize, usize)> {
        enum Explore {
            Open((usize, usize)),
            Close((usize, usize)),
        }

        let mut sand = HashSet::new();
        let mut explore_stack = vec![Explore::Open(start)];

        while let Some(e) = explore_stack.pop() {
            match e {
                Explore::Open((x, y)) => {
                    if y >= self.bottom {
                        if has_floor {
                            continue;
                        }
                        break;
                    }

                    explore_stack.push(Explore::Close((x, y)));
                    for n in [(x + 1, y + 1), (x - 1, y + 1), (x, y + 1)] {
                        if !(self.rocks.contains(&n) || sand.contains(&n)) {
                            explore_stack.push(Explore::Open(n));
                        }
                    }
                }
                Explore::Close(s) => {
                    sand.insert(s);
                }
            }
        }

        sand
    }
}

fn part1(input: &str) -> Result<usize, Error> {
    let map: Map = input.parse()?;
    let sand = map.fill_sand((500, 0), false);

    Ok(sand.len())
}

fn part2(input: &str) -> Result<usize, Error> {
    let map: Map = input.parse()?;
    let sand = map.fill_sand((500, 0), true);

    Ok(sand.len())
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/14")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 24);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 93);
    }
    static TEST_INPUT: &str = "498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9
";
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = Vec::new();
        for line in s.lines() {
            let mut line_parts = Vec::new();
            let parts = line.split(" -> ");
            for part in parts {
                let (x, y) = part
                    .split_once(',')
                    .context("could not split into coords")?;
                let x = x.parse::<usize>()?;
                let y = y.parse::<usize>()?;
                line_parts.push((x, y));
            }
            lines.push(line_parts);
        }

        let mut rocks = HashSet::new();

        for line in lines {
            for (s, t) in line.iter().tuple_windows() {
                let x_range = if s.0 < t.0 { s.0..=t.0 } else { t.0..=s.0 };
                rocks.extend(x_range.map(|x| (x, s.1)));
                let y_range = if s.1 < t.1 { s.1..=t.1 } else { t.1..=s.1 };
                rocks.extend(y_range.map(|y| (s.0, y)));
            }
        }

        let bottom = 2 + rocks
            .iter()
            .map(|(_, y)| y)
            .max()
            .context("could not find max y")?;

        Ok(Map { rocks, bottom })
    }
}
