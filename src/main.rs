use std::env;
use std::io;

mod graphust;

fn main() {
    let mut buffer = "".to_string();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        for line in io::stdin().lines() {
            buffer.push_str(&line.unwrap());
            buffer.push('\n');
        }
    } else {
        buffer = args[1].to_string();
    }

    let output = graphust::get_graph(&buffer);
    if let Ok(output) = output {
        println!("{}", output);
    } else {
        println!("Error: {}", output.unwrap_err());
    }
}
