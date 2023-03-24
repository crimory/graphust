use std::env;

mod graphust;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Example usage: graphust \"A -> B\"");
        return;
    }
    
    let output = graphust::get_graph(&args[1]);
    if let Ok(output) = output {
        println!("{}", output);
    } else {
        println!("Error: {}", output.unwrap_err());
    }
}
