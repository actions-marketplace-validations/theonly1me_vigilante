mod rust_comments;

use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use rust_comments::comment_lines;

const MAXIMUM_LINES: usize = 200;
const CODE_EXTENSIONS: [&str; 5] = [".rs", ".sh", ".mjs", ".js", ".py"];
const VENDORED_FOLDERS: [&str; 3] = [".agents/", ".claude/", "tests/fixtures/"];
const SYNTHETIC_HOMES: [&str; 3] = ["dev", "runner", "example"];
const HOME_FOLDERS: [&str; 2] = ["/Users/", "/home/"];
const PRIVATE_TERMS: &str = include_str!("../private_terms.txt");

struct TrackedFile {
    path: String,
    text: String,
}

impl TrackedFile {
    fn is_own_code(&self) -> bool {
        CODE_EXTENSIONS
            .iter()
            .any(|extension| self.path.ends_with(extension))
            && !VENDORED_FOLDERS
                .iter()
                .any(|folder| self.path.starts_with(folder))
    }
}

fn tracked_text_files() -> Vec<TrackedFile> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .expect("git ls-files runs");
    String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .filter_map(|path| {
            let text = fs::read_to_string(root.join(path)).ok()?;
            Some(TrackedFile {
                path: path.to_string(),
                text,
            })
        })
        .collect()
}

fn own_comment_lines(file: &TrackedFile) -> Vec<usize> {
    if file.path.ends_with(".rs") {
        return comment_lines(&file.text);
    }
    let marker = if file.path.ends_with(".sh") || file.path.ends_with(".py") {
        "#"
    } else {
        "//"
    };
    file.text
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            (trimmed.starts_with(marker) && !trimmed.starts_with("#!")) || trimmed.starts_with("/*")
        })
        .map(|(index, _)| index + 1)
        .collect()
}

fn private_words(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let hashes: Vec<&str> = PRIVATE_TERMS.lines().collect();
    let pairs = words.windows(2).map(|pair| pair.join(" "));
    words
        .iter()
        .map(|word| word.to_string())
        .chain(pairs)
        .filter(|term| hashes.contains(&hex_digest(term).as_str()))
        .collect()
}

fn hex_digest(term: &str) -> String {
    Sha256::digest(term)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn own_code_files_stay_under_200_lines() {
    let long: Vec<String> = tracked_text_files()
        .iter()
        .filter(|file| file.is_own_code() && file.text.lines().count() >= MAXIMUM_LINES)
        .map(|file| format!("{} has {} lines", file.path, file.text.lines().count()))
        .collect();

    assert_eq!(long, Vec::<String>::new());
}

#[test]
fn own_code_files_have_no_comments() {
    let comments: Vec<String> = tracked_text_files()
        .iter()
        .filter(|file| file.is_own_code())
        .flat_map(|file| {
            own_comment_lines(file)
                .into_iter()
                .map(move |line| format!("{}:{line}", file.path))
        })
        .collect();

    assert_eq!(comments, Vec::<String>::new());
}

#[test]
fn no_tracked_file_has_an_em_dash_or_an_en_dash() {
    let dashes: Vec<String> = tracked_text_files()
        .iter()
        .flat_map(|file| {
            file.text
                .lines()
                .enumerate()
                .filter(|(_, line)| line.contains(['\u{2013}', '\u{2014}']))
                .map(move |(index, _)| format!("{}:{}", file.path, index + 1))
        })
        .collect();

    assert_eq!(dashes, Vec::<String>::new());
}

#[test]
fn no_tracked_file_has_a_home_folder_of_a_real_machine() {
    let homes: Vec<String> = tracked_text_files()
        .iter()
        .flat_map(|file| {
            HOME_FOLDERS.iter().flat_map(move |folder| {
                file.text
                    .match_indices(folder)
                    .filter_map(move |(start, _)| {
                        let name: String = file.text[start + folder.len()..]
                            .chars()
                            .take_while(|character| {
                                character.is_alphanumeric() || matches!(character, '.' | '_' | '-')
                            })
                            .collect();
                        (!name.is_empty() && !SYNTHETIC_HOMES.contains(&name.as_str()))
                            .then(|| format!("{}: {folder}{name}", file.path))
                    })
            })
        })
        .collect();

    assert_eq!(homes, Vec::<String>::new());
}

#[test]
fn no_tracked_file_names_a_private_term() {
    let found: Vec<String> = tracked_text_files()
        .iter()
        .flat_map(|file| {
            private_words(&file.text)
                .into_iter()
                .map(move |_| format!("{} names a private term", file.path))
        })
        .collect();

    assert_eq!(found, Vec::<String>::new());
}
