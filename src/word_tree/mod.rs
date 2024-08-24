use std::{cell::RefCell, default::Default};

use log::{debug, error, info, trace, warn};
use pbr::ProgressBar;

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

const LETTER_FROM_ID: &[&str] = &[
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "\n",
];

fn letter_to_id(letter: char) -> usize {
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
        tree.generate(words);
        trace!(
            "First layer of tree {:?}",
            tree.root
                .children
                .clone()
                .into_iter()
                .zip(0..tree.root.children.len())
                .map(|(c, i)| match c.is_some() {
                    true => LETTER_FROM_ID[i],
                    false => "_",
                })
                .collect::<Vec<&str>>()
        );
        tree
    }

    pub fn generate(&mut self, words: &[String]) {
        let mut pb = ProgressBar::new(words.len() as u64);
        for word in words {
            // reverse the string for easy popping
            let word = word.chars().rev().collect::<String>();
            self.root.generate(word);
            pb.inc();
        }
        info!("Consumed {} words", words.len());
    }

    pub fn get_words(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.root.get_words(String::new(), &mut result);
        info!("Found {} total words in tree", result.len());
        result
    }
}

impl LetterNode {
    pub fn new(remaining_word: String) -> Self {
        let mut this = LetterNode::default();
        this.generate(remaining_word);
        this
    }

    pub fn generate(&mut self, mut remaining_word: String) {
        if let Some(next_letter) = remaining_word.pop() {
            let id = letter_to_id(next_letter);

            // Make the node if it doesn't exist yet
            if self.children[id].is_none() {
                info!("Adding child node {}", next_letter);
                self.children[id] = Some(LetterNode::new(remaining_word));
            } else if let Some(mut child) = self.children[id].clone() {
                // Go into the node and keep making any missing nodes
                child.generate(remaining_word);
                self.children[id] = Some(child);
            }
        } else {
            // This is a terminal node
            trace!("Marking this node as terminal");
            self.is_terminator = true;
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
                let next_word = working_word.clone() + LETTER_FROM_ID[i];
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
}

#[cfg(test)]
mod test {
    use crate::init_logger;

    use super::WordTree;

    #[test]
    fn zoo() {
        init_logger();
        let mut test_data = WordTree::default();
        test_data.generate(&["zoo".to_owned()]);
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
