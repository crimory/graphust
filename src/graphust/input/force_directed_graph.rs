pub struct PointApproximation {
    pub y: usize,
    pub x: usize,
}

pub struct NodeApproximation {
    pub name: String,
    pub position: PointApproximation,
}

#[derive(Debug, PartialEq)]
struct Point {
    y: f32,
    x: f32,
}

struct Node {
    name: String,
    position: Point,
}

struct Edge {
    from_index: usize,
    to_index: usize,
}

struct InnerGraphPlacement {
    next_x: f32,
    next_y: f32,
    offset: f32,
    max_x_count: usize,
}

pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    placement: InnerGraphPlacement,
}

impl InnerGraphPlacement {
    fn new() -> InnerGraphPlacement {
        InnerGraphPlacement {
            next_x: 0.0,
            next_y: 0.0,
            offset: 5.0,
            max_x_count: 3,
        }
    }
    fn get_next_position(&mut self) -> Point {
        let position = Point {
            x: self.next_x,
            y: self.next_y,
        };
        self.next_x += self.offset;
        if self.next_x > ((self.max_x_count - 1) as f32 * self.offset) {
            self.next_x = 0.0;
            self.next_y += self.offset;
        }
        position
    }
}

impl Edge {
    fn contains(&self, index: usize) -> bool {
        self.from_index == index || self.to_index == index
    }
}

fn calculate_distance(point1: &Point, point2: &Point) -> f32 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            placement: InnerGraphPlacement::new(),
        }
    }

    pub fn add_node(&mut self, node_label: &str) {
        if self.nodes.iter().any(|node| node.name == node_label) {
            return;
        }

        self.nodes.push(Node {
            name: node_label.to_string(),
            position: self.placement.get_next_position(),
        });
    }

    pub fn add_edge(&mut self, node_from_label: &str, node_to_label: &str) {
        if node_from_label == node_to_label {
            return;
        }

        let both_indices = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.name == node_from_label || node.name == node_to_label)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if both_indices.len() != 2 {
            panic!("Could not find both nodes");
        }

        let edge_already_exists = self.edges.iter().any(|edge| {
            edge.from_index == both_indices[0] && edge.to_index == both_indices[1]
                || edge.from_index == both_indices[1] && edge.to_index == both_indices[0]
        });
        if edge_already_exists {
            return;
        }

        self.edges.push(Edge {
            from_index: both_indices[0],
            to_index: both_indices[1],
        });
    }

    fn get_edges(&self, node_index: usize) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.contains(node_index))
            .collect()
    }

    fn apply_force_of_attraction(
        &self,
        node_index: usize,
        other_node_index: usize,
        force: &mut Point,
        attraction_strength: &f32,
    ) {
        let node = self.nodes.get(node_index).unwrap();
        let other_node = self.nodes.get(other_node_index).unwrap();
        let direction = Point {
            x: other_node.position.x - node.position.x,
            y: other_node.position.y - node.position.y,
        };
        let distance = calculate_distance(&node.position, &other_node.position);
        let force_magnitude = attraction_strength * (distance / 5.0).log10();
        force.x += force_magnitude * direction.x;
        force.y += force_magnitude * direction.y;
    }

    fn apply_force_of_repulsion(
        &self,
        node_index: usize,
        other_node_index: usize,
        force: &mut Point,
        repulsion_strength: &f32,
    ) {
        if node_index == other_node_index {
            return;
        }

        let node = self.nodes.get(node_index).unwrap();
        let other_node = self.nodes.get(other_node_index).unwrap();
        let direction = Point {
            x: node.position.x - other_node.position.x,
            y: node.position.y - other_node.position.y,
        };
        let distance = calculate_distance(&node.position, &other_node.position);
        let force_magnitude = repulsion_strength / distance.powi(2);
        force.x += force_magnitude * direction.x;
        force.y += force_magnitude * direction.y;
    }

    pub fn force_directed(
        &mut self,
        attraction_strength: Option<f32>,
        repulsion_strength: Option<f32>,
    ) -> Vec<NodeApproximation> {
        let attraction_strength = attraction_strength.unwrap_or(1.0);
        let repulsion_strength = repulsion_strength.unwrap_or(1.0);
        for _ in 0..100 {
            self.force_directed_iteration(attraction_strength, repulsion_strength);
        }
        self.get_transposed_nodes_approximation()
    }

    fn force_directed_iteration(&mut self, attraction_strength: f32, repulsion_strength: f32) {
        let mut forces = Vec::new();
        for (node_index, _) in self.nodes.iter().enumerate() {
            let mut force = Point { x: 0.0, y: 0.0 };
            for edge in self.get_edges(node_index) {
                let other_node_index = if edge.from_index == node_index {
                    edge.to_index
                } else {
                    edge.from_index
                };
                self.apply_force_of_attraction(
                    node_index,
                    other_node_index,
                    &mut force,
                    &attraction_strength,
                );
            }
            for (other_node_index, _) in self.nodes.iter().enumerate() {
                self.apply_force_of_repulsion(
                    node_index,
                    other_node_index,
                    &mut force,
                    &repulsion_strength,
                );
            }
            forces.push(force);
        }

        let mut forces_iter = forces.iter();
        self.nodes.iter_mut().for_each(|node| {
            let force = forces_iter.next().unwrap();
            if force.x == 0.0 && force.y == 0.0 {
                node.position.x += 0.1;
                node.position.y += 0.1;
            } else {
                node.position.x += force.x;
                node.position.y += force.y;
            }
        });
    }

    fn get_transposed_nodes_approximation(&self) -> Vec<NodeApproximation> {
        let min_x = self
            .nodes
            .iter()
            .map(|node| node.position.x)
            .reduce(f32::min)
            .unwrap()
            .round() as i32;
        let min_y = self
            .nodes
            .iter()
            .map(|node| node.position.y)
            .reduce(f32::min)
            .unwrap()
            .round() as i32;
        let x_offset = 0 - min_x;
        let y_offset = 0 - min_y;

        self.nodes
            .iter()
            .map(|inner_node| NodeApproximation {
                name: inner_node.name.to_owned(),
                position: PointApproximation {
                    x: (inner_node.position.x.round() as i32 + x_offset) as usize,
                    y: (inner_node.position.y.round() as i32 + y_offset) as usize,
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_graph_placement_example_01() {
        let mut under_test = InnerGraphPlacement::new();
        let p1 = under_test.get_next_position();
        let p2 = under_test.get_next_position();
        let p3 = under_test.get_next_position();
        let p4 = under_test.get_next_position();
        assert_eq!(p1, Point { x: 0.0, y: 0.0 });
        assert_eq!(p2, Point { x: 5.0, y: 0.0 });
        assert_eq!(p3, Point { x: 10.0, y: 0.0 });
        assert_eq!(p4, Point { x: 0.0, y: 5.0 });
    }

    fn get_nodes_approximation_picture(graph: &Graph) -> String {
        let nodes_approximation = graph.get_transposed_nodes_approximation();
        let max_x = nodes_approximation
            .iter()
            .map(|node| node.position.x)
            .reduce(usize::max)
            .unwrap();
        let max_y = nodes_approximation
            .iter()
            .map(|node| node.position.y)
            .reduce(usize::max)
            .unwrap();

        let mut builder = String::new();
        for y in 0..=max_y {
            for x in 0..=max_x {
                let potential_char = nodes_approximation
                    .iter()
                    .find(|a| a.position.x == x && a.position.y == y);
                match potential_char {
                    None => builder.push(' '),
                    Some(c) => builder.push(c.name.chars().next().unwrap()),
                }
            }
            builder.push('\n');
        }
        builder
    }

    fn test(attraction_strength: f32, repulsion_strength: f32) -> String {
        let mut graph = Graph::new();
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("C", "A");

        graph.force_directed(Some(attraction_strength), Some(repulsion_strength));
        get_nodes_approximation_picture(&graph)
    }

    fn test2(attraction_strength: f32, repulsion_strength: f32) -> String {
        let mut graph = Graph::new();
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_node("D");
        graph.add_node("E");
        graph.add_node("F");
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("C", "D");
        graph.add_edge("D", "B");
        graph.add_edge("D", "A");
        graph.add_edge("A", "E");
        graph.add_edge("E", "F");

        graph.force_directed(Some(attraction_strength), Some(repulsion_strength));
        get_nodes_approximation_picture(&graph)
    }

    fn test_with_extra_edges(attraction_strength: f32, repulsion_strength: f32) -> String {
        let mut graph = Graph::new();
        graph.add_node("A");
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_edge("A", "B");
        graph.add_edge("A", "B");
        graph.add_edge("A", "A");
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("C", "A");

        graph.force_directed(Some(attraction_strength), Some(repulsion_strength));
        get_nodes_approximation_picture(&graph)
    }

    #[test]
    fn get_example01() {
        let result = test(1.0, 2.0);
        let expected = "A     C
       
       
       
       
   B   
";
        assert_eq!(result, expected);
    }

    #[test]
    fn get_example01_with_unnecessary_edges() {
        let result = test_with_extra_edges(1.0, 2.0);
        let expected = "A     C
       
       
       
       
   B   
";
        assert_eq!(result, expected);
    }

    #[test]
    fn get_example02() {
        let result = test2(1.0, 1.0);
        let expected = "   C   
       
       
       
B      
      D
       
       
       
       
   A   
       
       
       
       
       
   E   
       
       
       
       
       
   F   
";
        assert_eq!(result, expected);
    }
}
