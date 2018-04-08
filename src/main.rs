extern crate urlparse;
extern crate yansi;
extern crate tiny_http;
extern crate rand;
extern crate json;
extern crate rustyline;
extern crate chrono;

mod api;
mod parser;
mod completion;
mod error;

use error::MyError;
use rand::Rng;
use tiny_http::{Server, Response};
use yansi::{Paint, Color};
use parser::Command;
use urlparse::{urlparse, GetQuery, quote};
use api::Api;
use std::io::{Write, Read};
use std::fs::File;
use std::borrow::Borrow;
use completion::Completer;
use completion::LastSeenList;
use json::JsonValue;
use rustyline::Editor;

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
        Err(_) => None,
    };
    user.update(&mut api).unwrap_or_else(|e| println!("{}", e));
    let mut line = String::new();
    let args = std::env::args();
    let completer: Completer = Completer::new();
    let mut ed = Editor::<(Completer)>::new();
    ed.set_completer(Some(completer));
    if args.len() > 1 {
        line = args.collect::<Vec<String>>()[1..].join(" ");
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, &mut user, &mut ed) {
            Ok(_) => (),
            Err(e) => println!("{}", Paint::red(format!("Error: {}", e))),
        }
        return;
    }
    loop {
        line.clear();
        use rustyline::error::ReadlineError::*;
        line = match ed.readline(&get_prompt(user.name.clone())) {
            Ok(l) => l,
            Err(e) => {
                match e {
                    Eof => break,
                    Interrupted => println!("Interrupted"),
                    _ => println!("{}", Paint::red(format!("Error: {}", e))),
                };
                continue;
            },
        };
        ed.add_history_entry(line.as_ref());
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, &mut user, &mut ed) {
            Err(e) => println!("{}", Paint::red(format!("Error: {}", e))),
            Ok(_) => {},
        }
    }
}

fn get_prompt<T: std::fmt::Display>(username: Option<T>) -> String {
    let color = Color::RGB(0x64, 0x41, 0xa5);
    if let Some(name) = username {
        format!("{} ", color.paint(format!("{}@twitch>", name)).bold())
    } else {
        format!("{} ", color.paint("twitch>").bold())
    }
}

fn execute_command(cmd: Command,
                   api: &mut Api,
                   mut user: &mut User,
                   editor: &mut Editor<Completer>)
        -> Result<(), MyError> {
    macro_rules! namelist {
        ($ed:expr) => {
            &mut $ed.get_completer().unwrap().names
        }
    }
    match cmd {
        Command::Empty => Ok(()),
        Command::Simple(c) => {
            match c[0].borrow() {
                "api" => {
                    show_api(api, user, &c)
                },
                "edit" => {
                    edit_var(api, user, &c, editor)
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
                    show_following(api, user, namelist!(editor))
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
                        search(api, user, &c, namelist!(editor))
                    }
                },
                "search" => {
                    search(api, user, &c, namelist!(editor))
                },
                "status" => {
                    show_status(api, user, &c)
                },
                "streams" => {
                    show_streams(api, user, &c)
                },
                "time" => {
                    show_time(api, user, &c)
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
                    Err(MyError::from(format!("Unknown command: {}", &c[0])))
                },
            }
        },
        Command::Assign(lhs, rhs) => {
            if lhs.len() == 0 {
                return Err(MyError::from("No variable"));
            }
            let joined_rhs = rhs.join(" ");
            match lhs[0].borrow() {
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
                    Err(MyError::from(format!("Unknown variable: {}", &lhs[0])))
                }
            }
        },
    }
}

fn string_from_file(filename: &str) -> Result<String, MyError> {
    let path = std::env::home_dir().unwrap().join(".twitch").join(filename);
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf.trim().to_owned())
}

fn string_to_file(filename: &str, string: &str) -> Result<(), MyError> {
    let path = std::env::home_dir().unwrap().join(".twitch").join(filename);
    let mut file = File::create(path)?;
    let buf = string.as_bytes();
    file.write_all(&buf)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn login(api: &mut Api, user: &mut User) -> Result<(), MyError> {
    let server = Server::http("127.0.0.1:49814").unwrap();
    let mut osrng = rand::os::OsRng::new().unwrap();
    let state: String = osrng.gen_ascii_chars().take(20).collect();
    println!("{}\n", api.get_login_url(&state));
    println!("Open the url above in a web browser and authorize with Twitch.");
    println!("Come back to this shell when you are done.");
    let rq = server.recv()?;
    let url = urlparse(rq.url());
    let query = url.get_parsed_query().ok_or(MyError::from(
        "Could not get code from Twitch (failed to parse url)".to_owned()
    ))?;
    let code = query.get_first_from_str("code").ok_or(MyError::from(
        "Could not get code from Twitch (failed to find the code in the url)"
        .to_owned()
    ))?;
    let got_state = query.get_first_from_str("state").ok_or(MyError::from(
        "Could not get state from Twitch (failed to find the state in the url)"
        .to_owned()
    ))?;
    if got_state != state {
        return Err(MyError::from(
            "Got an invalid response (the state is different)".to_owned()
        ));
    }
    println!("Logging in...");
    user.name = None;
    user.id = None;
    user.oauth = None;
    let obj = api.login(user, &code)?;
    let ref oauth = obj["access_token"];
    if oauth.is_null() {
        return Err(MyError::from("Could not get access token"));
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

fn show_api<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
              -> Result<(), MyError> {
    if cmd.len() > 2 {
        return Err(MyError::from("Usage: api [path]"));
    }
    let path = if cmd.len() == 2 {
        cmd[1].borrow()
    } else {
        ""
    };
    let obj = api.get(path, user)?;
    println!("{}", obj.pretty(2));
    Ok(())
}

fn edit_var<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>,
                            editor: &mut Editor<Completer>)
                           -> Result<(), MyError> {
    if cmd.len() != 2 {
        return Err(MyError::from("Usage: edit <var>"));
    }
    let var = cmd[1].borrow();
    let prompt = &Paint::yellow(format!("{}: ", var)).bold().to_string();
    let channel_obj = match user.id {
        Some(id) => api.get(&format!("channels/{}", id), user),
        None => api.get("channel", user),
    }?;
    let initial = match var {
        "status"|"game" => {
            if channel_obj[var].is_null() {
                "".to_owned()
            } else {
                channel_obj[var].to_string()
            }
        },
        _ => return Err(MyError::from("Variable not found")),
    };
    let line = match editor.readline_with_initial(prompt, &initial) {
        Ok(l) => l,
        Err(e) => {
            return Err(MyError::from("Variable was not changed"));
        },
    };
    match var {
        "status" => set_status(api, user, &line),
        "game" => set_game(api, user, &line),
        _ => Err(MyError::from("Variable not found")),
    }
}


fn follow<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
                         -> Result<(), MyError> {
    if cmd.len() < 2 {
        return Err(MyError::from("Usage: follow <channel...>"));
    }
    if let Some(my_id) = user.id {
        let ch_ids = api.get_user_ids(user, &cmd[1..])?;
        for ch_id in ch_ids {
            let url = format!("users/{}/follows/channels/{}", my_id, ch_id);
            let obj = api.put(&url, user, &[])?;
            println!("Followed {}", obj["channel"]["name"]);
        }
        Ok(())
    } else {
        Err(MyError::from("No user id"))
    }
}

fn unfollow<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
        -> Result<(), MyError> {
    if cmd.len() < 2 {
        return Err(MyError::from("Usage: unfollow <channel...>"));
    }
    if let Some(my_id) = user.id {
        let ch_ids = api.get_user_ids(user, &cmd[1..])?;
        for (i, ch_id) in ch_ids.iter().enumerate() {
            let url = format!("users/{}/follows/channels/{}", my_id, ch_id);
            api.delete(&url, user)?;
            println!("Unollowed {}", cmd[i + 1].borrow());
        }
        Ok(())
    } else {
        Err(MyError::from("No user id"))
    }
}

fn show_following(api: &mut Api, user: &User,
                  namelist: &mut LastSeenList<String>)
            -> Result<(), MyError> {
    let obj = api.get("streams/followed", user)?;
    let mut i = 0;
    let ref list = obj["streams"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{} playing {}\n  {}",
                 Paint::new(&l["channel"]["name"]).bold(),
                 Paint::green(&l["game"]),
                 l["channel"]["status"]);
        namelist.push(l["channel"]["name"].to_string());
        i += 1;
    }
    Ok(())
}

fn search<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>,
                          namelist: &mut LastSeenList<String>)
        -> Result<(), MyError> {
    let limit = 10;
    if cmd.len() < 2 {
        return Err(MyError::from("Usage: search <str> [page]"));
    }
    let offset = if cmd.len() > 2 {
        match cmd[2].borrow().parse::<i32>() {
            Ok(v) => (v - 1) * limit,
            Err(_) => {
                return Err(MyError::from("Page must be a number"))
            },
        }
    } else {
        0
    };
    let q = quote(cmd[1].borrow(), b"")?;
    let path = format!("search/streams?query={}&offset={}&limit={}",
                       q, offset, limit);
    let obj = api.get(&path, user)?;
    let mut i = 0;
    let ref list = obj["streams"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{} playing {}\n  {}",
                 Paint::new(&l["channel"]["name"]).bold(),
                 Paint::green(&l["game"]),
                 l["channel"]["status"]);
        namelist.push(l["channel"]["name"].to_string());
        i += 1;
    }
    Ok(())
}

fn show_vods<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
             -> Result<(), MyError> {
    let limit = 10;
    if cmd.len() > 3 {
        return Err(MyError::from("Usage: vods [channel [page]]"));
    }
    let offset = if cmd.len() > 2 {
        (cmd[2].borrow().parse::<i32>().unwrap() - 1) * limit
    } else {
        0
    };
    let id = if cmd.len() > 1 {
        let channel = quote(cmd[1].borrow(), b"")?;
        api.get_user_ids(user, &[channel])?[0]
    } else {
        user.id.ok_or(MyError::from("No user id"))?
    };
    let path = format!("channels/{}/videos?offset={}&limit={}",
                       id, offset, limit);
    let obj = api.get(&path, user)?;
    let mut i = 0;
    let ref list = obj["videos"];
    while !list[i].is_null() {
        let ref l = list[i];
        println!("{}: {} - {}\n  {}",
                 Paint::cyan(&l["broadcast_type"]), &l["url"],
                 &l["recorded_at"], &l["title"]);
                 //Paint::new(&l["channel"]["name"]).bold(),
                 //Paint::green(&l["game"]),
                 //l["channel"]["status"]);
        i += 1;
    }
    Ok(())
}

fn show_status<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
               -> Result<(), MyError> {
    let print_channel_info: fn(JsonValue) -> Result<(), MyError> = |o| {
        println!("{} playing {}\n  {}",
                 Paint::new(&o["name"]).bold(),
                 Paint::green(&o["game"]), o["status"]);
        Ok(())
    };
    if cmd.len() > 1 {
        let names = &cmd[1..];
        let ids = api.get_user_ids(user, names)?;
        for id in ids {
            print_channel_info(api.get(&format!("channels/{}", id), user)?)?;
        }
    } else {
        let obj = match user.id {
            Some(id) => api.get(&format!("channels/{}", id), user),
            None => api.get("channel", user),
        };
        print_channel_info(obj?)?;
    }
    Ok(())
}

fn show_streams<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
                -> Result<(), MyError> {
    let print_stream_info: fn(JsonValue) -> Result<(), MyError> = |o| {
        let st = o["stream"].clone();
        if !st.is_null() {
            let ch = st["channel"].clone();
            println!("{} playing {}\n  {}",
                     Paint::new(&ch["name"]).bold(),
                     Paint::green(&ch["game"]), ch["status"]);
        }
        Ok(())
    };
    if cmd.len() > 1 {
        let names = &cmd[1..];
        let ids = api.get_user_ids(user, names)?;
        for id in ids {
            print_stream_info(api.get(&format!("streams/{}", id), user)?)?;
        }
    } else {
        let obj = match user.id {
            Some(id) => api.get(&format!("streams/{}", id), user),
            None => return Err(MyError::from("No user id")),
        };
        print_stream_info(obj?)?;
    }
    Ok(())
}

fn show_time<S: Borrow<str>>(api: &mut Api, user: &User, cmd: &Vec<S>)
             -> Result<(), MyError> {
    let print_stream_info = |obj: Result<json::JsonValue, MyError>| {
        let o = obj?;
        let st = o["stream"].clone();
        let started = st["created_at"].as_str()
            .ok_or(MyError::from("Stream not found"))?;
        let dur = match chrono::DateTime::parse_from_rfc3339(started) {
            Ok(v) => {
                chrono::Local::now().signed_duration_since(v)
            },
            Err(e) => {
                return Err(MyError::from(format!("Could not parse date: {}", e)))
            },
        };
        println!("{}", dur_to_str(&dur));
        Ok(())
    };
    if cmd.len() > 1 {
        let names = &cmd[1..];
        let ids = api.get_user_ids(user, names)?;
        for id in ids {
            print_stream_info(api.get(&format!("streams/{}", id), user))?;
        }
    } else {
        let obj = match user.id {
            Some(id) => api.get(&format!("streams/{}", id), user),
            None => return Err(MyError::from("No user id")),
        };
        print_stream_info(obj)?;
    }
    Ok(())
}

fn show_user<S: Borrow<str>>(user: &User, cmd: &Vec<S>) -> Result<(), MyError> {
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
        Err(MyError::from("Usage: user"))
    }
}

fn set_user(api: &mut Api, user: &mut User, name: &str)
        -> Result<(), MyError> {
    if name.len() > 0 {
        user.name = Some(name.to_owned());
        user.id = None;
        user.oauth = None;
        user.update(api)?;
        user.save_all()
    } else {
        user.name = None;
        user.id = None;
        user.oauth = None;
        user.save_name()
    }
}

fn set_status(api: &mut Api, user: &User, status: &str)
              -> Result<(), MyError> {
    let data = String::new();
    let status_url = quote(status, b"")?;
    let path = match user.id {
        Some(id) => format!("channels/{}?channel[status]={}", id, status_url),
        None => return Err(MyError::from("No user")),
    };
    api.put(&path, user, data.as_bytes())?;
    show_status(api, user, &vec!["status"])
}

fn set_game(api: &mut Api, user: &User, game: &str)
              -> Result<(), MyError> {
    let data = String::new();
    let game_url = quote(game, b"")?;
    let path = match user.id {
        Some(id) => format!("channels/{}?channel[game]={}", id, game_url),
        None => return Err(MyError::from("No user")),
    };
    api.put(&path, user, data.as_bytes())?;
    show_status(api, user, &vec!["status"])
}

fn watch<S: Borrow<str>>(cmd: &Vec<S>) -> Result<(), MyError> {
    if cmd.len() < 2 {
        return Err(MyError::from("Usage: watch <channel>"));
    }
    let channel = &cmd[1];
    let url = format!("https://twitch.tv/{}", channel.borrow());
    let cmd = "mpv";
    match std::process::Command::new(cmd).arg(url).status() {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(MyError::from(format!("Could not start player {}: {}", cmd, e)))
        },
    }
}

fn print_help() {
    let p = |cmd: &str, desc: &str| {
        println!("  {:<24}{}", cmd, desc);
    };
    println!("Commands:");
    p("?", "Prints help text");
    p("api [path]", "Explore the api");
    p("edit <var>", "Lets you edit the value of a variable");
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
    p("time [channel...]",
      "Shows for how long the channels have been streaming");
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

    fn save_all(&self) -> Result<(), MyError> {
        self.save_name()?;
        self.save_id()?;
        self.save_oauth()?;
        Ok(())
    }

    fn save_name(&self) -> Result<(), MyError> {
        string_to_file("username", match self.name {
            Some(ref v) => v,
            None => "",
        })
    }

    fn save_id(&self) -> Result<(), MyError> {
        let name = self.name.as_ref()
            .ok_or(MyError::from("Could not save id because user has no name"))?;
        let id = self.id.as_ref()
            .ok_or(MyError::from("Could not save id because user has no id"))?;
        string_to_file(&format!(".{}.id", name), &id.to_string())
    }

    fn save_oauth(&self) -> Result<(), MyError> {
        let name = self.name.as_ref()
            .ok_or(MyError::from("Could not save oauth because user has no name"))?;
        let oauth = self.oauth.as_ref()
            .ok_or(MyError::from("Could not save oauth because user has no oauth"))?;
        string_to_file(&format!(".{}.oauth", name), &oauth)
    }

    fn update(&mut self, api: &mut Api) -> Result<(), MyError> {
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

fn dur_to_str(dur: &chrono::Duration) -> String {
    let h = dur.num_hours();
    let m = dur.num_minutes() - h*60;
    let s = dur.num_seconds() - dur.num_minutes()*60;
    format!("{}:{:02}:{:02}", h, m, s)
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
