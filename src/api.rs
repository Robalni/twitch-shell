extern crate curl;
extern crate json;
extern crate rand;
use self::curl::easy::{Easy, List};
use std::io::Read;
use self::json::JsonValue;
use self::rand::Rng;
use std::error::Error;
use ::User;

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

    pub fn get(&mut self, path: &str, user: &User)
               -> Result<JsonValue, String> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
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

    pub fn put(&mut self, path: &str, user: &User, data: &[u8])
               -> Result<JsonValue, String> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
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

    pub fn post(&mut self, path: &str, user: &User, data: &[u8])
                -> Result<JsonValue, String> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
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

    pub fn delete(&mut self, path: &str, user: &User)
                  -> Result<JsonValue, String> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
            http_method: HttpMethod::Delete,
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

    pub fn get_login_url(&mut self, state: &str) -> String {
        let state_url = self.easy.url_encode(state.as_bytes());
        format!("{}oauth2/authorize?client_id={}&redirect_uri=http://localhost:49814&response_type=code&scope=channel_editor+user_follows_edit&state={}",
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

    pub fn get_user_id(&mut self, user: &User) -> Result<i32, String> {
        let obj = match user.name {
            Some(ref name) => {
                let res = self.get(&format!("users?login={}", name), user);
                let obj = match res {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                if obj["_total"] != 0 {
                    obj["users"][0].clone()
                } else {
                    return Err("Could not get user information from Twitch"
                               .to_owned());
                }
            }
            None => {
                let res = self.get("channel", user);
                match res {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                }
            }
        };
        match obj["_id"].to_string().parse() {
            Ok(v) => Ok(v),
            Err(e) => Err(e.description().to_owned()),
        }
    }

    pub fn get_user_ids(&mut self, user: &User, names: &[&str])
                        -> Result<Vec<i32>, String> {
        let mut ids = Vec::new();
        let res = self.get(&format!("users?login={}", names.join(",")), user);
        let obj = match res {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if obj["_total"] == names.len() {
            let mut i = 0;
            while i < names.len() {
                let n = obj["users"][i]["_id"].to_string().parse();
                ids.push(n.unwrap());
                i += 1;
            }
        } else {
            return Err("Could not get ids for all channels".to_owned());
        }
        Ok(ids)
    }
}

enum HttpMethod {
    Get, Post, Put, Delete,
}

struct EasySettings<'a> {
    easy_handle: &'a mut Easy,
    oauth: &'a Option<String>,
    http_method: HttpMethod,
    url: &'a str,
    send_buf: Option<&'a [u8]>,
}

fn perform_curl(mut settings: EasySettings) -> Result<String, String> {
    settings.easy_handle.reset();
    match settings.http_method {
        HttpMethod::Get => {
            settings.easy_handle.get(true).unwrap();
        },
        HttpMethod::Post => {
            settings.easy_handle.post_field_size(settings.send_buf
                                                 .unwrap_or(&vec![])
                                                 .len() as u64).unwrap();
            settings.easy_handle.post(true).unwrap();
        },
        HttpMethod::Put => {
            settings.easy_handle.put(true).unwrap();
        },
        HttpMethod::Delete => {
            settings.easy_handle.custom_request("DELETE").unwrap();
        },
    };
    let mut headers = List::new();
    headers.append(&("Client-ID: ".to_owned() + CLIENT_ID)).unwrap();
    if let &Some(ref oauth) = settings.oauth {
        headers.append(&("Authorization: OAuth ".to_owned() + &oauth)).unwrap();
    }
    headers.append("Accept: application/vnd.twitchtv.v5+json").unwrap();
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
