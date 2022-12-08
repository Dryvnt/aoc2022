use std::collections::HashMap;
use std::fs;
use std::iter::Peekable;

use anyhow::{bail, Context, Error};
use itertools::Itertools;

#[derive(Debug)]
enum Node<'a> {
    Directory {
        size: u64,
        items: HashMap<&'a str, Node<'a>>,
    },
    File {
        size: u64,
    },
}

impl<'a> Node<'a> {
    fn size(&self) -> &u64 {
        match self {
            Node::Directory { size, items: _ } => size,
            Node::File { size } => size,
        }
    }
    fn parse_dir(lines: &mut Peekable<impl Iterator<Item = &'a str>>) -> Result<Self, Error> {
        let mut items = HashMap::new();

        while let Some(line) = lines.next() {
            match line.split_ascii_whitespace().collect_vec().as_slice() {
                ["$", "ls"] => {
                    let file_items = lines
                        .peeking_take_while(|line| !line.starts_with('$'))
                        .filter(|line| !line.starts_with("dir"))
                        .filter_map(|line| line.split_once(' '));

                    for (size, name) in file_items {
                        items.insert(
                            name,
                            Self::File {
                                size: size.parse()?,
                            },
                        );
                    }
                }
                ["$", "cd", ".."] => break,
                ["$", "cd", "/"] => bail!("did not expect to go to root oops"),
                ["$", "cd", rel] => {
                    let child = Self::parse_dir(lines)?;
                    items.insert(*rel, child);
                }
                _ => bail!("unrecognized command {}", line),
            }
        }

        let size = items.values().map(|node| node.size()).sum();

        Ok(Node::Directory { items, size })
    }

    fn parse_root(input: &'a str) -> Result<Self, Error> {
        let mut lines = input.lines();
        assert_eq!(lines.next(), Some("$ cd /"));

        Self::parse_dir(&mut lines.peekable())
    }

    fn dir_sizes(&self) -> HashMap<String, u64> {
        fn inner(n: &Node, path: String, out: &mut HashMap<String, u64>) {
            match n {
                Node::Directory { size, items } => {
                    out.insert(path.clone(), *size);
                    for (name, item) in items {
                        let mut other_path = path.clone();
                        other_path.push_str(name);
                        other_path.push('/');
                        inner(item, other_path, out);
                    }
                }
                Node::File { size: _ } => {}
            }
        }
        let mut out = HashMap::new();
        inner(self, "/".to_string(), &mut out);
        out
    }
}

fn part1(input: &str) -> Result<u64, Error> {
    let root = Node::parse_root(input)?;

    let sum = root
        .dir_sizes()
        .values()
        .filter(|&&size| size <= 100000)
        .sum();

    Ok(sum)
}

fn part2(input: &str) -> Result<u64, Error> {
    let root = Node::parse_root(input)?;

    let sizes = root.dir_sizes();

    const TOTAL_SPACE: u64 = 70000000;
    const REQUIRED_SPACE: u64 = 30000000;
    let used_space = sizes[&"/".to_string()];
    let min_size = REQUIRED_SPACE - (TOTAL_SPACE - used_space);

    let min = *sizes
        .values()
        .filter(|&&size| size >= min_size)
        .min()
        .context("no min value found?")?;

    Ok(min)
}

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input/7")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{part1, part2};

    static TEST_INPUT: &str = "$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    #[test]
    fn part1_example() {
        assert_eq!(part1(TEST_INPUT).unwrap(), 95437);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(TEST_INPUT).unwrap(), 24933642);
    }
}
