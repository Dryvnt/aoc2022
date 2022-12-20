use std::{clone, collections::HashMap, fmt::Display, fs, str::FromStr};

use anyhow::{anyhow, Error};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, opt},
    sequence::pair,
    IResult,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

impl Resource {
    const fn all() -> [Resource; 4] {
        [
            Resource::Ore,
            Resource::Clay,
            Resource::Obsidian,
            Resource::Geode,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceCollection {
    ore: usize,
    clay: usize,
    obsidian: usize,
}

impl ResourceCollection {
    fn of_type(&self, resource: Resource) -> usize {
        match resource {
            Resource::Ore => self.ore,
            Resource::Clay => self.clay,
            Resource::Obsidian => self.obsidian,
            Resource::Geode => 0,
        }
    }
}

impl Display for ResourceCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut do_and = false;

        for (name, amount) in [
            ("ore", self.ore),
            ("clay", self.clay),
            ("obsidian", self.obsidian),
        ] {
            if amount == 0 {
                continue;
            }

            if do_and {
                f.write_str(" and ")?;
            }
            do_and = true;
            f.write_fmt(format_args!("{} {}", amount, name))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Blueprint {
    number: usize,
    ore_robot: ResourceCollection,
    clay_robot: ResourceCollection,
    obsidian_robot: ResourceCollection,
    geode_robot: ResourceCollection,

    most_expensive: ResourceCollection,
}

impl Blueprint {
    fn new(
        number: usize,
        ore: ResourceCollection,
        clay: ResourceCollection,
        obsidian: ResourceCollection,
        geode: ResourceCollection,
    ) -> Self {
        let mut b = Blueprint {
            number,
            ore_robot: ore,
            clay_robot: clay,
            obsidian_robot: obsidian,
            geode_robot: geode,
            most_expensive: ResourceCollection {
                ore: 0,
                clay: 0,
                obsidian: 0,
            },
        };

        let expensive: HashMap<_, _> = Resource::all()
            .into_iter()
            .map(|r0| {
                Resource::all()
                    .into_iter()
                    .map(|r1| (r0, b.cost_of(r1).of_type(r0)))
                    .max()
                    .unwrap()
            })
            .collect();

        b.most_expensive = ResourceCollection {
            ore: expensive[&Resource::Ore],
            clay: expensive[&Resource::Clay],
            obsidian: expensive[&Resource::Obsidian],
        };

        b
    }
    fn cost_of(&self, r: Resource) -> &ResourceCollection {
        match r {
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

    resources: ResourceCollection,
    robots: ResourceCollection,

    blueprint: &'a Blueprint,
    relevant: Vec<Resource>,
}

impl<'a> State<'a> {
    fn new(blueprint: &'a Blueprint, max_ticks: usize) -> Self {
        State {
            remaining_ticks: max_ticks,
            score: 0,
            resources: ResourceCollection {
                ore: 0,
                clay: 0,
                obsidian: 0,
            },
            robots: ResourceCollection {
                ore: 1,
                clay: 0,
                obsidian: 0,
            },
            blueprint,
            relevant: Resource::all().into(),
        }
    }

    fn can_afford(&self, cost: &ResourceCollection) -> bool {
        self.resources.ore >= cost.ore
            && self.resources.clay >= cost.clay
            && self.resources.obsidian >= cost.obsidian
    }

    fn ticks_until_afford(&self, cost: &ResourceCollection) -> Option<usize> {
        fn weird_div_ceil(a: usize, b: usize) -> Option<usize> {
            if a == 0 {
                Some(0)
            } else {
                ((a + b) - 1).checked_div(b)
            }
        }

        let ore_ticks =
            weird_div_ceil(cost.ore.saturating_sub(self.resources.ore), self.robots.ore)?;
        let clay_ticks = weird_div_ceil(
            cost.clay.saturating_sub(self.resources.clay),
            self.robots.clay,
        )?;
        let obsidian_ticks = weird_div_ceil(
            cost.obsidian.saturating_sub(self.resources.obsidian),
            self.robots.obsidian,
        )?;

        Some(ore_ticks.max(clay_ticks).max(obsidian_ticks))
    }

    fn build_robot(&mut self, r: Resource) {
        match r {
            Resource::Ore => self.robots.ore += 1,
            Resource::Clay => self.robots.clay += 1,
            Resource::Obsidian => self.robots.obsidian += 1,
            Resource::Geode => {
                self.score += self.remaining_ticks;
            }
        };
    }

    fn try_pay(&mut self, cost: &ResourceCollection) -> bool {
        if !self.can_afford(cost) {
            return false;
        }
        self.resources.ore -= cost.ore;
        self.resources.clay -= cost.clay;
        self.resources.obsidian -= cost.obsidian;

        true
    }

    fn tick(&mut self, n_ticks: usize) {
        self.remaining_ticks -= n_ticks;
        self.resources.ore += self.robots.ore * n_ticks;
        self.resources.clay += self.robots.clay * n_ticks;
        self.resources.obsidian += self.robots.obsidian * n_ticks;

        self.relevant.retain(|&r| {
            let highest_cost = self.blueprint.most_expensive.of_type(r);
            let max_required = self.remaining_ticks * highest_cost;
            let min_available =
                self.resources.of_type(r) + self.remaining_ticks * self.robots.of_type(r);
            max_required <= min_available
        });
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
        let mut cheaper_obsidian = self.blueprint.obsidian_robot.clone();
        cheaper_obsidian.ore = 0;
        let mut cheaper_geode = self.blueprint.geode_robot.clone();
        cheaper_geode.ore = 0;

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

        for &r in &state.relevant {
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

    use itertools::Itertools;

    use crate::{explore_blueprint, part1, Blueprint};

    #[test]
    fn parsing() {
        let blueprints = TEST_INPUT
            .lines()
            .map(Blueprint::from_str)
            .collect::<Result<Vec<Blueprint>, _>>()
            .unwrap();

        let blueprints_str = blueprints.iter().map(|b| format!("{}\n", b)).join("");

        assert_eq!(blueprints_str, TEST_INPUT);
    }

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

impl Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Blueprint {}: Each ore robot costs {}. Each clay robot costs {}. Each obsidian robot costs {}. Each geode robot costs {}.", self.number, self.ore_robot, self.clay_robot, self.obsidian_robot, self.geode_robot))
    }
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

        fn parse_cost(s: &str) -> IResult<&str, ResourceCollection> {
            let (s, ore) = parse_cost_part("ore")(s)?;

            let (s, opt_clay) = opt(pair(tag(" and "), parse_cost_part("clay")))(s)?;
            let (s, opt_obsidian) = opt(pair(tag(" and "), parse_cost_part("obsidian")))(s)?;

            let clay = opt_clay.map(|(_, v)| v).unwrap_or_default();
            let obsidian = opt_obsidian.map(|(_, v)| v).unwrap_or_default();

            Ok((
                s,
                ResourceCollection {
                    ore,
                    clay,
                    obsidian,
                },
            ))
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
