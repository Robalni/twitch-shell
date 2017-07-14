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
        println!("{:?}", cmd);
        execute_command(cmd, &mut api, username);
    }
}

fn execute_command(cmd: parser::Command, api: &mut Api, username: &str)
                   -> Result<(), String> {
    match cmd {
        Command::Empty => {},
        Command::Simple(c) => {
            if c[0] == "status" {
                let obj = api.get(&("channels/".to_owned()
                                    + &encode(username)));
                let o = match obj {
                    Ok(v) => v,
                    Err(e) => { return Err(e); }
                };
                println!("{} playing {}\n  {}",
                         o["display_name"], o["game"], o["status"]);
            }
            println!("{}", c[0]);
        },
    }
    Ok(())
}
