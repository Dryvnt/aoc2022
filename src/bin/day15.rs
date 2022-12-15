use anyhow::{bail, Context, Error};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashSet, fs, ops::Range, str::FromStr};

#[derive(Debug)]
// A position in the rotated space, rotated 45 degrees so manhatten distance becomes max(x_dist, y_dist)
struct Position {
    x: i64,
    y: i64,
}

impl Position {
    fn new(x: i64, y: i64) -> Self {
        Position { x, y }
    }

    fn distance(&self, other: &Self) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn in_bounds(&self, bounds: u64) -> bool {
        0 <= self.x && self.x <= bounds as i64 && 0 <= self.y && self.y <= bounds as i64
    }
}

#[derive(Debug)]
struct Sensor {
    position: Position,
    distance: u64,
}

impl Sensor {
    fn x_span_at_y(&self, y: i64) -> Option<Range<i64>> {
        let diff = self.position.y.abs_diff(y);
        if diff > self.distance {
            None
        } else {
            let span = self.distance - diff;
            let start = self.position.x - span as i64;
            let end = self.position.x + span as i64;
            Some(start..end)
        }
    }

    // Coefficients for line equations of just outside the detection box
    // y =  x + a
    // y = -x + b
    fn line_coefficients(&self) -> ([i64; 2], [i64; 2]) {
        (
            [
                self.position.y - self.position.x + self.distance as i64 + 1,
                self.position.y - self.position.x - self.distance as i64 - 1,
            ],
            [
                self.position.y + self.position.x + self.distance as i64 + 1,
                self.position.y + self.position.x - self.distance as i64 - 1,
            ],
        )
    }
}

impl FromStr for Sensor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref SENSOR_RE: Regex = Regex::new(
                r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)"
            )
            .unwrap();
        }
        let captures = SENSOR_RE
            .captures(s)
            .with_context(|| format!("input '{}' did not match pattern", s))?;

        let position = Position::new(captures[1].parse()?, captures[2].parse()?);
        let beacon = Position::new(captures[3].parse()?, captures[4].parse()?);
        let distance = position.distance(&beacon);

        Ok(Sensor { position, distance })
    }
}

fn part1(input: &str, y: i64) -> Result<usize, Error> {
    let sensors = input
        .lines()
        .map(|l| l.parse::<Sensor>())
        .collect::<Result<Vec<_>, _>>()?;

    let mut ranges = Vec::new();
    for mut r in sensors.iter().filter_map(|s| s.x_span_at_y(y)) {
        ranges.retain(|c: &Range<i64>| {
            let max_start = std::cmp::max(r.start, c.start);
            let min_end = std::cmp::min(r.end, r.end);

            if max_start <= min_end {
                let start = std::cmp::min(r.start, c.start);
                let end = std::cmp::max(r.end, c.end);
                r = start..end;
                false
            } else {
                true
            }
        });
        ranges.push(r);
    }

    let count = ranges.drain(..).map(|r| r.count()).sum();

    Ok(count)
}

fn part2(input: &str, bounds: u64) -> Result<i64, Error> {
    let sensors = input
        .lines()
        .map(|l| l.parse::<Sensor>())
        .collect::<Result<Vec<_>, _>>()?;

    let mut a_coefficients = HashSet::new();
    let mut b_coefficients = HashSet::new();
    for s in &sensors {
        let (a, b) = s.line_coefficients();
        a_coefficients.extend(a.into_iter());
        b_coefficients.extend(b.into_iter());
    }

    for a in &a_coefficients {
        for b in &b_coefficients {
            let intersection = Position::new((b - a) / 2, (b + a) / 2);
            if !intersection.in_bounds(bounds) {
                continue;
            }
            if sensors
                .iter()
                .all(|s| s.position.distance(&intersection) > s.distance)
            {
                return Ok(4000000 * intersection.x + intersection.y);
            }
        }
    }

    bail!("Could not find a solution")
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/15")?;

    println!("Part 1: {}", part1(&input, 2000000)?);
    println!("Part 2: {}", part2(&input, 4000000)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT, 10).unwrap(), 26);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT, 20).unwrap(), 56000011);
    }

    static TEST_INPUT: &str = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3
";
}
