extern crate doco;

use std::env;
use std::process;

use doco::daikon::invariants;
use doco::Config;

fn usage(program_name: &str) {
    eprintln!(
        "Usage: {} <json config>|<path/to/config.json> <package> <class> <method signature>",
        program_name
    );
    process::exit(1);
}

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 5 {
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

    // construct the environment for JPF
    let (out_json_path, mut cmd) =
        doco::jpf::setup_environment(&config, &output_path, &args[2], &args[3], &args[4])
            .unwrap_or_else(|e| {
                eprintln!("Unable to setup JFP environment, err = {}", e.description());
                process::exit(1);
            });
    println!("Spawning JPF");
    let mut jpf = cmd.spawn().unwrap_or_else(|e| {
        eprintln!("Unable to execute JFP, err = {}", e);
        process::exit(1);
    });
    let jpf_succeeded: bool;
    match jpf.try_wait() {
        Ok(Some(status)) => jpf_succeeded = status.success(),
        _ => match jpf.wait() {
            Ok(status) => jpf_succeeded = status.success(),
            _ => jpf_succeeded = false,
        },
    }
    // JPF must have finished before the next if
    if jpf_succeeded {
        match doco::jpf::process_output(&out_json_path) {
            Ok(s) => println!("#doco-jpf {}", s),
            Err(e) => eprintln!("Error: {}", e.description()),
        }
    } else {
        eprintln!("JFP exited with an error");
    }

    println!("\nDynamic Analysis:");
    // match doco::daikon::infer(&config, &output_path) {
    //     Ok(_) => println!("Success!"),
    //     Err(err) => println!("Error running dynamic analysis: {}", err),
    // }
    let inv = invariants::Invariants::from_file("/tmp/inv.txt").unwrap();
    if let Some(rules) = inv.invariants_for("DataStructures.StackAr.isFull()") {
        for r in rules.iter() {
            println!("{}", r);
        }
    }
}
