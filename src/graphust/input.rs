use std::collections::{HashMap, HashSet};

use crate::graphust::domain;

#[derive(Debug)]
struct InnerMapping<'a> {
    source: &'a str,
    arrow: &'a str,
    target: &'a str,
}

const BOX_WIDTH: usize = 4;
const BOX_HEIGHT: usize = 3;
const ARROW_AND_SPACES_WIDTH: usize = 5;

fn add_nodes(map: &mut domain::Map, inner_mapping: &Vec<&InnerMapping>) {
    let mut inner_nodes = inner_mapping.iter().map(|x| x.source).collect::<Vec<_>>();
    inner_nodes.extend(inner_mapping.iter().map(|x| x.target));
    inner_nodes.iter().fold(0, |acc, label| {
        if map.nodes.iter().find(|x| x.1.name == **label).is_some() {
            return acc;
        }
        map.nodes.insert(
            domain::Point { x: acc, y: 0 },
            domain::Node {
                name: (**label).to_owned(),
                border: domain::BorderType::Box,
            },
        );
        acc + BOX_WIDTH + label.len() + ARROW_AND_SPACES_WIDTH
    });
}

enum Anchor {
    Left,
    Right,
    Top,
    Bottom,
}

fn get_point_next_to_box(
    nodes: &HashMap<domain::Point, domain::Node>,
    node_name: &str,
    direction: &Anchor,
) -> domain::Point {
    let node = nodes.iter().find(|x| x.1.name == node_name).unwrap();
    match direction {
        Anchor::Right => domain::Point {
            x: node.0.x + BOX_WIDTH + node.1.name.len() + 1,
            y: node.0.y + 1,
        },
        Anchor::Left => {
            let x = node.0.x as isize - 2;
            domain::Point {
                x: if x < 0 { 0 } else { x as usize },
                y: node.0.y + 1,
            }
        }
        Anchor::Bottom => domain::Point {
            x: node.0.x + 2,
            y: node.0.y + BOX_HEIGHT,
        },
        Anchor::Top => {
            let y = node.0.y as isize - 1;
            domain::Point {
                x: node.0.x + 2,
                y: if y < 0 { 0 } else { y as usize },
            }
        }
    }
}

fn add_arrows(map: &mut domain::Map, inner_mapping: &Vec<&InnerMapping>) {
    inner_mapping.iter().for_each(|x| {
        let source_point = get_point_next_to_box(&map.nodes, x.source, &Anchor::Right);
        let target_point = get_point_next_to_box(&map.nodes, x.target, &Anchor::Left);

        match (source_point, target_point) {
            (s, t) if s.y == t.y && t.x - s.x == 2 => {
                map.arrows.insert(domain::Arrow {
                    middle: domain::Point { x: s.x + 1, y: s.y },
                    start: s,
                    end: t,
                    body: domain::ArrowBody::Basic,
                    head: domain::ArrowHead::Basic,
                });
            }
            (_, _) => (),
        }
    });
}

fn get_inner_mappings(text: &str) -> Vec<Result<InnerMapping, String>> {
    text.lines()
        .map(|line| {
            let parts: Vec<_> = line.split(' ').collect();
            if parts.len() != 3 {
                return Err(format!("Cannot understand this line: {}", line));
            }
            Ok(InnerMapping {
                source: parts[0],
                arrow: parts[1],
                target: parts[2],
            })
        })
        .collect::<Vec<_>>()
}

pub fn read_input(text: &str) -> Result<domain::Map, String> {
    let inner_parts = get_inner_mappings(text);
    if inner_parts.iter().any(|x| x.is_err()) {
        return Err(inner_parts
            .iter()
            .find(|x| x.is_err())
            .unwrap()
            .as_ref()
            .unwrap_err()
            .to_owned());
    }

    let inner_mapping = inner_parts
        .iter()
        .map(|x| x.as_ref().unwrap())
        .collect::<Vec<_>>();
    let mut map = domain::Map {
        nodes: HashMap::new(),
        arrows: HashSet::new(),
    };
    add_nodes(&mut map, &inner_mapping);
    add_arrows(&mut map, &inner_mapping);
    Ok(map)
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn get_map_example01() {
        let input = "A -> B";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 0 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 10, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 6, y: 1 },
            middle: domain::Point { x: 7, y: 1 },
            end: domain::Point { x: 8, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(&input);
        if let Ok(mapped_result) = result {
            assert_eq!(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_error01() {
        let input = "A -> B -> C";
        let result = read_input(&input);
        if let Err(mapped_error) = result {
            assert_eq!("Cannot understand this line: A -> B -> C", mapped_error);
        } else {
            panic!("We were expecting an error for this input!");
        }
    }

    #[test]
    fn get_map_example02() {
        let input = "\
A -> B
B -> C";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 0 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 10, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 20, y: 0 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 6, y: 1 },
            middle: domain::Point { x: 7, y: 1 },
            end: domain::Point { x: 8, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 16, y: 1 },
            middle: domain::Point { x: 17, y: 1 },
            end: domain::Point { x: 18, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(&input);
        if let Ok(mapped_result) = result {
            assert_eq!(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_example03() {
        let input = "\
A -> B
B -> C
C -> A";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 0 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 10, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 20, y: 0 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 6, y: 1 },
            middle: domain::Point { x: 7, y: 1 },
            end: domain::Point { x: 8, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 16, y: 1 },
            middle: domain::Point { x: 17, y: 1 },
            end: domain::Point { x: 18, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 22, y: 3 },
            middle: domain::Point { x: 22, y: 5 },
            end: domain::Point { x: 2, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(&input);
        if let Ok(mapped_result) = result {
            assert_eq!(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }
}
