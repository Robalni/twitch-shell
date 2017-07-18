extern crate curl;
extern crate json;
extern crate rand;
use self::curl::easy::{Easy, List};
use std::io::Read;
use self::json::JsonValue;
use self::rand::Rng;

const BASE_URL: &str = "https://api.twitch.tv/kraken/";
const CLIENT_ID: &str = "dl1xe55lg2y26u8njj769lxhq3i47r";
const CLIENT_SECRET: &str = "39pxzfwymvpmaqoovj6fyj41idov1j";
const REDIRECT_URI: &str = "http://localhost:49814";

pub struct Api {
    easy: Easy,
}

impl Api {
    pub fn new() -> Self {
        curl::init();
        Api {
            easy: Easy::new(),
        }
    }

    pub fn get(&mut self, path: &str) -> Result<JsonValue, String> {
        self.easy.url(&(BASE_URL.to_owned() + path)).unwrap();
        self.easy.put(false).unwrap();
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
        let json_str = String::from_utf8(buf).unwrap();
        let obj = json::parse(&json_str);
        match obj {
            Ok(o) => {
                if o["error"].is_null() {
                    Ok(o)
                } else {
                    Err(o["error"].to_string()
                        + " (" + &(o["message"].to_string()) + ")")
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn put(&mut self, path: &str, mut data: &[u8], oauth: &str)
               -> Result<JsonValue, String> {
        self.easy.url(&(BASE_URL.to_owned() + path)).unwrap();
        let mut headers = List::new();
        headers.append(&("Client-ID: ".to_owned() + CLIENT_ID)).unwrap();
        headers.append(&("Authorization: OAuth ".to_owned() + oauth)).unwrap();
        self.easy.http_headers(headers).unwrap();
        self.easy.put(true).unwrap();
        self.easy.post_field_size(data.len() as u64).unwrap();
        let mut response = Vec::new();
        {
            let mut transfer = self.easy.transfer();
            transfer.read_function(|buf| {
                Ok(data.read(buf).unwrap_or(0))
            }).unwrap();
            transfer.write_function(|buf| {
                response.extend_from_slice(buf);
                Ok(buf.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        let json_str = String::from_utf8(response).unwrap();
        let obj = json::parse(&json_str);
        match obj {
            Ok(o) => {
                if o["error"].is_null() {
                    Ok(o)
                } else {
                    Err(o["error"].to_string()
                        + " (" + &(o["message"].to_string()) + ")")
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn post(&mut self, path: &str, mut data: &[u8], oauth: &str)
                -> Result<JsonValue, String> {
        self.easy.url(&(BASE_URL.to_owned() + path)).unwrap();
        let mut headers = List::new();
        headers.append(&("Client-ID: ".to_owned() + CLIENT_ID)).unwrap();
        headers.append(&("Authorization: OAuth ".to_owned() + oauth)).unwrap();
        self.easy.http_headers(headers).unwrap();
        self.easy.post(true).unwrap();
        self.easy.post_field_size(data.len() as u64).unwrap();
        let mut response = Vec::new();
        {
            let mut transfer = self.easy.transfer();
            transfer.read_function(|buf| {
                Ok(data.read(buf).unwrap_or(0))
            }).unwrap();
            transfer.write_function(|buf| {
                response.extend_from_slice(buf);
                Ok(buf.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        let json_str = String::from_utf8(response).unwrap();
        let obj = json::parse(&json_str);
        match obj {
            Ok(o) => {
                if o["error"].is_null() {
                    Ok(o)
                } else {
                    Err(o["error"].to_string()
                        + " (" + &(o["message"].to_string()) + ")")
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_login_url(&mut self, state: &str) -> String {
        let state_url = self.easy.url_encode(state.as_bytes());
        format!("{}oauth2/authorize?client_id={}&redirect_uri=http://localhost:49814&response_type=code&scope=channel_editor&state={}",
                BASE_URL, CLIENT_ID, state_url)
    }

    pub fn login(&mut self, code: &str) -> Result<JsonValue, String> {
        let mut osrng = rand::os::OsRng::new().unwrap();
        let state: String = osrng.gen_ascii_chars().take(20).collect();
        let url = format!("{}oauth2/token?client_id={}&client_secret={}&code={}\
                           &grant_type=authorization_code\
                           &redirect_uri={}&state={}",
                          BASE_URL, CLIENT_ID, CLIENT_SECRET, code,
                          REDIRECT_URI, state);
        self.easy.url(&url).unwrap();
        self.easy.post(true).unwrap();
        let mut data: &[u8] = &[0];
        let mut response = Vec::new();
        {
            let mut transfer = self.easy.transfer();
            transfer.read_function(|buf| {
                Ok(data.read(buf).unwrap_or(0))
            }).unwrap();
            transfer.write_function(|buf| {
                response.extend_from_slice(buf);
                Ok(buf.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        let json_str = String::from_utf8(response).unwrap();
        let obj = json::parse(&json_str);
        match obj {
            Ok(o) => {
                if o["error"].is_null() {
                    Ok(o)
                } else {
                    Err(o["error"].to_string()
                        + " (" + &(o["message"].to_string()) + ")")
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
