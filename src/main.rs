use std::{env, io, process};

use texts::{WordList, get_target_word_list};

mod render;
mod rng;
mod texts;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let word_list = match args.get(0).map(|s| s.as_str()) {
        None => WordList::Casual,
        Some(s) => s.parse().unwrap_or_else(|_| {
            eprintln!(
                "Usage: {} [dev|casual|music|chatting]",
                env::args().next().unwrap()
            );
            process::exit(1);
        }),
    };
    let target = get_target_word_list(word_list)?;
    render::render_loop(&target)?;
    Ok(())
}
