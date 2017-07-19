extern crate curl;
extern crate json;
extern crate rand;
use self::curl::easy::{Easy, List};
use std::io::Read;
use self::json::JsonValue;
use self::rand::Rng;
use std::error::Error;

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
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: None,
            http_method: HttpMethod::Get,
            url: &(BASE_URL.to_owned() + path),
            send_buf: None,
        };
        let json_str = match perform_curl(settings) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
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

    pub fn put(&mut self, path: &str, data: &[u8], oauth: &str)
               -> Result<JsonValue, String> {
        self.easy.post_field_size(data.len() as u64).unwrap();
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: Some(oauth),
            http_method: HttpMethod::Put,
            url: &(BASE_URL.to_owned() + path),
            send_buf: Some(data),
        };
        let json_str = match perform_curl(settings) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
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

    pub fn post(&mut self, path: &str, data: &[u8], oauth: &str)
                -> Result<JsonValue, String> {
        self.easy.post_field_size(data.len() as u64).unwrap();
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: Some(oauth),
            http_method: HttpMethod::Post,
            url: &(BASE_URL.to_owned() + path),
            send_buf: Some(data),
        };
        let json_str = match perform_curl(settings) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
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

enum HttpMethod {
    Get, Post, Put,
}

struct EasySettings<'a> {
    easy_handle: &'a mut Easy,
    oauth: Option<&'a str>,
    http_method: HttpMethod,
    url: &'a str,
    send_buf: Option<&'a [u8]>,
}

fn perform_curl(mut settings: EasySettings) -> Result<String, String> {
    let (post, put) = match settings.http_method {
        HttpMethod::Get => (false, false),
        HttpMethod::Post => (true, false),
        HttpMethod::Put => (false, true),
    };
    settings.easy_handle.put(put).unwrap();
    settings.easy_handle.post(post).unwrap();
    let mut headers = List::new();
    headers.append(&("Client-ID: ".to_owned() + CLIENT_ID)).unwrap();
    if let Some(oauth) = settings.oauth {
        headers.append(&("Authorization: OAuth ".to_owned() + oauth)).unwrap();
    }
    headers.append("Accept: application/vnd.twitchtv.v3+json").unwrap();
    settings.easy_handle.http_headers(headers).unwrap();
    settings.easy_handle.url(settings.url).unwrap();

    let mut buf = Vec::new();
    {
        let mut transfer = settings.easy_handle.transfer();
        if let Some(mut from) = settings.send_buf {
            transfer.read_function(move |to| {
                Ok(from.read(to).unwrap_or(0))
            }).unwrap();
        }
        transfer.write_function(|from| {
            buf.extend_from_slice(from);
            Ok(from.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    let string = String::from_utf8(buf);
    match string {
        Ok(v) => Ok(v),
        Err(e) => Err(e.description().to_owned()),
    }
}
