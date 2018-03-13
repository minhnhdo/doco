extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

use std::fs::File;
use std::io::{self, Read};
use std::process;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    path_to_klee_suite: String,
}

fn usage(program_name: &str) {
    eprintln!("Usage: {} path/to/config.json", program_name);
    process::abort();
}

fn read_config(path: &str) -> io::Result<String>  {
    let mut string = String::new();
    File::open(path)?.read_to_string(&mut string)?;
    Ok(string)
}

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 2 {
        usage(&args[0]);
    }
    let config: Config = match read_config(&args[1]) {
        Ok(string) => match json::from_str(&string) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Unable to parse configuration file {}, err = {}", args[1], e);
                process::abort();
            },
        },
        Err(e) => {
            eprintln!("Unable to open configuration file {}, err = {}", args[1], e);
            process::abort();
        },
    };
    println!("Found config {:?}", json::to_string(&config));
}
