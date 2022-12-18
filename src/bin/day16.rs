use anyhow::{bail, Error};
use bitvec::prelude::{BitArray, Lsb0};
use itertools::{iproduct, Itertools};
use ndarray::Array3;
use std::{cmp::Reverse, collections::HashMap, fs};

#[derive(Debug)]
struct Valve<'a> {
    name: &'a str,
    flow: u16,
    reachable: Vec<&'a str>,
}

impl<'a> TryFrom<&'a str> for Valve<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let words = s.split_ascii_whitespace().collect_vec();
        let name = words[1];
        let rate = words[4][5..words[4].len() - 1].parse::<u16>()?;
        let neighbors = words[9..].iter().map(|&n| n.strip_suffix(',').unwrap_or(n));

        Ok(Valve {
            name,
            flow: rate,
            reachable: neighbors.collect(),
        })
    }
}

struct SolveContext<'a> {
    valves: Vec<Valve<'a>>,
    name_idx: HashMap<&'a str, usize>,
    adjacency: Vec<Vec<usize>>,
    n_with_flow: usize,
    score: Array3<u16>,
}

impl<'a> TryFrom<&'a str> for SolveContext<'a> {
    type Error = Error;

    fn try_from(input: &'a str) -> Result<Self, Error> {
        let mut valves = input
            .lines()
            .map(|l| l.try_into())
            .collect::<Result<Vec<Valve>, _>>()?;

        valves.sort_unstable_by_key(|v| Reverse((v.flow, v.name == "AA")));

        let name_idx: HashMap<_, _> = valves
            .iter()
            .enumerate()
            .map(|(i, v)| (v.name, i))
            .collect();

        let adjacency = valves
            .iter()
            .map(|v| v.reachable.iter().map(|r| name_idx[r]).collect())
            .collect();

        let n_with_flow = valves.iter().filter(|v| v.flow > 0).count();

        if n_with_flow > 16 {
            bail!("too many nodes with flow for u32 bitset to handle");
        }
        let opened_set_size = 1 << n_with_flow;

        let score = Array3::default([30, valves.len(), opened_set_size]);

        Ok(SolveContext {
            name_idx,
            valves,
            n_with_flow,
            adjacency,
            score,
        })
    }
}

impl<'a> SolveContext<'a> {
    fn score_for_opening(
        &self,
        time_remaining: usize,
        standing_at: usize,
        opened: BitArray<usize>,
    ) -> u16 {
        let opening_score = self.valves[standing_at].flow * time_remaining as u16;

        let mut opened = opened;
        opened.set(standing_at, true);

        let afterwards_score = self.score[(time_remaining - 1, standing_at, opened.into_inner())];

        opening_score + afterwards_score
    }

    fn score_for_moving(
        &self,
        time_remaining: usize,
        moving_to: usize,
        opened: BitArray<usize>,
    ) -> u16 {
        self.score[(time_remaining - 1, moving_to, opened.into_inner())]
    }

    // dynamic programming ftw
    fn solve(&mut self) {
        let shape = self.score.shape().to_owned();
        for (time_remaining, standing_at, opened) in iproduct!(
            (1..shape[0]),
            (0..shape[1]),
            (0..shape[2]).map(BitArray::<usize, Lsb0>::from)
        ) {
            // Carry over score from doing nothing, it's the worst we can do
            let mut score = self.score[(time_remaining - 1, standing_at, opened.into_inner())];

            // Note: The order of nodes is important, we use it to determine if a given idx can be opened
            if standing_at < self.n_with_flow && !opened[standing_at] {
                score = score.max(self.score_for_opening(time_remaining, standing_at, opened));
            }

            for &moving_to in &self.adjacency[standing_at] {
                score = score.max(self.score_for_moving(time_remaining, moving_to, opened));
            }

            self.score[(time_remaining, standing_at, opened.into_inner())] = score;
        }
    }
}

fn part1(ctx: &SolveContext) -> Result<u16, Error> {
    Ok(ctx.score[(29, ctx.name_idx["AA"], 0)])
}

fn part2(ctx: &SolveContext) -> Result<u16, Error> {
    let opened_set_size = ctx.score.shape()[2];
    let mask = !(usize::MAX << ctx.n_with_flow);
    let mut max_score = 0;
    for human_ignore in 0..opened_set_size {
        let elephant_ignore = (!human_ignore) & mask;

        let human_score = ctx.score[(25, ctx.name_idx["AA"], human_ignore)];
        let elephant_score = ctx.score[(25, ctx.name_idx["AA"], elephant_ignore)];
        let score = human_score + elephant_score;
        max_score = max_score.max(score);
    }
    Ok(max_score)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/16")?;
    let mut ctx = SolveContext::try_from(input.as_str())?;
    ctx.solve();

    println!("Part 1: {}", part1(&ctx)?);
    println!("Part 2: {}", part2(&ctx)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{part1, part2, SolveContext};

    #[test]
    fn part1_example() {
        let mut ctx = SolveContext::try_from(TEST_INPUT).unwrap();
        ctx.solve();
        assert_eq!(part1(&ctx).unwrap(), 1651);
    }

    #[test]
    fn part2_example() {
        let mut ctx = SolveContext::try_from(TEST_INPUT).unwrap();
        ctx.solve();
        assert_eq!(part2(&ctx).unwrap(), 1707);
    }

    static TEST_INPUT: &str = "Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II
";
}
