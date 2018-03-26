use std::io::{self, Read};
use std::fs;
use std::fmt;
use std::collections::HashMap;
use regex::Regex;

pub struct Invariants {}

// maps a name (object or method) to a list of inferred pre- and post-conditions
pub struct InvariantList {
    map: HashMap<String, Vec<Inferences>>,
}

const DAIKON_OBJ: &str = "OBJECT";
const DAIKON_ENTER: &str = "ENTER";
const DAIKON_EXIT: &str = "EXIT";

const DAIKON_NULL: &str = "null";
const DAIKON_EQ: &str = "==";
const DAIKON_NOTEQ: &str = "!=";

type Expression = String;

impl fmt::Display for InvariantList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (entity, inferences) in &self.map {
            write!(f, "{}\n", entity)?;

            for inf in inferences.iter() {
                write!(f, "{}\n", inf)?;
            }
        }

        write!(f, "Total: {} entities", self.map.len())
    }
}

impl fmt::Display for Inferences {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"#doco-daikon {{"cond":""#)?;
        write!(f, r#"{}","pre":["#, self.cond)?;
        if self.pre.len() > 0 {
            write!(
                f,
                r#""{}"],"post":["#,
                self.pre
                    .iter()
                    .map(|e| format!("{}", e).replace("\"", "\\\""))
                    .collect::<Vec<String>>()
                    .join("\",\"")
            )?;
        } else {
            write!(f, r#"],"post":["#)?;
        }
        if self.post.len() > 0 {
            write!(
                f,
                r#""{}"]}}"#,
                self.post
                    .iter()
                    .map(|e| format!("{}", e).replace("\"", "\\\""))
                    .collect::<Vec<String>>()
                    .join("\",\"")
            )
        } else {
            write!(f, "]}}")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inferences {
    cond: Expression,     // when the pre- and post-conditions apply
    pre: Vec<Invariant>,  // list of pre-conditions
    post: Vec<Invariant>, // list of post-conditions
}

#[derive(Clone, Debug)]
enum Invariant {
    Null {
        exp: Expression,
    }, // x is NULL
    NotNull {
        exp: Expression,
    }, // x is not NULL
    Comparison {
        lhs: Expression,
        operator: String,
        rhs: Expression,
    }, // x >= y
    Original {
        source: Expression,
        target: Expression,
    }, // x == orig(y)
}
enum InfType {
    PreCondition,
    PostCondition,
}

impl fmt::Display for Invariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Invariant::Null { ref exp } => write!(f, "{} is NULL", exp),
            Invariant::NotNull { ref exp } => write!(f, "{} is not NULL", exp),
            Invariant::Comparison {
                ref lhs,
                ref operator,
                ref rhs,
            } => write!(f, "{} {} {}", lhs, operator, rhs),
            Invariant::Original {
                ref source,
                ref target,
            } => {
                if source == target {
                    return write!(f, "{} is unchanged", source);
                }

                write!(f, "{} == original value of {}", source, target)
            }
        }
    }
}

impl InvariantList {
    pub fn invariants_for(
        &self,
        package: &str,
        class: &str,
        method: &str,
    ) -> Option<&Vec<Inferences>> {
        if let Ok((_, signature)) = super::super::parse_java_method(package, class, method) {
            // DataStructures.StackArTester.top(i:int,s:String,)
            lazy_static! {
                static ref SIG_RE: Regex =
                    Regex::new(r"(?P<pref>[(, ])([^:]+):(?P<type>[^,]+)").unwrap();
                static ref COMMA_RE: Regex = Regex::new(r", *(?P<paren>\))$").unwrap();
            }
            let signature = SIG_RE.replace_all(&signature, "$pref$type");
            let signature = COMMA_RE.replace_all(&signature, "$paren");
            return self.map.get(&signature.to_string());
        }

        None
    }
}

impl Invariants {
    fn parse(daikon_inv: &str) -> InvariantList {
        let mut ret = HashMap::new();
        let mut inferences = Vec::new();
        let mut curr_cond = String::new();
        let mut pre = Vec::new();
        let mut post = Vec::new();

        let mut dstarted = false; // daikon invariants started
        lazy_static! {
            static ref SEP: Regex = Regex::new(r"^=+$").unwrap();
            static ref ENTITY_DEF: Regex = Regex::new(r"^(\S+):::([A-Za-z0-9]+);?(.*)$").unwrap();
            static ref CONDITION_RE: Regex = Regex::new(r#"condition="(.*)""#).unwrap();
            static ref IMPLIES: Regex = Regex::new(r"==>").unwrap();
            static ref PARTS: Regex = Regex::new(r"^(\S+) ([!=<>]+) (\S+)$").unwrap();
        }

        let mut inftype = InfType::PreCondition;
        let mut curr_entity = String::new();
        let mut skipping = false;

        for line in daikon_inv.split("\n") {
            // skip Daikon header
            if !dstarted {
                if SEP.is_match(line) {
                    dstarted = true;
                }

                continue;
            }

            // separator (====)
            if SEP.is_match(line) {
                continue;
            }

            // Object/Method start/end
            if ENTITY_DEF.is_match(line) {
                for cap in ENTITY_DEF.captures_iter(line) {
                    let mut new_cond = String::new();
                    skipping = false;

                    match &cap[2] {
                        DAIKON_OBJ | DAIKON_ENTER => inftype = InfType::PreCondition,
                        DAIKON_EXIT => inftype = InfType::PostCondition,
                        _ => {
                            // unknown rule type: ignore until next event
                            skipping = true;
                            continue;
                        }
                    };

                    // condition is supported?
                    if CONDITION_RE.is_match(&cap[3]) {
                        for cap in CONDITION_RE.captures_iter(&cap[3]) {
                            new_cond = String::from(&cap[1]);
                        }
                    } else {
                        new_cond = String::new();
                    }

                    // verify updates in method names and inference conditions
                    let changed_entity = curr_entity != cap[1].to_string() && curr_entity.len() > 0;
                    let same_entity = curr_entity == cap[1].to_string();
                    let cond_changed = curr_cond != new_cond;

                    if changed_entity || (same_entity && cond_changed) {
                        inferences.push(Inferences {
                            cond: curr_cond.to_owned(),
                            pre: pre.to_owned(),
                            post: post.to_owned(),
                        });

                        pre = Vec::new();
                        post = Vec::new();
                    }

                    if changed_entity {
                        ret.insert(curr_entity.to_owned(), inferences.to_owned());
                        inferences = Vec::new();
                    }

                    curr_entity = cap[1].to_string();
                    curr_cond = new_cond.to_owned();
                }

                continue;
            }

            if skipping {
                continue;
            }

            // invariant definition
            if IMPLIES.is_match(line) {
                continue;
            }

            if PARTS.is_match(line) {
                for cap in PARTS.captures_iter(line) {
                    let inv = match &cap[3] {
                        DAIKON_NULL => match &cap[2] {
                            DAIKON_EQ => Invariant::Null {
                                exp: cap[1].to_string(),
                            },
                            DAIKON_NOTEQ => Invariant::NotNull {
                                exp: cap[1].to_string(),
                            },
                            _ => panic!("Invalid operator on NULL: {}", &cap[2]),
                        },
                        _ => Invariant::Comparison {
                            lhs: cap[1].to_string(),
                            operator: cap[2].to_string(),
                            rhs: cap[3].to_string(),
                        },
                    };

                    match inftype {
                        InfType::PreCondition => pre.push(inv),
                        InfType::PostCondition => post.push(inv),
                    };
                }
            }
        }

        InvariantList { map: ret }
    }

    pub fn from_file(path: &str) -> Result<InvariantList, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        Ok(Invariants::parse(contents.as_str()))
    }
}
