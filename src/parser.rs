#[derive(Debug)]
pub enum Command<'a> {
    Empty, Simple(Vec<&'a str>), Assign(Vec<&'a str>, Vec<&'a str>)
}

pub fn parse(line: &str) -> Command {
    let mut cursor: usize = 0;
    let mut lhs = Vec::new();
    let mut words = Vec::new();
    let mut assign = false;
    loop {
        let (word, length) = read_word(&line[cursor..]);
        cursor += length;
        if word.len() == 0 {
            break;
        }
        if word == "=" {
            assign = true;
            lhs = words.clone();
            words.clear();
        } else {
            words.push(word);
        }
    }
    if assign {
        Command::Assign(lhs, words)
    } else if words.len() > 0 {
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
    if (&line[start..]).starts_with("=") {
        return ("=", start+1);
    }
    let length = match (&line[start..])
        .find(|c: char| {c.is_whitespace() || c == '='}) {
        Some(v) => v,
        None => line.len(),
    };
    (&line[start..start+length], start+length)
}
