use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::{Debug, Write},
    fs, iter,
};

use anyhow::{bail, Error};

const EXTRA_ROWS: usize = 7;
const LEFT_WALL: u8 = 0b1000_0000;
const ROCK_HORIZONTAL: u32 = u32::from_be_bytes([0b000_0000, 0b000_0000, 0b000_0000, 0b001_1110]);
const ROCK_CROSS: u32 = u32::from_be_bytes([0b000_0000, 0b000_1000, 0b001_1100, 0b000_1000]);
const ROCK_CORNER: u32 = u32::from_be_bytes([0b000_0000, 0b000_0100, 0b000_0100, 0b001_1100]);
const ROCK_VERTICAL: u32 = u32::from_be_bytes([0b001_0000, 0b001_0000, 0b001_0000, 0b001_0000]);
const ROCK_SQUARE: u32 = u32::from_be_bytes([0b000_0000, 0b000_0000, 0b001_1000, 0b001_1000]);
const ALL_ROCKS: [u32; 5] = [
    ROCK_HORIZONTAL,
    ROCK_CROSS,
    ROCK_CORNER,
    ROCK_VERTICAL,
    ROCK_SQUARE,
];

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '>' => Ok(Direction::Right),
            '<' => Ok(Direction::Left),
            e => bail!("could not parse '{}' as a direction", e),
        }
    }
}

#[derive(Clone)]
struct Chamber {
    rows: VecDeque<u8>,
    truncated_rows: usize,
}

impl Debug for Chamber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("total tower height {}\n", self.tower_height()))?;
        f.write_fmt(format_args!("truncated rows {}\n", self.truncated_rows))?;
        for r in self.rows.iter().rev() {
            f.write_char('|')?;
            for i in (0..7).rev() {
                if (r >> i) & 1 == 1 {
                    f.write_char('#')?;
                } else {
                    f.write_char('.')?;
                }
            }
            f.write_str("|\n")?;
        }
        f.write_str("+-------+")
    }
}

impl Chamber {
    fn new() -> Self {
        Chamber {
            rows: vec![LEFT_WALL; EXTRA_ROWS].into(),
            truncated_rows: 0,
        }
    }

    fn as_vec(&self) -> Vec<u8> {
        self.rows.iter().copied().collect()
    }

    fn tower_height(&self) -> usize {
        self.truncated_rows + self.rows.len() - 7
    }

    fn start_height(&self) -> usize {
        self.rows.len() + 3 - EXTRA_ROWS
    }

    fn skip(&mut self, height: usize) {
        self.truncated_rows += height;
    }

    fn fits(&self, rock: u32, height: usize) -> bool {
        let rock_rows = rock.to_le_bytes();

        let rows = self.rows.range(height..height + 4);
        rock_rows.iter().zip(rows).all(|(a, b)| a & b == 0)
    }

    fn ensure_capacity(&mut self, height: usize, rock: u32) {
        let rock_height = 4 - (rock.leading_zeros() as usize / 8);
        let new_top_height = height + rock_height;
        if let Some(h_diff) = new_top_height.checked_sub(self.rows.len() - EXTRA_ROWS) {
            self.rows.extend(iter::repeat(LEFT_WALL).take(h_diff));
        }
    }

    fn insert_rock(&mut self, rock: u32, height: usize) {
        self.ensure_capacity(height, rock);

        let rows = self.rows.range_mut(height..);
        let rock_bytes = rock.to_le_bytes();
        rock_bytes
            .iter()
            .zip(rows)
            .for_each(|(rock, row)| *row |= *rock);
    }

    fn lowest_reachable_height_of_rock(&self, rock: u32) -> usize {
        let mut explored = BTreeSet::new();
        let mut stack = vec![(self.start_height(), rock)];

        let mut min = usize::MAX;
        while let Some((height, rock)) = stack.pop() {
            if height < min {
                min = height;
            }
            if height == 0 {
                return height;
            }
            if explored.contains(&(height, rock)) {
                continue;
            }
            explored.insert((height, rock));

            for direction in [Direction::Left, Direction::Right] {
                let rock = self.push_rock(rock, height, direction);
                if self.fits(rock, height - 1) {
                    stack.push((height - 1, rock));
                }
            }
        }

        explored.iter().map(|&(height, _)| height).min().unwrap()
    }

    fn prune(&mut self) {
        let prune_heights = ALL_ROCKS.map(|rock| self.lowest_reachable_height_of_rock(rock));
        let prune_height = *prune_heights.iter().min().unwrap();
        if prune_height != 0 {
            self.skip(prune_height);
            drop(self.rows.drain(..prune_height));
        }
    }

    fn push_rock(&self, rock: u32, height: usize, direction: Direction) -> u32 {
        let shifted_rock = match direction {
            Direction::Left => rock.rotate_left(1),
            Direction::Right => rock.rotate_right(1),
        };
        if self.fits(shifted_rock, height) {
            shifted_rock
        } else {
            rock
        }
    }

    fn add_rock<'a>(
        &mut self,
        rocks: &mut impl Iterator<Item = (usize, &'a u32)>,
        directions: &mut impl Iterator<Item = (usize, &'a Direction)>,
    ) {
        let (_, rock) = rocks.next().unwrap();
        let mut rock = *rock;
        let mut height = self.start_height();
        loop {
            let (_, &direction) = directions.next().unwrap();
            rock = self.push_rock(rock, height, direction);
            if height == 0 || !self.fits(rock, height - 1) {
                break;
            }
            height -= 1;
        }

        self.insert_rock(rock, height);
    }
}

fn rock_fall(input: &str, total_rocks: usize) -> Result<usize, Error> {
    let directions = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.try_into())
        .collect::<Result<Vec<Direction>, _>>()?;

    let mut directions_inf = directions.iter().cycle().enumerate().peekable();
    let mut rocks_inf = ALL_ROCKS.iter().cycle().enumerate().peekable();

    let mut chamber = Chamber::new();
    let mut cache: BTreeMap<_, (usize, Chamber)> = BTreeMap::new();

    let mut n = 0;
    while n < total_rocks {
        let n_rock = rocks_inf.peek().unwrap().0 % ALL_ROCKS.len();
        let n_direction = directions_inf.peek().unwrap().0 % directions.len();
        let n_state = chamber.as_vec();
        let n_key = (n_rock, n_direction, n_state);

        if let Some((earlier_n, earlier_chamber)) = cache.get(&n_key) {
            let remaining_n = total_rocks - n;
            let n_diff = n - earlier_n;
            let h_diff = chamber.tower_height() - earlier_chamber.tower_height();
            let possible_jumps = remaining_n / n_diff;

            if possible_jumps > 0 {
                let n_jump = possible_jumps * n_diff;
                let h_jump = possible_jumps * h_diff;
                n += n_jump;
                chamber.skip(h_jump);
                continue;
            }
        } else {
            cache.insert(n_key, (n, chamber.clone()));
        }

        chamber.add_rock(&mut rocks_inf, &mut directions_inf);
        chamber.prune();

        n += 1;
    }

    Ok(chamber.tower_height())
}

fn part1(input: &str) -> Result<usize, Error> {
    rock_fall(input, 2022)
}

fn part2(input: &str) -> Result<usize, Error> {
    rock_fall(input, 1000000000000)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/17")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 3068);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 1514285714288);
    }

    static TEST_INPUT: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>
    ";
}
