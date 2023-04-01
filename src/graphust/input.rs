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

fn get_lines_of_nodes(inner_mapping: &[&InnerMapping]) -> Vec<Vec<String>> {
    let mut lines_of_nodes: Vec<Vec<String>> = Vec::new();
    for current_mapping in inner_mapping {
        let this_mapping_already_covered = lines_of_nodes
            .iter()
            .any(|y| {
                let possible_source_position = y.iter().position(|x| x == &current_mapping.source);
                let possible_target_position = y.iter().position(|x| x == &current_mapping.target);
                matches!(
                    (possible_source_position, possible_target_position),
                    (Some(source_position), Some(target_position)) if (source_position + 1) == target_position
                )
            });
        if this_mapping_already_covered {
            continue;
        }

        let this_mapping_already_covered_reversed = lines_of_nodes.iter().any(|y| {
            let possible_source_position = y.iter().position(|x| x == &current_mapping.source);
            let possible_target_position = y.iter().position(|x| x == &current_mapping.target);
            matches!(
                (possible_source_position, possible_target_position),
                (Some(source_position), Some(target_position)) if source_position > target_position
            )
        });
        if this_mapping_already_covered_reversed {
            lines_of_nodes.push(vec![
                current_mapping.source.to_owned(),
                current_mapping.target.to_owned(),
            ]);
            continue;
        }

        let source_as_last = lines_of_nodes
            .iter_mut()
            .find(|y| y.last().unwrap_or(&"".to_owned()) == &current_mapping.source);
        if let Some(line_ending_with_source) = source_as_last {
            line_ending_with_source.push(current_mapping.target.to_owned());
            continue;
        }

        let target_as_first = lines_of_nodes
            .iter_mut()
            .find(|y| y.first().unwrap_or(&"".to_owned()) == &current_mapping.target);
        if let Some(line_starting_with_target) = target_as_first {
            line_starting_with_target.insert(0, current_mapping.source.to_owned());
            continue;
        }

        lines_of_nodes.push(vec![
            current_mapping.source.to_owned(),
            current_mapping.target.to_owned(),
        ]);
    }
    lines_of_nodes
}

struct Inner2DMappingArrowHandler {
    number_of_arrows_needed: usize,
    number_of_arrows_added: usize,
}
enum Direction {
    Top,
    Bottom,
    Left,
    Right,
}
struct Inner2DMapping {
    anchor: domain::Point,
    label: String,
    top: Inner2DMappingArrowHandler,
    bottom: Inner2DMappingArrowHandler,
    left: Inner2DMappingArrowHandler,
    right: Inner2DMappingArrowHandler,
}
impl Inner2DMapping {
    fn get_width(&self) -> usize {
        let label_length = self.label.len();
        match (
            self.top.number_of_arrows_needed,
            self.bottom.number_of_arrows_needed,
        ) {
            (larger, smaller) if larger >= smaller && larger > label_length => larger + BOX_WIDTH,
            (smaller, larger) if larger >= smaller && larger > label_length => larger + BOX_WIDTH,
            _ => label_length + BOX_WIDTH,
        }
    }
    fn get_height(&self) -> usize {
        match (
            self.left.number_of_arrows_needed,
            self.right.number_of_arrows_needed,
        ) {
            (larger, smaller) if larger >= smaller && larger > BOX_HEIGHT => larger,
            (smaller, larger) if larger >= smaller && larger > BOX_HEIGHT => larger,
            _ => BOX_HEIGHT,
        }
    }
    fn get_next_arrow_position(&mut self, direction: &Direction) -> domain::Point {
        let x = match direction {
            Direction::Top => self.anchor.x + 2 + self.top.number_of_arrows_added,
            Direction::Bottom => self.anchor.x + 2 + self.bottom.number_of_arrows_added,
            Direction::Left => {
                let left = (self.anchor.x as isize) - 2;
                if left < 0 {
                    0
                } else {
                    left as usize
                }
            }
            Direction::Right => self.anchor.x + 1 + self.get_width(),
        };
        let y = match direction {
            Direction::Top => {
                let top = self.anchor.y as isize - 1;
                if top < 0 {
                    0
                } else {
                    top as usize
                }
            }
            Direction::Bottom => self.anchor.y + self.get_height(),
            Direction::Left => {
                let offset_from_top = if self.left.number_of_arrows_needed > 2 {
                    0
                } else {
                    1
                };
                self.anchor.y + self.left.number_of_arrows_added + offset_from_top
            }
            Direction::Right => {
                let offset_from_top = if self.right.number_of_arrows_needed > 2 {
                    0
                } else {
                    1
                };
                self.anchor.y + self.right.number_of_arrows_added + offset_from_top
            }
        };

        match direction {
            Direction::Top => self.top.number_of_arrows_added += 1,
            Direction::Bottom => self.bottom.number_of_arrows_added += 1,
            Direction::Left => self.left.number_of_arrows_added += 1,
            Direction::Right => self.right.number_of_arrows_added += 1,
        }

        domain::Point { x, y }
    }
    fn new(anchor: domain::Point, label: String) -> Inner2DMapping {
        Inner2DMapping {
            anchor,
            label,
            top: Inner2DMappingArrowHandler {
                number_of_arrows_needed: 0,
                number_of_arrows_added: 0,
            },
            bottom: Inner2DMappingArrowHandler {
                number_of_arrows_needed: 0,
                number_of_arrows_added: 0,
            },
            left: Inner2DMappingArrowHandler {
                number_of_arrows_needed: 0,
                number_of_arrows_added: 0,
            },
            right: Inner2DMappingArrowHandler {
                number_of_arrows_needed: 0,
                number_of_arrows_added: 0,
            },
        }
    }
}

fn is_node_between_horizontally(
    first: &domain::Point,
    second: &domain::Point,
    all: &[domain::Point],
) -> bool {
    if first.y != second.y {
        return false;
    }
    all.iter().any(|node| {
        node.y == first.y
            && ((node.x > first.x && node.x < second.x) || (node.x < first.x && node.x > second.x))
    })
}

fn get_initial_2d_mapping(lines_of_nodes: &Vec<Vec<String>>) -> HashMap<String, Inner2DMapping> {
    let mut inner_2d_mapping: HashMap<String, Inner2DMapping> = HashMap::new();
    let mut y = 0;
    let mut y_offset = 0;
    for line in lines_of_nodes {
        let mut x = 0;
        for i in 0..line.len() {
            let previous_index = if i == 0 { 0 } else { i - 1 };
            let mut current_and_previous = inner_2d_mapping.values_mut().filter(|mapping| {
                mapping.label == line[i]
                    || mapping.label == *line.get(previous_index).unwrap_or(&"".to_owned())
            });
            let (possible_previous_node, possible_current_node) =
                match (current_and_previous.next(), current_and_previous.next()) {
                    (None, None) => (None, None),
                    (Some(curr), None) | (None, Some(curr)) if curr.label == line[i] => {
                        (None, Some(curr))
                    }
                    (Some(prev), None) | (None, Some(prev)) => (Some(prev), None),
                    (Some(one), Some(two)) if one.label == line[i] => (Some(two), Some(one)),
                    (Some(one), Some(two)) => (Some(one), Some(two)),
                };
            match (possible_previous_node, possible_current_node) {
                (prev, None) => {
                    let mut new_2d_mapping =
                        Inner2DMapping::new(domain::Point { x, y }, line[i].to_owned());
                    if let Some(prev) = prev {
                        new_2d_mapping.left.number_of_arrows_needed += 1;
                        prev.right.number_of_arrows_needed += 1;
                    }
                    x = new_2d_mapping.anchor.x
                        + new_2d_mapping.get_width()
                        + ARROW_AND_SPACES_WIDTH;
                    inner_2d_mapping.insert(line[i].to_owned(), new_2d_mapping);
                }
                (None, Some(curr)) => {
                    x = curr.anchor.x + curr.get_width() + ARROW_AND_SPACES_WIDTH;
                }
                (Some(prev), Some(curr)) => {
                    x = curr.anchor.x + curr.get_width() + ARROW_AND_SPACES_WIDTH;
                    match (prev.anchor.y, curr.anchor.y) {
                        (prev_y, curr_y) if prev_y == curr_y => {
                            prev.bottom.number_of_arrows_needed += 1;
                            curr.bottom.number_of_arrows_needed += 1;
                            if y_offset == 0 {
                                y_offset += 2;
                                y += 2;
                            } else {
                                y_offset += 1;
                                y += 1;
                            }
                        }
                        (prev_y, curr_y) if prev_y < curr_y => {
                            prev.bottom.number_of_arrows_needed += 1;
                            curr.left.number_of_arrows_needed += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        let y_growth_from_this_line = inner_2d_mapping
            .values()
            .filter(|x| x.anchor.y == y)
            .map(|x| x.get_height())
            .max()
            .unwrap_or(0);
        if y_growth_from_this_line > 0 {
            y += y_growth_from_this_line + 1;
            y_offset = 0;
        }
    }
    inner_2d_mapping
}

fn arr_arrows(
    map: &mut domain::Map,
    lines_of_nodes: &Vec<Vec<String>>,
    inner_2d_mapping: &mut HashMap<String, Inner2DMapping>,
) {
    let read_only_copy_of_anchors = inner_2d_mapping
        .values()
        .map(|v| domain::Point { ..v.anchor })
        .collect::<Vec<_>>();
    let mut y_offset = 1;
    for line in lines_of_nodes {
        for pair in line.windows(2) {
            let mut test = inner_2d_mapping
                .values_mut()
                .filter(|x| x.label == pair[0] || x.label == pair[1]);
            let first = test.next().unwrap();
            let second = test.next().unwrap();
            let (source_2d, target_2d) = if first.label == pair[0] {
                (first, second)
            } else {
                (second, first)
            };
            let is_something_between = is_node_between_horizontally(
                &source_2d.anchor,
                &target_2d.anchor,
                &read_only_copy_of_anchors,
            );
            match (source_2d, target_2d, is_something_between) {
                (s, t, false) if s.anchor.x < t.anchor.x && s.anchor.y == t.anchor.y => {
                    let arrow_start = s.get_next_arrow_position(&Direction::Right);
                    let arrow_end = t.get_next_arrow_position(&Direction::Left);
                    let arrow_middle = domain::Point {
                        x: arrow_start.x + 1,
                        y: arrow_start.y,
                    };
                    map.arrows.insert(domain::Arrow {
                        start: arrow_start,
                        middle: arrow_middle,
                        end: arrow_end,
                        body: domain::ArrowBody::Basic,
                        head: domain::ArrowHead::Basic,
                    });
                }
                (s, t, false) if s.anchor.x > t.anchor.x && s.anchor.y == t.anchor.y => {
                    let arrow_start = s.get_next_arrow_position(&Direction::Left);
                    let arrow_end = t.get_next_arrow_position(&Direction::Right);
                    let arrow_middle = domain::Point {
                        x: arrow_end.x + 1,
                        y: arrow_end.y,
                    };
                    map.arrows.insert(domain::Arrow {
                        start: arrow_start,
                        middle: arrow_middle,
                        end: arrow_end,
                        body: domain::ArrowBody::Basic,
                        head: domain::ArrowHead::Basic,
                    });
                }
                (s, t, true) if s.anchor.x > t.anchor.x && s.anchor.y == t.anchor.y => {
                    let arrow_start = s.get_next_arrow_position(&Direction::Bottom);
                    let arrow_end = t.get_next_arrow_position(&Direction::Bottom);
                    let arrow_middle = domain::Point {
                        x: arrow_start.x,
                        y: arrow_start.y + y_offset,
                    };
                    y_offset += 1;
                    map.arrows.insert(domain::Arrow {
                        start: arrow_start,
                        middle: arrow_middle,
                        end: arrow_end,
                        body: domain::ArrowBody::Basic,
                        head: domain::ArrowHead::Basic,
                    });
                }
                (s, t, _) if s.anchor.x < t.anchor.x && s.anchor.y < t.anchor.y => {
                    y_offset = 0;
                    let arrow_start = s.get_next_arrow_position(&Direction::Bottom);
                    let arrow_end = t.get_next_arrow_position(&Direction::Left);
                    let arrow_middle = domain::Point {
                        x: arrow_start.x,
                        y: arrow_end.y,
                    };
                    map.arrows.insert(domain::Arrow {
                        start: arrow_start,
                        middle: arrow_middle,
                        end: arrow_end,
                        body: domain::ArrowBody::Basic,
                        head: domain::ArrowHead::Basic,
                    });
                }
                _ => {}
            }
        }
    }
}

fn add_nodes(map: &mut domain::Map, inner_2d_mapping: &HashMap<String, Inner2DMapping>) {
    for (label, mapping) in inner_2d_mapping {
        map.nodes.insert(
            domain::Point { ..mapping.anchor },
            domain::Node {
                name: label.to_owned(),
                border: domain::BorderType::Box,
            },
        );
    }
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
    let mut map = domain::Map {
        nodes: HashMap::new(),
        arrows: HashSet::new(),
    };

    let lines_of_nodes = get_lines_of_nodes(&inner_mapping);
    let mut inner_2d_mapping = get_initial_2d_mapping(&lines_of_nodes);
    arr_arrows(&mut map, &lines_of_nodes, &mut inner_2d_mapping);
    add_nodes(&mut map, &inner_2d_mapping);
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
