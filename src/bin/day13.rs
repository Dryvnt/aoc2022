use anyhow::{bail, Error};
use itertools::Itertools;
use std::{cmp::Ordering, fmt::Debug, fs, str::FromStr};

#[derive(Eq, PartialEq)]
enum Packet {
    Literal(u64),
    List(Vec<Packet>),
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        fn compare_slices(left: &[Packet], right: &[Packet]) -> Ordering {
            match (left, right) {
                ([], []) => Ordering::Equal,
                ([], [..]) => Ordering::Less,
                ([..], []) => Ordering::Greater,
                ([l, left @ ..], [r, right @ ..]) => match l.cmp(r) {
                    Ordering::Equal => compare_slices(left, right),
                    o => o,
                },
            }
        }

        match (self, other) {
            (Packet::Literal(left), Packet::Literal(right)) => left.cmp(right),
            (Packet::Literal(left), Packet::List(right)) => {
                compare_slices(&[Packet::Literal(*left)], right.as_slice())
            }
            (Packet::List(left), Packet::Literal(right)) => {
                compare_slices(left.as_slice(), &[Packet::Literal(*right)])
            }
            (Packet::List(left), Packet::List(right)) => {
                compare_slices(left.as_slice(), right.as_slice())
            }
        }
    }
}

fn get_packets(input: &str) -> Result<Vec<Packet>, Error> {
    let packets: Vec<Packet> = input
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.parse())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(packets)
}

fn part1(input: &str) -> Result<usize, Error> {
    let packets = get_packets(input)?;

    Ok(packets
        .iter()
        .tuples()
        .enumerate()
        .filter(|(_, (a, b))| a < b)
        .map(|(i, _)| i + 1)
        .sum())
}

fn part2(input: &str) -> Result<usize, Error> {
    let packets = get_packets(input)?;

    let divider_1 = Packet::List(vec![Packet::List(vec![Packet::Literal(2)])]);
    let divider_2 = Packet::List(vec![Packet::List(vec![Packet::Literal(6)])]);

    // We don't have to sort to find the positions of these :)
    let d1_pos = packets.iter().filter(|p| p < &&divider_1).count() + 1;
    let d2_pos = packets.iter().filter(|p| p < &&divider_2).count() + 2;

    Ok(d1_pos * d2_pos)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/13")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{get_packets, part1, part2, Packet};

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 13);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 140);
    }

    #[test]
    fn packet_sorting() {
        let mut packets = get_packets(TEST_INPUT).unwrap();
        let divider_1 = Packet::List(vec![Packet::List(vec![Packet::Literal(2)])]);
        let divider_2 = Packet::List(vec![Packet::List(vec![Packet::Literal(6)])]);
        packets.push(divider_1);
        packets.push(divider_2);
        packets.sort_unstable();

        let output = packets
            .iter()
            .map(|p| format!("{:?}", p))
            .collect::<Vec<String>>()
            .join("\n");

        assert_eq!(output, EXPECTED_SORTED)
    }

    static TEST_INPUT: &str = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
";

    static EXPECTED_SORTED: &str = "[]
[[]]
[[[]]]
[1,1,3,1,1]
[1,1,5,1,1]
[[1],[2,3,4]]
[1,[2,[3,[4,[5,6,0]]]],8,9]
[1,[2,[3,[4,[5,6,7]]]],8,9]
[[1],4]
[[2]]
[3]
[[4,4],4,4]
[[4,4],4,4,4]
[[6]]
[7,7,7]
[7,7,7,7]
[[8,7,6]]
[9]";
}

impl Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Literal(l) => f.write_fmt(format_args!("{}", l)),
            Packet::List(items) => f.write_fmt(format_args!(
                "[{}]",
                items.iter().map(|item| format!("{:?}", item)).join(",")
            )),
        }
    }
}

// Parsing frustrates me
impl FromStr for Packet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            bail!("malformed input")
        }

        if !(s.starts_with('[') && s.ends_with(']')) {
            return Ok(Packet::Literal(s.parse()?));
        }

        fn parse_list(
            s: &str,
            chars: &[char],
            mut start: usize,
        ) -> Result<(usize, Vec<Packet>), Error> {
            let mut items = Vec::new();
            let mut i = start;
            while i < chars.len() {
                let c = chars[i];
                match c {
                    '[' => {
                        let (sub_end, sub_items) = parse_list(s, chars, i + 1)?;
                        items.push(Packet::List(sub_items));
                        i = sub_end;
                        start = i;
                    }
                    ']' | ',' => {
                        if start != i {
                            let q = &s[start..i];
                            items.push(Packet::Literal(q.parse()?));
                        }
                        start = i + 1;
                        i += 1;
                        if c == ']' {
                            break;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            Ok((i, items))
        }

        let chars: Vec<_> = s.chars().collect();
        let (_, items) = parse_list(s, &chars, 1)?;

        Ok(Packet::List(items))
    }
}
