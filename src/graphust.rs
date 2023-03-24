mod domain;
mod input;

pub fn get_graph(input: &str) -> Result<String, String> {
    let map = input::read_input(input)?;
    Ok(map.get_picture())
}
