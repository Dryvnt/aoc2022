use std::{fs, str::FromStr};

use anyhow::{Context, Error};
use ndarray::{Array3, Axis};

#[derive(Default, Debug, Clone, Copy)]
enum State {
    #[default]
    Air,
    Steam,
    Lava,
}

fn try_build_grid(input: &str) -> Result<Array3<State>, Error> {
    let points = input
        .lines()
        .map(|l| {
            let parts = l
                .split(',')
                .map(usize::from_str)
                .collect::<Result<Vec<_>, _>>()?;
            Result::<_, Error>::Ok((parts[0], parts[1], parts[2]))
        })
        .collect::<Result<Vec<(usize, usize, usize)>, _>>()?;

    let max_x = points
        .iter()
        .map(|(x, _, _)| *x)
        .max()
        .context("could not find max x")?;
    let max_y = points
        .iter()
        .map(|(_, y, _)| *y)
        .max()
        .context("could not find max y")?;
    let max_z = points
        .iter()
        .map(|(_, _, z)| *z)
        .max()
        .context("could not find max z")?;

    // We want to have space for the edges to avoid annoying edge-case handling
    // +1 so we can _fit_ the max coord, +1 for low padding, +1 for high padding
    let mut grid = Array3::default([max_x + 3, max_y + 3, max_z + 3]);
    for (x, y, z) in points {
        grid[(x + 1, y + 1, z + 1)] = State::Lava;
    }

    Ok(grid)
}

fn adjacent(p: (usize, usize, usize)) -> [(usize, usize, usize); 6] {
    let (x, y, z) = p;
    [
        (x, y, z - 1),
        (x, y, z + 1),
        (x, y - 1, z),
        (x, y + 1, z),
        (x - 1, y, z),
        (x + 1, y, z),
    ]
}

fn count_exposed_sides(grid: &Array3<State>, counts_as_exposed: impl Fn(State) -> bool) -> usize {
    let mut exposed_sides = 0;

    for w in grid.windows([3, 3, 3]) {
        let p = (1, 1, 1);
        if matches!(w[p], State::Lava) {
            for p in adjacent(p) {
                if counts_as_exposed(w[p]) {
                    exposed_sides += 1;
                }
            }
        }
    }

    exposed_sides
}

fn part1(input: &str) -> Result<usize, Error> {
    let grid = try_build_grid(input)?;

    let exposed_sides = count_exposed_sides(&grid, |p| matches!(p, State::Air));

    Ok(exposed_sides)
}

fn part2(input: &str) -> Result<usize, Error> {
    let mut grid = try_build_grid(input)?;

    // Fill edges with steam to avoid having to worry about index-out-of-bounds edge cases
    for i in 0..3 {
        let axis = Axis(i);
        let len = grid.shape()[i];

        grid.index_axis_mut(axis, 0).fill(State::Steam);
        grid.index_axis_mut(axis, len - 1).fill(State::Steam);
    }

    // Simple BFS
    debug_assert!(matches!(grid[(0, 1, 1)], State::Steam));
    debug_assert!(matches!(grid[(1, 1, 1)], State::Air));
    let mut seen: Array3<bool> = Array3::default(grid.dim());
    let mut stack = vec![(1, 1, 1)];
    while let Some(p) = stack.pop() {
        grid[p] = State::Steam;

        for n in adjacent(p) {
            if matches!(grid[n], State::Air) && !seen[n] {
                seen[n] = true;
                stack.push(n);
            }
        }
    }

    let exposed_sides = count_exposed_sides(&grid, |p| matches!(p, State::Steam));

    Ok(exposed_sides)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/18")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    #[test]
    fn part1_example_small() {
        assert_eq!(part1("1,1,1\n2,1,1\n").unwrap(), 10);
    }

    #[test]
    fn part2_example_small() {
        assert_eq!(part2("1,1,1\n2,1,1\n").unwrap(), 10);
    }

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 64);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 58);
    }

    static TEST_INPUT: &str = "2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5
";
}
