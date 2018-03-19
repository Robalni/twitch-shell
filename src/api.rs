extern crate curl;
extern crate json;
extern crate rand;
use self::curl::easy::{Easy, List};
use std::io::Read;
use self::json::JsonValue;
use self::rand::Rng;
use ::User;
use ::MyError;
use std::borrow::Borrow;

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

    pub fn get<S: AsRef<str>>(&mut self, path: S, user: &User)
               -> Result<JsonValue, MyError> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
            http_method: HttpMethod::Get,
            url: &(BASE_URL.to_owned() + path.as_ref()),
            send_buf: None,
        };
        let json_str = perform_curl(settings)?;
        let obj = json::parse(&json_str)?;
        if obj["error"].is_null() {
            Ok(obj)
        } else {
            Err(MyError::from(format!("{} ({})", obj["error"], obj["message"])))
        }
    }

    pub fn put(&mut self, path: &str, user: &User, data: &[u8])
               -> Result<JsonValue, MyError> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
            http_method: HttpMethod::Put,
            url: &(BASE_URL.to_owned() + path),
            send_buf: Some(data),
        };
        let json_str = perform_curl(settings)?;
        let obj = json::parse(&json_str)?;
        if obj["error"].is_null() {
            Ok(obj)
        } else {
            Err(MyError::from(format!("{} ({})", obj["error"], obj["message"])))
        }
    }

    pub fn post(&mut self, path: &str, user: &User, data: &[u8])
                -> Result<JsonValue, MyError> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
            http_method: HttpMethod::Post,
            url: &(BASE_URL.to_owned() + path),
            send_buf: Some(data),
        };
        let json_str = perform_curl(settings)?;
        let obj = json::parse(&json_str)?;
        if obj["error"].is_null() {
            Ok(obj)
        } else {
            Err(MyError::from(format!("{} ({})", obj["error"], obj["message"])))
        }
    }

    pub fn delete(&mut self, path: &str, user: &User)
                  -> Result<JsonValue, MyError> {
        let settings = EasySettings {
            easy_handle: &mut self.easy,
            oauth: &user.oauth,
            http_method: HttpMethod::Delete,
            url: &(BASE_URL.to_owned() + path),
            send_buf: None,
        };
        let json_str = perform_curl(settings)?;
        let obj = json::parse(&json_str)?;
        if obj["error"].is_null() {
            Ok(obj)
        } else {
            Err(MyError::from(format!("{} ({})", obj["error"], obj["message"])))
        }
    }

    pub fn get_login_url(&mut self, state: &str) -> String {
        let state_url = self.easy.url_encode(state.as_bytes());
        format!("{}oauth2/authorize?client_id={}\
                &redirect_uri=http://localhost:49814\
                &response_type=code\
                &scope=channel_editor+user_follows_edit+channel_read\
                &state={}",
                BASE_URL, CLIENT_ID, state_url)
    }

    pub fn login(&mut self, user: &User, code: &str)
                 -> Result<JsonValue, MyError> {
        let mut osrng = rand::os::OsRng::new().unwrap();
        let state: String = osrng.gen_ascii_chars().take(20).collect();
        let path = format!("oauth2/token?client_id={}&client_secret={}&code={}\
                            &grant_type=authorization_code\
                            &redirect_uri={}&state={}",
                           CLIENT_ID, CLIENT_SECRET, code,
                           REDIRECT_URI, state);
        self.post(&path, user, &[])
    }

    pub fn get_user_id(&mut self, user: &User) -> Result<i32, MyError> {
        let obj = match user.name {
            Some(ref name) => {
                let obj = self.get(&format!("users?login={}", name), user)?;
                if obj["_total"] != 0 {
                    obj["users"][0].clone()
                } else {
                    return Err(MyError::from(
                        "Could not get user information from Twitch"
                    ))
                }
            }
            None => {
                self.get("channel", user)?
            }
        };
        let n = obj["_id"].to_string().parse::<i32>()?;
        Ok(n)
    }

    pub fn get_user_ids<S: Borrow<str>>(&mut self, user: &User, names: &[S])
                        -> Result<Vec<i32>, MyError> {
        let mut ids = Vec::new();
        let obj = self.get(&format!("users?login={}", names.join(",")), user)?;
        if obj["_total"] == names.len() {
            let mut i = 0;
            while i < names.len() {
                let n = obj["users"][i]["_id"].to_string().parse();
                ids.push(n.unwrap());
                i += 1;
            }
        } else {
            return Err(MyError::from("Could not get ids for all channels"));
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

fn perform_curl(settings: EasySettings) -> Result<String, MyError> {
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
        transfer.perform()?;
    }
    Ok(String::from_utf8(buf)?)
}

impl From<curl::Error> for MyError {
    fn from(e: curl::Error) -> Self {
        MyError::NetworkError(e)
    }
}
