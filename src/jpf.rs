use mustache::{self, MapBuilder};
use std::fs::File;
use std::path::PathBuf;
use std::process::{self, Command};

use super::Config;

static SPF_TEMPLATE: &str = r#"
shell=gov.nasa.jpf.jdart.summaries.MethodSummarizer
target=com.google.common.math.IntMath
report.console.start=
report.console.finished=
report.console.property_violation=
symbolic.dp=z3
symbolic.dp.z3.bitvectors=true
summary.methods=isPrime
concolic.method.isPrime=com.google.common.math.IntMath.isPrime(n: int)
concolic.method.isPrime.config=isPrime
jdart.configs.isPrime.symbolic.statics=com.google.common.math.IntMath
jdart.configs.isPrime.symbolic.include=com.google.common.math.IntMath.*;this.*
classpath={{classpath}}
jdart.summarystore={{output_path}}
"#;

fn construct_path(parent: &PathBuf, addition: &str) -> String {
    String::from(parent.join(addition).to_str().unwrap_or_else(|| {
        eprintln!("Unable to construct path to {}", addition);
        process::exit(1);
    }))
}

pub fn construct_command(config: &Config, output_path: &PathBuf) -> process::Command {
    let template = mustache::compile_str(SPF_TEMPLATE).unwrap();
    let jar_path = construct_path(&PathBuf::from(&config.jpf_home), "build/RunJPF.jar");
    let out_json_path = construct_path(output_path, "out.json");
    let run_jpf_path = construct_path(output_path, "run.jpf");
    let template_args = MapBuilder::new()
        .insert_str("classpath", "/home/minh/Documents/Workspace/guava/guava/target/classes;/home/minh/.m2/repository/org/checkerframework/checker-compat-qual/2.0.0/checker-compat-qual-2.0.0.jar")
        .insert_str("output_path", &out_json_path)
        .build();
    let mut run_jpf_file = File::create(&run_jpf_path).unwrap_or_else(|err| {
        eprintln!("Unable to create {}, err = {}", &run_jpf_path, err);
        process::exit(1);
    });
    template.render_data(&mut run_jpf_file, &template_args).unwrap_or_else(|err| {
        eprintln!("Unable to render {}, err = {}", &run_jpf_path, err);
        process::exit(1);
    });
    let mut args: Vec<&str> = config.jvm_flags.split(' ').collect();
    args.push("-jar");
    args.push(&jar_path);
    args.push(&run_jpf_path);
    let mut cmd = Command::new("java");
    cmd.env("JPF_HOME", &config.jpf_home)
       .env("JVM_FLAGS", &config.jvm_flags)
       .args(&args);
    cmd
}
