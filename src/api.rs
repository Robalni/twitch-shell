extern crate curl;
use self::curl::easy::Easy;

const BASE_URL: &str = "https://api.twitch.tv/kraken/";

pub struct Api {
    easy: Easy,
}

impl Api {
    pub fn new() -> Self {
        Api {
            easy: Easy::new(),
        }
    }

    pub fn get(&mut self, path: &str) {
        self.easy.url(&(BASE_URL.to_owned() + path)).unwrap();
        let mut buf = Vec::new();
        {
            let mut transfer = self.easy.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        println!("{}", String::from_utf8(buf).unwrap());
    }
}
