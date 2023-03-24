use std::env;

mod graphust;

fn main() {
    let args: Vec<String> = env::args().collect();
    // let test = graphust::input::read_input("A -> B");
    graphust::example_01();
}
