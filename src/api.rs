extern crate curl;
use self::curl::easy::{Easy, List};

const BASE_URL: &str = "https://api.twitch.tv/kraken/";
const CLIENT_ID: &str = "mpw1gvfyzd1mtnm7rzoln5icmkdecys";

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
        let mut headers = List::new();
        headers.append(&("Client-ID: ".to_owned() + CLIENT_ID)).unwrap();
        self.easy.http_headers(headers).unwrap();
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
