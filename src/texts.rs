use std::{io, process::Command};

use crate::rng::XorShift;

pub enum WordList {
    Dev,
    Casual,
    Music,
    Chatting,
}

impl WordList {
    pub fn words(&self) -> &'static str {
        match self {
            WordList::Dev => include_str!("texts/dev-words.txt"),
            WordList::Casual => include_str!("texts/casual-words.txt"),
            WordList::Music => include_str!("texts/music-words.txt"),
            WordList::Chatting => include_str!("texts/chatting-words.txt"),
        }
    }
}

impl std::str::FromStr for WordList {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(WordList::Dev),
            "casual" => Ok(WordList::Casual),
            "music" => Ok(WordList::Music),
            "chatting" => Ok(WordList::Chatting),
            _ => Err(()),
        }
    }
}

pub fn get_target_word_list(list: WordList) -> io::Result<String> {
    let tw = get_terminal_width()?;
    let mut buff = String::with_capacity(tw);
    let mut rng = XorShift::new();

    let words: Vec<&str> = list.words().lines().collect();

    while buff.len() < tw - 5 {
        let next_word = words[rng.next_bound(words.len())];
        if buff.len() + next_word.len() < tw - 5 {
            buff.push_str(next_word);
            buff.push(' ');
        } else {
            break;
        }
    }
    buff.pop();

    return Ok(buff);
}

fn get_terminal_width() -> io::Result<usize> {
    let output = Command::new("tput")
        .arg("cols")
        .stdin(std::fs::File::open("/dev/tty")?)
        .output()?;

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        s.trim()
            .parse::<usize>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "tput command failed"))
    }
}
