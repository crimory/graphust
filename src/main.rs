use std::env;

mod graphust;

fn main() {
    let args: Vec<String> = env::args().collect();

    graphust::test();
}
