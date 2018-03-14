extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{self, Command};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    jpf_home: String,
    jvm_flags: String,
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

    // construct the command line arguments to pass to jpf
    let jar_path = {
        let path = PathBuf::from(&config.jpf_home).join("build/RunJPF.jar");
        let s = path.to_str()
                    .unwrap_or_else(|| {
                        eprintln!("Unable to construct path to RunJPF.jar");
                        process::exit(1);
                    });
        String::from(s)
    };
    let mut args: Vec<&str> = config.jvm_flags.split(' ').collect();
    args.push("-jar");
    args.push(&jar_path);
    args.push("+shell=gov.nasa.jpf.jdart.summaries.MethodSummarizer");
    args.push("+target=examples.IsPositive");
    args.push("+report.console.start=");
    args.push("+report.console.finished=");
    args.push("+report.console.property_violation=");
    args.push("+symbolic.dp=z3");
    args.push("+symbolic.dp.z3.bitvectors=true");
    args.push("+summary.methods=isPositive,countPositives");
    args.push("+concolic.method.isPositive=examples.IsPositive.isPositive(i:int)");
    args.push("+concolic.method.countPositives=examples.IsPositive.countPositives(xs:int[])");
    args.push("+classpath=.");

    println!("{:?}", Command::new("java")
                             .env("JPF_HOME", &config.jpf_home)
                             .env("JVM_FLAGS", &config.jvm_flags)
                             .args(&args)
                             .output());
}
