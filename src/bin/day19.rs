use std::{
    fs,
    ops::{Index, IndexMut},
    str::FromStr,
};

use anyhow::{anyhow, Error};
use itertools::iproduct;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, opt},
    sequence::pair,
    IResult,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(usize)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

impl Resource {
    const fn all() -> [Resource; 4] {
        // Order matters - The faster we force ourselves up the tech tree, the earlier we get good lower bounds
        [
            Resource::Geode,
            Resource::Obsidian,
            Resource::Clay,
            Resource::Ore,
        ]
    }
}

impl Index<Resource> for [usize; 3] {
    type Output = usize;

    fn index(&self, index: Resource) -> &Self::Output {
        match index {
            Resource::Ore => &self[0],
            Resource::Clay => &self[1],
            Resource::Obsidian => &self[2],
            Resource::Geode => &0,
        }
    }
}

impl IndexMut<Resource> for [usize; 3] {
    fn index_mut(&mut self, index: Resource) -> &mut Self::Output {
        match index {
            Resource::Ore => &mut self[0],
            Resource::Clay => &mut self[1],
            Resource::Obsidian => &mut self[2],
            Resource::Geode => todo!(),
        }
    }
}

#[derive(Debug)]
struct Blueprint {
    number: usize,
    ore_robot: [usize; 3],
    clay_robot: [usize; 3],
    obsidian_robot: [usize; 3],
    geode_robot: [usize; 3],

    most_expensive: [usize; 3],
}

impl Blueprint {
    fn new(
        number: usize,
        ore: [usize; 3],
        clay: [usize; 3],
        obsidian: [usize; 3],
        geode: [usize; 3],
    ) -> Self {
        let mut b = Blueprint {
            number,
            ore_robot: ore,
            clay_robot: clay,
            obsidian_robot: obsidian,
            geode_robot: geode,
            most_expensive: [0, 0, 0],
        };

        for (r0, r1) in iproduct!(
            [Resource::Ore, Resource::Clay, Resource::Obsidian],
            Resource::all()
        ) {
            b.most_expensive[r0] = b.most_expensive[r0].max(b[r1][r0]);
        }

        b
    }
    fn cost_of(&self, r: Resource) -> &[usize; 3] {
        &self[r]
    }
}

impl Index<Resource> for Blueprint {
    type Output = [usize; 3];

    fn index(&self, index: Resource) -> &Self::Output {
        match index {
            Resource::Ore => &self.ore_robot,
            Resource::Clay => &self.clay_robot,
            Resource::Obsidian => &self.obsidian_robot,
            Resource::Geode => &self.geode_robot,
        }
    }
}

#[derive(Debug, Clone)]
struct State<'a> {
    remaining_ticks: usize,
    score: usize,

    resources: [usize; 3],
    robots: [usize; 3],

    blueprint: &'a Blueprint,
    relevant: [Option<Resource>; 4],
}

impl<'a> State<'a> {
    fn new(blueprint: &'a Blueprint, max_ticks: usize) -> Self {
        State {
            remaining_ticks: max_ticks,
            score: 0,
            resources: [0, 0, 0],
            robots: [1, 0, 0],
            blueprint,
            relevant: Resource::all().map(Some),
        }
    }

    fn can_afford(&self, cost: &[usize; 3]) -> bool {
        self.resources.iter().zip(cost.iter()).all(|(r, c)| r >= c)
    }

    fn ticks_until_afford(&self, cost: &[usize; 3]) -> Option<usize> {
        fn weird_div_ceil(a: usize, b: usize) -> Option<usize> {
            if a == 0 {
                Some(0)
            } else {
                ((a + b) - 1).checked_div(b)
            }
        }

        let mut max = 0;

        for (&c, (&res, &rob)) in cost
            .iter()
            .zip(self.resources.iter().zip(self.robots.iter()))
        {
            let v = weird_div_ceil(c.saturating_sub(res), rob)?;
            max = max.max(v);
        }
        Some(max)
    }

    fn build_robot(&mut self, r: Resource) {
        match r {
            Resource::Ore => self.robots[0] += 1,
            Resource::Clay => self.robots[1] += 1,
            Resource::Obsidian => self.robots[2] += 1,
            Resource::Geode => {
                self.score += self.remaining_ticks;
            }
        };
    }

    fn try_pay(&mut self, cost: &[usize; 3]) -> bool {
        if !self.can_afford(cost) {
            return false;
        }

        for (r, c) in self.resources.iter_mut().zip(cost.iter()) {
            *r -= c;
        }

        true
    }

    fn tick(&mut self, n_ticks: usize) {
        self.remaining_ticks -= n_ticks;
        for i in 0..3 {
            self.resources[i] += self.robots[i] * n_ticks;
        }

        for o in self.relevant.iter_mut() {
            if matches!(o, Some(Resource::Geode)) {
                continue;
            }

            if let Some(r) = o {
                let r = *r;
                let highest_cost = self.blueprint.most_expensive[r];
                let max_required = self.remaining_ticks * highest_cost;
                let min_available = self.resources[r] + self.remaining_ticks * self.robots[r];
                if max_required <= min_available {
                    *o = None;
                }
            }
        }
    }

    fn try_wait_and_build(&mut self, r: Resource) -> bool {
        let cost = self.blueprint.cost_of(r);
        if let Some(wait) = self
            .ticks_until_afford(cost)
            .filter(|&wait| wait < self.remaining_ticks)
        {
            self.tick(wait);
            debug_assert!(self.can_afford(cost));
            self.try_pay(cost);
            self.tick(1);
            self.build_robot(r);
            true
        } else {
            false
        }
    }

    fn upper_bound(&self) -> usize {
        let mut cheaper_obsidian = self.blueprint.obsidian_robot;
        cheaper_obsidian[0] = 0;
        let mut cheaper_geode = self.blueprint.geode_robot;
        cheaper_geode[0] = 0;

        let mut s = self.clone();
        while s.remaining_ticks > 0 {
            let build_clay = s.try_pay(&self.blueprint.clay_robot);
            let build_obsidian = s.try_pay(&cheaper_geode);
            let build_geode = s.try_pay(&cheaper_obsidian);

            s.tick(1);
            s.build_robot(Resource::Ore);
            if build_clay {
                s.build_robot(Resource::Clay);
            }
            if build_obsidian {
                s.build_robot(Resource::Obsidian);
            }
            if build_geode {
                s.build_robot(Resource::Geode);
            }
        }

        s.score
    }
}

// Several ideas going on here:
// * Branch and bound
// * Branch on resource-next-buildable instead of next-time
// * Restrict resources to those that will provide an actual benefit
fn explore_blueprint(b: &Blueprint, max_ticks: usize) -> usize {
    let mut stack = Vec::new();
    stack.push(State::new(b, max_ticks));

    let mut best_lower_bound = stack[0].score;
    while let Some(state) = stack.pop() {
        let lower_bound = state.score;
        best_lower_bound = best_lower_bound.max(lower_bound);

        for r in state.relevant.iter().filter_map(|&r| r) {
            let mut state = state.clone();
            if state.try_wait_and_build(r) {
                let upper_bound = state.upper_bound();
                if upper_bound > best_lower_bound {
                    stack.push(state);
                }
            }
        }
    }

    best_lower_bound
}

fn parse_blueprints(input: &str) -> Result<Vec<Blueprint>, Error> {
    input.lines().map(Blueprint::from_str).collect()
}

fn part1(input: &str) -> Result<usize, Error> {
    let blueprints = parse_blueprints(input)?;

    let s = blueprints
        .iter()
        .map(|b| explore_blueprint(b, 24) * b.number)
        .sum::<usize>();

    Ok(s)
}

fn part2(input: &str) -> Result<usize, Error> {
    let blueprints = parse_blueprints(input)?;

    let s = blueprints
        .iter()
        .take(3)
        .map(|b| explore_blueprint(b, 32))
        .product::<usize>();

    Ok(s)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/19")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{explore_blueprint, part1, Blueprint};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 33);
    }

    #[test]
    fn part2_example() {
        let b = TEST_INPUT
            .lines()
            .map(Blueprint::from_str)
            .collect::<Result<Vec<Blueprint>, _>>()
            .unwrap();
        assert_eq!(explore_blueprint(&b[0], 32), 56);
        assert_eq!(explore_blueprint(&b[1], 32), 62);
    }

    static TEST_INPUT: &str = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.
";
}

impl FromStr for Blueprint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_cost_part<'a>(t: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, usize> {
            move |s: &'a str| {
                let (s, amount) = map_res(digit1, |s: &str| s.parse())(s)?;
                let (s, _) = tag(" ")(s)?;
                let (s, _) = tag(t)(s)?;
                Ok((s, amount))
            }
        }

        fn parse_cost(s: &str) -> IResult<&str, [usize; 3]> {
            let (s, ore) = parse_cost_part("ore")(s)?;

            let (s, opt_clay) = opt(pair(tag(" and "), parse_cost_part("clay")))(s)?;
            let (s, opt_obsidian) = opt(pair(tag(" and "), parse_cost_part("obsidian")))(s)?;

            let clay = opt_clay.map(|(_, v)| v).unwrap_or_default();
            let obsidian = opt_obsidian.map(|(_, v)| v).unwrap_or_default();

            Ok((s, [ore, clay, obsidian]))
        }
        fn parse_blueprint(s: &str) -> IResult<&str, Blueprint> {
            let (s, _) = tag("Blueprint ")(s)?;
            let (s, number) = map_res(digit1, |s: &str| s.parse())(s)?;
            let (s, _) = tag(": Each ore robot costs ")(s)?;
            let (s, ore_robot) = parse_cost(s)?;
            let (s, _) = tag(". Each clay robot costs ")(s)?;
            let (s, clay_robot) = parse_cost(s)?;
            let (s, _) = tag(". Each obsidian robot costs ")(s)?;
            let (s, obsidian_robot) = parse_cost(s)?;
            let (s, _) = tag(". Each geode robot costs ")(s)?;
            let (s, geode_robot) = parse_cost(s)?;

            Ok((
                s,
                Blueprint::new(number, ore_robot, clay_robot, obsidian_robot, geode_robot),
            ))
        }

        let (_, blueprint) =
            parse_blueprint(s).map_err(|e| anyhow!("could not parse blueprint: {:?}", e))?;

        Ok(blueprint)
    }
}
