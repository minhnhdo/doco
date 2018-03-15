extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    jpf_home: String,
    jvm_flags: String,
}

impl Config {
    pub fn from_str(s: &str) -> Result<Config, json::Error> {
        json::from_str(s)
    }
}

pub fn construct_command(config: &Config, output_path: &PathBuf) -> process::Command {
    let jar_path = {
        let path = PathBuf::from(&config.jpf_home).join("build/RunJPF.jar");
        let s = path.to_str()
                    .unwrap_or_else(|| {
                        eprintln!("Unable to construct path to RunJPF.jar");
                        process::exit(1);
                    });
        String::from(s)
    };
    let out_json_path = {
        String::from(output_path.join("out.json").to_str().unwrap_or_else(|| {
            eprintln!("Unable to construct out.json path");
            process::exit(1);
        }))
    };
    let output_config = format!("+jdart.summarystore={}", out_json_path);
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
    args.push(&output_config);
    args.push("+classpath=.");
    let mut cmd = Command::new("java");
    cmd.env("JPF_HOME", &config.jpf_home)
       .env("JVM_FLAGS", &config.jvm_flags)
       .args(&args);
    cmd
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
