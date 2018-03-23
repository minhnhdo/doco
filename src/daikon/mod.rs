use std::fmt;
use std::path::{Path, PathBuf};
use std::result;

pub mod invariants;

use super::ftw;
use super::Config;

static OUTPUT_PREFIX: &str = "daikon";

type Result<T> = result::Result<T, Error>;

pub enum Error {
    InvalidTestsDirectory(String),
    DynamicAnalysisFailure(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidTestsDirectory(ref dir) => write!(f, "Invalid tests directory: {}", dir),
            Error::DynamicAnalysisFailure(ref file, ref reason) => {
                write!(f, "Daikon failure.\nFile: {}\nReason: {}", file, reason)
            }
        }
    }
}

fn on_test_file(file: &Path) -> ftw::Result {
    println!("\t{}", file.to_str().unwrap());
    Ok(())
}

fn on_test_dir(dir: &Path) -> ftw::Result {
    println!("{}", dir.to_str().unwrap());
    Ok(())
}

pub fn infer(config: &Config, output_path: &PathBuf) -> Result<()> {
    let daikon_out = output_path.join(OUTPUT_PREFIX);
    println!("Inferring to: {:?}", daikon_out);

    if let Err(err) = ftw::ftw(&config.tests_dir.clone(), on_test_dir, on_test_file) {
        return Err(Error::DynamicAnalysisFailure(err.path, err.message));
    }

    Ok(())
}
