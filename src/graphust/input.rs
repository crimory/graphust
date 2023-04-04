use crate::graphust::domain;
use std::collections::{HashMap, HashSet};

mod force_directed_graph;

#[derive(Debug)]
struct InnerMapping {
    source: String,
    arrow: String,
    target: String,
}

const BOX_WIDTH: usize = 4;
const BOX_HEIGHT: usize = 3;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Direction {
    Top,
    Bottom,
    Left,
    Right,
}

fn read_arrow_body(arrow: &str) -> domain::ArrowBody {
    match arrow {
        a if a.starts_with('-') => domain::ArrowBody::Basic,
        _ => domain::ArrowBody::Basic,
    }
}

fn read_arrow_head(arrow: &str) -> domain::ArrowHead {
    match arrow {
        a if a.ends_with('>') => domain::ArrowHead::Basic,
        _ => domain::ArrowHead::Basic,
    }
}

fn get_line_parts_respecting_quotes(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut in_quotes = false;
    for c in line.chars() {
        if c == ' ' && !in_quotes {
            parts.push(current_part);
            current_part = String::new();
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else {
            current_part.push(c);
        }
    }
    parts.push(current_part);
    parts
}

fn get_inner_mappings(text: &str) -> Vec<Result<InnerMapping, String>> {
    text.lines()
        .map(|line| {
            let parts = get_line_parts_respecting_quotes(line);
            if parts.len() != 3 {
                return Err(format!("Cannot understand this line: {}", line));
            }
            let arrow_correct_direction = !parts[1].starts_with('<');
            if arrow_correct_direction {
                Ok(InnerMapping {
                    source: parts[0].to_owned(),
                    arrow: parts[1].to_owned(),
                    target: parts[2].to_owned(),
                })
            } else {
                Ok(InnerMapping {
                    source: parts[2].to_owned(),
                    arrow: parts[1].chars().rev().collect::<String>(),
                    target: parts[0].to_owned(),
                })
            }
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

    let mut force_directed_graph = force_directed_graph::Graph::new();
    for mapping in &inner_mapping {
        force_directed_graph.add_node(&mapping.source);
        force_directed_graph.add_node(&mapping.target);
        force_directed_graph.add_edge(&mapping.source, &mapping.target);
    }
    let approximation = force_directed_graph.force_directed(None, None);
    let mut map = domain::Map {
        nodes: HashMap::new(),
        arrows: HashSet::new(),
    };
    include_nodes(&mut map, &approximation);
    include_arrows(&mut map, &inner_mapping);
    Ok(map)
}

fn include_nodes(
    map: &mut domain::Map,
    approximation: &Vec<force_directed_graph::NodeApproximation>,
) {
    for node in approximation {
        map.nodes.insert(
            domain::Point {
                x: node.position.x * 4,
                y: node.position.y,
            },
            domain::Node {
                name: node.name.to_owned(),
                border: domain::BorderType::Box,
            },
        );
    }
}

struct ArrowAnchorsForNode<'a> {
    anchor: &'a domain::Point,
    node_label: &'a str,
    used_anchors: HashMap<Direction, usize>,
}
impl ArrowAnchorsForNode<'_> {
    fn new<'a>(node_label: &'a str, anchor: &'a domain::Point) -> ArrowAnchorsForNode<'a> {
        let mut used_anchors = HashMap::new();
        used_anchors.insert(Direction::Top, 2);
        used_anchors.insert(Direction::Bottom, 2);
        used_anchors.insert(Direction::Left, 1);
        used_anchors.insert(Direction::Right, 1);
        ArrowAnchorsForNode {
            anchor,
            node_label,
            used_anchors,
        }
    }
    fn get_arrow_anchor_offset(&mut self, direction: Direction) -> usize {
        let output = self.used_anchors.get(&direction).unwrap().to_owned();
        let next_output = match (direction, output) {
            (Direction::Left, 2) | (Direction::Right, 2) => 0,
            _ => output + 1,
        };
        self.used_anchors.insert(direction, next_output);
        output
    }
    fn get_arrow_anchor(&mut self, direction: Direction) -> domain::Point {
        let offset = self.get_arrow_anchor_offset(direction);
        let (x, y) = match direction {
            Direction::Top => (self.anchor.x + offset, self.anchor.y - 1),
            Direction::Bottom => (self.anchor.x + offset, self.anchor.y + BOX_HEIGHT),
            Direction::Left => (self.anchor.x - 1, self.anchor.y + offset),
            Direction::Right => (
                self.anchor.x + BOX_WIDTH + self.node_label.len(),
                self.anchor.y + offset,
            ),
        };
        domain::Point { x, y }
    }
}

fn include_arrows(map: &mut domain::Map, inner_mappings: &Vec<&InnerMapping>) {
    let mut anchors_for_nodes = vec![];
    for (anchor, node) in map.nodes.iter() {
        anchors_for_nodes.push(ArrowAnchorsForNode::new(&node.name, anchor));
    }
    for mapping in inner_mappings {
        let mut mutable_nodes = anchors_for_nodes
            .iter_mut()
            .filter(|node| node.node_label == mapping.source || node.node_label == mapping.target);
        let (node_from, node_to) = match (mutable_nodes.next(), mutable_nodes.next()) {
            (Some(from), Some(to)) if from.node_label == mapping.source => (from, to),
            (Some(to), Some(from)) if from.node_label == mapping.source => (from, to),
            _ => panic!("Could not find nodes for arrow"),
        };

        let (arrow_start, arrow_end) = match (
            node_from.anchor.x,
            node_from.anchor.y,
            node_to.anchor.x,
            node_to.anchor.y,
        ) {
            (x1, y1, x2, y2) if x1 < x2 && y1 < y2 => (
                node_from.get_arrow_anchor(Direction::Bottom),
                node_to.get_arrow_anchor(Direction::Left),
            ),
            (x1, y1, x2, y2) if x1 > x2 && y1 > y2 => (
                node_from.get_arrow_anchor(Direction::Left),
                node_to.get_arrow_anchor(Direction::Bottom),
            ),
            (x1, y1, x2, y2) if x1 > x2 && y1 < y2 => (
                node_from.get_arrow_anchor(Direction::Bottom),
                node_to.get_arrow_anchor(Direction::Right),
            ),
            (x1, y1, x2, y2) if x1 < x2 && y1 > y2 => (
                node_from.get_arrow_anchor(Direction::Right),
                node_to.get_arrow_anchor(Direction::Bottom),
            ),
            (x1, y1, x2, y2) if x1 == x2 && y1 < y2 => (
                node_from.get_arrow_anchor(Direction::Bottom),
                node_to.get_arrow_anchor(Direction::Top),
            ),
            (x1, y1, x2, y2) if x1 == x2 && y1 > y2 => (
                node_from.get_arrow_anchor(Direction::Top),
                node_to.get_arrow_anchor(Direction::Bottom),
            ),
            (x1, y1, x2, y2) if x1 < x2 && y1 == y2 => (
                node_from.get_arrow_anchor(Direction::Right),
                node_to.get_arrow_anchor(Direction::Left),
            ),
            (x1, y1, x2, y2) if x1 > x2 && y1 == y2 => (
                node_from.get_arrow_anchor(Direction::Left),
                node_to.get_arrow_anchor(Direction::Right),
            ),
            _ => continue,
        };

        let arrow_middle = match (arrow_start.x, arrow_start.y, arrow_end.x, arrow_end.y) {
            (x1, y1, x2, y2) if x1 < x2 && y1 < y2 => domain::Point { x: x1, y: y2 },
            (x1, y1, x2, y2) if x1 > x2 && y1 > y2 => domain::Point { x: x2, y: y1 },
            (x1, y1, x2, y2) if x1 > x2 && y1 < y2 => domain::Point { x: x1, y: y2 },
            (x1, y1, x2, y2) if x1 < x2 && y1 > y2 => domain::Point { x: x2, y: y1 },
            (x1, y1, x2, y2) if x1 == x2 && y1 < y2 => domain::Point { x: x1, y: y1 + 1 },
            (x1, y1, x2, y2) if x1 == x2 && y1 > y2 => domain::Point { x: x1, y: y2 + 1 },
            (x1, y1, x2, y2) if x1 < x2 && y1 == y2 => domain::Point { x: x1 + 1, y: y1 },
            (x1, y1, x2, y2) if x1 > x2 && y1 == y2 => domain::Point { x: x2 + 1, y: y1 },
            _ => continue,
        };

        map.arrows.insert(domain::Arrow {
            start: arrow_start,
            middle: arrow_middle,
            end: arrow_end,
            body: read_arrow_body(&mapping.arrow),
            head: read_arrow_head(&mapping.arrow),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;

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
            domain::Point { x: 20, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 5, y: 1 },
            middle: domain::Point { x: 6, y: 1 },
            end: domain::Point { x: 19, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_error01() {
        let input = "A -> B -> C";
        let result = read_input(input);
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
            domain::Point { x: 24, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 48, y: 0 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 5, y: 1 },
            middle: domain::Point { x: 6, y: 1 },
            end: domain::Point { x: 23, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 29, y: 1 },
            middle: domain::Point { x: 30, y: 1 },
            end: domain::Point { x: 47, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
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
            domain::Point { x: 12, y: 5 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 24, y: 0 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 2, y: 3 },
            middle: domain::Point { x: 2, y: 6 },
            end: domain::Point { x: 11, y: 6 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 17, y: 6 },
            middle: domain::Point { x: 26, y: 6 },
            end: domain::Point { x: 26, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 23, y: 1 },
            middle: domain::Point { x: 6, y: 1 },
            end: domain::Point { x: 5, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_example04() {
        let input = "\
A -> B
B -> A";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 0 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 20, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 5, y: 1 },
            middle: domain::Point { x: 6, y: 1 },
            end: domain::Point { x: 19, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 19, y: 2 },
            middle: domain::Point { x: 6, y: 2 },
            end: domain::Point { x: 5, y: 2 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_example05() {
        let input = "\
A -> B
B -> C
C -> D
D -> A
D -> B";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 1 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 20, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 36, y: 3 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 16, y: 5 },
            domain::Node {
                name: "D".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 15, y: 6 },
            middle: domain::Point { x: 2, y: 6 },
            end: domain::Point { x: 2, y: 4 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 21, y: 7 },
            middle: domain::Point { x: 24, y: 7 },
            end: domain::Point { x: 24, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 5, y: 2 },
            middle: domain::Point { x: 5, y: 3 },
            end: domain::Point { x: 22, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 23, y: 3 },
            middle: domain::Point { x: 23, y: 4 },
            end: domain::Point { x: 35, y: 4 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 38, y: 6 },
            middle: domain::Point { x: 22, y: 6 },
            end: domain::Point { x: 21, y: 6 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_example06() {
        let input = "\
A -> B
B -> C
C -> D
B -> E
E -> F";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 0, y: 3 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 24, y: 5 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 40, y: 0 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 64, y: 0 },
            domain::Node {
                name: "D".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 36, y: 10 },
            domain::Node {
                name: "E".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 52, y: 15 },
            domain::Node {
                name: "F".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 2, y: 6 },
            middle: domain::Point { x: 3, y: 6 },
            end: domain::Point { x: 23, y: 6 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 29, y: 6 },
            middle: domain::Point { x: 42, y: 6 },
            end: domain::Point { x: 42, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 45, y: 1 },
            middle: domain::Point { x: 46, y: 1 },
            end: domain::Point { x: 63, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 26, y: 8 },
            middle: domain::Point { x: 26, y: 11 },
            end: domain::Point { x: 35, y: 11 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 38, y: 13 },
            middle: domain::Point { x: 38, y: 16 },
            end: domain::Point { x: 51, y: 16 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    #[test]
    fn get_map_example07() {
        let input = "\
A -> B
B -> C
C -> A
A -> D";
        let mut nodes: HashMap<domain::Point, domain::Node> = HashMap::new();
        nodes.insert(
            domain::Point { x: 16, y: 5 },
            domain::Node {
                name: "A".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 24, y: 0 },
            domain::Node {
                name: "B".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 36, y: 4 },
            domain::Node {
                name: "C".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 0, y: 9 },
            domain::Node {
                name: "D".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        let mut arrows: HashSet<domain::Arrow> = HashSet::new();
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 21, y: 6 },
            middle: domain::Point { x: 26, y: 6 },
            end: domain::Point { x: 26, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 27, y: 3 },
            middle: domain::Point { x: 27, y: 5 },
            end: domain::Point { x: 35, y: 5 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 38, y: 7 },
            middle: domain::Point { x: 22, y: 7 },
            end: domain::Point { x: 21, y: 7 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 18, y: 8 },
            middle: domain::Point { x: 18, y: 10 },
            end: domain::Point { x: 5, y: 10 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        let expected = domain::Map { nodes, arrows };

        let result = read_input(input);
        if let Ok(mapped_result) = result {
            assert_maps(expected, mapped_result);
        } else {
            panic!("Map should not be None for this input!");
        }
    }

    fn assert_maps(expected: domain::Map, result: domain::Map) {
        assert_eq!(expected.nodes.len(), result.nodes.len());
        for (key, value) in expected.nodes {
            assert_eq!(value, result.nodes[&key]);
        }
        assert_eq!(expected.arrows.len(), result.arrows.len());
        for arrow in expected.arrows {
            assert!(result.arrows.contains(&arrow));
        }
    }
}
