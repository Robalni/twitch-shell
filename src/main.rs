extern crate urlparse;
extern crate yansi;
extern crate tiny_http;
extern crate rand;

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
        Ok(v) => Some(v),
        Err(e) => { println!("{}", e); None },
    };
    user.id = if let Some(ref name) = user.name {
        match string_from_file(&format!(".{}.id", name)) {
            Ok(v) => Some(v.parse().unwrap()),
            Err(_) => match api.get_user_id(&user) {
                Ok(v) => Some(v),
                Err(e) => { println!("{}", e); None },
            },
        }
    } else {
        None
    };
    user.oauth = if let Some(ref name) = user.name {
        match string_from_file(&format!(".{}.oauth", name)) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    } else {
        None
    };
    let mut line = String::new();
    let args = std::env::args();
    if args.len() > 1 {
        line = args.collect::<Vec<String>>()[1..].join(" ");
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, &user) {
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
        match execute_command(cmd, &mut api, &user) {
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

fn execute_command(cmd: Command, api: &mut Api, user: &User)
                   -> Result<(), String> {
    match cmd {
        Command::Empty => Ok(()),
        Command::Simple(c) => {
            match c[0] {
                "exit" => {
                    std::process::exit(0);
                },
                "help"|"?" => {
                    print_help();
                    Ok(())
                },
                "login" => {
                    login(api, user)
                },
                "s" => {
                    if c.len() == 1 {
                        status(api, user)
                    } else {
                        search(api, user, &c)
                    }
                },
                "search" => {
                    search(api, user, &c)
                },
                "status" => {
                    status(api, user)
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
            let joined_rhs = rhs.join(" ");
            match lhs[0] {
                "status" => {
                    let data = format!("channel[status]={}", joined_rhs);
                    let path = match user.id {
                        Some(id) => format!("channels/{}", id),
                        None => return Err("No user".to_owned()),
                    };
                    let s = api.put(&path, user, data.as_bytes());
                    match s {
                        Ok(_) => {
                            println!("set {} to {}", lhs[0], joined_rhs);
                            Ok(())
                        },
                        Err(e) => Err(e),
                    }
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

fn login(api: &mut Api, user: &User) -> Result<(), String> {
    let username = match user.name {
        Some(ref v) => v,
        None => return Err("No user".to_owned()),
    };
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
    let obj = api.login(&code);
    let o = match obj {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let ref oauth = o["access_token"];
    println!("Writing oauth token to file...");
    match string_to_file(&format!(".{}.oauth", username), &oauth.to_string()) {
        Err(e) => return Err(e),
        _ => (),
    }
    let r = "Everything went well. Now go back to the shell.";
    rq.respond(Response::from_string(r)).unwrap();
    println!("Done!");
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
    let obj = api.get(&path, user);
    let o = match obj {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut i = 0;
    let ref list = o["streams"];
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

fn status(api: &mut Api, user: &User) -> Result<(), String> {
    let obj = match user.id {
        Some(id) => api.get(&format!("channels/{}", id), user),
        None => api.get("channel", user),
    };
    let o = match obj {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    println!("{} playing {}\n  {}",
             Paint::new(&o["display_name"]).bold(),
             Paint::green(&o["game"]), o["status"]);
    Ok(())
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
    p("exit", "Exits the shell");
    p("help", "Prints help text");
    p("login", "Logs in to Twitch");
    p("s [str [page]]", "Alias for search or status if no arguments");
    p("search <str> [page]", "Searches for streams");
    p("status", "Prints information about your channel");
    p("w <channel>", "Alias for watch");
    p("watch <channel>", "Watch a stream (using mpv)");
    println!();
    println!("Variables:");
    p("status", "Status/title of the stream");
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
}
