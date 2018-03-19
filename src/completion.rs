use rustyline;
use rustyline::error::ReadlineError;

pub struct Completer {
    pub names: LastSeenList<String>,
}

impl Completer {
    pub fn new() -> Completer {
        Completer {
            names: LastSeenList::new(64),
        }
    }
}

impl rustyline::completion::Completer for Completer {
    fn complete(&self, line: &str, pos: usize)
                    -> Result<(usize, Vec<String>), ReadlineError> {
        let mut v = Vec::new();
        let word = line.split(' ').last().unwrap_or("").to_owned();
        for n in self.names.iter().rev() {
            if n.starts_with(&word) {
                v.push(n.to_owned());
            }
        }
        Ok((line.len() - word.len(), v))
    }
}

pub struct LastSeenList<T: PartialEq> {
    // The last seen element is at the end.
    vec: Vec<T>,
}

impl<T: PartialEq> LastSeenList<T> {
    pub fn new(capacity: usize) -> Self {
        LastSeenList {
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: T) {
        let index = self.vec.iter().rposition(|x| x.eq(&value));
        match index {
            Some(i) => {
                self.vec.remove(i);
                self.vec.push(value);
            },
            None => {
                if self.vec.len() >= self.vec.capacity() {
                    self.vec.remove(0);
                }
                self.vec.push(value);
            },
        }
    }

    pub fn iter(&self) -> ::std::slice::Iter<T> {
        self.vec.iter()
    }
}
