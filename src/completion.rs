use rustyline;
use rustyline::error::ReadlineError;

pub struct Completer {
    pub names: Vec<String>,
}

impl Completer {
    pub fn new() -> Completer {
        Completer {
            names: Vec::new(),
        }
    }
}

impl rustyline::completion::Completer for Completer {
    fn complete(&self, line: &str, pos: usize)
                    -> Result<(usize, Vec<String>), ReadlineError> {
        let mut v = Vec::new();
        let word = line.split(' ').last().unwrap_or("").to_owned();
        for n in &self.names {
            if n.starts_with(&word) {
                v.push(n.to_owned());
            }
        }
        Ok((line.len() - word.len(), v))
    }
}

