#[derive(Debug)]
pub enum Command<'a> {
    Empty, Simple(Vec<&'a str>)
}

pub fn parse(line: &str) -> Command {
    let mut cursor: usize = 0;
    let mut words = Vec::new();
    loop {
        let (word, length) = read_word(&line[cursor..]);
        cursor += length;
        if word.len() == 0 {
            break;
        }
        words.push(word);
    }
    if words.len() > 0 {
        Command::Simple(words)
    } else {
        Command::Empty
    }
}

fn read_word(line: &str) -> (&str, usize) {
    let start = match line.find(|c: char| !c.is_whitespace()) {
        Some(v) => v,
        None => return ("", line.len()),
    };
    let length = match (&line[start..]).find(char::is_whitespace) {
        Some(v) => v,
        None => line.len(),
    };
    (&line[start..start+length], start+length)
}
