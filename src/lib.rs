#[macro_use]
extern crate lazy_static;
extern crate mustache;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

pub mod jpf;

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    jpf_home: String,
    jvm_flags: String,
    classpath: Vec<String>,
}

impl Config {
    pub fn from_str(s: &str) -> Result<Config, json::Error> {
        json::from_str(s)
    }
}

pub fn random_alphanumeric_string(size: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(size).collect()
}

pub fn create_random_path(parent: &Path, prefix: &str, size: usize) -> io::Result<PathBuf> {
    let mut output_dir = String::from(prefix);
    loop {
        output_dir.push_str(&random_alphanumeric_string(size));
        if !PathBuf::from(&output_dir).exists() {
            break;
        }
        output_dir.clear();
        output_dir.push_str(prefix);
    }
    let path = parent.join(&output_dir);
    fs::create_dir_all(&path).map(|()| path)
}

pub fn read_file_to_string(path: &str) -> io::Result<String>  {
    let mut string = String::new();
    File::open(path)?.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
