use anyhow::{bail, Error};
use itertools::Itertools;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::digit0, combinator::map_res,
    multi::separated_list0, IResult,
};
use std::{cmp::Ordering, fmt::Debug, fs, str::FromStr};

#[derive(Eq, PartialEq)]
enum Packet {
    Literal(u64),
    List(Vec<Packet>),
}

impl Packet {
    fn as_slice(&self) -> &[Self] {
        if let Self::List(items) = self {
            items.as_slice()
        } else {
            std::slice::from_ref(self)
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        if let (Packet::Literal(left), Packet::Literal(right)) = (self, other) {
            left.cmp(right)
        } else {
            self.as_slice().cmp(other.as_slice())
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

    let divider_1 = "[[2]]".parse()?;
    let divider_2 = "[[6]]".parse()?;

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
    use super::{get_packets, part1, part2};

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
        let divider_1 = "[[2]]".parse().unwrap();
        let divider_2 = "[[6]]".parse().unwrap();
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

impl FromStr for Packet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_list_packet(s: &str) -> IResult<&str, Packet> {
            let (s, _) = tag("[")(s)?;
            let (s, out) = separated_list0(tag(","), parse_packet)(s)?;
            let (s, _) = tag("]")(s)?;

            Ok((s, Packet::List(out)))
        }

        fn parse_literal_packet(s: &str) -> IResult<&str, Packet> {
            let (s, literal) = map_res(digit0, |s: &str| s.parse())(s)?;

            Ok((s, Packet::Literal(literal)))
        }

        fn parse_packet(s: &str) -> IResult<&str, Packet> {
            alt((parse_list_packet, parse_literal_packet))(s)
        }

        match parse_packet(s) {
            Ok((_, p)) => Ok(p),
            Err(error) => bail!("{}", error),
        }
    }
}
