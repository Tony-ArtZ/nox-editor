use std::{
    fs::{self, DirEntry, File, ReadDir},
    io::{self, BufRead},
};

use nox_editor::FileManager;

pub fn open_file(path: &str) -> io::Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    Ok(lines)
}
