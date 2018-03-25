#[macro_use]
extern crate lazy_static;
extern crate mustache;
#[macro_use]
extern crate nom;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

pub mod daikon;
pub mod ftw;
pub mod jpf;
pub mod range;

use regex::Regex;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    jpf_home: String,
    jvm_flags: String,
    classpath: Vec<String>,
    daikon_classpath: Vec<String>,
    max_depth: u32,
}

#[derive(Debug)]
struct JavaArgParseError {
    description: String,
}

impl JavaArgParseError {
    fn from(arg: &str, method: &str) -> JavaArgParseError {
        JavaArgParseError {
            description: format!("Malformed argument {} in method {}", arg, method),
        }
    }
}

impl fmt::Display for JavaArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.description)
    }
}

impl Error for JavaArgParseError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl Config {
    pub fn from_str(s: &str) -> Result<Config, json::Error> {
        json::from_str(s)
    }
}

fn random_alphanumeric_string(size: usize) -> String {
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

pub fn read_file_to_string(path: &str) -> io::Result<String> {
    let mut string = String::new();
    File::open(path)?.read_to_string(&mut string)?;
    Ok(string)
}

pub fn parse_java_method(
    package: &str,
    class: &str,
    decl: &str,
) -> Result<(String, String), Box<Error>> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?P<name>\w+)[ \t]*\([ \t]*(?P<arglist>[^\)]*)[ \t]*\)").unwrap();
    }
    let mut ret = String::new();
    let mut name = String::new();
    for cap in RE.captures_iter(decl) {
        name = String::from(&cap["name"]);
        ret.push_str(package);
        ret.push('.');
        ret.push_str(class);
        ret.push('.');
        ret.push_str(&name);
        ret.push('(');
        if cap["arglist"].len() != 0 {
            for arg in cap["arglist"].split(',') {
                let processed: Vec<&str> = arg.split_whitespace()
                    .filter(|e| !e.starts_with('@'))
                    .collect();
                if processed.len() != 2 {
                    return Err(Box::new(JavaArgParseError::from(arg, &name)));
                }
                ret.push_str(processed[1]);
                ret.push(':');
                ret.push_str(processed[0]);
                ret.push(',');
            }
        }
        ret.push(')');
    }
    Ok((name, ret))
}

#[cfg(test)]
mod test {
    use super::parse_java_method;

    #[test]
    fn test_parse_nullary_method() {
        assert_eq!(
            (
                String::from("isEmpty"),
                String::from("DataStructures.StackAr.isEmpty()")
            ),
            parse_java_method("DataStructures", "StackAr", "    public boolean isEmpty( )")
                .unwrap()
        );
    }
}
