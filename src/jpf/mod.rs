use mustache::{self, MapBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Write};
use std::fs::File;
use std::path::PathBuf;
use std::process::{self, Command};

use self::expression::Condition;
use super::{json, Config};

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

#[derive(Debug)]
struct NoValidValue {
    description: String,
}

impl NoValidValue {
    fn for_variable(name: &str) -> NoValidValue {
        NoValidValue {
            description: format!("No valid value for '{}'", name),
        }
    }
}

impl fmt::Display for NoValidValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.description)
    }
}

impl Error for NoValidValue {
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
struct InvalidPath {
    description: String,
}

impl InvalidPath {
    fn from(addition: &str) -> InvalidPath {
        InvalidPath {
            description: format!("Unable to construct path to {}", addition),
        }
    }
}

impl fmt::Display for InvalidPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.description)
    }
}

impl Error for InvalidPath {
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
struct JavaArgParseError {
    description: String,
}

impl JavaArgParseError {
    fn from(arg: &str, method: &str) -> JavaArgParseError {
        JavaArgParseError {
            description: format!("Malformed argument {} in method {}", arg, method),
        }
    }
}

impl fmt::Display for JavaArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.description)
    }
}

impl Error for JavaArgParseError {
    fn description(&self) -> &str {
        &self.description
    }
}

fn construct_path(parent: &PathBuf, addition: &str) -> Result<String, Box<Error>> {
    parent
        .join(addition)
        .to_str()
        .map(String::from)
        .ok_or_else(|| Box::new(InvalidPath::from(addition)) as Box<Error>)
}

fn parse_java_method(
    package: &str,
    class: &str,
    decl: &str,
) -> Result<(String, String), Box<Error>> {
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
            let processed: Vec<&str> = arg.split_whitespace()
                .filter(|e| !e.starts_with('@'))
                .collect();
            if processed.len() != 2 {
                return Err(Box::new(JavaArgParseError::from(arg, &name)));
            }
            ret.push_str(processed[1]);
            ret.push(':');
            ret.push_str(processed[0]);
            ret.push(',');
        }
        ret.push(')');
    }
    Ok((name, ret))
}

fn ranges_to_string(
    ranges: &[(i64, i64)],
    name: &str,
    lower: i64,
    upper: i64,
) -> Result<String, Box<Error>> {
    if ranges.len() == 0 {
        return Err(Box::new(NoValidValue::for_variable(name)));
    }
    if ranges.len() == 1 && ranges[0] == (lower, upper) {
        return Ok(String::new());
    }
    let mut s = String::new();
    let mut conditions = Vec::new();
    for &(l, u) in ranges.iter() {
        s.clear();
        s.push('(');
        if l > lower {
            write!(&mut s, "'{}' >= {}", name, l)?;
            if u < upper {
                s.push_str(" && ");
            } else {
                s.push(')');
            }
        }
        if u < upper {
            write!(&mut s, "'{}' <= {})", name, u)?;
        }
        conditions.push(s.clone());
    }
    Ok(format!("{}", conditions.join(" || ")))
}

fn variable_conditions_to_string(
    m: &HashMap<String, expression::Variable>,
) -> Result<String, Box<Error>> {
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
    if s.len() > 4 {
        let len = s.len() - 4;
        s.truncate(len);
    }
    Ok(s)
}

pub fn process_output(out_json_path: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(out_json_path)?;
    let method_summary: MethodSummary = json::from_reader(&mut file)?;
    let mut unparsable = Vec::new();
    let mut parsable_with_one_variable = HashMap::new();
    let mut parsable_with_multiple_variables = Vec::new();
    let mut has_error_paths = false;
    for (_method_name, summary) in method_summary.summaries.iter() {
        if let json::Value::Array(ref v) = summary["errorPaths"] {
            has_error_paths = v.len() > 0;
        }
        match summary["okPaths"] {
            json::Value::Array(ref v) => for ok_path in v.iter() {
                match ok_path["pathCondition"] {
                    json::Value::String(ref s) => match expression::Expression::from_str(s) {
                        expression::Expression::Unparsable(s) => unparsable.push(s),
                        expression::Expression::Parsed(Condition::True) => {
                            return Ok(String::from("True"))
                        }
                        expression::Expression::Parsed(Condition::Conditions(mut m)) => {
                            if m.len() == 1 {
                                match m.drain().take(1).next() {
                                    Some((name, var)) => {
                                        if !parsable_with_one_variable.contains_key(&name) {
                                            parsable_with_one_variable.insert(name, var);
                                        } else {
                                            let range = parsable_with_one_variable[&name]
                                                .range
                                                .union(&var.range);
                                            parsable_with_one_variable
                                                .get_mut(&name)
                                                .unwrap()
                                                .range = range;
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                parsable_with_multiple_variables.push(m);
                            }
                        }
                    },
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }
    if unparsable.len() == 0 && parsable_with_one_variable.len() == 0
        && parsable_with_multiple_variables.len() == 0
    {
        return Ok(String::from(if has_error_paths {
            "True"
        } else {
            "No satisfiable value"
        }));
    }
    let single_var_conditions = variable_conditions_to_string(&parsable_with_one_variable)?;
    if single_var_conditions.len() != 0 {
        unparsable.push(single_var_conditions);
    }
    for cond in parsable_with_multiple_variables.iter() {
        unparsable.push(variable_conditions_to_string(cond)?);
    }
    if unparsable.len() == 1 {
        return Ok(unparsable[0].clone());
    }
    let ret = unparsable.join(") || (");
    if ret == "" {
        return Ok(String::from("True"));
    }
    Ok(format!("({})", ret))
}

pub fn setup_environment(
    config: &Config,
    output_path: &PathBuf,
    package: &str,
    class: &str,
    method: &str,
) -> Result<(String, process::Command), Box<Error>> {
    let (method_name, method_signature) = parse_java_method(package, class, method)?;
    let template = mustache::compile_str(SPF_TEMPLATE).unwrap();
    let jar_path = construct_path(&PathBuf::from(&config.jpf_home), "build/RunJPF.jar")?;
    let out_json_path = construct_path(output_path, "out.json")?;
    let run_jpf_path = construct_path(output_path, "run.jpf")?;
    let template_args = MapBuilder::new()
        .insert_str("classpath", config.classpath.join(";"))
        .insert_str("output_path", out_json_path.clone())
        .insert_str("package", package)
        .insert_str("class", class)
        .insert_str("method_name", method_name)
        .insert_str("method_signature", method_signature)
        .build();
    let mut run_jpf_file = File::create(&run_jpf_path)?;
    template.render_data(&mut run_jpf_file, &template_args)?;
    let mut args: Vec<&str> = config.jvm_flags.split(' ').collect();
    args.push("-jar");
    args.push(&jar_path);
    args.push(&run_jpf_path);
    let mut cmd = Command::new("java");
    cmd.env("JPF_HOME", &config.jpf_home)
        .env("JVM_FLAGS", &config.jvm_flags)
        .args(&args);
    Ok((out_json_path, cmd))
}
