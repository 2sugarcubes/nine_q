use log::info;
use pbr::{ProgressBar, Units};
use rayon::prelude::*;
use std::fs::{metadata, read_to_string};
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::time::Instant;
use std::{io, sync, thread};
use word_tree::WordTree;

pub mod game;
pub mod word_tree;

pub fn load_words_from_disk<P>(path: P) -> io::Result<WordTree>
where
    P: AsRef<Path> + ToString,
{
    let lines = read_to_string(path)?
        .par_lines()
        .map(String::from)
        .collect::<Vec<String>>();

    Ok(WordTree::new(&lines))
}

pub fn load_9p_like_words<P>(path: P) -> io::Result<WordTree>
where
    P: AsRef<Path> + ToString,
{
    let mut progress_bar = ProgressBar::new(metadata(&path)?.len());
    progress_bar.set_units(Units::Bytes);

    let mut start_time = Instant::now();

    // Create the result vec
    // Create MPSC for byte count

    let result = Arc::new(sync::Mutex::new(Vec::<String>::new()));
    let sent_result = result.clone();

    let (tx, rx) = mpsc::channel();

    let file = read_to_string(path)?;
    info!("Sucessfully read file");
    // Spawn thread for computing lines
    rayon::spawn(move || {
        let result = sent_result;
        let mut lock = result.lock().unwrap();
        lock.extend(
            file.par_lines()
                .filter_map(|s: &str| -> Option<String> {
                    // Let the main thread know how many bytes we just read
                    let _ = tx.send(s.bytes().len() + 2);
                    match s.len() {
                        4..=9 => Some(s.to_string()),
                        _ => None,
                    }
                })
                .collect::<Vec<String>>(),
        );
    });

    // Join thread and print progress
    for bytes in rx {
        progress_bar.add(bytes as u64);
    }

    progress_bar.finish_print("Finished loading words");
    info!("Loaded words");

    let lines = result.lock().unwrap();

    let mut duration = start_time - Instant::now();
    info!(
        "Took {}.{}s to load from disk",
        duration.as_secs(),
        duration.subsec_millis()
    );
    start_time = Instant::now();

    let result = Ok(WordTree::new(&lines));
    duration = start_time - Instant::now();
    info!(
        "Took {}.{}s to make tree",
        duration.as_secs(),
        duration.subsec_millis()
    );
    result
}

#[cfg(test)]
fn init_logger() {
    use log::LevelFilter;

    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::max())
        .try_init();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_english_words() {
        //init_logger();
        //let res = load_words_from_disk("words_eng.txt");
        //assert!(res.is_ok());
    }

    #[test]
    fn filter_english_words() {
        init_logger();
        let res = load_9p_like_words("words_eng.txt");
        assert!(res.is_ok());
    }
}
