extern crate urlparse;
extern crate yansi;
extern crate tiny_http;
extern crate rand;
extern crate json;

mod api;
mod parser;

use rand::Rng;
use tiny_http::{Server, Response};
use yansi::{Paint, Color};
use parser::Command;
use urlparse::{urlparse, GetQuery, quote};
use api::Api;
use std::io::{Write, Read};
use std::fs::File;
use std::error::Error;

fn main() {
    let mut user = User::new();
    let mut api = Api::new();
    user.name = match string_from_file("username") {
        Ok(v) => {
            if v.len() > 0 {
                Some(v)
            } else {
                None
            }
        },
        Err(e) => None,
    };
    user.update(&mut api).unwrap_or_else(|e| println!("{}", e));
    let mut line = String::new();
    let args = std::env::args();
    if args.len() > 1 {
        line = args.collect::<Vec<String>>()[1..].join(" ");
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, &mut user) {
            Ok(_) => (),
            Err(e) => println!("{}", Paint::red(format!("Error: {}", e))),
        }
        return;
    }
    loop {
        line.clear();
        show_prompt(user.name.clone());
        let chars_read = std::io::stdin().read_line(&mut line).unwrap();
        if chars_read == 0 {
            println!();
            break;
        }
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, &mut user) {
            Err(e) => println!("{}", Paint::red(format!("Error: {}", e))),
            Ok(_) => {},
        }
    }
}

fn show_prompt<T: std::fmt::Display>(username: Option<T>) {
    let color = Color::RGB(0x64, 0x41, 0xa5);
    if let Some(name) = username {
        print!("{} ", color.paint(format!("{}@twitch>", name)).bold());
    } else {
        print!("{} ", color.paint("twitch>").bold());
    }
    std::io::stdout().flush().unwrap();
}

fn execute_command(cmd: Command, api: &mut Api, mut user: &mut User)
                   -> Result<(), String> {
    match cmd {
        Command::Empty => Ok(()),
        Command::Simple(c) => {
            match c[0] {
                "api" => {
                    show_api(api, user, &c)
                },
                "exit" => {
                    std::process::exit(0);
                },
                "follow" => {
                    follow(api, user, &c)
                },
                "\x46\x72\x61\x6e\x6b\x65\x72\x5a" => {
                    print_some_shit();
                    Ok(())
                },
                "following"|"f" => {
                    show_following(api, user)
                },
                "help"|"?" => {
                    print_help();
                    Ok(())
                },
                "\x4b\x61\x70\x70\x61" => {
                    panic!("Can't handle it.");
                },
                "login" => {
                    login(api, &mut user)
                },
                "s" => {
                    if c.len() == 1 {
                        show_status(api, user, &c)
                    } else {
                        search(api, user, &c)
                    }
                },
                "search" => {
                    search(api, user, &c)
                },
                "status" => {
                    show_status(api, user, &c)
                },
                "streams" => {
                    show_streams(api, user, &c)
                },
                "unfollow" => {
                    unfollow(api, user, &c)
                },
                "user" => {
                    show_user(user, &c)
                },
                "vods" => {
                    show_vods(api, user, &c)
                },
                "watch"|"w" => {
                    watch(&c)
                },
                _ => {
                    Err("Unknown command: ".to_owned() + c[0])
                },
            }
        },
        Command::Assign(lhs, rhs) => {
            if lhs.len() == 0 {
                return Err("No variable".to_owned());
            }
            let joined_rhs = rhs.join(" ");
            match lhs[0] {
                "game" => {
                    set_game(api, user, &joined_rhs)
                },
                "status" => {
                    set_status(api, user, &joined_rhs)
                },
                "user" => {
                    set_user(api, user, &joined_rhs)
                },
                _ => {
                    Err("Unknown variable: ".to_owned() + lhs[0])
                }
            }
        },
    }
}

fn string_from_file(filename: &str) -> Result<String, String> {
    let path = std::env::home_dir().unwrap().join(".twitch").join(filename);
    match File::open(path) {
        Ok(mut f) => {
            let mut buf = String::new();
            match f.read_to_string(&mut buf) {
                Ok(_) => Ok(buf.trim().to_owned()),
                Err(e) => Err(e.description().to_owned()),
            }
        },
        Err(e) => Err(e.description().to_owned()),
    }
}

fn string_to_file(filename: &str, string: &str) -> Result<(), String> {
    let path = std::env::home_dir().unwrap().join(".twitch").join(filename);
    match File::create(path) {
        Ok(mut f) => {
            let buf = string.as_bytes();
            match f.write_all(&buf) {
                Ok(_) => {
                    match f.write_all(b"\n") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.description().to_owned()),
                    }
                },
                Err(e) => Err(e.description().to_owned()),
            }
        },
        Err(e) => Err(e.description().to_owned()),
    }
}

fn login(api: &mut Api, user: &mut User) -> Result<(), String> {
    let server = Server::http("127.0.0.1:49814").unwrap();
    let mut osrng = rand::os::OsRng::new().unwrap();
    let state: String = osrng.gen_ascii_chars().take(20).collect();
    println!("{}\n", api.get_login_url(&state));
    println!("Open the url above in a web browser and authorize with Twitch.");
    println!("Come back to this shell when you are done.");
    let rq = match server.recv() {
        Ok(rq) => rq,
        Err(e) => return Err(e.to_string()),
    };
    let url = urlparse(rq.url());
    let query = match url.get_parsed_query() {
        Some(v) => v,
        None => return Err("Could not get code from Twitch \
                            (failed to parse url)".to_owned()),
    };
    let code = match query.get_first_from_str("code") {
        Some(v) => v,
        None => return Err("Could not get code from Twitch \
                            (failed to find the code in the url)"
                           .to_owned()),
    };
    let got_state = match query.get_first_from_str("state") {
        Some(v) => v,
        None => return Err("Could not get state from Twitch \
                            (failed to find the state in the url)"
                           .to_owned()),
    };
    if got_state != state {
        return Err("Got an invalid response (the state is different)"
                   .to_owned());
    }
    println!("Logging in...");
    user.name = None;
    user.id = None;
    user.oauth = None;
    let obj = api.login(user, &code)?;
    let ref oauth = obj["access_token"];
    if oauth.is_null() {
        return Err("Could not get access token".to_owned());
    }
    user.oauth = Some(oauth.to_string());
    println!("Getting user information...");
    let chobj = api.get("channel", user)?;
    user.name = Some(chobj["name"].to_string());
    user.id = Some(chobj["_id"].to_string().parse().unwrap());
    println!("Saving user information...");
    user.save_all()?;
    let r = "Everything went well. Now go back to the shell.";
    rq.respond(Response::from_string(r)).unwrap();
    println!("Done!");
    Ok(())
}

fn show_api(api: &mut Api, user: &User, cmd: &Vec<&str>)
              -> Result<(), String> {
    if cmd.len() > 2 {
        return Err("Usage: api [path]".to_owned());
    }
    let path = if cmd.len() == 2 {
        cmd[1]
    } else {
        ""
    };
    let obj = api.get(path, user)?;
    println!("{}", obj.pretty(2));
    Ok(())
}

fn follow(api: &mut Api, user: &User, cmd: &Vec<&str>) -> Result<(), String> {
    if cmd.len() < 2 {
        return Err("Usage: follow <channel...>".to_owned());
    }
    if let Some(my_id) = user.id {
        let ch_ids = api.get_user_ids(user, &cmd[1..])?;
        for ch_id in ch_ids {
            let url = format!("users/{}/follows/channels/{}", my_id, ch_id);
            let obj = api.put(&url, user, &[])?;
            println!("Followed {}", obj["channel"]["display_name"]);
        }
        Ok(())
    } else {
        Err("No user id".to_owned())
    }
}

fn unfollow(api: &mut Api, user: &User, cmd: &Vec<&str>) -> Result<(), String> {
    if cmd.len() < 2 {
        return Err("Usage: unfollow <channel...>".to_owned());
    }
    if let Some(my_id) = user.id {
        let ch_ids = api.get_user_ids(user, &cmd[1..])?;
        for (i, ch_id) in ch_ids.iter().enumerate() {
            let url = format!("users/{}/follows/channels/{}", my_id, ch_id);
            api.delete(&url, user);
            println!("Unollowed {}", cmd[i + 1]);
        }
        Ok(())
    } else {
        Err("No user id".to_owned())
    }
}

fn show_following(api: &mut Api, user: &User) -> Result<(), String> {
    let obj = api.get("streams/followed", user)?;
    let mut i = 0;
    let ref list = obj["streams"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{} playing {}\n  {}",
                 Paint::new(&l["channel"]["display_name"]).bold(),
                 Paint::green(&l["game"]),
                 l["channel"]["status"]);
        i += 1;
    }
    Ok(())
}

fn search(api: &mut Api, user: &User, cmd: &Vec<&str>) -> Result<(), String> {
    let limit = 10;
    if cmd.len() < 2 {
        return Err("Usage: search <str> [page]".to_owned());
    }
    let offset = if cmd.len() > 2 {
        (cmd[2].parse::<i32>().unwrap() - 1) * limit
    } else {
        0
    };
    let q = match quote(cmd[1], b"") {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };
    let path = format!("search/streams?query={}&offset={}&limit={}",
                       q, offset, limit);
    let obj = api.get(&path, user)?;
    let mut i = 0;
    let ref list = obj["streams"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{} playing {}\n  {}",
                 Paint::new(&l["channel"]["display_name"]).bold(),
                 Paint::green(&l["game"]),
                 l["channel"]["status"]);
        i += 1;
    }
    Ok(())
}

fn show_vods(api: &mut Api, user: &User, cmd: &Vec<&str>)
             -> Result<(), String> {
    let limit = 10;
    if cmd.len() > 3 {
        return Err("Usage: vods [channel [page]]".to_owned());
    }
    let offset = if cmd.len() > 2 {
        (cmd[2].parse::<i32>().unwrap() - 1) * limit
    } else {
        0
    };
    let id = if cmd.len() > 1 {
        let channel = match quote(cmd[1], b"") {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };
        api.get_user_ids(user, &[&channel])?[0]
    } else {
        match user.id {
            Some(v) => v,
            None => return Err("No user id".to_owned()),
        }
    };
    let path = format!("channels/{}/videos?offset={}&limit={}",
                       id, offset, limit);
    let obj = api.get(&path, user)?;
    let mut i = 0;
    let ref list = obj["videos"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{}: {}\n  {}",
                 Paint::cyan(&l["broadcast_type"]), &l["url"], &l["title"]);
                 //Paint::new(&l["channel"]["display_name"]).bold(),
                 //Paint::green(&l["game"]),
                 //l["channel"]["status"]);
        i += 1;
    }
    Ok(())
}

fn show_status(api: &mut Api, user: &User, cmd: &Vec<&str>)
               -> Result<(), String> {
    let print_channel_info = |obj: Result<json::JsonValue, String>| {
        let o = match obj {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        println!("{} playing {}\n  {}",
                 Paint::new(&o["display_name"]).bold(),
                 Paint::green(&o["game"]), o["status"]);
        Ok(())
    };
    if cmd.len() > 1 {
        let names = &cmd[1..];
        let ids = api.get_user_ids(user, names)?;
        for id in ids {
            print_channel_info(api.get(&format!("channels/{}", id), user));
        }
    } else {
        let obj = match user.id {
            Some(id) => api.get(&format!("channels/{}", id), user),
            None => api.get("channel", user),
        };
        print_channel_info(obj)?;
    }
    Ok(())
}

fn show_streams(api: &mut Api, user: &User, cmd: &Vec<&str>)
                -> Result<(), String> {
    let print_stream_info = |obj: Result<json::JsonValue, String>| {
        let o = match obj {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let st = o["stream"].clone();
        if !st.is_null() {
            let ch = st["channel"].clone();
            println!("{} playing {}\n  {}",
                     Paint::new(&ch["display_name"]).bold(),
                     Paint::green(&ch["game"]), ch["status"]);
        }
        Ok(())
    };
    if cmd.len() > 1 {
        let names = &cmd[1..];
        let ids = api.get_user_ids(user, names)?;
        for id in ids {
            print_stream_info(api.get(&format!("streams/{}", id), user));
        }
    } else {
        let obj = match user.id {
            Some(id) => api.get(&format!("streams/{}", id), user),
            None => return Err("No user id".to_owned()),
        };
        print_stream_info(obj)?;
    }
    Ok(())
}

fn show_user(user: &User, cmd: &Vec<&str>) -> Result<(), String> {
    if cmd.len() == 1 {
        let name = match user.name {
            Some(ref v) => v,
            None => "",
        };
        let id_str = match user.id {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };
        let has_oauth = match user.oauth {
            Some(_) => "yes",
            None => "no",
        };
        println!("{} (id:{}, has_oauth:{})", name, id_str, has_oauth);
        Ok(())
    } else {
        Err("Usage: user".to_owned())
    }
}

fn set_user(api: &mut Api, user: &mut User, name: &str) -> Result<(), String> {
    if name.len() > 0 {
        user.name = Some(name.to_owned());
        user.id = None;
        user.oauth = None;
        match user.update(api) {
            Ok(_) => user.save_all(),
            Err(e) => Err(e),
        }
    } else {
        user.name = None;
        user.id = None;
        user.oauth = None;
        user.save_name()
    }
}

fn set_status(api: &mut Api, user: &User, status: &str)
              -> Result<(), String> {
    let data = String::new();
    let status_url = match quote(status, b"") {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };
    let path = match user.id {
        Some(id) => format!("channels/{}?channel[status]={}", id, status_url),
        None => return Err("No user".to_owned()),
    };
    let s = api.put(&path, user, data.as_bytes());
    match s {
        Ok(_) => {
            show_status(api, user, &vec!["status"])
        },
        Err(e) => Err(e),
    }
}

fn set_game(api: &mut Api, user: &User, game: &str)
              -> Result<(), String> {
    let data = String::new();
    let game_url = match quote(game, b"") {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };
    let path = match user.id {
        Some(id) => format!("channels/{}?channel[game]={}", id, game_url),
        None => return Err("No user".to_owned()),
    };
    let s = api.put(&path, user, data.as_bytes());
    match s {
        Ok(_) => {
            show_status(api, user, &vec!["status"])
        },
        Err(e) => Err(e),
    }
}

fn watch(cmd: &Vec<&str>) -> Result<(), String> {
    if cmd.len() < 2 {
        return Err("Usage: watch <channel>".to_owned());
    }
    let channel = cmd[1];
    let url = format!("https://twitch.tv/{}", channel);
    let cmd = "mpv";
    match std::process::Command::new(cmd).arg(url).status() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Could not start player {}: {}", cmd, e)),
    }
}

fn print_help() {
    let p = |cmd: &str, desc: &str| {
        println!("  {:<24}{}", cmd, desc);
    };
    println!("Commands:");
    p("?", "Prints help text");
    p("api [path]", "Explore the api");
    p("exit", "Exits the shell");
    p("f", "Alias for following");
    p("follow <channel...>", "Follows the channel(s)");
    p("unfollow <channel...>", "Unfollows the channel(s)");
    p("following", "Shows online streams you follow");
    p("help", "Prints help text");
    p("login", "Logs in to Twitch");
    p("s [str [page]]", "Alias for search or status if no arguments");
    p("search <str> [page]", "Searches for streams");
    p("status [channel...]",
      "Shows info about channels (or your channel)");
    p("streams [channel...]",
      "Shows info about streams (or your stream) if online");
    p("user", "Prints information about current user");
    p("vods [channel [page]]", "Shows a list of videos from the channel");
    p("w <channel>", "Alias for watch");
    p("watch <channel>", "Watch a stream (using mpv)");
    println!();
    println!("Variables:");
    p("game", "The game you are playing");
    p("status", "Status/title of the stream");
    p("user", "Name of current user");
    println!();
}

pub struct User {
    id: Option<i32>,
    name: Option<String>,
    oauth: Option<String>,
}

impl User {
    fn new() -> Self {
        User {
            id: None,
            name: None,
            oauth: None,
        }
    }

    fn save_all(&self) -> Result<(), String> {
        self.save_name()?;
        self.save_id()?;
        self.save_oauth()?;
        Ok(())
    }

    fn save_name(&self) -> Result<(), String> {
        string_to_file("username", match self.name {
            Some(ref v) => v,
            None => "",
        })
    }

    fn save_id(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if let Some(ref id) = self.id {
                string_to_file(&format!(".{}.id", name), &id.to_string())?;
                Ok(())
            } else {
                Err("Could not save id because user has no id".to_owned())
            }
        } else {
            Err("Could not save id because user has no name".to_owned())
        }
    }

    fn save_oauth(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if let Some(ref oauth) = self.oauth {
                string_to_file(&format!(".{}.oauth", name), &oauth)?;
                Ok(())
            } else {
                Err("Could not save oauth because user has no oauth".to_owned())
            }
        } else {
            Err("Could not save oauth because user has no name".to_owned())
        }
    }

    fn update(&mut self, api: &mut Api) -> Result<(), String> {
        self.id = if let Some(ref name) = self.name {
            match string_from_file(&format!(".{}.id", name)) {
                Ok(v) => Some(v.parse().unwrap()),
                Err(_) => match api.get_user_id(self) {
                    Ok(v) => Some(v),
                    Err(e) => { println!("{}", e); None },
                },
            }
        } else {
            None
        };
        if self.id.is_some() {
            self.save_id().unwrap();
        }
        self.oauth = if let Some(ref name) = self.name {
            match string_from_file(&format!(".{}.oauth", name)) {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        } else {
            None
        };
        Ok(())
    }
}

fn print_some_shit() {
    println!(
        "{}",
        "\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2584}\u{20}\u{20}\
         \u{2584}\u{2584}\u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\
         \u{2584}\u{2584}\u{2584}\u{20}\u{20}\u{2584}\u{a}\u{20}\u{20}\u{20}\
         \u{2584}\u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2592}\
         \u{2592}\u{2592}\u{2591}\u{2591}\u{2591}\u{2591}\u{2592}\u{2592}\
         \u{2592}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}\u{2584}\
         \u{a}\u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\
         \u{2588}\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}\u{2591}\u{2591}\
         \u{20}\u{2591}\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}\u{2588}\
         \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}\u{a}\
         \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\
         \u{2592}\u{2591}\u{2588}\u{2588}\u{2591}\u{2592}\u{2591}\u{20}\
         \u{2591}\u{2592}\u{2592}\u{2591}\u{2588}\u{2588}\u{2592}\u{2588}\
         \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{a}\
         \u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{20}\
         \u{2592}\u{2592}\u{2592}\u{2592}\u{2592}\u{2591}\u{2591}\u{20}\
         \u{2591}\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}\u{2580}\
         \u{20}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{a}\u{20}\
         \u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{20}\u{2592}\
         \u{2592}\u{2592}\u{2592}\u{2591}\u{2591}\u{20}\u{20}\u{2591}\
         \u{2591}\u{2591}\u{2592}\u{2592}\u{2592}\u{20}\u{20}\u{2588}\
         \u{2588}\u{2588}\u{2588}\u{2588}\u{2580}\u{a}\u{20}\u{20}\u{20}\
         \u{2580}\u{2580}\u{2580}\u{20}\u{20}\u{20}\u{2592}\u{2592}\u{2591}\
         \u{2591}\u{2591}\u{2591}\u{2584}\u{2584}\u{2584}\u{2591}\u{2591}\
         \u{2592}\u{20}\u{20}\u{20}\u{20}\u{2580}\u{2580}\u{2580}\u{a}\u{20}\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2592}\
         \u{2592}\u{2591}\u{2591}\u{2591}\u{2588}\u{2588}\u{2588}\u{2591}\
         \u{2591}\u{2592}\u{a}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{2592}\u{2592}\u{2591}\u{2591}\
         \u{2591}\u{2591}\u{2592}\u{2592}\u{a}\u{20}\u{20}\u{20}\u{20}\u{20}\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2592}\
         \u{2592}\u{2592}\u{2592}"
    );
    loop {
        std::thread::sleep(std::time::Duration::from_millis(472));
        print!("\x46\x72\x61\x6e\x6b\x65\x72\x5a\x20");
        std::io::stdout().flush().unwrap();
    }
}
