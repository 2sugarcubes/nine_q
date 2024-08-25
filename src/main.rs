use std::{
    io::{self, Error},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use nine_q_lib::{game::NineP, load_9p_like_words};
use rayon::slice::ParallelSliceMut;

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

    /// Sorting method to use when displaying results
    #[arg(value_enum, default_value_t = Sorting::Alpha)]
    sorting: Sorting,

    #[arg(short, long, action = clap::ArgAction::Count, value_parser = clap::value_parser!(u8).range(0..6), default_value_t = 0_u8)]
    verbosity: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Sorting {
    /// Sorts words A-Z
    Alpha,
    /// Sorts words Z-A
    RevAlpha,
    /// Sort by ascending length of the words
    Length,
    /// Sort by decending length of the words
    RevLength,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    // Set verbosity level
    let level = match cli.verbosity {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => unreachable!(
            "Verbosity level '{}' is not a valid level, select one between 0 and 5 (inclusive)",
            cli.verbosity
        ),
    };
    env_logger::builder()
        .default_format()
        .filter_level(level)
        .init();

    let word_list = cli.word_list.into_os_string();

    if word_list.to_str().is_none() {
        return std::io::Result::Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Path {:?} was not a valid path", word_list),
        ));
    }

    let word_tree = load_9p_like_words(word_list.to_str().unwrap())?;
    let board = NineP::new(cli.board, word_tree);

    let mut lines = board.solve();

    match cli.sorting {
        Sorting::RevAlpha => lines.reverse(),
        Sorting::Length => lines.par_sort_unstable_by(|a, b| a.len().cmp(&b.len())),
        Sorting::RevLength => lines.par_sort_unstable_by(|a, b| b.len().cmp(&a.len())),
        // No need to do anything for alphabetically sorting
        Sorting::Alpha => (),
    }

    for line in lines {
        println!("{}", line);
    }
    Ok(())
}
