use std::io::{self, Read};
use std::fs;
use std::fmt;
use std::collections::HashMap;
use regex::Regex;

pub struct Invariants {}

// maps a name (object or method) to a list of inferred pre- and post-conditions
pub struct InvariantList {
    map: HashMap<String, Inferences>,
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
        let mut pres = 0;
        let mut posts = 0;

        for (entity, inferences) in &self.map {
            write!(f, "{}\n", entity);

            if inferences.pre.len() > 0 {
                write!(f, "\tPre-conditions:\n");
                for inv in inferences.pre.iter() {
                    pres += 1;
                    write!(f, "\t\t{}\n", inv);
                }
            }

            if inferences.post.len() > 0 {
                write!(f, "\tPost-conditions:\n");
                for inv in inferences.post.iter() {
                    posts += 1;
                    write!(f, "\t\t{}\n", inv);
                }
            }
        }

        write!(f,
               "Total: {} entities, {} pre-conditions, {} post-conditions\n",
               self.map.len(),
               pres,
               posts)
    }
}

#[derive(Clone, Debug)]
pub struct Inferences {
    pre: Vec<Invariant>, // list of pre-conditions
    post: Vec<Invariant>, // list of post-conditions
}

#[derive(Clone, Debug)]
enum Invariant {
    Null { exp: Expression }, // x is NULL
    NotNull { exp: Expression }, // x is not NULL
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

impl Invariants {
    fn parse(daikon_inv: &str) -> InvariantList {
        let mut ret = HashMap::new();
        let mut pre = Vec::new();
        let mut post = Vec::new();

        let mut dstarted = false; // daikon invariants started

        let sep = Regex::new(r"^=+$").unwrap();
        let entity_def = Regex::new(r"^(\S+):::([\S]+)$").unwrap();

        let mut inftype = InfType::PreCondition;
        let mut curr_entity = String::from("");

        for (i, line) in daikon_inv.split("\n").enumerate() {
            // skip Daikon header
            if !dstarted {
                if sep.is_match(line) {
                    dstarted = true;
                }

                continue;
            }

            // separator (====)
            if sep.is_match(line) {
                continue;
            }

            // Object/Method start/end
            if entity_def.is_match(line) {
                for cap in entity_def.captures_iter(line) {
                    match &cap[2] {
                        DAIKON_OBJ => inftype = InfType::PreCondition,
                        DAIKON_ENTER => {
                            if curr_entity.len() > 0 {
                                ret.insert(curr_entity,
                                           Inferences {
                                               pre: pre.to_owned(),
                                               post: post.to_owned(),
                                           });
                            }

                            inftype = InfType::PreCondition;
                        }
                        DAIKON_EXIT => inftype = InfType::PostCondition,
                        _ => println!("Skipping (line {}): {}", i + 1, &cap[2]),
                    };

                    curr_entity = cap[1].to_string();
                }

                continue;
            }

            // invariant definition
            let implies = Regex::new(r"==>").unwrap();
            if implies.is_match(line) {
                println!("Skipping unsupported syntax: {}", line);
                continue;
            }

            let parts = Regex::new(r"^(\S+) ([!=<>]+) (\S+)$").unwrap();
            if parts.is_match(line) {
                for cap in parts.captures_iter(line) {
                    let inv = match &cap[3] {
                        DAIKON_NULL => {
                            match &cap[2] {
                                DAIKON_EQ => Invariant::Null { exp: cap[1].to_string() },
                                DAIKON_NOTEQ => Invariant::NotNull { exp: cap[1].to_string() },
                                _ => panic!("Invalid operator on NULL: {}", &cap[2]),
                            }
                        }
                        _ => {
                            Invariant::Comparison {
                                lhs: cap[1].to_string(),
                                operator: cap[2].to_string(),
                                rhs: cap[3].to_string(),
                            }
                        }
                    };

                    match inftype {
                        InfType::PreCondition => pre.push(inv),
                        InfType::PostCondition => post.push(inv),
                    }
                }
            } else {
                println!("Warning: skipping line {}: {}", i + 1, line);
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
