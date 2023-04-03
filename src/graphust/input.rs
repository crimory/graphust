use std::collections::{HashMap, HashSet};
use crate::graphust::domain;

mod force_directed_graph;

#[derive(Debug)]
struct InnerMapping {
    source: String,
    arrow: String,
    target: String,
}

const BOX_WIDTH: usize = 4;
const BOX_HEIGHT: usize = 3;

fn get_arrow_anchor(node_label: &str, anchor: &domain::Point, direction: Direction) -> domain::Point {
    let (x, y) = match direction {
        Direction::Top => (anchor.x + 2, anchor.y - 1),
        Direction::Bottom => (anchor.x + 2, anchor.y + BOX_HEIGHT),
        Direction::Left => (anchor.x - 1, anchor.y + 1),
        Direction::Right => (anchor.x + BOX_WIDTH + node_label.len(), anchor.y + 1),
    };
    domain::Point { x, y }
}

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

fn include_nodes(map: &mut domain::Map, approximation: &Vec<force_directed_graph::NodeApproximation>) {
    for node in approximation {
        map.nodes.insert(
            domain::Point { x: node.position.x * 4, y: node.position.y },
            domain::Node {
                name: node.name.to_owned(),
                border: domain::BorderType::Box,
            },
        );
    }
}

fn include_arrows(map: &mut domain::Map, inner_mappings: &Vec<&InnerMapping>) {
    for mapping in inner_mappings {
        let (position_from, node_from) = map.nodes.iter().find(|(_, node)| node.name == mapping.source).unwrap();
        let (position_to, node_to) = map.nodes.iter().find(|(_, node)| node.name == mapping.target).unwrap();

        let mut arrow_start = domain::Point { ..*position_from };
        let mut arrow_end = domain::Point { ..*position_to };
        let arrow_middle = match (position_from.x, position_from.y, position_to.x, position_to.y) {
            (x1, y1, x2, y2) if x1 < x2 && y1 < y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Bottom);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Left);
                domain::Point { x: arrow_start.x, y: arrow_end.y }
            },
            (x1, y1, x2, y2) if x1 > x2 && y1 > y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Left);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Bottom);
                domain::Point { x: arrow_end.x, y: arrow_start.y }
            },
            (x1, y1, x2, y2) if x1 > x2 && y1 < y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Bottom);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Right);
                domain::Point { x: arrow_start.x, y: arrow_end.y }
            },
            (x1, y1, x2, y2) if x1 < x2 && y1 > y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Right);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Bottom);
                domain::Point { x: arrow_end.x, y: arrow_start.y }
            },
            (x1, y1, x2, y2) if x1 == x2 && y1 < y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Bottom);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Top);
                domain::Point { x: arrow_start.x, y: arrow_start.y + 1 }
            },
            (x1, y1, x2, y2) if x1 == x2 && y1 > y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Top);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Bottom);
                domain::Point { x: arrow_start.x, y: arrow_end.y + 1 }
            },
            (x1, y1, x2, y2) if x1 < x2 && y1 == y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Right);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Left);
                domain::Point { x: arrow_start.x + 1, y: arrow_start.y }
            },
            (x1, y1, x2, y2) if x1 > x2 && y1 == y2 => {
                arrow_start = get_arrow_anchor(&node_from.name, position_from, Direction::Left);
                arrow_end = get_arrow_anchor(&node_to.name, position_to, Direction::Right);
                domain::Point { x: arrow_end.x + 1, y: arrow_start.y }
            },
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
            middle: domain::Point { x: 22, y: 4 },
            end: domain::Point { x: 2, y: 3 },
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
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 8, y: 2 },
            middle: domain::Point { x: 7, y: 2 },
            end: domain::Point { x: 6, y: 2 },
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
        nodes.insert(
            domain::Point { x: 30, y: 0 },
            domain::Node {
                name: "D".to_owned(),
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
            start: domain::Point { x: 26, y: 1 },
            middle: domain::Point { x: 27, y: 1 },
            end: domain::Point { x: 28, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 32, y: 3 },
            middle: domain::Point { x: 32, y: 4 },
            end: domain::Point { x: 2, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 33, y: 3 },
            middle: domain::Point { x: 33, y: 5 },
            end: domain::Point { x: 12, y: 3 },
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
        nodes.insert(
            domain::Point { x: 30, y: 0 },
            domain::Node {
                name: "D".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 20, y: 4 },
            domain::Node {
                name: "E".to_owned(),
                border: domain::BorderType::Box,
            },
        );
        nodes.insert(
            domain::Point { x: 30, y: 4 },
            domain::Node {
                name: "F".to_owned(),
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
            start: domain::Point { x: 26, y: 1 },
            middle: domain::Point { x: 27, y: 1 },
            end: domain::Point { x: 28, y: 1 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 12, y: 3 },
            middle: domain::Point { x: 12, y: 5 },
            end: domain::Point { x: 18, y: 5 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 26, y: 5 },
            middle: domain::Point { x: 27, y: 5 },
            end: domain::Point { x: 28, y: 5 },
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
        nodes.insert(
            domain::Point { x: 10, y: 6 },
            domain::Node {
                name: "D".to_owned(),
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
            middle: domain::Point { x: 22, y: 4 },
            end: domain::Point { x: 2, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 3, y: 3 },
            middle: domain::Point { x: 3, y: 7 },
            end: domain::Point { x: 8, y: 7 },
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
