extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

use std::fs::File;
use std::io::{self, Read};
use std::process;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
}

fn usage(program_name: &str) {
    eprintln!("Usage: {} <json config>|<path/to/config.json>", program_name);
    process::exit(1);
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
    let config: Config = {
        let content = if args[1].ends_with(".json") {
            // reading from file
            read_config(&args[1]).unwrap_or_else(|e| {
                eprintln!("Unable to read configuration file {}, err = {}", args[1], e);
                process::exit(1);
            })
        } else {
            // the config is provided in the command line argument
            args[1].clone()
        };
        json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Unable to parse configuration {}, err = {}", content, e);
            process::exit(1);
        })
    };
    println!("Found config {:?}", json::to_string(&config));
}
