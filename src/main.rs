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
    let username = &match string_from_file("username") {
        Ok(v) => v,
        Err(e) => { println!("{}", e); return; },
    };
    let oauth = &match string_from_file(&(format!(".{}.oauth", username))) {
        Ok(v) => v,
        Err(e) => { println!("{}", e); String::new() },
    };
    let mut line = String::new();
    let mut api = Api::new();
    loop {
        line.clear();
        show_prompt(username);
        let chars_read = std::io::stdin().read_line(&mut line).unwrap();
        if chars_read == 0 {
            println!();
            break;
        }
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, username, oauth) {
            Err(e) => println!("{}", Paint::red(format!("Error: {}", e))),
            Ok(_) => {},
        }
    }
}

fn show_prompt(username: &str) {
    print!("{} ", Color::RGB(0x64, 0x41, 0xa5)
           .paint(format!("{}@twitch>", username)).bold());
    std::io::stdout().flush().unwrap();
}

fn execute_command(cmd: parser::Command, api: &mut Api,
                   username: &str, oauth: &str)
                   -> Result<(), String> {
    match cmd {
        Command::Empty => { Ok(()) },
        Command::Simple(c) => {
            match c[0] {
                "exit" => {
                    std::process::exit(0);
                },
                "help" => {
                    print_help();
                    Ok(())
                },
                "login" => {
                    login(api, username)
                },
                "search" => {
                    let limit = 7;
                    let offset = if c.len() > 2 {
                        (c[2].parse::<i32>().unwrap() - 1) * limit
                    } else {
                        0
                    };
                    let q = match quote(c[1], b"") {
                        Ok(v) => v,
                        Err(e) => return Err(e.to_string()),
                    };
                    let path = format!("search/streams?q={}&offset={}&limit={}",
                                       q, offset, limit);
                    let obj = api.get(&(path));
                    let o = match obj {
                        Ok(v) => v,
                        Err(e) => { return Err(e); },
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
                },
                "status" => {
                    let uname_url = match quote(username, b"") {
                        Ok(v) => v,
                        Err(e) => return Err(e.to_string()),
                    };
                    let obj = api.get(&("channels/".to_owned()
                                        + &uname_url));
                    let o = match obj {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };
                    println!("{} playing {}\n  {}",
                             o["display_name"], o["game"], o["status"]);
                    Ok(())
                },
                _ => {
                    return Err("Unknown command: ".to_owned() + c[0]);
                },
            }
        },
        Command::Assign(lhs, rhs) => {
            let joined_rhs = rhs.join(" ");
            match lhs[0] {
                "status" => {
                    let uname_url = match quote(username, b"") {
                        Ok(v) => v,
                        Err(e) => return Err(e.to_string()),
                    };
                    let path = format!("channels/{}", uname_url);
                    let data = format!("channel[status]={}", joined_rhs);
                    let s = api.put(&path, data.as_bytes(), oauth);
                    match s {
                        Ok(_) => {},
                        Err(e) => return Err(e),
                    }
                    println!("set {} to {}", lhs[0], joined_rhs);
                    Ok(())
                },
                _ => {
                    return Err("Unknown variable: ".to_owned() + lhs[0]);
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

fn login(api: &mut Api, username: &str) -> Result<(), String> {
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
    let obj = api.login(&code);
    let o = match obj {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let ref oauth = o["access_token"];
    match string_to_file(&(format!(".{}.oauth", username)), &oauth.to_string()) {
        Err(e) => return Err(e),
        _ => (),
    }
    let r = "Everything went well. Now go back to the shell.";
    rq.respond(Response::from_string(r)).unwrap();
    println!("Done!");
    Ok(())
}

fn print_help() {
    let p = |cmd: &str, desc: &str| {
        println!("  {:<24}{}", cmd, desc);
    };
    println!("Commands:");
    p("exit", "Exits the shell");
    p("help", "Prints help text");
    p("login", "Logs in to Twitch");
    p("search <str> [page]", "Searches for streams");
    p("status", "Prints information about your channel");
    println!();
    println!("Variables:");
    p("status", "Status/title of the stream");
    println!();
}
