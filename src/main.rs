use std::{env, io, process};

use texts::{WordList, get_target_word_list};

mod render;
mod rng;
mod texts;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let word_list = match args.first().map(String::as_str) {
        None => WordList::Casual,
        Some(s) => s.parse().unwrap_or_else(|_| {
            let bin_name = env::args().next().unwrap_or_else(|| "kb".to_string());
            eprintln!("Usage: {} [dev|casual|music|chatting]", bin_name);
            process::exit(1);
        }),
    };
    let target = get_target_word_list(word_list)?;
    render::render_loop(&target)?;
    Ok(())
}
