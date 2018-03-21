use std::path::PathBuf;
use std::process::{self, Command};

use super::Config;

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
    args.push("+concolic.method.isPositive=examples.IsPositive.isPositive(other:examples.IsPositive,i:int)");
    args.push("+concolic.method.countPositives=examples.IsPositive.countPositives(xs:int[])");
    args.push(&output_config);
    args.push("+classpath=.");
    let mut cmd = Command::new("java");
    cmd.env("JPF_HOME", &config.jpf_home)
       .env("JVM_FLAGS", &config.jvm_flags)
       .args(&args);
    cmd
}
