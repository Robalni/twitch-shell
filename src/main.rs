mod api;
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
            api.get(&("channels/".to_owned() + username));
        } else {
            println!("{}", line);
        }
    }
}
