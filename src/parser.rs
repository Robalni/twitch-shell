#[derive(Debug)]
pub enum Command {
    Empty, Simple(Vec<String>), Assign(Vec<String>, Vec<String>)
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

fn read_word(line: &str) -> (String, usize) {
    let mut word = String::new();
    let start = match line.find(|c: char| !c.is_whitespace()) {
        Some(v) => v,
        None => return ("".to_owned(), line.len()),
    };
    if (&line[start..]).starts_with("=") {
        return ("=".to_owned(), start+1);
    }
    let mut length = 0;
    let mut quoted: Option<char> = None;
    for c in line[start..].chars() {
        if let Some(q) = quoted {
            if q == c {
                quoted = None;
                length += 1;
                continue;
            }
        } else if c == '\'' || c == '"' {
            quoted = Some(c);
            length += 1;
            continue;
        }
        if quoted.is_none() && (c.is_whitespace() || c == '=') {
            break;
        }
        word.push(c);
        length += 1;
    }
    (word, start + length)
}
