use std::io::Write;

fn main() {
    let mut line = String::new();
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
            
        } else {
            println!("{}", line);
        }
    }
}

struct Api {

}

impl Api {
    fn get() {

    }
}
