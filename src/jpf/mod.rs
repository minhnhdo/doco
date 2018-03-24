use mustache::{self, MapBuilder};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;
use std::process::{self, Command};

use self::expression::Condition;
use super::{json, range, Config};

pub mod expression;

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
summary.methods={{method_name}}
concolic.method.{{method_name}}={{method_signature}}
concolic.method.{{method_name}}.config={{method_name}}
jdart.configs.{{method_name}}.symbolic.statics={{package}}.{{class}}
jdart.configs.{{method_name}}.symbolic.include=this.*;{{package}}.{{class}}.*
";

#[derive(Debug, Serialize, Deserialize)]
struct MethodSummary {
    summaries: HashMap<String, json::Value>,
}

fn construct_path(parent: &PathBuf, addition: &str) -> String {
    String::from(parent.join(addition).to_str().unwrap_or_else(|| {
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
            let type_ = split_arg.next().unwrap_or_else(|| {
                eprintln!(
                    "Unable to extract the type for argument {} of method {}",
                    arg, name
                );
                process::exit(1);
            });
            let arg_name = split_arg.next().unwrap_or_else(|| {
                eprintln!(
                    "Unable to extract the name for argument {} of method {}",
                    arg, name
                );
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

fn ranges_to_string(ranges: &[(i64, i64)], name: &str, lower: i64, upper: i64) -> Option<String> {
    if ranges.len() == 0 {
        return None;
    }
    if ranges.len() == 1 && ranges[0] == (lower, upper) {
        return Some(String::new());
    }
    let mut s = String::new();
    let mut conditions = Vec::new();
    for &(l, u) in ranges.iter() {
        s.clear();
        s.push('(');
        if l > lower {
            write!(&mut s, "'{}' >= {}", name, l);
            if u < upper {
                s.push_str(" && ");
            } else {
                s.push(')');
            }
        }
        if u < upper {
            write!(&mut s, "'{}' <= {})", name, u);
        }
        conditions.push(s.clone());
    }
    Some(format!("{}", conditions.join(" || ")))
}

fn variable_conditions_to_string(m: &HashMap<String, expression::Variable>) -> Option<String> {
    let mut s = String::new();
    for (_, var) in m.iter() {
        let (l, u) = {
            let range = var.typ.range();
            range.get_ranges()[0]
        };
        let c = ranges_to_string(var.range.get_ranges(), &var.name, l, u)?;
        if c.len() != 0 {
            s.push_str(&c);
        }
        s.push_str(" && ");
    }
    let len = s.len() - 4;
    s.truncate(len);
    Some(s)
}

pub fn process_output(out_json_path: &str) -> Option<String> {
    let mut conditions = Vec::new();
    let mut file = File::open(out_json_path).ok()?;
    let method_summary: MethodSummary = json::from_reader(&mut file).ok()?;
    for (method_name, summary) in method_summary.summaries.iter() {
        match summary["okPaths"] {
            json::Value::Array(ref v) => for ok_path in v.iter() {
                match ok_path["pathCondition"] {
                    json::Value::String(ref s) => {
                        conditions.push(expression::Expression::from_str(s));
                    }
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }
    // special cases
    if conditions.len() == 0 {
        return Some(String::from("True"));
    }
    if conditions.iter().any(|e| {
        if let &expression::Expression::Parsed(Condition::True) = e {
            true
        } else {
            false
        }
    }) {
        return Some(String::from("True"));
    }
    let mut simple = true;
    let mut vars = HashSet::new();
    for cond in conditions.iter() {
        if let &expression::Expression::Parsed(Condition::Conditions(ref m)) = cond {
            vars.extend(m.keys());
        } else {
            simple = false;
            break;
        }
    }
    if simple && vars.len() == 1 {
        let mut name = String::new();
        let mut typ = expression::Type::SInt8;
        let mut range = range::Range::from(3, 1); // empty range
        match conditions[0] {
            expression::Expression::Parsed(Condition::Conditions(ref m)) => for v in m.values() {
                typ = v.typ.clone();
                name = v.name.clone();
            },
            _ => unreachable!(),
        }
        for cond in conditions.iter() {
            match cond {
                &expression::Expression::Parsed(Condition::Conditions(ref m)) => {
                    for v in m.values() {
                        range = range.union(&v.range);
                    }
                }
                _ => unreachable!(),
            }
        }
        let mut m = HashMap::new();
        m.insert(name.clone(), expression::Variable { name, typ, range });
        return match variable_conditions_to_string(&m) {
            Some(ref s) if s == "" => Some(String::from("True")),
            ret @ _ => ret,
        };
    }
    // nothing works :( apply best effort
    let mut vs = Vec::new();
    for cond in conditions.iter() {
        vs.push(match cond {
            &expression::Expression::Parsed(Condition::Conditions(ref m)) => {
                variable_conditions_to_string(&m)?
            }
            &expression::Expression::Unparsable(ref s) => s.clone(),
            _ => unreachable!(),
        })
    }
    Some(format!("({})", vs.join(") || (")))
}

pub fn construct_command(
    config: &Config,
    output_path: &PathBuf,
    package: &str,
    class: &str,
    method: &str,
) -> (String, process::Command) {
    let (method_name, method_signature) = parse_java_method(package, class, method);
    let template = mustache::compile_str(SPF_TEMPLATE).unwrap();
    let jar_path = construct_path(&PathBuf::from(&config.jpf_home), "build/RunJPF.jar");
    let out_json_path = construct_path(output_path, "out.json");
    let run_jpf_path = construct_path(output_path, "run.jpf");
    let template_args = MapBuilder::new()
        .insert_str("classpath", config.classpath.join(";"))
        .insert_str("output_path", out_json_path.clone())
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
    (out_json_path, cmd)
}
