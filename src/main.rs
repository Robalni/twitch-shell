extern crate urlencoding;

mod api;

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
        if line == "\n" {
        } else if line == "status\n" {
            let obj = api.get(&("channels/".to_owned() + &encode(username)));
            let o = match obj {
                Ok(v) => v,
                Err(e) => { println!("Error: {}", e); continue; }
            };
            println!("{} playing {}\n  {}",
                     o["display_name"], o["game"], o["status"]);
        } else {
            println!("{}", line);
        }
    }
}
