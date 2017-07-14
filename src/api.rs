extern crate curl;
use self::curl::easy::Easy;

pub struct Api {
    easy: Easy,
}

impl Api {
    pub fn new() -> Self {
        Api {
            easy: Easy::new(),
        }
    }

    pub fn get(&self) {
        
    }
}
