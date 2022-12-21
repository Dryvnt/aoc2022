use std::{collections::HashSet, fmt::Display, fs, str::FromStr};

use anyhow::{anyhow, Error};
use itertools::Itertools;

// Doubly linked list
#[derive(Debug)]
struct List {
    zero_idx: usize,
    entries: Vec<ListEntry>,
}
#[derive(Debug)]
struct ListEntry {
    value: i64,
    prev: usize,
    next: usize,
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        let mut seen = HashSet::new();
        let mut cursor = 0;
        while !seen.contains(&cursor) {
            seen.insert(cursor);
            parts.push(self.entries[cursor].value);
            cursor = self.entries[cursor].next;
        }

        f.debug_list().entries(parts.iter()).finish()
    }
}

impl FromStr for List {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .lines()
            .map(|l| {
                l.parse::<i64>()
                    .map_err(|e| anyhow!("could parse input: {:?}", e))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let entries: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(pos, &value)| ListEntry {
                value,
                prev: if pos > 0 { pos - 1 } else { values.len() - 1 },
                next: if pos < values.len() - 1 { pos + 1 } else { 0 },
            })
            .collect();
        let zero_idx = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.value == 0)
            .map(|(i, _)| i)
            .exactly_one()
            .map_err(|e| anyhow!("could not find index of zero: {}", e))?;

        Ok(List { zero_idx, entries })
    }
}

impl List {
    fn remove(&mut self, i: usize) {
        let prev = self.entries[i].prev;
        let next = self.entries[i].next;
        self.entries[next].prev = prev;
        self.entries[prev].next = next;
    }

    fn seek(&self, mut i: usize, mut amount: i64, pretend_removed: bool) -> usize {
        amount %= if pretend_removed {
            self.entries.len() - 1
        } else {
            self.entries.len()
        } as i64;

        while amount > 0 {
            amount -= 1;
            i = self.entries[i].next;
        }
        if amount < 0 {
            amount -= 1;
        }
        while amount < 0 {
            amount += 1;
            i = self.entries[i].prev;
        }

        i
    }

    fn insert(&mut self, i: usize, new_prev: usize) {
        let new_next = self.entries[new_prev].next;
        self.entries[new_prev].next = i;
        self.entries[new_next].prev = i;

        self.entries[i].next = new_next;
        self.entries[i].prev = new_prev;
    }

    fn mix(&mut self) {
        for i in 0..self.entries.len() {
            let value = self.entries[i].value;
            let new_prev = self.seek(i, value, true);
            if new_prev == i {
                continue;
            }
            self.remove(i);
            self.insert(i, new_prev);
        }
    }

    fn score_coordinates(&self) -> i64 {
        [1000, 2000, 3000]
            .into_iter()
            .map(|a| self.entries[self.seek(self.zero_idx, a, false)].value)
            .sum::<i64>()
    }
}

fn part1(input: &str) -> Result<i64, Error> {
    let mut list = input.parse::<List>()?;
    list.mix();

    Ok(list.score_coordinates())
}

fn part2(input: &str) -> Result<i64, Error> {
    let mut list = input.parse::<List>()?;
    list.entries.iter_mut().for_each(|i| i.value *= 811589153);
    for _ in 0..10 {
        list.mix();
    }

    Ok(list.score_coordinates())
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/20")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{part1, part2};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 3);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 1623178306);
    }

    static TEST_INPUT: &str = "1
2
-3
3
-2
0
4
";
}
