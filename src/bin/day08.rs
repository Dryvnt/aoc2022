use anyhow::{Context, Error};
use ndarray::{Array2, Axis};
use std::{
    fs,
    ops::{Index, IndexMut},
};

fn parse_heights(input: &str) -> Result<Array2<usize>, Error> {
    let mut trees_vec = Vec::new();
    let max_x = input.lines().next().unwrap().chars().count();
    let max_y = input.lines().count();
    for line in input.lines() {
        for c in line.chars() {
            let height = c
                .to_digit(10)
                .with_context(|| format!("could not convert char '{}' to tree height", c))?
                as usize;

            trees_vec.push(height);
        }
    }

    Ok(Array2::from_shape_vec((max_x, max_y), trees_vec)?)
}

fn calculate_visibility(heights: &Array2<usize>) -> Result<Array2<bool>, Error> {
    let shape = heights.shape();
    let mut visibility = Array2::default([shape[0], shape[1]]);
    for axis in [Axis(0), Axis(1)] {
        for (axis_idx, height_axis) in heights.axis_iter(axis).enumerate() {
            fn sweep_axis(
                visiblity: &mut impl IndexMut<usize, Output = bool>,
                height: &impl Index<usize, Output = usize>,
                mut r: impl Iterator<Item = usize>,
            ) {
                let start = r.next().unwrap();
                visiblity[start] = true;
                let mut max_previous = height[start];
                for i in r {
                    let other = height[i];
                    if other > max_previous {
                        visiblity[i] = true;
                        max_previous = other;
                    }
                }
            }

            let axis_len = height_axis.len();
            let mut visibility_axis = visibility.index_axis_mut(axis, axis_idx);
            sweep_axis(&mut visibility_axis, &height_axis, 0..axis_len);
            sweep_axis(&mut visibility_axis, &height_axis, (0..axis_len).rev());
        }
    }

    Ok(visibility)
}

fn calculate_scores(heights: &Array2<usize>) -> Result<Array2<usize>, Error> {
    let shape = heights.shape();
    let mut scores = Vec::with_capacity(heights.iter().count());
    for ((cursor_x, cursor_y), &height) in heights.indexed_iter() {
        fn find_distance(
            tree_height: usize,
            heights: &impl Index<usize, Output = usize>,
            r: impl Iterator<Item = usize>,
        ) -> usize {
            let mut count = 0;
            for other in r {
                count += 1;
                let other_height = heights[other];
                if other_height >= tree_height {
                    break;
                }
            }
            count
        }

        let x_column = heights.index_axis(Axis(0), cursor_x);
        let y_row = heights.index_axis(Axis(1), cursor_y);

        let l = find_distance(height, &y_row, (0..cursor_x).rev());
        let r = find_distance(height, &y_row, cursor_x + 1..shape[0]);
        let u = find_distance(height, &x_column, cursor_y + 1..shape[1]);
        let d = find_distance(height, &x_column, (0..cursor_y).rev());

        scores.push(l * r * u * d);
    }

    Ok(Array2::from_shape_vec([shape[0], shape[1]], scores)?)
}

fn part1(input: &str) -> Result<usize, Error> {
    let forest = parse_heights(input)?;
    let visibility = calculate_visibility(&forest)?;

    Ok(visibility.iter().filter(|&&visible| visible).count())
}

fn part2(input: &str) -> Result<usize, Error> {
    let forest = parse_heights(input)?;
    let scores = calculate_scores(&forest)?;

    scores.iter().max().copied().context("could not find max")
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/8")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    static TEST_INPUT: &str = "30373
25512
65332
33549
35390
";

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 21);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 8);
    }
}
