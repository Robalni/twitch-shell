extern crate urlencoding;

mod api;
mod parser;

use parser::Command;
use urlencoding::encode;
use api::Api;
use std::io::Write;

fn main() {
    let username = "robalni";
    let mut line = String::new();
    let mut api = Api::new();
    loop {
        line.clear();
        print!("twitch> ");
        std::io::stdout().flush().unwrap();
        let chars_read = std::io::stdin().read_line(&mut line).unwrap();
        if chars_read == 0 {
            println!();
            break;
        }
        let cmd = parser::parse(&line);
        match execute_command(cmd, &mut api, username) {
            Err(e) => println!("Error: {}", e),
            Ok(_) => {},
        }
    }
}

fn execute_command(cmd: parser::Command, api: &mut Api, username: &str)
                   -> Result<(), String> {
    match cmd {
        Command::Empty => {},
        Command::Simple(c) => {
            match c[0] {
                "exit" => {
                    std::process::exit(0);
                },
                "search" => {
                    let limit = 7;
                    let offset = if c.len() > 2 {
                        (c[2].parse::<i32>().unwrap() - 1) * limit
                    } else {
                        0
                    };
                    let path = format!("search/streams?q={}&offset={}&limit={}",
                                       encode(c[1]), offset, limit);
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
                                 l["channel"]["display_name"],
                                 l["game"],
                                 l["channel"]["status"]);
                        i += 1;
                    }
                },
                "status" => {
                    let obj = api.get(&("channels/".to_owned()
                                        + &encode(username)));
                    let o = match obj {
                        Ok(v) => v,
                        Err(e) => { return Err(e); },
                    };
                    println!("{} playing {}\n  {}",
                             o["display_name"], o["game"], o["status"]);
                },
                _ => {
                    return Err("Unknown command: ".to_owned() + c[0]);
                },
            }
        },
    }
    Ok(())
}
