use std::fs::read_to_string;
use std::io::Error;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(String::from(value));
        }

        Ok(Self { lines })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
