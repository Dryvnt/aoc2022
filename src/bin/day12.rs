use anyhow::{Context, Error};
use ndarray::Array2;
use std::{
    collections::{HashSet, VecDeque},
    fs,
    str::FromStr,
};

#[derive(Debug)]
struct Map {
    heights: Array2<u64>,
    start: (usize, usize),
    end: (usize, usize),
}

impl Map {
    fn neighbors(
        &self,
        (x, y): (usize, usize),
        reachability_check: impl Fn(u64, u64) -> bool,
    ) -> Vec<(usize, usize)> {
        let h = self.heights[(x, y)];

        let mut out = Vec::new();
        // What a disgusting mess lol, all to avoid underflow
        for other in [
            (x.checked_sub(1), Some(y)),
            (x.checked_add(1), Some(y)),
            (Some(x), y.checked_sub(1)),
            (Some(x), y.checked_add(1)),
        ] {
            if let (Some(x), Some(y)) = other {
                if let Some(&n) = self.heights.get((x, y)) {
                    if reachability_check(h, n) {
                        out.push((x, y));
                    }
                }
            }
        }

        out
    }
}

fn bfs(
    map: &Map,
    start: (usize, usize),
    check_goal: impl Fn((usize, usize)) -> bool,
    reachability: impl Fn(u64, u64) -> bool,
) -> Option<u64> {
    let mut to_explore = VecDeque::with_capacity(map.heights.len());
    let mut visited = HashSet::new();

    // Standard library BinaryHeap is a max-heap, so use Reverse to make it a min-heap
    to_explore.push_back((0, start));

    while let Some((d, node)) = to_explore.pop_front() {
        if check_goal(node) {
            return Some(d);
        }

        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        for neighbor in map.neighbors(node, &reachability) {
            to_explore.push_back((d + 1, neighbor));
        }
    }
    None
}

fn part1(input: &str) -> Result<u64, Error> {
    let map: Map = input.parse()?;

    bfs(&map, map.start, |node| node == map.end, |h, n| n <= h + 1).context("could not find end")
}

fn part2(input: &str) -> Result<u64, Error> {
    let map: Map = input.parse()?;

    bfs(
        &map,
        map.end,
        |node| map.heights[node] == 0,
        |h, n| h <= n + 1,
    )
    .context("could not find end")
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/12")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 31);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 29);
    }

    static TEST_INPUT: &str = "Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi
";
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let y = s.lines().count();
        let x = s.lines().next().context("input contains no lines")?.len();

        let mut heights = Array2::default((x, y));

        let mut start = None;
        let mut end = None;
        for (y, line) in s.lines().enumerate() {
            for (x, mut c) in line.char_indices() {
                if c == 'S' {
                    start = Some((x, y));
                    c = 'a';
                }
                if c == 'E' {
                    end = Some((x, y));
                    c = 'z';
                }

                let h = c as u64 - ('a' as u64);
                heights[(x, y)] = h;
            }
        }

        Ok(Map {
            heights,
            start: start.context("map contained no start")?,
            end: end.context("map contained no end")?,
        })
    }
}
