use std::{
    io::{self, Error},
    path::PathBuf,
};

use clap::Parser;
use nine_q_lib::{game::NineP, load_9p_like_words};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Text file containing newline seperated values for all valid words in the game. e.g.
    /// `words_eng.txt`
    #[arg(short, long, value_name = "FILE", default_value = "words_eng.txt")]
    word_list: PathBuf,

    /// The available letters to play with. e.g. `abcdefghi`
    #[arg(short, long, value_name = "LETTERS")]
    board: String,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let word_list = cli.word_list.into_os_string();

    if word_list.to_str().is_none() {
        return std::io::Result::Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Path {:?} was not a valid path", word_list),
        ));
    }

    let word_tree = load_9p_like_words(word_list.to_str().unwrap())?;
    let board = NineP::new(cli.board, word_tree);

    for line in board.solve() {
        println!("{}", line);
    }
    Ok(())
}
