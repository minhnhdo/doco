extern crate doco;

use std::env;
use std::process;

use doco::Config;

fn usage(program_name: &str) {
    eprintln!(
        "Usage: {} <json config>|<path/to/config.json>",
        program_name
    );
    process::exit(1);
}

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 2 {
        usage(&args[0]);
    }
    let config = {
        let content = if args[1].ends_with(".json") {
            // reading from file
            doco::read_file_to_string(&args[1]).unwrap_or_else(|e| {
                eprintln!("Unable to read configuration file {}, err = {}", args[1], e);
                process::exit(1);
            })
        } else {
            // the config is provided in the command line argument
            args[1].clone()
        };
        Config::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Unable to parse configuration {}, err = {}", content, e);
            process::exit(1);
        })
    };

    let output_path = doco::create_random_path(&env::temp_dir(), "doco", 28).unwrap_or_else(|e| {
        eprintln!("Unable to create output dir, err = {}", e);
        process::exit(1);
    });

    // construct the command line arguments to pass to jpf
    let mut cmd = doco::jpf::construct_command(&config, &output_path);
    println!("Static Analysis:");
    if let Ok(process::Output { stderr, stdout, .. }) = cmd.output() {
        if let Ok(s) = std::str::from_utf8(&stderr) {
            println!("stderr:\n{}\n", s);
        }
        if let Ok(s) = std::str::from_utf8(&stdout) {
            println!("stdout:\n{}\n", s);
        }
    }

    println!("\nDynamic Analysis:");
    match doco::daikon::infer(&config, &output_path) {
        Ok(_) => println!("Success!"),
        Err(err) => println!("Error running dynamic analysis: {}", err),
    }
}
