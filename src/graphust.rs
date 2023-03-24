use std::collections::{HashMap, HashSet};

mod domain;
pub mod input;

pub fn example_01() {
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
    let map = dbg!(domain::Map { nodes, arrows });
    let output = map.get_picture();

    println!("{}", output);
}
