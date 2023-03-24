use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
pub enum BorderType {
    Box,
}
impl BorderType {
    fn get_matching_char(&self, part: &BorderPart) -> char {
        match (self, part) {
            (BorderType::Box, BorderPart::Horizontal) => '-',
            (BorderType::Box, BorderPart::Vertical) => '|',
            (BorderType::Box, BorderPart::LeftTopCorner) => '+',
            (BorderType::Box, BorderPart::RightTopCorner) => '+',
            (BorderType::Box, BorderPart::LeftBottomCorner) => '+',
            (BorderType::Box, BorderPart::RightBottomCorner) => '+',
        }
    }
}

enum BorderPart {
    Horizontal,
    Vertical,
    LeftTopCorner,
    RightTopCorner,
    LeftBottomCorner,
    RightBottomCorner,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}
impl Point {
    fn new_with_anchor(x: usize, y: usize, anchor: &Point) -> Self {
        Self {
            x: anchor.x + x,
            y: anchor.y + y,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub name: String,
    pub border: BorderType,
}
impl Node {
    fn grab_characters(&self, anchor: &Point) -> HashMap<Point, char> {
        let mut output = HashMap::new();
        let end_x = self.name.len() + 3;
        self.add_corners(end_x, anchor, &mut output);
        self.add_top_and_bottom(end_x, anchor, &mut output);
        self.add_sides(end_x, anchor, &mut output);
        self.add_text(anchor, &mut output);
        output
    }

    fn add_corners(&self, end_x: usize, anchor: &Point, output: &mut HashMap<Point, char>) {
        output.insert(
            Point::new_with_anchor(0, 0, anchor),
            self.border.get_matching_char(&BorderPart::LeftTopCorner),
        );
        output.insert(
            Point::new_with_anchor(end_x, 0, anchor),
            self.border.get_matching_char(&BorderPart::RightTopCorner),
        );
        output.insert(
            Point::new_with_anchor(0, 2, anchor),
            self.border.get_matching_char(&BorderPart::LeftBottomCorner),
        );
        output.insert(
            Point::new_with_anchor(end_x, 2, anchor),
            self.border
                .get_matching_char(&BorderPart::RightBottomCorner),
        );
    }
    fn add_top_and_bottom(&self, end_x: usize, anchor: &Point, output: &mut HashMap<Point, char>) {
        for x in 1..end_x {
            output.insert(
                Point::new_with_anchor(x, 0, anchor),
                self.border.get_matching_char(&BorderPart::Horizontal),
            );
            output.insert(
                Point::new_with_anchor(x, 2, anchor),
                self.border.get_matching_char(&BorderPart::Horizontal),
            );
        }
    }
    fn add_sides(&self, end_x: usize, anchor: &Point, output: &mut HashMap<Point, char>) {
        output.insert(
            Point::new_with_anchor(0, 1, anchor),
            self.border.get_matching_char(&BorderPart::Vertical),
        );
        output.insert(
            Point::new_with_anchor(end_x, 1, anchor),
            self.border.get_matching_char(&BorderPart::Vertical),
        );
    }
    fn add_text(&self, anchor: &Point, output: &mut HashMap<Point, char>) {
        self.name.chars().fold(2, |acc, c| {
            output.insert(Point::new_with_anchor(acc, 1, anchor), c);
            acc + 1
        });
    }
}

enum ArrowDirection {
    HorizontalLeft,
    HorizontalRight,
    VerticalUp,
    VerticalDown,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ArrowBody {
    Basic,
}
impl ArrowBody {
    fn get_matching_character(&self, direction: &ArrowDirection) -> char {
        match (self, direction) {
            (ArrowBody::Basic, ArrowDirection::HorizontalLeft)
            | (ArrowBody::Basic, ArrowDirection::HorizontalRight) => '-',
            (ArrowBody::Basic, ArrowDirection::VerticalUp)
            | (ArrowBody::Basic, ArrowDirection::VerticalDown) => '|',
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ArrowHead {
    Basic,
}
impl ArrowHead {
    fn get_matching_character(&self, direction: &ArrowDirection) -> char {
        match (self, direction) {
            (ArrowHead::Basic, ArrowDirection::HorizontalLeft) => '<',
            (ArrowHead::Basic, ArrowDirection::HorizontalRight) => '>',
            (ArrowHead::Basic, ArrowDirection::VerticalUp) => '^',
            (ArrowHead::Basic, ArrowDirection::VerticalDown) => 'v',
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Arrow {
    pub start: Point,
    pub middle: Point,
    pub end: Point,
    pub body: ArrowBody,
    pub head: ArrowHead,
}
impl Arrow {
    fn grab_characters(&self) -> HashMap<Point, char> {
        let mut output = HashMap::new();
        self.add_arrow_body(&self.start, &self.middle, &mut output);
        self.add_arrow_body(&self.middle, &self.end, &mut output);
        let end_diff_x = self.end.x as isize - self.middle.x as isize;
        let end_diff_y = self.end.y as isize - self.middle.y as isize;
        let arrow_head_char = match (end_diff_y, end_diff_x) {
            (0, a) if a < 0 => self
                .head
                .get_matching_character(&ArrowDirection::HorizontalLeft),
            (0, _) => self
                .head
                .get_matching_character(&ArrowDirection::HorizontalRight),
            (a, _) if a < 0 => self
                .head
                .get_matching_character(&ArrowDirection::VerticalUp),
            (_, _) => self
                .head
                .get_matching_character(&ArrowDirection::VerticalDown),
        };
        output.insert(
            Point {
                x: self.end.x,
                y: self.end.y,
            },
            arrow_head_char,
        );
        output
    }
    fn add_arrow_body(&self, start: &Point, end: &Point, output: &mut HashMap<Point, char>) {
        let x_diff = end.x as isize - start.x as isize;
        if x_diff != 0 {
            let horizontal_direction = match x_diff {
                a if a < 0 => ArrowDirection::HorizontalLeft,
                _ => ArrowDirection::HorizontalRight,
            };
            let x_range = if x_diff < 0 {
                (end.x + 1)..(start.x + 1)
            } else {
                start.x..end.x
            };
            for x in x_range {
                output.insert(
                    Point { x, y: start.y },
                    self.body.get_matching_character(&horizontal_direction),
                );
            }
        }
        let y_diff = end.y as isize - start.y as isize;
        if y_diff != 0 {
            let vertical_direction = match y_diff {
                a if a < 0 => ArrowDirection::VerticalUp,
                _ => ArrowDirection::VerticalDown,
            };
            let y_range = if y_diff < 0 {
                (end.y + 1)..(start.y + 1)
            } else {
                start.y..end.y
            };
            for y in y_range {
                output.insert(
                    Point { x: end.x, y },
                    self.body.get_matching_character(&vertical_direction),
                );
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Map {
    pub nodes: HashMap<Point, Node>,
    pub arrows: HashSet<Arrow>,
}
impl Map {
    pub fn get_picture(&self) -> String {
        let mut chars: HashMap<Point, char> = HashMap::new();
        for node in &self.nodes {
            let characters = node.1.grab_characters(&node.0);
            for character in characters {
                chars.insert(character.0, character.1);
            }
        }
        for arrow in &self.arrows {
            let characters = arrow.grab_characters();
            for character in characters {
                chars.insert(character.0, character.1);
            }
        }
        let mut builder = String::new();
        let max_x = chars.iter().max_by_key(|c| c.0.x).unwrap().0.x;
        let max_y = chars.iter().max_by_key(|c| c.0.y).unwrap().0.y;
        for y in 0..=max_y {
            for x in 0..=max_x {
                let potential_char = chars.iter().find(|c| c.0.x == x && c.0.y == y);
                match potential_char {
                    None => builder.push(' '),
                    Some(c) => builder.push(*c.1),
                }
            }
            builder.push_str("\n");
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_returns_with_two_arrows() {
        let mut nodes: HashMap<Point, Node> = HashMap::new();
        nodes.insert(
            Point { x: 0, y: 0 },
            Node {
                name: "A".to_owned(),
                border: BorderType::Box,
            },
        );
        nodes.insert(
            Point { x: 10, y: 0 },
            Node {
                name: "B".to_owned(),
                border: BorderType::Box,
            },
        );
        let mut arrows: HashSet<Arrow> = HashSet::new();
        arrows.insert(Arrow {
            start: Point { x: 6, y: 1 },
            middle: Point { x: 7, y: 1 },
            end: Point { x: 8, y: 1 },
            body: ArrowBody::Basic,
            head: ArrowHead::Basic,
        });
        arrows.insert(Arrow {
            start: Point { x: 12, y: 3 },
            middle: Point { x: 12, y: 5 },
            end: Point { x: 2, y: 3 },
            body: ArrowBody::Basic,
            head: ArrowHead::Basic,
        });
        let map = Map { nodes, arrows };
        let expected = "\
+---+     +---+
| A | --> | B |
+---+     +---+
  ^         |  
  |         |  
  |----------  
";
        let output = map.get_picture();
        assert_eq!(expected, output);
    }

    #[test]
    fn map_returns_with_arrow() {
        let mut nodes: HashMap<Point, Node> = HashMap::new();
        nodes.insert(
            Point { x: 0, y: 0 },
            Node {
                name: "A".to_owned(),
                border: BorderType::Box,
            },
        );
        nodes.insert(
            Point { x: 10, y: 0 },
            Node {
                name: "B".to_owned(),
                border: BorderType::Box,
            },
        );
        let mut arrows: HashSet<Arrow> = HashSet::new();
        arrows.insert(Arrow {
            start: Point { x: 6, y: 1 },
            middle: Point { x: 7, y: 1 },
            end: Point { x: 8, y: 1 },
            body: ArrowBody::Basic,
            head: ArrowHead::Basic,
        });
        let map = Map { nodes, arrows };
        let expected = "\
+---+     +---+
| A | --> | B |
+---+     +---+
";
        let output = map.get_picture();
        assert_eq!(expected, output);
    }

    #[test]
    fn map_returns() {
        let mut nodes: HashMap<Point, Node> = HashMap::new();
        nodes.insert(
            Point { x: 0, y: 0 },
            Node {
                name: "A".to_owned(),
                border: BorderType::Box,
            },
        );
        nodes.insert(
            Point { x: 5, y: 0 },
            Node {
                name: "B".to_owned(),
                border: BorderType::Box,
            },
        );
        let map = Map {
            nodes,
            arrows: HashSet::new(),
        };
        let expected = "\
+---++---+
| A || B |
+---++---+
";
        let output = map.get_picture();
        assert_eq!(expected, output);
    }

    #[test]
    fn node_returns() {
        let node = Node {
            name: "T".to_owned(),
            border: BorderType::Box,
        };
        let anchor = Point { x: 0, y: 0 };
        let output = node.grab_characters(&anchor);

        let mut expected: HashMap<Point, char> = HashMap::new();
        expected.insert(Point { x: 0, y: 0 }, '+');
        expected.insert(Point { x: 1, y: 0 }, '-');
        expected.insert(Point { x: 2, y: 0 }, '-');
        expected.insert(Point { x: 3, y: 0 }, '-');
        expected.insert(Point { x: 4, y: 0 }, '+');

        expected.insert(Point { x: 0, y: 1 }, '|');
        expected.insert(Point { x: 2, y: 1 }, 'T');
        expected.insert(Point { x: 4, y: 1 }, '|');

        expected.insert(Point { x: 0, y: 2 }, '+');
        expected.insert(Point { x: 1, y: 2 }, '-');
        expected.insert(Point { x: 2, y: 2 }, '-');
        expected.insert(Point { x: 3, y: 2 }, '-');
        expected.insert(Point { x: 4, y: 2 }, '+');

        assert_eq!(expected.len(), output.len());
        for expected_item in expected {
            let matching_output: Vec<(&Point, &char)> = output
                .iter()
                .filter(|o| *o.0 == expected_item.0 && *o.1 == expected_item.1)
                .collect();
            assert_eq!(matching_output.len(), 1);
        }
    }
}
