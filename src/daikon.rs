use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result;

use super::Config;

type Result<T> = result::Result<T, Error>;

pub enum Error {
    InvalidTestsDirectory(String),
    DynamicAnalysisFailure(String, String),
}

impl<'a> fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidTestsDirectory(ref dir) => write!(f, "Invalid tests directory: {}", dir),
            Error::DynamicAnalysisFailure(ref file, ref reason) => {
                write!(f, "Daikon failure.\nFile: {}\nReason: {}", file, reason)
            }
        }
    }
}

fn process_junit_files(file: String) -> Result<()> {
    println!("Found file: {}", file);
    Ok(())
}

fn ftw_rec(path: &Path, callback: fn(String) -> Result<()>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let curr_path = entry.path();

            if curr_path.is_dir() {
                ftw_rec(&curr_path, callback)?;
            } else {
                callback(String::from(curr_path.to_str().unwrap()));
            }
        }
    }

    Ok(())
}

fn ftw(path: &String, callback: fn(String) -> Result<()>) -> Result<()> {
    let p = Path::new(path);
    if !p.is_dir() {
        return Err(Error::InvalidTestsDirectory(path.clone()));
    }

    match ftw_rec(p, callback) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::InvalidTestsDirectory(path.clone())),
    }
}

pub fn infer(config: &Config, output_path: &PathBuf) -> Result<bool> {
    if let Err(err) = ftw(&config.tests_dir.clone(), process_junit_files) {
        return Err(err);
    }
    println!("Inferring to: {:?}", output_path);

    Ok(true)
}
