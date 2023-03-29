use std::collections::{HashMap, HashSet};

use crate::graphust::domain;

#[derive(Debug)]
struct InnerMapping {
    source: String,
    arrow: String,
    target: String,
}

const BOX_WIDTH: usize = 4;
const BOX_HEIGHT: usize = 3;
const ARROW_AND_SPACES_WIDTH: usize = 5;

fn add_nodes(map: &mut domain::Map, inner_mapping: &[&InnerMapping]) {
    let mut inner_nodes = inner_mapping.iter().map(|x| &x.source).collect::<Vec<_>>();
    inner_nodes.extend(inner_mapping.iter().map(|x| &x.target));
    inner_nodes.iter().fold(0, |acc, label| {
        if map.nodes.iter().any(|x| x.1.name == **label) {
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

fn has_arrow_already(map: &domain::Map, start: &domain::Point, end: &domain::Point) -> bool {
    map.arrows
        .iter()
        .any(|a| a.start == *start && a.end == *end || a.start == *end && a.end == *start)
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

fn add_horizontal_arrows(map: &mut domain::Map, inner_mapping: &[&InnerMapping]) {
    let mapping_point_pairs = [(Anchor::Right, Anchor::Left), (Anchor::Left, Anchor::Right)];
    inner_mapping.iter().for_each(|x| {
        mapping_point_pairs
            .iter()
            .for_each(|(source_anchor, target_anchor)| {
                let source_point = get_point_next_to_box(&map.nodes, &x.source, source_anchor);
                let target_point = get_point_next_to_box(&map.nodes, &x.target, target_anchor);

                let possible_new_arrow = match (source_point, target_point) {
                    (s, t) if s.y == t.y && (t.x as isize - s.x as isize).abs() == 2 => {
                        let middle_x_relating_to_start: isize = if s.x < t.x { 1 } else { -1 };
                        let middle_point = domain::Point {
                            x: (s.x as isize + middle_x_relating_to_start) as usize,
                            y: s.y,
                        };
                        if has_arrow_already(map, &s, &t) {
                            Some(domain::Arrow {
                                middle: domain::Point {
                                    y: s.y + 1,
                                    ..middle_point
                                },
                                start: domain::Point { y: s.y + 1, ..s },
                                end: domain::Point { y: t.y + 1, ..t },
                                body: read_arrow_body(&x.arrow),
                                head: read_arrow_head(&x.arrow),
                            })
                        } else {
                            Some(domain::Arrow {
                                middle: middle_point,
                                start: s,
                                end: t,
                                body: read_arrow_body(&x.arrow),
                                head: read_arrow_head(&x.arrow),
                            })
                        }
                    }
                    (_, _) => None,
                };
                if let Some(new_arrow) = possible_new_arrow {
                    map.arrows.insert(new_arrow);
                }
            });
    });
}

fn add_vertical_arrows(map: &mut domain::Map, inner_mapping: &[&InnerMapping]) {
    let mut y_diff = 1;
    inner_mapping.iter().for_each(|x| {
        let source_point = get_point_next_to_box(&map.nodes, &x.source, &Anchor::Bottom);
        let target_point = get_point_next_to_box(&map.nodes, &x.target, &Anchor::Bottom);
        let expected_x_diff = (x.source.len() + ARROW_AND_SPACES_WIDTH + BOX_WIDTH) as isize;

        let possible_new_arrow = match (source_point, target_point) {
            (s, t) if s.y == t.y && (t.x as isize - s.x as isize).abs() > expected_x_diff => {
                y_diff += 1;
                Some(domain::Arrow {
                    middle: domain::Point {
                        x: s.x,
                        y: s.y + y_diff,
                    },
                    start: s,
                    end: t,
                    body: read_arrow_body(&x.arrow),
                    head: read_arrow_head(&x.arrow),
                })
            }
            (_, _) => None,
        };
        if let Some(new_arrow) = possible_new_arrow {
            map.arrows.insert(new_arrow);
        }
    });
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
    let mut map = domain::Map {
        nodes: HashMap::new(),
        arrows: HashSet::new(),
    };
    add_nodes(&mut map, &inner_mapping);
    add_horizontal_arrows(&mut map, &inner_mapping);
    add_vertical_arrows(&mut map, &inner_mapping);
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

        let result = read_input(&input);
        if let Ok(mapped_result) = result {
            assert_eq!(expected, mapped_result);
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
            middle: domain::Point { x: 32, y: 5 },
            end: domain::Point { x: 2, y: 3 },
            body: domain::ArrowBody::Basic,
            head: domain::ArrowHead::Basic,
        });
        arrows.insert(domain::Arrow {
            start: domain::Point { x: 32, y: 3 },
            middle: domain::Point { x: 32, y: 6 },
            end: domain::Point { x: 12, y: 3 },
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
