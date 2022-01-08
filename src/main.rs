use clap::{ArgGroup, Parser};
use itertools::izip;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::iter::repeat;
use std::str;

const WORD_LENGTH: usize = 5;
const GUESS_COUNT: usize = 6;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(group = ArgGroup::new("art").required(true))]
struct Args {
    /// The solution to today's Wordle puzzle
    solution: String,
    /// The target pattern (lines may be separated by / or newlines)
    #[clap(short, long, group = "art")]
    pattern: Option<String>,
    /// Path to artfile containing the target pattern
    #[clap(short = 'f', long, group = "art")]
    artfile: Option<String>,
}

fn main() {
    let args = Args::parse();

    assert!(
        args.solution.len() == WORD_LENGTH,
        "Solution should be 5 letters",
    );

    let all_words: Vec<&str> = std::str::from_utf8(include_bytes!("../dict.txt"))
        .expect("Could not read dictionary")
        .lines()
        .collect();

    let goal_shape = match (args.pattern, args.artfile) {
        (Some(pattern), _) => pattern_from_string(&pattern),
        (_, Some(artfile)) => pattern_from_file(&artfile).expect("Could not read artfile"),
        // Arg validation requires that one of the above must match.
        // Ideally `clap` would allow reading in as an enum in such cases..
        _ => unreachable!(),
    };

    let answer: Vec<Vec<&str>> = goal_shape
        .iter()
        .map(|goal_row| find_matches(&all_words, &args.solution, goal_row))
        .collect();

    println!("{}", format_answer(&answer));
}

fn pattern_from_string(string: &str) -> Vec<Vec<bool>> {
    string
        .split(&['/', '\n'][..])
        .map(pattern_for_line)
        .chain(repeat(vec![false; WORD_LENGTH]))
        .take(GUESS_COUNT)
        .collect()
}

fn pattern_from_file(path: &str) -> io::Result<Vec<Vec<bool>>> {
    let contents = fs::read_to_string(path)?;
    Ok(pattern_from_string(&contents))
}

fn pattern_for_line<S: AsRef<str>>(line: S) -> Vec<bool> {
    line.as_ref()
        .chars()
        .chain(repeat(' '))
        .take(WORD_LENGTH)
        .map(|c| c != ' ')
        .collect()
}

fn find_matches<'a>(all_words: &[&'a str], solution: &str, goal_row: &[bool]) -> Vec<&'a str> {
    all_words
        .iter()
        .cloned()
        .filter(|&test_word| does_match(test_word, solution, goal_row))
        .collect()
}

fn does_match(test_word: &str, solution: &str, goal_row: &[bool]) -> bool {
    // TODO hash set is an inaccurate way to do this, need sensitivity to repeats
    let mut yellow_letters: HashSet<char> = solution.chars().collect();

    for (test_char, solution_char, &should_match) in
        izip!(test_word.chars(), solution.chars(), goal_row.iter())
    {
        let does_match = test_char == solution_char;
        if should_match != does_match {
            return false;
        }
        if does_match {
            yellow_letters.remove(&solution_char);
        }
    }

    // One final iteration over the non-matching charactes to ensure none of them will be yellow.
    test_word
        .chars()
        .enumerate()
        .all(|(i, c)| goal_row[i] || !yellow_letters.contains(&c))
}

fn format_answer(answer: &[Vec<&str>]) -> String {
    let mut lines = Vec::new();
    let mut used_words = HashSet::new();
    let mut rng = thread_rng();

    for all_row_answers in answer {
        let unused_row_answers: Vec<&str> = all_row_answers
            .iter()
            .cloned()
            .filter(|&word| !used_words.contains(word))
            .collect();
        let row_answers = if unused_row_answers.is_empty() {
            all_row_answers
        } else {
            &unused_row_answers
        };
        // TODO give dictionary weights based on actual commonality?
        let weights: Vec<usize> = (1..=row_answers.len()).rev().map(|w| w * w).collect();
        match WeightedIndex::new(&weights) {
            Ok(dist) => {
                let word = row_answers[dist.sample(&mut rng)];
                lines.push(word.to_uppercase());
                used_words.insert(word);
            }
            Err(_) => lines.push("[no solution]".to_string()),
        }
    }

    lines.join("\n")
}
