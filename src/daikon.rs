use std::path::PathBuf;
use std::fmt;
use std::result;
use super::Config;

type Result<T> = result::Result<T, Error>;

pub enum Error {
    TestsDirDoesNotExist(String),
    DynamicAnalysisFailure(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TestsDirDoesNotExist(ref dir) => write!(f, "Invalid tests directory: {}", dir),
            Error::DynamicAnalysisFailure(ref file, ref reason) => {
                write!(f, "Daikon failure.\nFile: {}\nReason: {}", file, reason)
            }
        }
    }
}

pub fn infer(config: &Config, output_path: &PathBuf) -> Result<bool> {
    println!("Running dynamic analysis:");
    println!("\tTests Dir: {}", config.tests_dir);
    println!("\tData dir: {}", output_path.as_path().to_str().unwrap());

    Ok(true)
}
