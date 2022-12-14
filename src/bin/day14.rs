use anyhow::{Context, Error};
use itertools::Itertools;
use ndarray::Array2;
use std::{fmt::Debug, fs, str::FromStr};

#[derive(Default, Clone)]
enum Space {
    #[default]
    Air,
    Rock,
    Sand,
}

struct Map {
    area: Array2<Space>,
}

impl Map {
    fn fill_sand(&mut self, start: (usize, usize), has_abyss: bool) {
        enum Explore {
            Open((usize, usize)),
            Close((usize, usize)),
        }

        let mut explore_stack = vec![Explore::Open(start)];
        let bottom = self.area.dim().1;

        while let Some(e) = explore_stack.pop() {
            match e {
                Explore::Open((x, y)) => {
                    if has_abyss && y + 1 >= bottom {
                        break;
                    }
                    explore_stack.push(Explore::Close((x, y)));
                    for n in [(x + 1, y + 1), (x - 1, y + 1), (x, y + 1)] {
                        if let Some(Space::Air) = self.area.get(n) {
                            explore_stack.push(Explore::Open(n));
                        }
                    }
                }
                Explore::Close(s) => self.area[s] = Space::Sand,
            }
        }
    }
}

fn part1(input: &str) -> Result<usize, Error> {
    let mut map: Map = input.parse()?;
    map.fill_sand((500, 0), true);

    let count = map.area.iter().filter(|s| matches!(s, Space::Sand)).count();

    Ok(count)
}

fn part2(input: &str) -> Result<usize, Error> {
    let mut map: Map = input.parse()?;
    map.fill_sand((500, 0), false);

    let count = map.area.iter().filter(|s| matches!(s, Space::Sand)).count();

    Ok(count)
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

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .area
            .columns()
            .into_iter()
            .map(|row| {
                row.iter()
                    .map(|s| match s {
                        Space::Air => '.',
                        Space::Rock => '#',
                        Space::Sand => 'o',
                    })
                    .join("")
            })
            .join("\n");

        f.write_str(&s)
    }
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

        let all_coords = lines.iter().flat_map(|l| l.iter()).collect_vec();
        let max_x = 2 * all_coords
            .iter()
            .map(|&&(x, _)| x)
            .max()
            .context("could not find max x")?;
        let max_y = all_coords
            .iter()
            .map(|&&(_, y)| y)
            .max()
            .context("could not find max y")?;

        let dims = (max_x, max_y + 2);

        let mut area = Array2::default(dims);

        for line in lines {
            let mut parts_iter = line.iter();
            let mut s = *parts_iter.next().context("line does not have a start?")?;
            area[s] = Space::Rock;

            for &t in parts_iter {
                let x_range = if s.0 < t.0 { s.0..=t.0 } else { t.0..=s.0 };
                for x in x_range {
                    area[(x, s.1)] = Space::Rock;
                }
                let y_range = if s.1 < t.1 { s.1..=t.1 } else { t.1..=s.1 };
                for y in y_range {
                    area[(s.0, y)] = Space::Rock;
                }
                s = t;
            }
        }

        Ok(Map { area })
    }
}
