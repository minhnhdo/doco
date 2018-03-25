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
    if args.len() != 6 {
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
    let (out_json_path, mut jpfcmd) =
        doco::jpf::setup_environment(&config, &output_path, &args[2], &args[3], &args[4])
            .unwrap_or_else(|e| {
                eprintln!("Unable to setup JPF environment, err = {}", e.description());
                process::exit(1);
            });

    // construct the environment for Daikon
    let (out_inv_path, mut dyncompcmd, mut chicorycmd) =
        doco::daikon::setup_environment(&config, &output_path, &args[2], &args[5]).unwrap_or_else(
            |e| {
                eprintln!(
                    "Unable to setup Daikon environment, err = {}",
                    e.description()
                );
                process::exit(1);
            },
        );
    eprintln!("Daikon output to: {}", out_inv_path);

    eprintln!("Spawning JPF");
    let mut jpf = jpfcmd.spawn().unwrap_or_else(|e| {
        eprintln!("Unable to execute JPF, err = {}", e);
        process::exit(1);
    });

    eprintln!("Spawning Daikon instrumentation and inference engine");
    let mut dyncomp = dyncompcmd.spawn().unwrap_or_else(|e| {
        eprintln!("Unable to execute daikon.DynComp, err = {}", e);
        process::exit(1);
    });

    match jpf.wait() {
        Ok(status) if status.success() => match doco::jpf::process_output(&out_json_path) {
            Ok(s) => println!("#doco-jpf {}", s),
            Err(e) => eprintln!("Error: {}", e.description()),
        },
        _ => eprintln!("JPF exited with an error"),
    }

    match dyncomp.wait() {
        Ok(status) if status.success() => match chicorycmd.output() {
            Ok(ref output) if output.status.success() => {
                let inv = invariants::Invariants::from_file(&out_inv_path).unwrap();
                if let Some(rules) = inv.invariants_for(&args[2], &args[3], &args[4]) {
                    eprintln!("\nInvariants found for method: {}\n", &args[4]);

                    for r in rules.iter() {
                        println!("{}", r);
                    }
                }
            }
            _ => eprintln!("daikon.Chicory exited with an error"),
        },
        _ => eprintln!("daikon.DynComp exited with an error"),
    }
}
