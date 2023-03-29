mod domain;
mod input;

pub fn get_graph(input: &str) -> Result<String, String> {
    let map = input::read_input(input)?;
    Ok(map.get_picture())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_error() {
        let input = "A -> -> B";
        let output = get_graph(&input);
        assert!(output.is_err());
        if let Err(message) = output {
            assert_eq!(format!("Cannot understand this line: {}", input), message);
        }
    }

    #[test]
    fn example01() {
        let input = "\
A -> B
B -> C
C -> A";
        let expected = "\
+---+     +---+     +---+
| A | --> | B | --> | C |
+---+     +---+     +---+
  ^                   |  
  |                   |  
  |--------------------  
";
        let output = get_graph(&input);

        assert!(output.is_ok());
        if let Ok(ok_output) = output {
            assert_eq!(expected, ok_output);
        }
    }

    #[test]
    fn example02() {
        let input = "\
\"This is our test\" -> B
B -> C
C -> \"This is our test\"";
        let expected = "\
+------------------+     +---+     +---+
| This is our test | --> | B | --> | C |
+------------------+     +---+     +---+
  ^                                  |  
  |                                  |  
  |-----------------------------------  
";
        let output = get_graph(&input);

        assert!(output.is_ok());
        if let Ok(ok_output) = output {
            assert_eq!(expected, ok_output);
        }
    }
}
