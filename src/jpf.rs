use mustache::{self, MapBuilder};
use regex::Regex;
use std::fs::File;
use std::path::PathBuf;
use std::process::{self, Command};

use super::Config;

static SPF_TEMPLATE: &str = r"
shell=gov.nasa.jpf.jdart.summaries.MethodSummarizer
report.console.start=
report.console.finished=
report.console.property_violation=
symbolic.dp=z3
symbolic.dp.z3.bitvectors=true
target={{package}}.{{class}}
classpath={{classpath}}
jdart.summarystore={{output_path}}
summary.methods=isPrime
concolic.method.{{method_name}}={{method_signature}}
concolic.method.{{method_name}}.config={{method_name}}
jdart.configs.{{method_name}}.symbolic.statics={{package}}.{{class}}
jdart.configs.{{method_name}}.symbolic.include=this.*;{{package}}.{{class}}.*
";

fn construct_path(parent: &PathBuf, addition: &str) -> String {
    String::from(parent
                     .join(addition)
                     .to_str()
                     .unwrap_or_else(|| {
                                         eprintln!("Unable to construct path to {}", addition);
                                         process::exit(1);
                                     }))
}

fn parse_java_method(package: &str, class: &str, decl: &str) -> (String, String) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<name>\w+)[ \t]*\([ \t]*(?P<arglist>[^\)]*)[ \t]*\)").unwrap();
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
        for arg in cap["arglist"].split(',') {
            let mut split_arg = arg.split_whitespace();
            let type_ = split_arg
                .next()
                .unwrap_or_else(|| {
                    eprintln!("Unable to extract the type for argument {} of method {}",
                              arg,
                              name);
                    process::exit(1);
                });
            let arg_name = split_arg
                .next()
                .unwrap_or_else(|| {
                    eprintln!("Unable to extract the name for argument {} of method {}",
                              arg,
                              name);
                    process::exit(1);
                });
            if split_arg.next().is_some() {
                eprintln!("Malformed argument {} in method {}", arg, name);
                process::exit(1);
            }
            ret.push_str(arg_name);
            ret.push(':');
            ret.push_str(type_);
            ret.push(',');
        }
        ret.push(')');
    }
    (name, ret)
}

pub fn construct_command(config: &Config, output_path: &PathBuf) -> process::Command {
    let package = "com.google.common.math";
    let class = "IntMath";
    let (method_name, method_signature) =
        parse_java_method(package, class, "public static boolean isPrime(int n) {");
    let template = mustache::compile_str(SPF_TEMPLATE).unwrap();
    let jar_path = construct_path(&PathBuf::from(&config.jpf_home), "build/RunJPF.jar");
    let out_json_path = construct_path(output_path, "out.json");
    let run_jpf_path = construct_path(output_path, "run.jpf");
    let template_args = MapBuilder::new()
        .insert_str("classpath", config.classpath.join(";"))
        .insert_str("output_path", &out_json_path)
        .insert_str("package", package)
        .insert_str("class", class)
        .insert_str("method_name", method_name)
        .insert_str("method_signature", method_signature)
        .build();
    let mut run_jpf_file = File::create(&run_jpf_path).unwrap_or_else(|err| {
        eprintln!("Unable to create {}, err = {}", &run_jpf_path, err);
        process::exit(1);
    });
    template
        .render_data(&mut run_jpf_file, &template_args)
        .unwrap_or_else(|err| {
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
