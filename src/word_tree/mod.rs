use std::{cell::RefCell, default::Default};

use log::{debug, error, info, trace, warn};
use pbr::ProgressBar;
use rayon::prelude::*;

#[derive(Default, Clone)]
pub struct WordTree {
    root: LetterNode,
}

#[derive(Clone, Debug)]
pub struct LetterNode {
    children: Vec<Option<LetterNode>>,
    is_terminator: bool,
}

impl Default for LetterNode {
    fn default() -> Self {
        LetterNode {
            // 26 Nones
            children: vec![None; 26],
            is_terminator: false,
        }
    }
}

const LETTER_FROM_ID: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

fn letter_to_id(letter: &char) -> usize {
    match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        'i' => 8,
        'j' => 9,
        'k' => 10,
        'l' => 11,
        'm' => 12,
        'n' => 13,
        'o' => 14,
        'p' => 15,
        'q' => 16,
        'r' => 17,
        's' => 18,
        't' => 19,
        'u' => 20,
        'v' => 21,
        'w' => 22,
        'x' => 23,
        'y' => 24,
        'z' => 25,
        '\n' => 26,
        '\r' => 26,
        _ => {
            error!("Character '{letter}' was not found in lookup table.");
            unreachable!()
        }
    }
}

impl WordTree {
    pub fn new(words: &[String]) -> Self {
        let mut tree = Self::default();
        let mut words = words.to_owned();
        tree.generate(&mut words);
        trace!(
            "First layer of tree {:?}",
            tree.root
                .children
                .clone()
                .into_iter()
                .zip(0..tree.root.children.len())
                .map(|(c, i)| match c.is_some() {
                    true => LETTER_FROM_ID[i],
                    false => '_',
                })
                .collect::<Vec<char>>()
        );
        tree
    }

    pub fn generate(&mut self, words: &mut [String]) {
        words.par_sort();
        let mut vec_words: Vec<String> = words.into();
        vec_words.dedup();
        self.root = LetterNode::new(vec_words);
        info!("Consumed {} words", words.len());
    }

    pub fn get_words(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.root.get_words(String::new(), &mut result);
        info!("Found {} total words in tree", result.len());
        result
    }
    pub fn solve(&self, available_letters: &String) -> Vec<String> {
        let mut available_letters: Vec<char> = available_letters.chars().collect();
        available_letters.sort();
        let mut results = Vec::new();
        self.root
            .solve(available_letters, String::new(), &mut results);
        results
    }
}

impl LetterNode {
    pub fn new(remaining_words: Vec<String>) -> Self {
        // The empty string should be sorted to the end of the list
        let is_terminator = remaining_words
            .first()
            .unwrap_or(&"".to_string())
            .is_empty();

        for i in 1..remaining_words.len() - 1 {
            if remaining_words[i].is_empty() {
                warn!("Empty string was found at index {}", i);
            }
        }

        let mut children: [Option<LetterNode>; 26] = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None,
        ];

        let mut end = remaining_words.len();
        if remaining_words.last() == Some(&"".to_string()) {
            // Ignore the last string if it is empty
            end -= 1;
        }
        for (i, c) in (0..26_usize).zip(LETTER_FROM_ID.iter()).rev() {
            let start = remaining_words
                .binary_search(&c.to_string())
                .unwrap_or_else(|i| i);

            if start >= end {
                // No words found for this letter.
                children[i] = None;
                //trace!("Char '{}' not found.", c);
            } else {
                // Scope to words starting with the right letter
                if remaining_words.len() <= 10 {
                    trace!("words={:?}", remaining_words);
                }
                let dequed_words = remaining_words[start..end]
                    .iter()
                    // Deque the first letter of the word (we already removed the empty string if there
                    // was one, so this shouldn't panic)
                    .map(|s| {
                        if s.is_empty() {
                            panic!("String should not be empty")
                        } else if !s.starts_with(*c) {
                            panic!(
                                "String ({}) is not sorted, expected {} start={} end={}",
                                s, c, start, end
                            );
                        }
                        s[1..].to_string()
                    })
                    .collect::<Vec<String>>();
                children[i] = Some(LetterNode::new(dequed_words));

                if start == 0 {
                    break;
                }
                // Look for the next letter
                end = start;
            }
        }

        LetterNode {
            children: children.into(),
            is_terminator,
        }
    }

    fn is_terminator(&self) -> bool {
        self.is_terminator
    }

    pub fn get_words(&self, working_word: String, results: &mut Vec<String>) {
        trace!(
            "Current word is {}, viable children = {}",
            working_word,
            self.children
                .clone()
                .into_iter()
                .filter(|c| c.is_some())
                .count()
                + if self.is_terminator { 1 } else { 0 }
        );
        for (i, child) in (0_usize..self.children.len()).zip(self.children.iter()) {
            if let Some(child) = child {
                let mut next_word = working_word.clone();
                next_word.push(LETTER_FROM_ID[i]);
                trace!(
                    "Decending to child {} with word {}",
                    LETTER_FROM_ID[i],
                    next_word
                );
                (*child).get_words(next_word, results);
            }
        }

        if self.is_terminator() {
            info!("Adding word {} to results", working_word);
            results.push(working_word);
        }
    }

    fn solve(&self, available_letters: Vec<char>, current_word: String, results: &mut Vec<String>) {
        let mut thin_letters = available_letters.clone();
        thin_letters.dedup();
        for c in thin_letters.iter() {
            if self.children[letter_to_id(c)].is_some() {
                // There is at least one word that has this caracter in this location
                // This character must exist in the pool of available_letters
                let first_index = available_letters.binary_search(c).unwrap();
                let mut next_available_letters = available_letters.clone();
                next_available_letters.remove(first_index);

                // Set up next word
                let mut next_word = current_word.clone();
                next_word.push(*c);

                // Recurse
                self.children
                    .get(letter_to_id(c))
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .solve(next_available_letters, next_word, results);
            }
        }
        if self.is_terminator() {
            results.push(current_word);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::init_logger;

    use super::WordTree;

    #[test]
    fn zoo() {
        init_logger();
        let mut test_data = WordTree::default();
        test_data.generate(&mut ["zoo".to_string()]);
        assert_eq!(test_data.get_words(), ["zoo".to_string()]);
    }

    #[test]
    fn many_zoos() {
        init_logger();
        let test_data = WordTree::new(&["zoo".to_owned(), "zoo".to_owned(), "zoo".to_owned()]);
        assert_eq!(test_data.get_words(), ["zoo".to_string()]);
    }

    #[test]
    fn happy_words() {
        init_logger();
        let words: &[String] = &[
            "fabulous".to_string(),
            "happiness".to_string(),
            "happy".to_string(),
            "kinder".to_string(),
            "kind".to_string(),
            "radiant".to_string(),
        ];
        let test_data = WordTree::new(words);

        assert_eq!(test_data.get_words(), words);
    }
}
